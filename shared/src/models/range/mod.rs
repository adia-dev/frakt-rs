use super::point::Point;

use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub min: Point,
    pub max: Point,
}
