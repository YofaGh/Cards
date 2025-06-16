use std::{env, io::Error as IoError, num::ParseIntError};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::{constants::INVALID_RESPONSE, models::Player, prelude::*};

pub async fn get_player_choice(
    player: &mut Player,
    prompt: &str,
    passable: bool,
    max_value: usize,
) -> Result<PlayerChoice> {
    let mut pre: String = String::new();
    loop {
        player.send_message(&format!("{pre}{prompt}"), 1).await?;
        let response: String = player.receive_message().await?;
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

pub async fn handle_client(connection: &mut TcpStream) -> Result<String> {
    send_message(connection, "1$_$_$Enter your username:").await?;
    receive_message(connection).await
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
