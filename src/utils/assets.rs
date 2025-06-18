use std::{env, io::Error as IoError, num::ParseIntError};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::{
    constants::{INVALID_RESPONSE, PROTOCOL_SEP},
    models::Player,
    prelude::*,
};

pub async fn get_player_choice(
    player: &mut Player,
    prompt: &str,
    msg_type: MessageType,
    passable: bool,
    max_value: usize,
) -> Result<PlayerChoice> {
    let mut pre: String = String::new();
    loop {
        player
            .send_message(&format!("{pre}{prompt}"), msg_type)
            .await?;
        let response_raw: String = player.receive_message().await?;
        let (response, _) = match get_message(response_raw, msg_type) {
            Ok(msg) => msg,
            Err(_) => {
                pre = INVALID_RESPONSE.to_owned();
                continue;
            }
        };
        if response == "pass" {
            if passable {
                return Ok(PlayerChoice::Pass);
            }
            pre = "You can't pass this one".to_owned();
        } else if let Ok(choice) = response.parse::<usize>() {
            if choice <= max_value {
                return Ok(PlayerChoice::Choice(choice));
            }
            pre = format!("Choice can't be greater than {max_value}");
        } else {
            pre = INVALID_RESPONSE.to_owned();
        }
    }
}

pub async fn send_message(connection: &mut TcpStream, message: &str) -> Result<()> {
    let message_bytes: &[u8] = message.as_bytes();
    let length: u32 = message_bytes.len() as u32;
    connection
        .write_all(&length.to_be_bytes())
        .await
        .map_err(Error::connection)?;
    connection
        .write_all(message_bytes)
        .await
        .map_err(Error::connection)?;
    connection.flush().await.map_err(Error::connection)
}

pub async fn receive_message(connection: &mut TcpStream) -> Result<String> {
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
    String::from_utf8(message_buf).map_err(|err: std::string::FromUtf8Error| {
        Error::Tcp(format!("Connection error, Invalid UTF-8 in message: {err}"))
    })
}

pub async fn close_connection(connection: &mut TcpStream) -> Result<()> {
    connection.shutdown().await.map_err(Error::connection)
}

pub async fn get_listener() -> Result<TcpListener> {
    let address: &str = &get_bind_address()?;
    let listener: std::net::TcpListener = std::net::TcpListener::bind(address)
        .map_err(|err: IoError| Error::bind_address(address, err))?;
    listener.set_nonblocking(true).map_err(|err: IoError| {
        Error::Tcp(format!("Failed to enable non blocking: {address}: {err}"))
    })?;
    TcpListener::from_std(listener).map_err(|err: IoError| Error::bind_address(address, err))
}

pub async fn handshake(connection: &mut TcpStream) -> Result<()> {
    let message_type: MessageType = MessageType::Handshake;
    let message: String = set_message("", message_type);
    send_message(connection, &message).await?;
    let response_raw: String = receive_message(connection).await?;
    get_message(response_raw, message_type)?;
    Ok(())
}

pub async fn handle_client(connection: &mut TcpStream) -> Result<String> {
    handshake(connection).await?;
    let message_type: MessageType = MessageType::Username;
    let message: String = set_message("Enter your username:", message_type);
    send_message(connection, &message).await?;
    let response_raw: String = receive_message(connection).await?;
    let (response, _) = get_message(response_raw, message_type)?;
    Ok(response)
}

pub fn set_message(message: &str, message_type: MessageType) -> String {
    format!("{}{PROTOCOL_SEP}{message}", message_type as u8)
}

pub fn get_message(
    message: String,
    expected_message_type: MessageType,
) -> Result<(String, MessageType)> {
    if let Some((msg_type, msg)) = message.split_once(PROTOCOL_SEP) {
        let message_type: MessageType = MessageType::from(msg_type);
        if message_type != expected_message_type {
            return Err(Error::InvalidResponse(expected_message_type, message_type));
        }
        return Ok((msg.to_string(), message_type));
    }
    Ok((message, MessageType::Unknown))
}

fn get_bind_address() -> Result<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return Err(Error::Arg(format!("Usage: {} <host> <port>", args[0])));
    }
    let port: u16 = args[2].parse().map_err(|err: ParseIntError| {
        Error::Arg(format!(
            "Failed to parse port number: {}, err: {}",
            args[2], err
        ))
    })?;
    Ok(format!("{}:{}", &args[1], port))
}
