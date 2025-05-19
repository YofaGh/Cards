use std::{
    io::{Error as IoError, Read, Write},
    net::TcpListener,
};

use crate::prelude::*;

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

pub fn get_listener(host: &str, port: u16) -> Result<TcpListener> {
    TcpListener::bind(format!("{}:{}", host, port))
        .map_err(|err: IoError| Error::bind_port(host, port, err))
}
