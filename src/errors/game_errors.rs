use std::{io::Error as IoError, num::ParseIntError, str::ParseBoolError};
use tokio::time::error::Elapsed;

use crate::core::{PlayerId, TeamId};

#[derive(Debug)]
pub enum Error {
    Config(Vec<String>),
    InvalidResponse(String, String),
    Other(String),
    Tcp(String),
    RmpSerde(String),
    Tls(String),
    FileOperation(String),
    Timeout(String),
    NoValidCard,
}

impl Error {
    pub fn connection(err: IoError) -> Self {
        Self::Tcp(format!("Connection error {err}"))
    }
    pub fn bind_address(address: &str, err: IoError) -> Self {
        Self::Tcp(format!("Failed to bind address: {address}, {err}"))
    }
    pub fn player_not_found(id: PlayerId) -> Self {
        Self::id_not_found(id, "player")
    }
    pub fn team_not_found(id: TeamId) -> Self {
        Self::id_not_found(id, "team")
    }
    pub fn id_not_found(id: uuid::Uuid, object: &str) -> Self {
        Self::Other(format!("{object} with ID {id} not found"))
    }
    pub fn deserialization(err: rmp_serde::decode::Error) -> Self {
        Self::RmpSerde(format!("Deserialization error: {err}"))
    }
    pub fn serialization(err: rmp_serde::encode::Error) -> Self {
        Self::RmpSerde(format!("Serialization error: {err}"))
    }
    pub fn read_file(err: IoError) -> Self {
        Self::FileOperation(format!("unable to read file error: {err}"))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Other(msg)
            | Error::Tcp(msg)
            | Error::RmpSerde(msg)
            | Error::Tls(msg)
            | Error::Timeout(msg)
            | Error::FileOperation(msg) => {
                write!(f, "{msg}")
            }
            Error::Config(errors) => write!(f, "{}", errors.join("\n")),
            Error::InvalidResponse(req, res) => {
                write!(f, "Expected {res} type from client, got {req} type")
            }
            Error::NoValidCard => write!(f, "No valid card was found"),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<ParseBoolError> for Error {
    fn from(err: ParseBoolError) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<Elapsed> for Error {
    fn from(_: Elapsed) -> Self {
        Error::Timeout("Operation timed out".to_string())
    }
}
