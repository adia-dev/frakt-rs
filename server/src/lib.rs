pub mod portal;

use std::{
    mem::size_of,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use log::{debug, error, info, trace};

use shared::{
    dtos::{portal_dto::PortalDto, rendering_data::RenderingData},
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
        worker::Worker,
    },
    rendering::launch_graphics_engine,
};
use tokio::{
    net::{TcpListener, TcpStream},
    signal,
    sync::mpsc::{self, Sender},
};

use crate::portal::run_portal;

/// Executes the main server loop for handling connections and processing graphics if configured
/// that way.
///
/// This function initializes the TCP server, sets up channel communications for rendering data and portal interactions,
/// and spawns tasks for handling incoming connections, rendering graphics, and processing portal requests.
///
/// # Arguments
///
/// * `config` - Configuration settings for the server.
///
/// # Returns
///
/// A `NetworkingResult<()>` indicating success or error in server execution.
pub async fn run_server(config: &ServerConfig) {
    run_wrapper(&config).await;
}

pub async fn run_wrapper(config: &ServerConfig) {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    {
        let shutdown_tx = shutdown_tx.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
            info!("CTRL+C pressed. Shutting down...");
            _ = shutdown_tx.send(());
        })
        .expect("Error setting CTRL+C handler");
    }

    let r = running.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
        r.store(false, Ordering::SeqCst);
    });

    while running.load(Ordering::SeqCst) {
        let config = config.clone();
        match execute_server(&config, shutdown_tx.clone()).await {
            Ok(_) => info!("Server shut down gracefully."),
            Err(e) => error!("Server encountered an error: {}", e),
        }
    }

    info!("Shutting down gracefully...");
    info!("~bye~");
    // TODO: here maybe save the state or some kind of data somewhere
}

use tokio::sync::broadcast;

async fn execute_server(
    config: &ServerConfig,
    shutdown_tx: broadcast::Sender<()>,
) -> NetworkingResult<()> {
    let server_address = format!("{}:{}", config.address, config.port);
    let listener = initialize_server(&server_address).await?;
    info!("Server is listening on {}", server_address);

    let render_shutdown_tx = shutdown_tx.clone();
    let connection_handler_shutdown_tx = shutdown_tx.clone();

    let mut shutdown_signal = shutdown_tx.subscribe();

    let (render_tx, render_rx) = mpsc::channel::<RenderingData>(32);
    let (portal_request_tx, mut portal_request_rx) = mpsc::channel::<FragmentRequest>(32);
    let (portal_tx, portal_rx) = mpsc::channel::<PortalDto>(32);
    let server = create_server(config, Some(render_tx.clone()), Some(portal_tx.clone()));

    let connection_handler = tokio::spawn(handle_connections(
        listener,
        server.clone(),
        render_tx.clone(),
        portal_tx.clone(),
        connection_handler_shutdown_tx.subscribe(),
    ));

    if config.portal {
        let server = server.clone();
        tokio::spawn(run_portal(portal_request_tx, portal_rx, server.clone()));
        tokio::spawn(async move {
            while let Some(request) = portal_request_rx.recv().await {
                process_portal_fragment_request(request, server.clone()).await;
            }
        });
    }

    if config.graphics {
        let graphics_handler =
            launch_graphics_engine(server, render_rx, render_shutdown_tx.subscribe());

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C signal received. Initiating shutdown...");
            },
            _ = shutdown_signal.recv() => {
                info!("Shutdown signal received. Initiating shutdown...");
            },
            _ = connection_handler => {
                info!("Connection handler shutdown completed.");
            },
            _ = graphics_handler => {
                info!("Graphics handler shutdown completed.");
            }
        };
    } else {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl+C signal received. Initiating shutdown...");
            },
            _ = shutdown_signal.recv() => {
                info!("Shutdown signal received. Initiating shutdown...");
            },
            _ = connection_handler => {
                info!("Connection handler shutdown completed.");
            }
        };
    }

    // Signal all components to shutdown
    let _ = shutdown_tx.send(());

    Ok(())
}

fn create_server(
    config: &ServerConfig,
    render_tx: Option<Sender<RenderingData>>,
    portal_tx: Option<Sender<PortalDto>>,
) -> Arc<Mutex<Server>> {
    let server = Server::new(config.clone(), render_tx, portal_tx);
    Arc::new(Mutex::new(server))
}

