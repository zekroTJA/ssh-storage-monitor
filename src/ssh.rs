use crate::config;
use anyhow::Result;
use ssh2::Session;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};

pub struct CommandResult {
    pub exit_code: i32,
    pub output: String,
}

impl CommandResult {
    pub fn ok(&self) -> bool {
        self.exit_code == 0
    }

    pub fn check_exit_code(self) -> Result<Self> {
        if self.ok() {
            return Ok(self);
        }

        if self.output.is_empty() {
            return Err(anyhow::anyhow!(
                "command failed with status {}",
                self.exit_code
            ));
        }

        Err(anyhow::anyhow!(
            "command failed with status {}: {}",
            self.exit_code,
            self.output
        ))
    }
}

pub struct Client {
    session: Session,
}

impl Client {
    pub fn new<A: ToSocketAddrs, S: AsRef<str>>(
        host_addr: A,
        username: S,
        auth: &config::AuthMethod,
    ) -> Result<Self> {
        let tcp = TcpStream::connect(host_addr)?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        match auth {
            config::AuthMethod::Password { password } => {
                session.userauth_password(username.as_ref(), password)?
            }
            config::AuthMethod::Keyfile {
                private_key,
                public_key,
                passphrase,
            } => session.userauth_pubkey_file(
                username.as_ref(),
                public_key.as_deref(),
                private_key,
                passphrase.as_deref(),
            )?,
        }

        if !session.authenticated() {
            return Err(anyhow::anyhow!("not authenticated"));
        }

        Ok(Self { session })
    }

    pub fn exec<S: AsRef<str>>(&self, command: S) -> Result<CommandResult> {
        let mut channel = self.session.channel_session()?;

        channel.exec(command.as_ref())?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;

        channel.wait_close()?;
        let exit_code = channel.exit_status()?;

        Ok(CommandResult { exit_code, output })
    }
}
