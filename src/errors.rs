use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    sync::PoisonError,
};
use uuid::Uuid;

use crate::types::{PlayerId, TeamId};

#[derive(Debug)]
pub enum Error {
    Arg(String),
    Other(String),
    Tcp(String),
    NoValidCard,
}

impl Error {
    pub fn connection(err: IoError) -> Self {
        Self::Tcp(format!("Connection error {}", err))
    }
    pub fn lock_connection<T>(err: PoisonError<T>) -> Self {
        Self::Tcp(format!("Failed to lock connection: {}", err))
    }
    pub fn bind_address(address: &str, err: IoError) -> Self {
        Self::Tcp(format!("Failed to bind address: {}, {}", address, err))
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
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::Arg(msg) | Error::Other(msg) | Error::Tcp(msg) => {
                write!(f, "{msg}")
            }
            Error::NoValidCard => write!(f, "No valid card was found"),
        }
    }
}
