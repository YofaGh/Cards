use once_cell::sync::OnceCell;
use std::{env, path::PathBuf};

use crate::{
    constants::{SERVER_HOST, SERVER_PORT, TLS_CERTS_PATH},
    prelude::*,
};

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub tls: TlsConfig,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        let tls_path: PathBuf =
            PathBuf::from(env::var("TLS_CERTS_PATH").unwrap_or(TLS_CERTS_PATH.to_string()));
        let config: Config = Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or(SERVER_HOST.to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or(SERVER_PORT.to_string())
                    .parse()?,
            },
            tls: TlsConfig {
                cert: tls_path.join("cert.pem"),
                key: tls_path.join("key.pem"),
            },
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();
        let cert_path: &PathBuf = &self.tls.cert;
        let key_path: &PathBuf = &self.tls.key;
        if self.server.host.is_empty() {
            errors.push("Server host cannot be empty".to_string());
        }
        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }
        if !cert_path.exists() {
            errors.push(format!("Tls cert.pem was not found: {cert_path:?}"));
        }
        if !key_path.exists() {
            errors.push(format!("Tls key.pem was not found: {key_path:?}"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Config(errors))
        }
    }
}

pub fn get_config() -> &'static Config {
    CONFIG.get().expect("CONFIG not initialized")
}

pub fn init_config() -> Result<()> {
    let config: Config = Config::from_env()?;
    CONFIG.set(config).expect("Failed to set CONFIG");
    Ok(())
}
