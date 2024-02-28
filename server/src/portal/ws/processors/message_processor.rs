use actix::{Actor, AsyncContext, StreamHandler};
use actix_web_actors::ws::{self};
use shared::{
    dtos::portal_dto::PortalDto, models::fragments::fragment_request::FragmentRequest,
    networking::server::Server,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::portal::ws::messages::PortalMessage;

pub struct WsMessageProcessor {
    pub fragment_request_tx: Sender<FragmentRequest>,
    pub portal_dto_rx: Arc<Mutex<Receiver<PortalDto>>>,
    pub server: Arc<Mutex<Server>>,
}

impl WsMessageProcessor {
    fn start_polling_rendering_data(&self, ctx: &mut <Self as Actor>::Context) {
        let rx = self.portal_dto_rx.clone();
        let actor_address = ctx.address();

        ctx.run_interval(std::time::Duration::from_millis(100), move |_, _| {
            let mut rx_lock = rx.lock().unwrap();
            if let Ok(dto) = rx_lock.try_recv() {
                actor_address.do_send(PortalMessage(dto));
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsMessageProcessor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            match serde_json::from_str::<FragmentRequest>(&text) {
                Ok(fragment_request) => {
                    let tx = self.fragment_request_tx.clone();
                    ctx.text(format!("Fragment Request sent: {:?}", fragment_request));
                    tokio::spawn(async move {
                        let _ = tx.send(fragment_request).await;
                    });

                    let server = self.server.clone();
                    tokio::spawn(async move {
                        let _ = server.lock().unwrap().notify_portal();
                    });
                }
                Err(_) => ctx.text("Error parsing FragmentRequest"),
            }
        }
    }
}

impl Actor for WsMessageProcessor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_polling_rendering_data(ctx);
    }
}
