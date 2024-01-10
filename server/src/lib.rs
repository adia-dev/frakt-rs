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
    env, logger,
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
    env::init();
    logger::init();

    match run(&server).await {
        Ok(()) => info!("Server shutdown gracefully"),
        Err(e) => error!("Server error: {}", e),
    }
}

async fn run(server: &Server) -> NetworkingResult<()> {
    let server_addr = format!("{}:{}", server.address, server.port);
    let listener = start_server(&server_addr).await?;

    info!("Server listening on {}", server_addr);

    let (tx, _) = mpsc::channel(32);
    let server_state = ServerState {
        metrics: HashMap::new(),
        workers: HashMap::new(),
    };
    let state = Arc::new(Mutex::new(server_state));

    let metrics_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let state = metrics_state.lock().unwrap();
            for (_, worker) in &state.workers {
                debug!("{:#?}", worker);
            }
        }
    });

    loop {
        let (mut socket, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                continue;
            }
        };

        let tx = tx.clone();
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            _ = handle_connection(&mut socket, tx, state).await;
        });
    }
}

async fn start_server(addr: &str) -> NetworkingResult<TcpListener> {
    Ok(TcpListener::bind(addr).await?)
}

async fn handle_connection(
    mut socket: &mut TcpStream,
    _tx: mpsc::Sender<Metrics>,
    _state: Arc<Mutex<ServerState>>,
) -> NetworkingResult<()> {
    debug!("Handling new connection...");
    let raw_message = read_message_raw(&mut socket).await?;
    if let Ok(result) = FragmentResult::from_json(&raw_message.json_message) {
        debug!("Received a result ! {:?}", result);
        debug!("Data{:?}", result);
        _ = tokio::spawn(async move {
            let pixel_intensities: Vec<PixelIntensity> = raw_message
                .data
                .chunks_exact(size_of::<PixelIntensity>())
                .map(|pixel_intensity_chunk| {
                    let (f32_bytes, _) = pixel_intensity_chunk.split_at(size_of::<f32>());
                    PixelIntensity {
                        zn: f32::from_ne_bytes(f32_bytes.try_into().unwrap()),
                        count: f32::from_ne_bytes(f32_bytes.try_into().unwrap()),
                    }
                })
                .collect();
            debug!("{:?}", pixel_intensities);
        }).await;
        return Ok(());
    };

    let request = FragmentRequest::from_json(&raw_message.json_message)?;

    // IT WAS SO PRETTYYYYY NOOOOOOO
    // let (_, request) = read_fragment::<FragmentRequest>(&mut socket).await?;

    info!("FragmentRequest received successfully");
    debug!("FragmentRequest: {:?}", request);

    send_fragment_task(socket).await?;

    Ok(())
}

async fn send_fragment_task(stream: &mut TcpStream) -> NetworkingResult<()> {
    let id = U8Data::new(0, 16);

    let julia = Julia::new(Complex::new(0.285, 0.013), 4.0);
    let fractal_descriptor = FractalDescriptor::Julia(julia);

    let range = Range::new(Point::new(0.6, -1.2), Point::new(1.2, -0.6));
    let resolution = Resolution::new(100, 100);

    let task = FragmentTask::new(id, fractal_descriptor, 256, resolution, range).to_json()?;

    let serialized_fragment_task = serde_json::to_string(&task)?;
    let serialized_fragment_task_bytes = serialized_fragment_task.as_bytes();
    debug!(
        "Sending FragmentTask to server: {:?}",
        serialized_fragment_task
    );
    if let Err(e) = send_message(stream, serialized_fragment_task_bytes).await {
        error!("Failed to send task: {}", e);
        return Err(e.into());
    }
    info!("Task sent");
    Ok(())
}
