

use serde::{Deserialize};


#[derive(Debug, Deserialize)]
pub struct TextureInfo {
    pub path: String,
    pub tiles: u32,
    pub emission: f32,
}