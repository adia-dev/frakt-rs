use std::{
    env,
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_web::{
    http::StatusCode, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use log::info;
use shared::{
    dtos::portal_dto::PortalDto, models::fragments::fragment_request::FragmentRequest,
    networking::server::Server,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::portal::{
    handlers::{cycle_fractal, move_fractal},
    ws::handlers::websocket_route,
};

pub async fn health() -> impl Responder {
    HttpResponse::new(StatusCode::OK)
}

pub mod handlers;
pub mod ws;

/// Starts the portal websocket server.
///
/// This function configures and runs an Actix web server dedicated for handling websocket connections.
/// It sets up shared state for sending and receiving messages through channels and initiates the websocket route.
///
/// # Arguments
///
/// * `tx` - Sender channel for sending fragment requests to the processing logic.
/// * `rx` - Receiver channel for receiving portal DTOs (Data Transfer Objects) from the processing logic.
/// * `server` - server state, contains config, workers etc...
///
/// # Returns
///
/// A `Result` which is `Ok` if the server runs successfully, or an `Err` with an `io::Error` if an error occurs.
pub async fn run_portal(
    tx: Sender<FragmentRequest>,
    rx: Receiver<PortalDto>,
    server: Arc<Mutex<Server>>,
) -> std::io::Result<()> {
    let rx = Arc::new(Mutex::new(rx));

    let host =
        env::var("PORTAL_HOST").expect("Please make sure a `PORTAL_HOST` env variable is setup.");
    let port: u16 = env::var("PORTAL_PORT")
        .expect("Please make sure a `PORTAL_PORT` env variable is setup.")
        .parse()
        .expect("Please make sure the `PORTAL_PORT` env variable is a valid integer");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(rx.clone()))
            .app_data(web::Data::new(server.clone()))
            .route("/health", web::get().to(health))
            .route("/api/fractal/move", web::post().to(move_fractal))
            .route("/api/fractal/cycle", web::post().to(cycle_fractal))
            .route("/ws/", web::get().to(websocket_route))
    })
    .bind((host.as_str(), port))?
    .run();

    info!(
        "🌀 Starting the Portal websocket server at {}:{}",
        host, port
    );

    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            info!("Server error: {:?}", e);
        }
    });

    let _ = server_handle.await;

    info!("🌀 Portal terminated gracefully.");

    Ok(())
}
