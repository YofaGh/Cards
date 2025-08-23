use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::prelude::*;

pub async fn send_message<W>(writer: &mut W, message: &GameMessage) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    let data: Vec<u8> = rmp_serde::to_vec(message).map_err(Error::serialization)?;
    let length: u32 = data.len() as u32;
    writer
        .write_all(&length.to_be_bytes())
        .await
        .map_err(Error::connection)?;
    writer.write_all(&data).await.map_err(Error::connection)?;
    writer.flush().await.map_err(Error::connection)
}

pub async fn receive_message<R>(reader: &mut R) -> Result<GameMessage>
where
    R: AsyncReadExt + Unpin,
{
    let mut length_buf: [u8; 4] = [0u8; 4];
    reader
        .read_exact(&mut length_buf)
        .await
        .map_err(Error::connection)?;
    let message_length: usize = u32::from_be_bytes(length_buf) as usize;
    let mut message_buf: Vec<u8> = vec![0u8; message_length];
    reader
        .read_exact(&mut message_buf)
        .await
        .map_err(Error::connection)?;
    rmp_serde::from_slice(&message_buf).map_err(Error::deserialization)
}

pub async fn close_connection(connection: &mut Stream) -> Result<()> {
    connection.shutdown().await.map_err(Error::connection)
}
