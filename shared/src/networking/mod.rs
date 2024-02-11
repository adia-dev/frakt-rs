pub mod error;
pub mod result;
pub mod server;
pub mod worker;

use log::debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::models::fragments::fragment::Fragment;

use self::result::NetworkingResult;

#[derive(Debug, Clone)]
pub struct RawMessage {
    pub message_length: u32,
    pub json_length: u32,
    pub json_message: String,
    pub data: Vec<u8>,
}

pub async fn send_message(
    stream: &mut TcpStream,
    json_message: &[u8],
    data: Option<&[u8]>,
) -> NetworkingResult<()> {
    let json_message_size = json_message.len() as u32;
    let data_size = match data {
        Some(data) => data.len() as u32,
        None => 0,
    };
    let total_message_size = json_message_size + data_size;

    let mut buffer = Vec::new();
    buffer.extend_from_slice(&total_message_size.to_be_bytes());
    buffer.extend_from_slice(&json_message_size.to_be_bytes());
    buffer.extend_from_slice(json_message);
    if let Some(data) = data {
        buffer.extend_from_slice(data);
    };

    stream.write_all(&buffer).await?;
    Ok(stream.flush().await?)
}

pub async fn read_message_length(stream: &mut TcpStream) -> NetworkingResult<u32> {
    let mut length_bytes = [0u8; 4];
    stream.read_exact(&mut length_bytes).await?;
    Ok(u32::from_be_bytes(length_bytes))
}

pub async fn read_json_message(stream: &mut TcpStream, length: usize) -> NetworkingResult<String> {
    let mut json_message = vec![0u8; length as usize];
    stream.read_exact(&mut json_message).await?;
    Ok(String::from_utf8_lossy(&json_message).to_string())
}

pub async fn read_binary_data(stream: &mut TcpStream, length: usize) -> NetworkingResult<Vec<u8>> {
    let mut data_message = vec![0u8; length];
    stream.read_exact(&mut data_message).await?;
    Ok(data_message)
}

// QUESTION: Is this executing a heavy copy to the struct
// or just transfering the ownership ?
pub async fn read_message_raw(mut stream: &mut TcpStream) -> NetworkingResult<RawMessage> {
    let message_length = read_message_length(&mut stream).await?;
    let json_length = read_message_length(&mut stream).await?;
    let json_message = read_json_message(&mut stream, json_length as usize).await?;
    let data = read_binary_data(&mut stream, (message_length - json_length) as usize).await?;

    Ok(RawMessage {
        message_length,
        json_length,
        json_message,
        data,
    })
}

pub async fn read_fragment<T>(mut stream: &mut TcpStream) -> NetworkingResult<(Vec<u8>, T)>
where
    T: Fragment,
{
    let message_length = read_message_length(&mut stream).await?;
    let json_length = read_message_length(&mut stream).await?;
    let json_message = read_json_message(&mut stream, json_length as usize).await?;
    let data_message =
        read_binary_data(&mut stream, (message_length - json_length) as usize).await?;

    let fragment = T::from_json(&json_message)?;
    Ok((data_message, fragment))
}

pub async fn write_json_message(
    stream: &mut TcpStream,
    json_message: &str,
) -> NetworkingResult<()> {
    let message_bytes = json_message.as_bytes();
    let message_length = message_bytes.len() as u32;

    // Write the length of the JSON message
    stream.write_u32(message_length).await?;

    // Write the JSON message
    stream.write_all(message_bytes).await?;
    Ok(stream.flush().await?)
}
pub async fn write_binary_data(stream: &mut TcpStream, data: &[u8]) -> NetworkingResult<()> {
    // Write the binary data
    stream.write_all(data).await?;
    Ok(stream.flush().await?)
}

pub async fn send_result(
    stream: &mut TcpStream,
    json_message: &str,
    binary_data: &[u8],
    signature: &[u8],
) -> NetworkingResult<()> {
    let total_message_size =
        (json_message.as_bytes().len() + binary_data.len() + signature.len()) as u32;
    stream.write_u32(total_message_size).await?;
    stream.flush().await?;

    write_json_message(stream, json_message).await?;
    write_binary_data(stream, signature).await?;
    write_binary_data(stream, binary_data).await?;
    Ok(())
}
