use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::networking::result::NetworkingResult;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelIntensity {
    pub zn: f32,
    pub count: f32,
}

impl PixelIntensity {
    pub fn new(zn: f32, count: f32) -> Self {
        Self { zn, count }
    }

    pub async fn from_bytes(mut bytes: &[u8]) -> NetworkingResult<Self> {
        let zn = match bytes.read_f32().await {
            Ok(zn) => zn.clone(),
            Err(err) => return Err(err.into()),
        };

        let count = match bytes.read_f32().await {
            Ok(count) => count.clone(),
            Err(err) => return Err(err.into()),
        };

        Ok(Self { zn, count })
    }
}
