use actix::{Handler, Message};
use serde_json::json;
use shared::dtos::{rendering_data::RenderingData, server_dto::ServerDto};

use super::processors::fragment_processor::WsFragmentProcessor;

#[derive(Message)]
#[rtype(result = "()")]
pub enum PortalMessage {
    SyncServerMessage(ServerDto),
    RenderingDataMessage(RenderingData),
}

impl Handler<PortalMessage> for WsFragmentProcessor {
    type Result = ();

    fn handle(&mut self, msg: PortalMessage, ctx: &mut Self::Context) {
        let (message_type, message_payload) = match msg {
            PortalMessage::SyncServerMessage(server) => {
                let payload = serde_json::to_string(&server)
                    .unwrap_or_else(|_| "Error serializing server data".to_string());

                ("server_sync", payload)
            }
            PortalMessage::RenderingDataMessage(rendering_data) => {
                let payload = serde_json::to_string(&rendering_data)
                    .unwrap_or_else(|_| "Error serializing rendering data".to_string());

                ("rendering_data", payload)
            }
        };
        let json = json!({"type": message_type, "payload": message_payload}).to_string();
        ctx.text(json);
    }
}
