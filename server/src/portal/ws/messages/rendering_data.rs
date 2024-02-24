use actix::{Handler, Message};
use shared::dtos::rendering_data::RenderingData;

use crate::portal::ws::fragment_processor::WsFragmentProcessor;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RenderingDataMessage(pub RenderingData);

impl Handler<RenderingDataMessage> for WsFragmentProcessor {
    type Result = ();

    fn handle(&mut self, msg: RenderingDataMessage, ctx: &mut Self::Context) {
        let data_json =
            serde_json::to_string(&msg.0).unwrap_or_else(|_| "Error serializing data".to_string());
        // Send the serialized data through the WebSocket
        ctx.text(data_json);
    }
}
