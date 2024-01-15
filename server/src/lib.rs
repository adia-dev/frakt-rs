pub mod metrics;
pub mod server_state;

use std::{
    collections::HashMap,
    mem::size_of,
    sync::{Arc, Mutex},
    time::Duration,
};

use complex_rs::complex::Complex;
use log::{debug, error, info};
use metrics::Metrics;
use server_state::ServerState;
use shared::{
    env,
    logger,
    models::{
        fractal::{fractal_descriptor::FractalDescriptor, julia::Julia},
        fragments::{
            fragment::Fragment, fragment_request::FragmentRequest, fragment_result::FragmentResult,
            fragment_task::FragmentTask,
        },
        pixel::pixel_intensity::PixelIntensity,
        point::Point,
        range::Range,
        resolution::Resolution,
        u8_data::U8Data,
    },
    networking::{read_message_raw, result::NetworkingResult, send_message, server::Server},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time,
};

pub async fn run_server(server: &Server) {
    match run(&server).await {
        Ok(()) => info!("Server shutdown gracefully"),
        Err(e) => error!("Server error: {}", e),
    }
}

async fn run(server: &Server) -> NetworkingResult<()> {
    let server_addr = format!("{}:{}", server.address, server.port);
    let listener = start_server(&server_addr).await?;

    info!("Server listening on {}", server_addr);

    // let (metrics_tx, _) = mpsc::channel(32);
    let server_state = ServerState {
        metrics: HashMap::new(),
        workers: HashMap::new(),
    };
    let state = Arc::new(Mutex::new(server_state));

    let metrics_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let state = metrics_state.lock().unwrap();
            for (_, worker) in &state.workers {
                debug!("WORKER: {:#?}", worker);
            }
        }
    });

    let (fragment_result_tx, fragment_result_rx) = mpsc::channel::<(FragmentResult, Vec<u8>)>(32);
    let workers_handle = tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };

            let tx = fragment_result_tx.clone();
            let state = Arc::clone(&state);

            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut socket, tx, state).await {
                    match e {
                        shared::networking::error::NetworkingError::IoError(io_error) => {
                            if io_error.kind() == std::io::ErrorKind::ConnectionReset {
                                debug!("CONNECTION RESET !!!!");
                            }
                        }
                        shared::networking::error::NetworkingError::JsonError(_) => todo!(),
                        shared::networking::error::NetworkingError::Error(_) => todo!(),
                    }
                }
            });
        }
    });

    // Graphics process
    run_graphics(server.width, server.height, fragment_result_rx).await;

    _ = workers_handle.await;

    Ok(())
}

async fn start_server(addr: &str) -> NetworkingResult<TcpListener> {
    Ok(TcpListener::bind(addr).await?)
}

async fn handle_connection(
    mut socket: &mut TcpStream,
    tx: mpsc::Sender<(FragmentResult, Vec<u8>)>,
    _state: Arc<Mutex<ServerState>>,
) -> NetworkingResult<()> {
    debug!("Handling new connection...");
    let raw_message = read_message_raw(&mut socket).await?;

    if let Ok(result) = FragmentResult::from_json(&raw_message.json_message) {
        debug!("Received a result ! {:?}", result);
        _ = tokio::spawn(async move {
            // Read the first 4 bytes
            let (signature_bytes, pixel_data) = raw_message.data.split_at(16);

            info!("Sending the FragmentResult to the rendering channel.");
            match tx.send((result, pixel_data.to_vec())).await {
                Ok(it) => {
                    debug!(
                        "Successfully sent a FragmentResult to the rendering channel. {:?}",
                        it
                    );
                }
                Err(err) => {
                    error!(
                        "Failed to send the FragmentResult through the channel. {:?}",
                        err
                    );
                }
            };

            debug!("SignatureBytes: {:02X?}", signature_bytes);

            // Now process the rest as pixel intensities
            let _pixel_intensities: Vec<PixelIntensity> = pixel_data
                .chunks_exact(size_of::<PixelIntensity>())
                .map(|pixel_intensity_chunk| {
                    let (zn_bytes, count_bytes) = pixel_intensity_chunk.split_at(4);
                    PixelIntensity {
                        zn: f32::from_be_bytes(zn_bytes.try_into().unwrap()),
                        count: f32::from_be_bytes(count_bytes.try_into().unwrap()),
                    }
                })
                .collect();
        })
        .await;
        return Ok(());
    };

    let request = FragmentRequest::from_json(&raw_message.json_message)?;
    info!("FragmentRequest received successfully");
    debug!("FragmentRequest: {:?}", request);

    send_fragment_task(socket).await?;

    Ok(())
}

async fn send_fragment_task(stream: &mut TcpStream) -> NetworkingResult<()> {
    let id = U8Data::new(0, 16);

    let julia = Julia::new(Complex::new(0.285, 0.013), 4.0);
    let fractal_descriptor = FractalDescriptor::Julia(julia);
    let max_iterations: u32 = 64;

    let resolution = Resolution::new(1, 1);
    let range = Range::new(Point::new(-1.2, -1.2), Point::new(1.2, 1.2));

    let task = FragmentTask::new(id, fractal_descriptor, max_iterations, resolution, range);
    let task_json = task.to_json()?;

    // TODO: Randomize the signature for each task sent
    let signature = [0u8; 16];

    let serialized_fragment_task = serde_json::to_string(&task_json)?;
    let serialized_fragment_task_bytes = serialized_fragment_task.as_bytes();

    debug!("Sending FragmentTask to worker: {:?}", task);
    if let Err(e) = send_message(stream, serialized_fragment_task_bytes, Some(&signature)).await {
        error!("Failed to send task: {}", e);
        return Err(e.into());
    }
    info!("FragmentTask sent to a worker");
    Ok(())
}
