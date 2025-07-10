use std::result::Result as StdResult;
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;
use uuid::Uuid;

use crate::{errors::Error, games::Game};

pub type PlayerId = Uuid;
pub type TeamId = Uuid;
pub type Result<T, E = Error> = StdResult<T, E>;
pub type Stream = TlsStream<TcpStream>;
pub type BoxGame = Box<dyn Game>;
pub type GameFactory = fn() -> BoxGame;