async fn initialize_server(address: &str) -> NetworkingResult<TcpListener> {
    TcpListener::bind(address).await.map_err(Into::into)
}

async fn handle_connections(
    listener: TcpListener,
    server: Arc<Mutex<Server>>,
    render_tx: Sender<RenderingData>,
    portal_tx: Sender<PortalDto>,
    mut shutdown_rx: broadcast::Receiver<()>,
) {
    info!("Starting to handle incoming connections.");
    loop {
        tokio::select! {
            Ok((socket, socket_addr)) = listener.accept() => {
                debug!("Accepted new connection.");
                let render_tx = render_tx.clone();
                let portal_tx = portal_tx.clone();
                tokio::spawn(handle_connection(
                    socket,
                    socket_addr,
                    server.clone(),
                    render_tx,
                    portal_tx,
                ));
            },
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received in connection handler.");
                return;
            },
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    socket_addr: SocketAddr,
    server: Arc<Mutex<Server>>,
    render_tx: Sender<RenderingData>,
    portal_tx: Sender<PortalDto>,
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
        debug!("📐 Processing FragmentResult.");
        process_fragment_result(
            fragment_result,
            &raw_message.data,
            render_tx,
            portal_tx,
            socket_addr,
            server,
        )
        .await;
    } else if let Ok(request) = FragmentRequest::from_json(&raw_message.json_message) {
        debug!("🤌 Processing FragmentRequest.");
        process_fragment_request(request, server.clone(), &mut socket, socket_addr).await;
    }
}

async fn process_fragment_result(
    result: FragmentResult,
    data: &[u8],
    render_tx: Sender<RenderingData>,
    portal_tx: Sender<PortalDto>,
    socket_addr: SocketAddr,
    server: Arc<Mutex<Server>>,
) {
    info!("Processing received FragmentResult.");
    trace!("FragmentResult details: {:?}", result);

    // Skip the offset bytes of the data
    let offset = result.pixels.offset;
    let data = &data[(offset as usize)..];
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

    let worker = {
        let server = server.lock().unwrap();
        if let Some(worker) = server.get_worker(&socket_addr) {
            worker.name.to_string()
        } else {
            format!("worker-{}", uuid::Uuid::new_v4())
        }
    };

    let rendering_data = RenderingData {
        result,
        iterations,
        worker,
    };

    let (graphics_enabled, portal_enabled) = {
        let server = server.lock().unwrap();
        (server.config.graphics, server.config.portal)
    };

    if graphics_enabled {
        if let Err(e) = render_tx.send(rendering_data.clone()).await {
            error!("Failed to send rendering data: {}", e);
        } else {
            info!("🎨 Sent rendering data to the rendering window");
        }
    }

    if portal_enabled {
        match portal_tx
            .send(PortalDto::RenderindData(rendering_data))
            .await
        {
            Err(e) => {
                error!("Failed to send rendering data to the portal: {}", e);
            }
            Ok(_) => {
                info!("🌀 Sent rendering data to the portal");
                server.lock().unwrap().notify_portal();
            }
        }
    }
}

async fn process_portal_fragment_request(request: FragmentRequest, server: Arc<Mutex<Server>>) {
    info!("🤌🌀 Received FragmentRequest from the portal");
    trace!("FragmentRequest details: {:?}", request);
    let mut server = server.lock().unwrap();

    match server.create_fragment_task() {
        Some(task) => {
            info!("Task queued: {:?}", task);
            server.enqueue_task(task);
        }
        None => {}
    }
}

async fn process_fragment_request(
    request: FragmentRequest,
    server: Arc<Mutex<Server>>,
    socket: &mut TcpStream,
    socket_addr: SocketAddr,
) {
    info!(
        "Received FragmentRequest for worker: {}",
        request.worker_name
    );
    trace!("FragmentRequest details: {:?}", request);
    let task = {
        let mut server = server.lock().unwrap();

        let worker = Worker::new(
            request.worker_name.to_string(),
            request.maximal_work_load,
            server.config.address.to_string(),
            server.config.port,
        );
        server.register_worker(socket_addr, worker);

        match server.dequeue_task() {
            Some(task) => Some(task),
            None => server.create_fragment_task(),
        }
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

    info!("📋 Sending fragment task to worker: {}", worker_name);
    send_message(socket, task_json.as_bytes(), Some(&signature))
        .await
        .map_err(Into::into)
}
