use std::sync::{Arc, Mutex};

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use shared::{dtos::portal_dto::PortalDto, models::fragments::fragment_request::FragmentRequest};
use tokio::sync::mpsc::{Receiver, Sender};

use super::processors::fragment_processor::WsMessageProcessor;

pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    tx: web::Data<Sender<FragmentRequest>>,
    rx: web::Data<Arc<Mutex<Receiver<PortalDto>>>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsMessageProcessor {
            fragment_request_tx: tx.get_ref().clone(),
            portal_dto_rx: rx.get_ref().clone(),
        },
        &req,
        stream,
    )
}
