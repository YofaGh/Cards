use std::{
    env,
    io::{Error as IoError, Read, Write},
    net::TcpListener,
    num::ParseIntError,
};

use crate::{constants::INVALID_RESPONSE, enums::PlayerChoice, models::Player, prelude::*};

pub fn get_player_choice(
    player: &Player,
    prompt: &str,
    passable: bool,
    max_value: usize,
) -> Result<PlayerChoice> {
    let mut pre: String = String::new();
    loop {
        player.send_message(&format!("{pre}{prompt}"), 1)?;
        let response: String = player.receive_message()?;
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

pub fn send_message(mut connection: &TcpStream, message: &str) -> Result<()> {
    let message_bytes: &[u8] = message.as_bytes();
    connection
        .write_all(&message_bytes.len().to_be_bytes())
        .map_err(Error::connection)?;
    connection
        .write_all(message_bytes)
        .map_err(Error::connection)?;
    connection.flush().map_err(Error::connection)
}

pub fn receive_message(mut connection: &TcpStream) -> Result<String> {
    let mut buf: [u8; 1024] = [0; 1024];
    let bytes_read: usize = connection.read(&mut buf).map_err(Error::connection)?;
    Ok(String::from_utf8_lossy(&buf[..bytes_read]).to_string())
}

pub fn get_listener() -> Result<TcpListener> {
    let address: &str = &get_bind_address()?;
    TcpListener::bind(address).map_err(|err: IoError| Error::bind_address(address, err))
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
