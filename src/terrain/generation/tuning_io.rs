//! JSON overlay for designer-tunable `noise_sampling` + `biome_tuning` (optional file on disk).
//! Full path default: `assets/config/world_gen_tuning.json`

use serde::{Deserialize, Serialize};

use crate::terrain::biome::BiomeTuning;

use super::terrain_noise::NoiseSamplingTuning;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorldGenTuningOverlay {
    pub noise_sampling: Option<NoiseSamplingTuning>,
    pub biome_tuning: Option<BiomeTuning>,
}

pub fn load_overlay(path: &str) -> std::io::Result<Option<WorldGenTuningOverlay>> {
    if !std::path::Path::new(path).exists() {
        return Ok(None);
    }
    let s = std::fs::read_to_string(path)?;
    let o: WorldGenTuningOverlay = serde_json::from_str(&s).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("JSON: {e}"))
    })?;
    Ok(Some(o))
}
