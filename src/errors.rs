use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
    num::ParseIntError,
};
use uuid::Uuid;

use crate::{
    enums::MessageType,
    types::{PlayerId, TeamId},
};

#[derive(Debug)]
pub enum Error {
    Config(Vec<String>),
    InvalidResponse(MessageType, MessageType),
    Other(String),
    Tcp(String),
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
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::Other(msg) | Error::Tcp(msg) => write!(f, "{msg}"),
            Error::Config(errors) => write!(f, "{}", errors.join("\n")),
            Error::InvalidResponse(req, res) => {
                write!(f, "Expected {:?} type from client, got {:?} type", req, res)
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
