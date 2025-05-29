use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    net::TcpStream,
    sync::{MutexGuard, PoisonError},
};
use uuid::Uuid;

use crate::types::{PlayerId, TeamId};

#[derive(Debug)]
pub enum Error {
    Other(String),
    Lock(String),
    TcpError(String),
    NoValidCard,
}

impl Error {
    pub fn connection(err: IoError) -> Self {
        Self::TcpError(format!("Connection error {}", err))
    }
    pub fn lock_connection(err: PoisonError<MutexGuard<TcpStream>>) -> Self {
        Self::TcpError(format!("Failed to lock connection: {}", err.to_string()))
    }
    pub fn bind_port(host: &str, port: u16, err: IoError) -> Self {
        Self::TcpError(format!(
            "Failed to bind host: {}, port: {}, {}",
            host,
            port,
            err.to_string()
        ))
    }
    pub fn player_not_found(id: PlayerId) -> Self {
        Self::id_not_found(id, "player")
    }
    pub fn team_not_found(id: TeamId) -> Self {
        Self::id_not_found(id, "team")
    }
    pub fn id_not_found(id: Uuid, object: &str) -> Self {
        Self::Other(format!("{} with ID {} not found", object, id))
    }
    pub fn rw_read<T>(err: PoisonError<T>) -> Self {
        Self::Lock(format!("Read lock error {}", err.to_string()))
    }
    pub fn rw_write<T>(err: PoisonError<T>) -> Self {
        Self::Lock(format!("Write lock error {}", err.to_string()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Other(msg) | Error::Lock(msg) | Error::TcpError(msg) => write!(f, "{msg}"),
            Error::NoValidCard => write!(f, "No valid card found to determine winner"),
        }
    }
}
