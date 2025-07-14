use once_cell::sync::OnceCell;
use std::{env, path::PathBuf, time::Duration};

use crate::prelude::*;

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub tls: TlsConfig,
    pub timeout: TimeoutConfig,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub queue_clean_up_interval: Duration,
}

#[derive(Debug)]
pub struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug)]
pub struct TimeoutConfig {
    pub player_choice_enabled: bool,
    pub team_selection: Duration,
    pub player_choice: Duration,
    pub queue_cutoff: Duration,
}

const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_SERVER_PORT: &str = "0";
const DEFAULT_TLS_CERTS_PATH: &str = ".";
const DEFAULT_PLAYER_CHOICE_TIMEOUT_ENABLED: bool = true;
const DEFAULT_QUEUE_CLEAN_UP_INTERVAL: u64 = 300;
const DEFAULT_TEAM_SELECTION_TIMEOUT: u64 = 300;
const DEFAULT_PLAYER_CHOICE_TIMEOUT: u64 = 30;
const DEFAULT_QUEUE_CUTOFF_TIMEOUT: u64 = 600;

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        dotenv::dotenv().ok();
        let tls_path: PathBuf =
            PathBuf::from(env::var("TLS_CERTS_PATH").unwrap_or(DEFAULT_TLS_CERTS_PATH.to_string()));
        let config: Config = Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or(DEFAULT_SERVER_HOST.to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or(DEFAULT_SERVER_PORT.to_string())
                    .parse()?,
                queue_clean_up_interval: get_env_var_as_duration(
                    "QUEUE_CLEAN_UP_INTERVAL",
                    DEFAULT_QUEUE_CLEAN_UP_INTERVAL,
                )?,
            },
            tls: TlsConfig {
                cert: tls_path.join("cert.pem"),
                key: tls_path.join("key.pem"),
            },
            timeout: TimeoutConfig {
                player_choice_enabled: env::var("PLAYER_CHOICE_TIMEOUT_ENABLED")
                    .unwrap_or(DEFAULT_PLAYER_CHOICE_TIMEOUT_ENABLED.to_string())
                    .parse()?,
                team_selection: get_env_var_as_duration(
                    "TEAM_SELECTION_TIMEOUT",
                    DEFAULT_TEAM_SELECTION_TIMEOUT,
                )?,
                player_choice: get_env_var_as_duration(
                    "PLAYER_CHOICE_TIMEOUT",
                    DEFAULT_PLAYER_CHOICE_TIMEOUT,
                )?,
                queue_cutoff: get_env_var_as_duration(
                    "QUEUE_CUTOFF_TIMEOUT",
                    DEFAULT_QUEUE_CUTOFF_TIMEOUT,
                )?,
            },
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut errors: Vec<String> = Vec::new();
        if self.server.host.is_empty() {
            errors.push("Server host cannot be empty".to_string());
        }
        if self.server.port == 0 {
            errors.push("Server port must be greater than 0".to_string());
        }
        if self.server.queue_clean_up_interval.is_zero() {
            errors
                .push("Server queue clean up interval must be greater than 0 seconds".to_string());
        }
        if !self.tls.cert.exists() {
            errors.push(format!("TLS cert.pem was not found: {:?}", self.tls.cert));
        }
        if !self.tls.key.exists() {
            errors.push(format!("TLS key.pem was not found: {:?}", self.tls.key));
        }
        if self.timeout.team_selection.is_zero() {
            errors.push("team selection timeout must be greater than 0 seconds".to_string());
        }
        if self.timeout.player_choice_enabled && self.timeout.player_choice.is_zero() {
            errors.push("player choice timeout must be greater than 0 seconds".to_string());
        }
        if self.timeout.queue_cutoff.is_zero() {
            errors.push("queue cutoff timeout must be greater than 0 seconds".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Config(errors))
        }
    }
}

fn get_env_var_as_duration(key: &str, default_value: u64) -> Result<Duration> {
    Ok(Duration::from_secs(
        env::var(key)
            .unwrap_or(default_value.to_string())
            .parse::<u64>()?,
    ))
}

pub fn get_config() -> &'static Config {
    CONFIG.get().expect("CONFIG not initialized")
}

pub fn init_config() -> Result<(), Error> {
    let config: Config = Config::from_env()?;
    CONFIG.set(config).expect("Failed to set CONFIG");
    Ok(())
}
