use uuid::Uuid;

pub type PlayerId = Uuid;
pub type TeamId = Uuid;
pub type GameId = Uuid;
pub type UserId = Uuid;
pub type Result<T, E = crate::errors::Error> = std::result::Result<T, E>;
pub type Stream = tokio_rustls::TlsStream<tokio::net::TcpStream>;
pub type BoxGame = Box<dyn crate::core::Game + Send>;
pub type GameFactory = fn() -> BoxGame;
