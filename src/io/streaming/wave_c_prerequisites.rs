//! Entry gates before Wave C streaming work (after Wave S + P).

use crate::gui::editor::world_preview::{
    gather_wave_p_readiness, wave_p_readiness_passes, PreviewLayers, PreviewPathAuthority,
};
use crate::io::save::{SAVE_WORLD_MANIFEST_SCHEMA_VERSION, SAVED_CHUNK_BODY_SCHEMA_VERSION};

/// Closed when `product_shell.ron` hydrate + `wave_c_live.json` tile apply witness ship (post–Stage 6).
pub const WAVE_C_OPEN_BACKLOG_ITEMS: &[&str] = &[];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaveCPrerequisitesReport {
    pub wave_s_manifest_schema_version: u32,
    pub wave_s_chunk_schema_version: u32,
    pub wave_p_readiness_ok: bool,
    pub open_backlog_items: u32,
}

#[must_use]
pub fn gather_wave_c_prerequisites(authority: &PreviewPathAuthority) -> WaveCPrerequisitesReport {
    let wave_p = gather_wave_p_readiness(PreviewLayers::BIOME, authority);
    WaveCPrerequisitesReport {
        wave_s_manifest_schema_version: SAVE_WORLD_MANIFEST_SCHEMA_VERSION,
        wave_s_chunk_schema_version: SAVED_CHUNK_BODY_SCHEMA_VERSION,
        wave_p_readiness_ok: wave_p_readiness_passes(&wave_p),
        open_backlog_items: WAVE_C_OPEN_BACKLOG_ITEMS.len() as u32,
    }
}

#[must_use]
pub fn wave_c_prerequisites_passes(report: &WaveCPrerequisitesReport) -> bool {
    report.wave_s_manifest_schema_version > 0
        && report.wave_s_chunk_schema_version > 0
        && report.wave_p_readiness_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_c_prerequisites_pass_with_default_preview_authority() {
        let report = gather_wave_c_prerequisites(&PreviewPathAuthority::default());
        assert!(wave_c_prerequisites_passes(&report));
    }
}
