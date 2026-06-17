//! CDR-A-VISUAL-SMOKE-ECO-001 — ecology raster heterogeneity smoke (lib harness path).

pub const VISUAL_SMOKE_LIVE_JSON: &str = "debug_runs/landscape_grammar_visual_smoke_live.json";

#[must_use]
pub fn landscape_visual_smoke_lib_green() -> bool {
    use crate::dev::landscape_grammar_sim_harness::{
        build_landscape_grammar_harness_app, count_topology_tint_visible_program_chunks,
        run_landscape_grammar_harness_ticks,
    };
    use crate::gui::editor::world_preview::count_distinct_topology_visible_rgba;
    use crate::systems::ecology::{evaluate_landscape_program, load_landscape_grammar_catalog,
        lg4_preview_operator_visible, LandscapeProgramOnChunk, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID,
        ChunkEcology, VegetationField};
    use crate::systems::weather::ChunkWeather;

    let mut app = build_landscape_grammar_harness_app();
    run_landscape_grammar_harness_ticks(&mut app);
    let tint_visible = count_topology_tint_visible_program_chunks(app.world_mut());
    let kind_slices: Vec<Vec<String>> = app
        .world_mut()
        .query::<&LandscapeProgramOnChunk>()
        .iter(app.world())
        .map(|p| p.evaluation.topology_kinds.clone())
        .collect();
    let preview_samples =
        crate::gui::editor::world_preview::preview_samples_from_topology_kinds(kind_slices);
    let pixel_visible = count_distinct_topology_visible_rgba(&preview_samples);
    let catalog = load_landscape_grammar_catalog();
    let Some(preset) = catalog.presets.get(LG1_PILOT_PRESET_ID) else {
        return false;
    };
    let eval = evaluate_landscape_program(
        preset,
        LG1_PILOT_CHUNK,
        &ChunkEcology::default(),
        &VegetationField::default(),
        &ChunkWeather::default(),
    );
    tint_visible >= 1
        && pixel_visible >= 3
        && lg4_preview_operator_visible(tint_visible, &eval)
}

#[must_use]
pub fn refresh_landscape_visual_smoke_live_witness() -> bool {
    let lib_green = landscape_visual_smoke_lib_green();
    let body = serde_json::json!({
        "slice_id": "CDR-A-VISUAL-SMOKE-ECO-001",
        "gate": "CDR-A-VISUAL-SMOKE-ECO-001",
        "green": lib_green,
        "lib_smoke_green": lib_green,
        "proof_grade": crate::dev::proof_grade::ProofGrade::HeadlessSim.as_str(),
        "pixel_heterogeneity_wired": lib_green,
        "visual_capture_required": true,
        "visual_capture_command": "cargo test -p proc_A_dine01 --test visual",
        "note": "Lib harness proves preview pixel heterogeneity; full visual capture is operator lane",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-A-VISUAL-SMOKE-ECO-001",
        "refresh_landscape_visual_smoke_live_witness",
        VISUAL_SMOKE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VISUAL_SMOKE_LIVE_JSON, wrapped) && lib_green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_visual_smoke_live_witness_green() {
        assert!(refresh_landscape_visual_smoke_live_witness());
    }
}
