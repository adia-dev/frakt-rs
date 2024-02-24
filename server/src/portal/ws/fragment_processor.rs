use actix::{Actor, AsyncContext, Handler, Message, StreamHandler, Context, Addr};
use actix_web_actors::ws::{self, WebsocketContext, Message as WsMessage, ProtocolError};
use shared::{dtos::rendering_data::RenderingData, models::fragments::fragment_request::FragmentRequest};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{Sender, Receiver};

use super::messages::rendering_data::RenderingDataMessage;

pub struct WsFragmentProcessor {
    pub tx: Sender<FragmentRequest>,
    pub rx: Arc<Mutex<Receiver<RenderingData>>>,
}

impl WsFragmentProcessor {
    fn start_polling_rendering_data(&self, ctx: &mut <Self as Actor>::Context) {
        let rx = self.rx.clone();
        let actor_address = ctx.address();

        ctx.run_interval(std::time::Duration::from_secs(1), move |_, _| {
            let mut rx_lock = rx.lock().unwrap();
            if let Ok(rendering_data) = rx_lock.try_recv() {
                // Send the rendering data to the actor
                actor_address.do_send(RenderingDataMessage(rendering_data));
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsFragmentProcessor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = msg {
            match serde_json::from_str::<FragmentRequest>(&text) {
                Ok(fragment_request) => {
                    let tx = self.tx.clone();
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
