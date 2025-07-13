use once_cell::sync::OnceCell;
use std::{env, path::PathBuf};

use crate::errors::Error;

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

const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_SERVER_PORT: &str = "0";
const DEFAULT_TLS_CERTS_PATH: &str = ".";

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        dotenv::dotenv().ok();
        let tls_path: PathBuf = PathBuf::from(
            env::var("TLS_CERTS_PATH").unwrap_or(DEFAULT_TLS_CERTS_PATH.to_string())
        );
        let config = Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or(DEFAULT_SERVER_HOST.to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or(DEFAULT_SERVER_PORT.to_string())
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

    pub fn validate(&self) -> Result<(), Error> {
        let mut errors = Vec::new();
        
        if self.server.host.is_empty() {
            errors.push("Server host cannot be empty".to_string());
        }
        
        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }
        
        if !self.tls.cert.exists() {
            errors.push(format!("TLS cert.pem was not found: {:?}", self.tls.cert));
        }
        
        if !self.tls.key.exists() {
            errors.push(format!("TLS key.pem was not found: {:?}", self.tls.key));
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

pub fn init_config() -> Result<(), Error> {
    let config: Config = Config::from_env()?;
    CONFIG.set(config).expect("Failed to set CONFIG");
    Ok(())
}