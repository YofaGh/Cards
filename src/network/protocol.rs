use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::prelude::*;

pub async fn send_message(connection: &mut Stream, message: &GameMessage) -> Result<()> {
    let data: Vec<u8> = rmp_serde::to_vec(message).map_err(Error::serialization)?;
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

pub async fn receive_message(connection: &mut Stream) -> Result<GameMessage> {
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
    rmp_serde::from_slice(&message_buf).map_err(Error::deserialization)
}

pub async fn close_connection(connection: &mut Stream) -> Result<()> {
    connection.shutdown().await.map_err(Error::connection)
}
