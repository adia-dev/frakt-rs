use actix::{Handler, Message};
use log::info;
use serde_json::json;
use shared::dtos::portal_dto::PortalDto;

use super::processors::message_processor::WsMessageProcessor;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PortalMessage(pub PortalDto);

impl Handler<PortalMessage> for WsMessageProcessor {
    type Result = ();

    fn handle(&mut self, msg: PortalMessage, ctx: &mut Self::Context) {
        let (message_type, message_payload) = match msg.0 {
            PortalDto::Server(server) => {
                let payload = serde_json::to_string(&server)
                    .unwrap_or_else(|_| "Error serializing server data".to_string());

                info!("🌀 Server some sync data to the good ol' websocket client");

                ("server_sync", payload)
            }
            PortalDto::RenderindData(rendering_data) => {
                let payload = serde_json::to_string(&rendering_data)
                    .unwrap_or_else(|_| "Error serializing rendering data".to_string());

                ("rendering_data", payload)
            }
        };
        let json = json!({"type": message_type, "payload": message_payload}).to_string();
        ctx.text(json);
    }
}
