use rmp_serde::{from_slice, to_vec};
use std::io::Error as IoError;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::{models::Player, prelude::*};

pub async fn get_player_choice(
    player: &mut Player,
    message: &mut GameMessage,
    passable: bool,
    max_value: usize,
) -> Result<PlayerChoice> {
    let mut error: String = String::new();
    loop {
        message.set_error(error.clone());
        player.send_message(message).await?;
        match player.receive_message().await? {
            GameMessage::PlayerChoice { index, passed } => {
                if passed {
                    if passable {
                        return Ok(PlayerChoice::Pass);
                    }
                    error = "You can't pass this one".to_owned();
                } else {
                    if index <= max_value {
                        return Ok(PlayerChoice::Choice(index));
                    }
                    error = format!("Choice can't be greater than {max_value}");
                }
            }
            invalid => {
                error = format!(
                    "Expected message type PlayerChoice, but received {}",
                    invalid.message_type()
                );
            }
        }
    }
}

pub async fn send_message(connection: &mut TcpStream, message: &GameMessage) -> Result<()> {
    let data: Vec<u8> = to_vec(message).map_err(Error::serialization)?;
    let length: u32 = data.len() as u32;
    connection
        .write_all(&length.to_be_bytes())
        .await
        .map_err(Error::connection)?;
    connection
        .write_all(&data)
        .await
        .map_err(Error::connection)?;
    connection.flush().await.map_err(Error::connection)
}

pub async fn receive_message(connection: &mut TcpStream) -> Result<GameMessage> {
    let mut length_buf: [u8; 4] = [0u8; 4];
    connection
        .read_exact(&mut length_buf)
        .await
        .map_err(Error::connection)?;
    let message_length: usize = u32::from_be_bytes(length_buf) as usize;
    let mut message_buf: Vec<u8> = vec![0u8; message_length];
    connection
        .read_exact(&mut message_buf)
        .await
        .map_err(Error::connection)?;
    from_slice(&message_buf).map_err(Error::deserialization)
}

pub async fn close_connection(connection: &mut TcpStream) -> Result<()> {
    connection.shutdown().await.map_err(Error::connection)
}

pub fn get_listener() -> Result<TcpListener> {
    let config: &'static Config = get_config();
    let address: &str = &format!("{}:{}", config.server.host, config.server.port);
    let listener: std::net::TcpListener = std::net::TcpListener::bind(address)
        .map_err(|err: IoError| Error::bind_address(address, err))?;
    listener.set_nonblocking(true).map_err(|err: IoError| {
        Error::Tcp(format!("Failed to enable non blocking: {address}: {err}"))
    })?;
    TcpListener::from_std(listener).map_err(|err: IoError| Error::bind_address(address, err))
}

pub async fn handshake(connection: &mut TcpStream) -> Result<()> {
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

pub async fn handle_client(connection: &mut TcpStream) -> Result<String> {
    handshake(connection).await?;
    send_message(connection, &GameMessage::Username).await?;
    match receive_message(connection).await? {
        GameMessage::UsernameResponse { username } => Ok(username),
        invalid => {
            close_connection(connection).await?;
            Err(Error::InvalidResponse(
                "UsernameResponse".to_string(),
                invalid.message_type(),
            ))
        }
    }
}
