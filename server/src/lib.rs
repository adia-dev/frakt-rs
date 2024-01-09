pub mod metrics;
pub mod server_state;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration, fs::File,
};

use log::{debug, error, info};
use metrics::Metrics;
use server_state::ServerState;
use shared::{
    env, logger,
    models::{
        fractal::{fractal_descriptor::FractalDescriptor, julia::Julia, mandelbrot::Mandelbrot},
        fragments::{
            fragment::Fragment, fragment_request::FragmentRequest, fragment_result::FragmentResult,
            fragment_task::FragmentTask,
        },
        point::Point,
        range::Range,
        resolution::Resolution,
        u8_data::U8Data,
    },
    networking::{
        read_binary_data, read_fragment, read_json_message, read_message_length, read_message_raw,
        result::NetworkingResult, send_message, server::Server, worker::Worker,
    },
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time,
};

pub async fn run_server(server: Server) {
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
    let id = U8Data::new(500, 1);
    let fractal_descriptor = FractalDescriptor::Mandelbrot(Mandelbrot::new());
    let range = Range::new(Point::new(0_f64, 0_f64), Point::new(100_f64, 100_f64));
    let resolution = Resolution::new(8, 8);

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
