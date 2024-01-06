use image::EncodableLayout;
use log::{debug, error, info};
use serde_json;
use shared::{
    env, logger,
    models::fragments::{
        fragment::Fragment, fragment_request::FragmentRequest, fragment_result::FragmentResult,
        fragment_task::FragmentTask,
    },
    networking::{
        read_binary_data, read_json_message, read_message_length, send_request, send_result,
    },
};
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    env::init();
    logger::init();

    let handle = tokio::spawn(async move {
        loop {
            if let Err(e) = run().await {
                error!("Application error: {}", e);
            }
        }
    });

    _ = handle.await;
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = conntect_to_server("localhost:8787").await?;
    send_fragment_request(&mut stream).await?;

    let mut task: FragmentTask;
    let mut data_message: Vec<u8>;
    let mut result: FragmentResult;
    let mut data: Vec<u8>;

    (data_message, task) = match read_fragment_task(&mut stream).await {
        Ok(value) => value,
        Err(value) => return value,
    };

    (result, data) = match perform_task(&task) {
        Ok(value) => value,
        Err(value) => return value,
    };

    loop {
        let mut inner_stream = conntect_to_server("localhost:8787").await?;
        send_fragment_result(
            result.clone(),
            &mut inner_stream,
            data.clone(),
            data_message.clone(),
        )
        .await?;

        (data_message, task) = match read_fragment_task(&mut inner_stream).await {
            Ok(value) => value,
            Err(value) => return value,
        };

        (result, data) = match perform_task(&task) {
            Ok(value) => value,
            Err(value) => return value,
        };
        inner_stream.shutdown().await?;
    }
}

fn perform_task(
    task: &FragmentTask,
) -> Result<(FragmentResult, Vec<u8>), Result<(), Box<dyn std::error::Error>>> {
    let (result, data) = match task.perform() {
        Ok((result, data)) => (result, data),
        Err(e) => {
            error!("Failed to perform the FragmentTask: {}", e);
            return Err(Err(e.into()));
        }
    };
    info!("FragmentTask performed successfully");
    debug!("FragmentResult: {:?}", result);
    Ok((result, data))
}

async fn send_fragment_result(
    result: FragmentResult,
    inner_stream: &mut TcpStream,
    data: Vec<u8>,
    data_message: Vec<u8>,
) -> io::Result<()> {
    let serialized_fragment_result = FragmentResult::to_json(&result)?.to_string();
    debug!("FragmentResult: {}", serialized_fragment_result);
    send_result(
        inner_stream,
        &serialized_fragment_result,
        &data,
        &data_message.as_bytes(),
    )
    .await?;
    info!("FragmentResult sent successfully");
    Ok(())
}

async fn read_fragment_task(
    mut stream: &mut TcpStream,
) -> Result<(Vec<u8>, FragmentTask), Result<(), Box<dyn std::error::Error>>> {
    let message_length = match read_message_length(&mut stream).await {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read message length: {}", e);
            return Err(Err(e.into()));
        }
    };
    let json_length = match read_message_length(&mut stream).await {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read json length: {}", e);
            return Err(Err(e.into()));
        }
    };
    let json_message = match read_json_message(&mut stream, json_length as usize).await {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to read JSON message: {}", e);
            return Err(Err(e.into()));
        }
    };
    debug!("Received JSON message: {}", json_message);
    let data_message =
        match read_binary_data(&mut stream, (message_length - json_length) as usize).await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to read DATA message: {}", e);
                return Err(Err(e.into()));
            }
        };
    debug!("Received DATA message: {:?}", data_message);
    let task = match FragmentTask::from_json(&json_message) {
        Ok(task) => task,
        Err(e) => {
            error!("Failed to deserialize JSON into FragmentTask: {}", e);
            return Err(Err(e.into()));
        }
    };
    info!("FragmentTask deserialized successfully");
    debug!("Deserialized FragmentTask: {:?}", task);
    Ok((data_message, task))
}

async fn send_fragment_request(stream: &mut TcpStream) -> io::Result<()> {
    let worker_name = "adia-dev";
    info!("Worker launched: {}", worker_name);
    let request = FragmentRequest::new(worker_name.to_string(), 250).to_json()?;
    let serialized_fragment_request = serde_json::to_string(&request)?;
    let serialized_fragment_request_bytes = serialized_fragment_request.as_bytes();
    debug!(
        "Sending FragmentRequest to server: {:?}",
        serialized_fragment_request
    );
    if let Err(e) = send_request(stream, serialized_fragment_request_bytes).await {
        error!("Failed to send request: {}", e);
        return Err(e.into());
    }
    info!("Request sent");
    Ok(())
}

async fn conntect_to_server(addr: &str) -> Result<TcpStream, io::Error> {
    let stream = match TcpStream::connect(addr).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to server: {}", e);
            return Err(e.into());
        }
    };
    info!("Connected to server");
    Ok(stream)
}
