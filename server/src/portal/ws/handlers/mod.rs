use std::sync::{Arc, Mutex};

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use shared::{
    dtos::rendering_data::RenderingData, models::fragments::fragment_request::FragmentRequest,
};
use tokio::sync::mpsc::{Receiver, Sender};

use super::processors::fragment_processor::WsFragmentProcessor;

pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    tx: web::Data<Sender<FragmentRequest>>,
    rx: web::Data<Arc<Mutex<Receiver<RenderingData>>>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsFragmentProcessor {
            fragment_request_tx: tx.get_ref().clone(),
            rendering_data_rx: rx.get_ref().clone(),
        },
        &req,
        stream,
    )
}
