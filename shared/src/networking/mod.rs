pub mod error;
pub mod result;
pub mod server;
pub mod worker;

use log::{debug, error};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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
    let data_size = data.map_or(0, |d| d.len() as u32);
    let total_message_size = json_message_size + data_size;

    let mut buffer = Vec::new();
    buffer.extend_from_slice(&total_message_size.to_be_bytes());
    buffer.extend_from_slice(&json_message_size.to_be_bytes());
    buffer.extend_from_slice(json_message);
    if let Some(data) = data {
        buffer.extend_from_slice(data);
    }

    if let Err(e) = stream.write_all(&buffer).await {
        error!("Failed to send message: {}", e);
        return Err(e.into());
    }

    if let Err(e) = stream.flush().await {
        error!("Failed to flush stream after sending message: {}", e);
        return Err(e.into());
    }

    debug!(
        "Message sent successfully, total size: {}",
        total_message_size
    );
    Ok(())
}

pub async fn read_message_length(stream: &mut TcpStream) -> NetworkingResult<u32> {
    let mut length_bytes = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut length_bytes).await {
        error!("Failed to read message length: {}", e);
        return Err(e.into());
    }
    let length = u32::from_be_bytes(length_bytes);
    debug!("Read message length: {}", length);
    Ok(length)
}

pub async fn read_json_message(stream: &mut TcpStream, length: usize) -> NetworkingResult<String> {
    let mut json_message = vec![0u8; length];
    if let Err(e) = stream.read_exact(&mut json_message).await {
        error!("Failed to read JSON message: {}", e);
        return Err(e.into());
    }
    let message = String::from_utf8_lossy(&json_message).to_string();
    debug!("JSON message read successfully, length: {}", length);
    Ok(message)
}

pub async fn read_binary_data(stream: &mut TcpStream, length: usize) -> NetworkingResult<Vec<u8>> {
    let mut data_message = vec![0u8; length];
    if let Err(e) = stream.read_exact(&mut data_message).await {
        error!("Failed to read binary data: {}", e);
        return Err(e.into());
    }
    debug!("Binary data read successfully, length: {}", length);
    Ok(data_message)
}

pub async fn write_json_message(
    stream: &mut TcpStream,
    json_message: &str,
) -> NetworkingResult<()> {
    let message_bytes = json_message.as_bytes();
    let message_length = message_bytes.len() as u32;

    if let Err(e) = stream.write_all(&message_length.to_be_bytes()).await {
        error!("Failed to write JSON message length: {}", e);
        return Err(e.into());
    }

    if let Err(e) = stream.write_all(message_bytes).await {
        error!("Failed to write JSON message: {}", e);
        return Err(e.into());
    }

    if let Err(e) = stream.flush().await {
        error!("Failed to flush stream after writing JSON message: {}", e);
        return Err(e.into());
    }

    debug!(
        "JSON message written successfully, length: {}",
        message_length
    );
    Ok(())
}

pub async fn write_binary_data(stream: &mut TcpStream, data: &[u8]) -> NetworkingResult<()> {
    if let Err(e) = stream.write_all(data).await {
        error!("Failed to write binary data: {}", e);
        return Err(e.into());
    }
    if let Err(e) = stream.flush().await {
        error!("Failed to flush stream after writing binary data: {}", e);
        return Err(e.into());
    }
    debug!(
        "Successfully wrote binary data of length {} bytes",
        data.len()
    );
    Ok(())
}

pub async fn send_result(
    stream: &mut TcpStream,
    json_message: &str,
    binary_data: &[u8],
    signature: &[u8],
) -> NetworkingResult<()> {
    let json_bytes = json_message.as_bytes();
    let total_message_size = (json_bytes.len() + binary_data.len() + signature.len()) as u32;

    // Write the total message size
    if let Err(e) = stream.write_u32(total_message_size).await {
        error!("Failed to write total message size: {}", e);
        return Err(e.into());
    }

    // Write components: JSON message, signature, and binary data
    write_json_message(stream, json_message).await?;
    write_binary_data(stream, signature).await?;
    write_binary_data(stream, binary_data).await?;

    debug!(
        "Result sent: {} bytes of JSON, {} bytes of signature, and {} bytes of binary data",
        json_bytes.len(),
        signature.len(),
        binary_data.len()
    );
    Ok(())
}

pub async fn read_message_raw(stream: &mut TcpStream) -> NetworkingResult<RawMessage> {
    debug!("Starting to read a raw message from the stream.");

    // Read the overall message length.
    let message_length = read_message_length(stream).await.map_err(|e| {
        error!("Failed to read message length: {}", e);
        e
    })?;
    debug!("Message length: {}", message_length);

    // Read the length of the JSON message.
    let json_length = read_message_length(stream).await.map_err(|e| {
        error!("Failed to read JSON length: {}", e);
        e
    })?;
    debug!("JSON message length: {}", json_length);

    // Read the JSON message itself.
    let json_message = read_json_message(stream, json_length as usize)
        .await
        .map_err(|e| {
            error!("Failed to read JSON message: {}", e);
            e
        })?;
    debug!("Successfully read JSON message.");

    // Calculate the length of the binary data and read it.
    let binary_data_length = message_length - json_length;
    let data = if binary_data_length > 0 {
        read_binary_data(stream, binary_data_length as usize)
            .await
            .map_err(|e| {
                error!("Failed to read binary data: {}", e);
                e
            })?
    } else {
        Vec::new()
    };
    debug!("Successfully read binary data of length: {}", data.len());

    Ok(RawMessage {
        message_length,
        json_length,
        json_message,
        data,
    })
}
