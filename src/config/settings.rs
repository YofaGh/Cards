use once_cell::sync::OnceCell;
use std::{env, path::PathBuf, time::Duration};

use crate::prelude::{Error, Result};

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug)]
pub struct Config {
    pub game_server: GameServerConfig,
    pub tls: TlsConfig,
    pub timeout: TimeoutConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub api_server: ApiServerConfig,
}

#[derive(Debug)]
pub struct GameServerConfig {
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

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug)]
pub struct JwtConfig {
    pub secret: String,
    pub expire_time: u32,
}

#[derive(Debug)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
}

const DEFAULT_GAME_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_GAME_SERVER_PORT: &str = "0";
const DEFAULT_TLS_CERTS_PATH: &str = ".";
const DEFAULT_PLAYER_CHOICE_TIMEOUT_ENABLED: bool = true;
const DEFAULT_QUEUE_CLEAN_UP_INTERVAL: u64 = 300;
const DEFAULT_TEAM_SELECTION_TIMEOUT: u64 = 300;
const DEFAULT_PLAYER_CHOICE_TIMEOUT: u64 = 30;
const DEFAULT_QUEUE_CUTOFF_TIMEOUT: u64 = 600;
const DEFAULT_DATABASE_URL: &str = "postgresql://localhost:5432/cards_game";
const DEFAULT_DATABASE_MAX_CONNECTIONS: &str = "10";
const DEFAULT_DATABASE_MIN_CONNECTIONS: &str = "2";
const DEFAULT_JWT_SECRET: &str = "abababababababababababababababab";
const DEFAULT_JWT_EXPIRE_DURATION: &str = "24";
const DEFAULT_API_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_API_SERVER_PORT: &str = "0";

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        dotenv::dotenv().ok();
        let tls_path: PathBuf =
            PathBuf::from(env::var("TLS_CERTS_PATH").unwrap_or(DEFAULT_TLS_CERTS_PATH.to_string()));
        let config: Config = Config {
            game_server: GameServerConfig {
                host: env::var("GAME_SERVER_HOST").unwrap_or(DEFAULT_GAME_SERVER_HOST.to_string()),
                port: env::var("GAME_SERVER_PORT")
                    .unwrap_or(DEFAULT_GAME_SERVER_PORT.to_string())
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
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or(DEFAULT_DATABASE_URL.to_string()),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or(DEFAULT_DATABASE_MAX_CONNECTIONS.to_string())
                    .parse()?,
                min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or(DEFAULT_DATABASE_MIN_CONNECTIONS.to_string())
                    .parse()?,
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or(DEFAULT_JWT_SECRET.to_string()),
                expire_time: env::var("JWT_EXPIRE_DURATION")
                    .unwrap_or(DEFAULT_JWT_EXPIRE_DURATION.to_string())
                    .parse()?,
            },
            api_server: ApiServerConfig {
                host: env::var("API_SERVER_HOST").unwrap_or(DEFAULT_API_SERVER_HOST.to_string()),
                port: env::var("API_SERVER_PORT")
                    .unwrap_or(DEFAULT_API_SERVER_PORT.to_string())
                    .parse()?,
            },
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Error> {
        let mut errors: Vec<String> = Vec::new();
        if self.game_server.host.is_empty() {
            errors.push("Game server host cannot be empty".to_string());
        }
        if self.game_server.port == 0 {
            errors.push("Game server port must be greater than 0".to_string());
        }
        if self.game_server.queue_clean_up_interval.is_zero() {
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
        if self.jwt.secret.is_empty() {
            errors.push("Jwt secret cannot be empty".to_string());
        }
        if self.jwt.expire_time == 0 {
            errors.push("Jwt expire time must be greater than 0".to_string());
        }
        if self.api_server.host.is_empty() {
            errors.push("Api server host cannot be empty".to_string());
        }
        if self.api_server.port == 0 {
            errors.push("Api server port must be greater than 0".to_string());
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
