use actix::{Actor, AsyncContext, StreamHandler};
use actix_web_actors::ws::{self};
use shared::{
    dtos::rendering_data::RenderingData, models::fragments::fragment_request::FragmentRequest,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::portal::ws::messages::PortalMessage;

pub struct WsFragmentProcessor {
    pub fragment_request_tx: Sender<FragmentRequest>,
    pub rendering_data_rx: Arc<Mutex<Receiver<RenderingData>>>,
}

impl WsFragmentProcessor {
    fn start_polling_rendering_data(&self, ctx: &mut <Self as Actor>::Context) {
        let rx = self.rendering_data_rx.clone();
        let actor_address = ctx.address();

        ctx.run_interval(std::time::Duration::from_millis(100), move |_, _| {
            let mut rx_lock = rx.lock().unwrap();
            if let Ok(rendering_data) = rx_lock.try_recv() {
                // Send the rendering data to the actor
                actor_address.do_send(PortalMessage::RenderingDataMessage(rendering_data));
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsFragmentProcessor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            match serde_json::from_str::<FragmentRequest>(&text) {
                Ok(fragment_request) => {
                    let tx = self.fragment_request_tx.clone();
                    ctx.text(format!("Fragment Request sent: {:?}", fragment_request));
                    tokio::spawn(async move {
                        let _ = tx.send(fragment_request).await;
                    });
                }
                Err(_) => ctx.text("Error parsing FragmentRequest"),
            }
        }
    }
}

impl Actor for WsFragmentProcessor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_polling_rendering_data(ctx);
    }
}
