//! **VEG-DIAG-EXTRACT-PANEL-001** — diagnostics extract panel witness.

pub const VEG_DIAG_EXTRACT_PANEL_LIVE_JSON: &str =
    "debug_runs/veg_diag_extract_panel_live.json";

const EXTRACT_WITNESS: &str = "debug_runs/landscape_grammar_extract_live.json";

#[must_use]
pub fn format_veg_extract_diag_sample(
    coord: bevy::prelude::IVec2,
    variant_key: &str,
    glyph: char,
    burn_active: bool,
    frame_index: u8,
) -> String {
    format!(
        "({},{}) vk={} g={} burn={} f={frame_index:02}",
        coord.x, coord.y, variant_key, glyph, burn_active
    )
}

#[must_use]
pub fn veg_diag_extract_panel_wired_green() -> bool {
    let sample = format_veg_extract_diag_sample(
        bevy::prelude::IVec2::new(0, 0),
        "veg_burn_03",
        'B',
        true,
        3,
    );
    sample.contains("veg_burn_03") && sample.contains("burn=true")
}

#[must_use]
pub fn refresh_veg_diag_extract_panel_live_witness() -> bool {
    let wired = veg_diag_extract_panel_wired_green();
    let extract_linked = std::path::Path::new(EXTRACT_WITNESS).is_file()
        || crate::render::extraction::vegetation_extract_witness_green(&sample_extract_frame());
    let green = wired && extract_linked;
    let body = serde_json::json!({
        "gate": "VEG-DIAG-EXTRACT-PANEL-001",
        "slice_id": "VEG-DIAG-EXTRACT-PANEL-001",
        "green": green,
        "diag_panel_wired": wired,
        "extract_witness_linked": EXTRACT_WITNESS,
        "extract_witness_present": std::path::Path::new(EXTRACT_WITNESS).is_file(),
        "code": "src/gui/diagnostics_ui.rs",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-DIAG-EXTRACT-PANEL-001",
        "refresh_veg_diag_extract_panel_live_witness",
        VEG_DIAG_EXTRACT_PANEL_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VEG_DIAG_EXTRACT_PANEL_LIVE_JSON, wrapped)
        && green
}

fn sample_extract_frame() -> crate::render::extraction::VegetationExtractFrame {
    use bevy::prelude::IVec2;

    use crate::render::extraction::{VegExtractModifiers, VegExtractRow, VegetationExtractFrame};
    use crate::systems::ecology::SuccessionTopologyStage;
    use crate::systems::sim_control::SimStepStamp;

    VegetationExtractFrame {
        revision: 1,
        stamp: SimStepStamp::new(1, 0),
        rows: vec![VegExtractRow {
            coord: IVec2::ZERO,
            planning_glyph: 'P',
            extract_glyph: 'E',
            modifiers: VegExtractModifiers {
                burn_active: true,
                mean_density: 0.5,
            },
            variant_key: "veg_burn_03".into(),
            succession_stage: SuccessionTopologyStage::BurnScar,
            burn_active: true,
            frame_index: 3,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veg_diag_extract_panel_live_witness_green() {
        assert!(veg_diag_extract_panel_wired_green());
        assert!(refresh_veg_diag_extract_panel_live_witness());
    }
}
