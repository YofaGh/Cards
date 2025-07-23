use std::{path::PathBuf, time::Duration};

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
