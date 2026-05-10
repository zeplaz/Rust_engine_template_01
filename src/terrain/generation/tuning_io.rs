//! Optional overlay for designer-tunable `noise_sampling` + `biome_tuning`.
//!
//! **Format policy:** **RON** is the canonical on-disk format for the game (`world_gen_tuning.ron`).
//! **JSON** remains for the Python asset editor and hand-shared snippets (`world_gen_tuning.json`).
//! Default load order: **`.ron` if present**, else **`.json`**. Per-path load uses the file extension;
//! unknown extension tries **RON then JSON** (mirrors transport G4).
//!
//! Example files: `assets/config/world_gen_tuning.example.ron` (and `.example.json`).

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::terrain::biome::BiomeTuning;

use super::terrain_noise::NoiseSamplingTuning;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorldGenTuningOverlay {
    pub noise_sampling: Option<NoiseSamplingTuning>,
    pub biome_tuning: Option<BiomeTuning>,
}

fn overlay_from_json_str(s: &str) -> Result<WorldGenTuningOverlay, io::Error> {
    serde_json::from_str(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("JSON: {e}")))
}

fn overlay_from_ron_str(s: &str) -> Result<WorldGenTuningOverlay, io::Error> {
    ron::de::from_str(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("RON: {e}")))
}

/// Parse UTF-8 text using `path`'s extension: `.json` → JSON, `.ron` → RON; other / none → RON then JSON.
pub fn overlay_from_text_for_path(text: &str, path: &Path) -> io::Result<WorldGenTuningOverlay> {
    let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("json") => overlay_from_json_str(text),
        Some("ron") => overlay_from_ron_str(text),
        None | Some(_) => overlay_from_ron_str(text).or_else(|_| overlay_from_json_str(text)),
    }
}

/// Read a single file if it exists; dispatch format by extension (see [`overlay_from_text_for_path`]).
pub fn load_overlay_from_path(path: &Path) -> io::Result<Option<WorldGenTuningOverlay>> {
    if !path.exists() {
        return Ok(None);
    }
    let s = std::fs::read_to_string(path)?;
    overlay_from_text_for_path(&s, path).map(Some)
}

/// Back-compat: load from a path string (relative or absolute).
pub fn load_overlay(path: &str) -> io::Result<Option<WorldGenTuningOverlay>> {
    load_overlay_from_path(Path::new(path))
}

/// Prefer `ron_path` when that file exists; otherwise load `json_path` if it exists.
pub fn load_overlay_prefer_ron(
    ron_path: impl AsRef<Path>,
    json_path: impl AsRef<Path>,
) -> io::Result<Option<WorldGenTuningOverlay>> {
    let ron_path = ron_path.as_ref();
    let json_path = json_path.as_ref();
    if ron_path.exists() {
        load_overlay_from_path(ron_path)
    } else if json_path.exists() {
        load_overlay_from_path(json_path)
    } else {
        Ok(None)
    }
}

fn pretty_ron_config() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::new().depth_limit(8).indentor("    ".into())
}

/// Canonical save for in-game / editor — **RON** pretty.
pub fn save_overlay_ron(path: impl AsRef<Path>, overlay: &WorldGenTuningOverlay) -> io::Result<()> {
    let s = ron::ser::to_string_pretty(overlay, pretty_ron_config())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("RON: {e}")))?;
    if let Some(parent) = path.as_ref().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path.as_ref(), s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn assert_overlay_json_eq(a: &WorldGenTuningOverlay, b: &WorldGenTuningOverlay) {
        let ja = serde_json::to_value(a).expect("a to json");
        let jb = serde_json::to_value(b).expect("b to json");
        assert_eq!(ja, jb);
    }

    #[test]
    fn example_json_round_trips_through_ron() {
        let json = include_str!("../../../assets/config/world_gen_tuning.example.json");
        let o0: WorldGenTuningOverlay = serde_json::from_str(json).unwrap();
        let ron = ron::ser::to_string_pretty(&o0, pretty_ron_config()).unwrap();
        let o1: WorldGenTuningOverlay = ron::de::from_str(&ron).unwrap();
        assert_overlay_json_eq(&o0, &o1);
    }

    #[test]
    fn example_ron_matches_example_json_semantics() {
        let json = include_str!("../../../assets/config/world_gen_tuning.example.json");
        let ron = include_str!("../../../assets/config/world_gen_tuning.example.ron");
        let oj: WorldGenTuningOverlay = serde_json::from_str(json).unwrap();
        let oron: WorldGenTuningOverlay = ron::de::from_str(ron).unwrap();
        assert_overlay_json_eq(&oj, &oron);
    }

    #[test]
    fn from_path_respects_json_extension() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/world_gen_tuning.example.json");
        let o = load_overlay_from_path(&path).unwrap().expect("example json");
        assert!(o.noise_sampling.is_some());
    }

    #[test]
    fn from_path_respects_ron_extension() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/world_gen_tuning.example.ron");
        let o = load_overlay_from_path(&path).unwrap().expect("example ron");
        assert!(o.biome_tuning.is_some());
    }
}
