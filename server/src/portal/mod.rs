use std::{
    env,
    sync::{Arc, Mutex},
};

use actix_cors::Cors;
use actix_web::{
    http::StatusCode, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use log::info;
use serde::{Deserialize, Serialize};
use shared::{
    dtos::portal_dto::PortalDto, models::fragments::fragment_request::FragmentRequest,
    networking::server::Server,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::portal::ws::handlers::websocket_route;

pub async fn health() -> impl Responder {
    HttpResponse::new(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct DirectionQuery {
    direction: String,
}

pub async fn cycle_fractal(
    query: web::Query<DirectionQuery>, // Use Query extractor for query parameters
    server: web::Data<Arc<Mutex<Server>>>,
) -> impl Responder {
    let mut server = server.lock().unwrap();
    match query.direction.to_lowercase().as_str() {
        "top" => server.cycle_fractal(),
        "left" => server.cycle_fractal(),
        _ => {
            server.cycle_fractal();
        }
    }

    HttpResponse::Ok().body(format!("Cycled fractal {:?}", query.direction))
}

pub async fn move_fractal(
    query: web::Query<DirectionQuery>, // Use Query extractor for query parameters
    server: web::Data<Arc<Mutex<Server>>>,
) -> impl Responder {
    let mut server = server.lock().unwrap();
    match query.direction.to_lowercase().as_str() {
        "top" => server.move_up(),
        "right" => server.move_right(),
        "bottom" => server.move_down(),
        "left" => server.move_left(),
        _ => {
            return HttpResponse::BadRequest().body(format!("The direction {} is not supported, please enter one of `top`, `left`, `right`, `bottom`.", query.direction));
        }
    }

    HttpResponse::Ok().body(format!("Moved fractal {:?}", query.direction))
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
            .wrap(Cors::default())
            .wrap(Logger::default())
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(rx.clone()))
            .app_data(web::Data::new(server.clone()))
            .route("/health", web::get().to(health))
            // TODO: refactor this disgusting ass code
            .route("/fractal/move", web::get().to(move_fractal))
            .route("/fractal/cycle", web::get().to(cycle_fractal))
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
