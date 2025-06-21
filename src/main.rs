use crate::config::{Config, Server};
use crate::usage::DiskUsage;
use anyhow::Result;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::borrow::Cow;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

mod config;
mod metrics;
mod ssh;
mod usage;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let config = Arc::new(Config::parse(cli.config)?);

    let log_level = config
        .log_level
        .as_ref()
        .map(|l| tracing::Level::from_str(l))
        .transpose()?
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(std::io::stdout)
        .init();

    let app = Router::new()
        .route("/metrics", get(metrics))
        .with_state(config.clone());

    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn complete_address(addr: &str) -> Cow<'_, str> {
    match addr.split_once(':') {
        Some((host, port)) => {
            if !host.is_empty() && !port.is_empty() {
                return Cow::Borrowed(addr);
            }
            let host = if host.is_empty() { "localhost" } else { host };
            let port = if port.is_empty() { "22" } else { port };
            Cow::Owned(format!("{host}:{port}"))
        }
        None => Cow::Owned(format!("{addr}:22")),
    }
}

fn get_disk_usage(server: &Server) -> Result<DiskUsage> {
    let address = complete_address(&server.address);
    let client = ssh::Client::new(address.as_ref(), &server.username, &server.auth)?;
    let result = client.exec("df")?.check_exit_code()?;
    result.output.parse()
}

fn disk_usage_to_metrics(server: &Server, disk_usage: &DiskUsage) -> String {
    let mut s = String::new();

    for entry in &disk_usage.entries {
        let mut labels = vec![
            ("id", server.id.as_str()),
            ("address", server.address.as_str()),
            ("filesystem", entry.filesystem.as_str()),
            ("mount", entry.mount.as_str()),
        ];

        if let Some(extra_labels) = &server.extra_labels {
            extra_labels.iter().for_each(|(k, v)| labels.push((k, v)));
        };

        metrics::write_metric(&mut s, "ssm_blocks", entry.blocks, &labels).ok();
        metrics::write_metric(&mut s, "ssm_available", entry.available, &labels).ok();
        metrics::write_metric(&mut s, "ssm_used", entry.used, &labels).ok();
    }

    s
}

async fn metrics(State(cfg): State<Arc<Config>>) -> (StatusCode, String) {
    let results: Vec<String> = cfg
        .servers
        .par_iter()
        .map(|server| (server, get_disk_usage(server)))
        .filter_map(|(server, res)| match res {
            Err(err) => {
                tracing::error!(
                    "failed getting disk usage for server {}: {}",
                    server.id,
                    err
                );
                None
            }
            Ok(usage) => Some((server, usage)),
        })
        .map(|(server, usage)| disk_usage_to_metrics(server, &usage))
        .collect();

    let metrics = results.join("\n");

    (StatusCode::OK, metrics)
}
