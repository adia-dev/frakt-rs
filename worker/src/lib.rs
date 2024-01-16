use std::{thread, time::Duration};

use image::EncodableLayout;
use log::{debug, error, info};
use serde_json;
use shared::{
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

pub async fn run_worker(worker: Worker) {
    // TODO: maybe add a counter of consecutive errors with a threshold
    // to detect and shutdown the connection if it is too recurrent
    let handle = tokio::spawn(async move {
        loop {
            if let Err(e) = run(&worker).await {
                error!("Application error: {}", e);
                break;
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

        // NOTE: little sleepy sleep to make the logs readable and emulate a big FRAGMENT TASK
        thread::sleep(Duration::from_millis(500));

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

// TODO: maybe handle each result function calls with a match expression
// to be able to have more descriptive error messages
// TODO: a more modular read_fragment function that takes a type T inheriting from the
// Fragment trait
async fn read_fragment_task(
    mut stream: &mut TcpStream,
) -> NetworkingResult<(Vec<u8>, FragmentTask)> {
    let message_length = read_message_length(&mut stream).await?;
    let json_length = read_message_length(&mut stream).await?;
    let json_message = read_json_message(&mut stream, json_length as usize).await?;

    let data_message =
        read_binary_data(&mut stream, (message_length - json_length) as usize).await?;

    let task = FragmentTask::from_json(&json_message)?;

    info!("FragmentTask deserialized successfully");
    debug!("FragmentTask: {:?}", task);

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
