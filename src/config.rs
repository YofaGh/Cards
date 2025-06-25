use once_cell::sync::OnceCell;
use std::env;

use crate::{
    constants::{SERVER_HOST, SERVER_PORT},
    prelude::*,
};

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        let config: Config = Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or(SERVER_HOST.to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or(SERVER_PORT.to_string())
                    .parse()?,
            },
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        let mut errors: Vec<String> = Vec::new();
        if self.server.host.is_empty() {
            errors.push("Server host cannot be empty".to_string());
        }
        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
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
