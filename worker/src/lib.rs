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
        read_binary_data, read_json_message, read_message_length, result::NetworkingResult,
        send_message, send_result, worker::Worker,
    },
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

// #[tokio::main]
// async fn main() {
//     run_worker().await;
// }

pub async fn run_worker(worker: Worker) {
    env::init();
    logger::init();

    // TODO: maybe add a counter of consecutive errors with a threshold
    // to detect and shutdown the connection if it is too recurrent
    let handle = tokio::spawn(async move {
        loop {
            if let Err(e) = run(&worker).await {
                error!("Application error: {}", e);
            }
        }
    });

    _ = handle.await;
}

async fn run(worker: &Worker) -> NetworkingResult<()> {
    let server_addr = format!("{}:{}", worker.address, worker.port);
    let mut stream = connect_to_server(&server_addr).await?;

    loop {
        send_fragment_request(&mut stream, &worker).await?;
        let (data_message, task) = read_fragment_task(&mut stream).await?;
        let (result, data) = perform_task(&task)?;

        let mut stream = connect_to_server(&server_addr).await?;
        send_fragment_result(&result, &mut stream, &data, &data_message).await?;

        _ = stream.shutdown().await?;
    }
}

fn perform_task(task: &FragmentTask) -> NetworkingResult<(FragmentResult, Vec<u8>)> {
    let (result, data) = match task.perform() {
        Ok((result, data)) => (result, data),
        Err(e) => {
            error!("Failed to perform the FragmentTask: {}", e);
            return Err(e.into());
        }
    };
    info!("FragmentTask performed successfully");
    debug!("FragmentResult: {:?}", result);
    Ok((result, data))
}

async fn send_fragment_result(
    result: &FragmentResult,
    inner_stream: &mut TcpStream,
    data: &Vec<u8>,
    data_message: &Vec<u8>,
) -> NetworkingResult<()> {
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
) -> NetworkingResult<(Vec<u8>, FragmentTask)> {
    let message_length = match read_message_length(&mut stream).await {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read message length: {}", e);
            return Err(e.into());
        }
    };
    let json_length = match read_message_length(&mut stream).await {
        Ok(length) => length,
        Err(e) => {
            error!("Failed to read json length: {}", e);
            return Err(e.into());
        }
    };
    let json_message = match read_json_message(&mut stream, json_length as usize).await {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to read JSON message: {}", e);
            return Err(e.into());
        }
    };
    debug!("Received JSON message: {}", json_message);
    let data_message =
        match read_binary_data(&mut stream, (message_length - json_length) as usize).await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to read DATA message: {}", e);
                return Err(e.into());
            }
        };
    debug!("Received DATA message: {:?}", data_message);
    let task = match FragmentTask::from_json(&json_message) {
        Ok(task) => task,
        Err(e) => {
            error!("Failed to deserialize JSON into FragmentTask: {}", e);
            return Err(e.into());
        }
    };
    info!("FragmentTask deserialized successfully");
    debug!("Deserialized FragmentTask: {:?}", task);
    Ok((data_message, task))
}

async fn send_fragment_request(stream: &mut TcpStream, worker: &Worker) -> NetworkingResult<()> {
    info!("Worker launched: {}", worker.name);
    let request =
        FragmentRequest::new(worker.name.to_owned(), worker.maximal_work_load).to_json()?;
    let serialized_fragment_request = serde_json::to_string(&request)?;
    let serialized_fragment_request_bytes = serialized_fragment_request.as_bytes();
    debug!(
        "Sending FragmentRequest to server: {:?}",
        serialized_fragment_request
    );
    if let Err(e) = send_message(stream, serialized_fragment_request_bytes, None).await {
        error!("Failed to send request: {}", e);
        return Err(e.into());
    }
    info!("Request sent");
    Ok(())
}

async fn connect_to_server(addr: &str) -> NetworkingResult<TcpStream> {
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
