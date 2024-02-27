use std::{
    env,
    sync::{Arc, Mutex},
};

use actix_web::{
    http::StatusCode,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use log::info;
use shared::{
    dtos::{portal_dto::PortalDto, rendering_data::RenderingData},
    models::fragments::fragment_request::FragmentRequest,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::portal::ws::handlers::websocket_route;

pub async fn health() -> impl Responder {
    HttpResponse::new(StatusCode::OK)
}

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
///
/// # Returns
///
/// A `Result` which is `Ok` if the server runs successfully, or an `Err` with an `io::Error` if an error occurs.
pub async fn run_portal(
    tx: Sender<FragmentRequest>,
    rx: Receiver<PortalDto>,
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
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(rx.clone()))
            .route("/health", web::get().to(health))
            // TODO: refactor this disgusting ass code
            // .route("/move-right", web::get().to(move_right))
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
