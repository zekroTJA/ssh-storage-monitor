use anyhow::Result;
use core::fmt;
use figment::Figment;
use figment::providers::{Env, Format, Toml, Yaml};
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};

macro_rules! package_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

macro_rules! local_config_name {
    ($ext:expr) => {
        concat!(package_name!(), $ext)
    };
}

fn default_bind_address() -> String {
    "0.0.0.0:9091".into()
}

#[derive(Deserialize)]
pub struct Secret<T>(T);

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "***REDACTED***")
    }
}

impl<T> Deref for Secret<T>
where
    T: Deref,
{
    type Target = T::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum AuthMethod {
    Password {
        password: Secret<String>,
    },
    Keyfile {
        private_key: PathBuf,
        public_key: Option<PathBuf>,
        passphrase: Option<Secret<String>>,
    },
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub id: String,
    pub address: String,
    pub username: String,
    pub auth: AuthMethod,
    pub extra_labels: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    pub log_level: Option<String>,
    pub servers: Vec<Server>,
}

impl Config {
    pub fn parse<T: AsRef<Path>>(dir: Option<T>) -> Result<Self> {
        dir.map(Self::parse_from_file)
            .unwrap_or_else(Self::parse_from_cfgdir)
    }

    pub fn parse_from_cfgdir() -> Result<Self> {
        let dirs = dirs::config_dir()
            .map(|d| d.join(package_name!()))
            .ok_or_else(|| anyhow::anyhow!("could not resolve project directories"))?;

        Ok(Figment::new()
            .merge(Env::prefixed("SSM_"))
            .merge(Toml::file(local_config_name!(".toml")))
            .merge(Yaml::file(local_config_name!(".yaml")))
            .merge(Toml::file(dirs.join("config.toml")))
            .merge(Yaml::file(dirs.join("config.yaml")))
            .extract()?)
    }

    pub fn parse_from_file<T: AsRef<Path>>(path: T) -> Result<Self> {
        let ext = path.as_ref().extension().unwrap_or_default();
        let mut figment = Figment::new();

        figment = match ext.to_string_lossy().deref() {
            "yml" | "yaml" => figment.merge(Yaml::file(path)),
            "toml" => figment.merge(Toml::file(path)),
            _ => anyhow::bail!("invalid config file type"),
        };

        Ok(figment.extract()?)
    }
}
