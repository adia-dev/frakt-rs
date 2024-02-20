use std::{
    mem::size_of,
    sync::{Arc, Mutex},
};

use log::{debug, error, info, trace};
use shared::{
    dtos::rendering_data::RenderingData,
    graphics::launch_graphics_engine,
    models::{
        fragments::{
            fragment::Fragment, fragment_request::FragmentRequest, fragment_result::FragmentResult,
            fragment_task::FragmentTask,
        },
        pixel::pixel_intensity::PixelIntensity,
    },
    networking::{
        read_message_raw,
        result::NetworkingResult,
        send_message,
        server::{Server, ServerConfig},
    },
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Sender},
};

pub async fn run_graphics_server(config: &ServerConfig) {
    match execute_server(config).await {
        Ok(_) => info!("Server shut down gracefully."),
        Err(e) => error!("Server encountered an error: {}", e),
    }
}

async fn execute_server(config: &ServerConfig) -> NetworkingResult<()> {
    let server_address = format!("{}:{}", config.address, config.port);
    let listener = initialize_server(&server_address).await?;
    info!("Server is listening on {}", server_address);

    let (render_tx, render_rx) = mpsc::channel::<RenderingData>(32);
    let server = create_server(config, &render_tx);

    let connection_handler = tokio::spawn(handle_connections(
        listener,
        server.clone(),
        render_tx.clone(),
    ));
    let graphics_handler = launch_graphics_engine(server, render_rx);

    let _ = tokio::join!(connection_handler, graphics_handler);

    Ok(())
}

fn create_server(config: &ServerConfig, render_tx: &Sender<RenderingData>) -> Arc<Mutex<Server>> {
    let server = Server::new(config.clone(), render_tx.clone());
    Arc::new(Mutex::new(server))
}

async fn initialize_server(address: &str) -> NetworkingResult<TcpListener> {
    TcpListener::bind(address).await.map_err(Into::into)
}

async fn handle_connections(
    listener: TcpListener,
    server: Arc<Mutex<Server>>,
    render_tx: Sender<RenderingData>,
) {
    info!("Starting to handle incoming connections.");
    while let Ok((socket, _)) = listener.accept().await {
        debug!("Accepted new connection.");
        let tx_clone = render_tx.clone();
        tokio::spawn(handle_connection(socket, server.clone(), tx_clone));
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    server: Arc<Mutex<Server>>,
    render_tx: Sender<RenderingData>,
) {
    debug!("Initiating connection handling.");
    let raw_message = match read_message_raw(&mut socket).await {
        Ok(msg) => {
            trace!("Received raw message: {:?}", msg);
            msg
        }
        Err(e) => {
            error!("Failed to read message: {:?}", e);
            return;
        }
    };
    trace!("Raw message: {:?}", raw_message);

    if let Ok(fragment_result) = FragmentResult::from_json(&raw_message.json_message) {
        debug!("Processing FragmentResult.");
        process_fragment_result(fragment_result, &raw_message.data, render_tx).await;
    } else if let Ok(request) = FragmentRequest::from_json(&raw_message.json_message) {
        debug!("Processing FragmentRequest.");
        process_fragment_request(request, server.clone(), &mut socket).await;
    }
}

async fn process_fragment_result(
    result: FragmentResult,
    data: &[u8],
    render_tx: Sender<RenderingData>,
) {
    info!("Processing received FragmentResult.");
    trace!("FragmentResult details: {:?}", result);
    if data.len() % size_of::<PixelIntensity>() != 0 {
        error!("Data size is not aligned with PixelIntensity size.");
        return;
    }

    let pixel_intensities: Vec<PixelIntensity> = data
        .chunks_exact(size_of::<PixelIntensity>())
        .filter_map(|chunk| {
            let zn_bytes = chunk.get(0..4)?.try_into().ok()?;
            let count_bytes = chunk.get(4..8)?.try_into().ok()?;
            Some(PixelIntensity {
                zn: f32::from_be_bytes(zn_bytes),
                count: f32::from_be_bytes(count_bytes),
            })
        })
        .collect();

    //NOTE: we currenlty only care about the count
    let iterations: Vec<f64> = pixel_intensities.iter().map(|pi| pi.count as f64).collect();

    let rendering_data = RenderingData {
        result,
        iterations,
        worker: "adia-dev".to_owned(),
    };

    if let Err(e) = render_tx.send(rendering_data).await {
        error!("Failed to send rendering data: {}", e);
    }
}

async fn process_fragment_request(
    request: FragmentRequest,
    server: Arc<Mutex<Server>>,
    socket: &mut TcpStream,
) {
    info!(
        "Received FragmentRequest for worker: {}",
        request.worker_name
    );
    trace!("FragmentRequest details: {:?}", request);
    let task = {
        let mut server = server.lock().unwrap();
        server.create_fragment_task()
    };

    match task {
        Some(task) => {
            if let Err(e) = send_fragment_task(socket, &request.worker_name, &task).await {
                error!("Failed to send fragment task: {}", e);
            }
        }
        None => {
            info!("No more fragment tasks to send.");
        }
    }
}

async fn send_fragment_task(
    socket: &mut TcpStream,
    worker_name: &str,
    task: &FragmentTask,
) -> NetworkingResult<()> {
    let serialized_task = task.to_json()?;
    let task_json = serde_json::to_string(&serialized_task)?;
    let signature = [0u8; 16]; // Placeholder for actual signature logic

    info!("Sending fragment task to worker: {}", worker_name);
    send_message(socket, task_json.as_bytes(), Some(&signature))
        .await
        .map_err(Into::into)
}
