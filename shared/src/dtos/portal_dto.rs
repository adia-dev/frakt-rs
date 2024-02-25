use serde::{Deserialize, Serialize};

use super::{rendering_data::RenderingData, server_dto::ServerDto};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PortalDto {
    RenderindData(RenderingData),
    Server(ServerDto),
}
