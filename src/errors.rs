use rmp_serde::{decode::Error as DecodeError, encode::Error as EncodeError};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    num::ParseIntError,
};
use uuid::Uuid;

use crate::types::{PlayerId, TeamId};

#[derive(Debug)]
pub enum Error {
    Config(Vec<String>),
    InvalidResponse(String, String),
    Other(String),
    Tcp(String),
    RmpSerde(String),
    Tls(String),
    FileOperation(String),
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
    pub fn id_not_found(id: Uuid, object: &str) -> Self {
        Self::Other(format!("{object} with ID {id} not found"))
    }
    pub fn deserialization(err: DecodeError) -> Self {
        Self::RmpSerde(format!("Deserialization error: {err}"))
    }
    pub fn serialization(err: EncodeError) -> Self {
        Self::RmpSerde(format!("Serialization error: {err}"))
    }
    pub fn read_file(err: IoError) -> Self {
        Self::FileOperation(format!("unable to read file error: {err}"))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::Other(msg)
            | Error::Tcp(msg)
            | Error::RmpSerde(msg)
            | Error::Tls(msg)
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
