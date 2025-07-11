use std::io::Error as IoError;
use tokio::net::TcpListener;

use crate::{
    network::{close_connection, receive_message, send_message},
    prelude::*,
};

pub async fn get_listener() -> Result<TcpListener> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.server.host, config.server.port);
    TcpListener::bind(address)
        .await
        .map_err(|err: IoError| Error::bind_address(address, err))
}

pub async fn handshake(connection: &mut Stream) -> Result<()> {
    send_message(connection, &GameMessage::Handshake).await?;
    match receive_message(connection).await? {
        GameMessage::HandshakeResponse => Ok(()),
        invalid => {
            close_connection(connection).await?;
            Err(Error::InvalidResponse(
                GameMessage::HandshakeResponse.message_type(),
                invalid.message_type(),
            ))
        }
    }
}

pub async fn handle_client(connection: &mut Stream) -> Result<String> {
    handshake(connection).await?;
    let mut message: GameMessage = GameMessage::demand(DemandMessage::Username);
    loop {
        send_message(connection, &message).await?;
        match receive_message(connection).await? {
            GameMessage::PlayerChoice { choice } => {
                if !choice.is_empty() {
                    return Ok(choice);
                }
                message.set_demand_error("Username can not be empty!".to_owned());
            }
            invalid => {
                close_connection(connection).await?;
                return Err(Error::InvalidResponse(
                    "PlayerChoice".to_string(),
                    invalid.message_type(),
                ));
            }
        }
    }
}
