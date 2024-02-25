use std::sync::{Arc, Mutex};

use actix_web::{web::{self, Data}, App, HttpServer};
use log::info;
use shared::{
    dtos::{portal_dto::PortalDto, rendering_data::RenderingData}, models::fragments::fragment_request::FragmentRequest,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::portal::ws::handlers::websocket_route;

pub mod ws;

pub async fn run_portal(
    tx: Sender<FragmentRequest>,
    rx: Receiver<PortalDto>,
) -> std::io::Result<()> {
    info!("🌀 Starting the Portal websocket server");
    let rx = Arc::new(Mutex::new(rx));

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(tx.clone()))
            .app_data(Data::new(rx.clone()))
            .route("/ws/", web::get().to(websocket_route))
    })
    .bind("127.0.0.1:8686")?
    .run();

    let server_handle = tokio::spawn(async move {
        _ = server.await;
    });

    _ = tokio::try_join!(server_handle);

    Ok(())
}
