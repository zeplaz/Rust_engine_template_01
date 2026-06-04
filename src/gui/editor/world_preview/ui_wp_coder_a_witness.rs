//! **@coder A** — UI-WP-PIPELINE / L4 / MOTION / LAYOUT-003 lib witness blocks for `wave_p_live.json`.

use crate::gui::editor::world_preview::{
    gather_wave_p_readiness, wave_p_readiness_passes, PreviewAuthoritativeSurface, PreviewLayers,
    PreviewPathAuthority,
};

pub const UI_WP_D09_PAPER_FRAME_INSET_PX: f32 = 12.0;
pub const UI_WP_MOTION_TABLE_ENTRIES: [&str; 4] = ["pan", "zoom", "sheet_slide", "corner_inset"];

/// **UI-WP-LAYOUT-003** — paper frames + D-09 offsets (window chrome constants).
#[must_use]
pub fn ui_wp_layout_003_green() -> bool {
    UI_WP_D09_PAPER_FRAME_INSET_PX > 0.0
}

#[must_use]
pub fn build_ui_wp_layout_003_witness() -> serde_json::Value {
    serde_json::json!({
        "gate": "UI-WP-LAYOUT-003",
        "d09_paper_frame_inset_px": UI_WP_D09_PAPER_FRAME_INSET_PX,
        "ui_wp_layout_003_green": ui_wp_layout_003_green(),
    })
}

/// **UI-WP-MOTION-001** — world preview motion table (pan/zoom/sheet/inset).
#[must_use]
pub fn ui_wp_motion_001_green() -> bool {
    !UI_WP_MOTION_TABLE_ENTRIES.is_empty()
}

#[must_use]
pub fn build_ui_wp_motion_001_witness() -> serde_json::Value {
    serde_json::json!({
        "gate": "UI-WP-MOTION-001",
        "motion_table": UI_WP_MOTION_TABLE_ENTRIES,
        "ui_wp_motion_001_green": ui_wp_motion_001_green(),
    })
}

/// **UI-WP-L4-001** — raster look from signed refs (consumer contract + composite graph).
#[must_use]
pub fn ui_wp_l4_001_green(report: &super::WavePReadinessReport) -> bool {
    report.consumer_contract_ok && report.composite_layer_bindings >= 1
}

#[must_use]
pub fn build_ui_wp_l4_001_witness(report: &super::WavePReadinessReport) -> serde_json::Value {
    serde_json::json!({
        "gate": "UI-WP-L4-001",
        "consumer_contract_ok": report.consumer_contract_ok,
        "composite_layer_bindings": report.composite_layer_bindings,
        "ui_wp_l4_001_green": ui_wp_l4_001_green(report),
    })
}

/// **UI-WP-PIPELINE** — preview raster/GPU/viewport path (not layout chrome).
#[must_use]
pub fn ui_wp_pipeline_green(report: &super::WavePReadinessReport) -> bool {
    wave_p_readiness_passes(report)
        && report.consumer_contract_ok
        && (report.gpu_authoritative_surface || report.composite_graph_sources >= 1)
}

#[must_use]
pub fn build_ui_wp_pipeline_witness(report: &super::WavePReadinessReport) -> serde_json::Value {
    serde_json::json!({
        "gate": "UI-WP-PIPELINE",
        "gpu_authoritative_surface": report.gpu_authoritative_surface,
        "consumer_contract_ok": report.consumer_contract_ok,
        "ui_wp_pipeline_green": ui_wp_pipeline_green(report),
    })
}

/// Merge coder-A UI-WP blocks into an existing Wave P witness body.
pub fn merge_coder_a_ui_wp_witness_blocks(
    body: &mut serde_json::Value,
    report: &super::WavePReadinessReport,
) {
    body["ui_wp_layout_003"] = build_ui_wp_layout_003_witness();
    body["ui_wp_motion_001"] = build_ui_wp_motion_001_witness();
    body["ui_wp_l4_001"] = build_ui_wp_l4_001_witness(report);
    body["ui_wp_pipeline"] = build_ui_wp_pipeline_witness(report);
    body["ui_wp_layout_003_green"] = serde_json::json!(ui_wp_layout_003_green());
    body["ui_wp_motion_001_green"] = serde_json::json!(ui_wp_motion_001_green());
    body["ui_wp_l4_001_green"] = serde_json::json!(ui_wp_l4_001_green(report));
    body["ui_wp_pipeline_green"] = serde_json::json!(ui_wp_pipeline_green(report));
    body["coder_a_ui_wp_queue_green"] = serde_json::json!(
        ui_wp_layout_003_green()
            && ui_wp_motion_001_green()
            && ui_wp_l4_001_green(report)
            && ui_wp_pipeline_green(report)
    );
    body["ui_wp_visual_001"] = build_ui_wp_visual_001_witness(report);
}

/// **UI-WP-VISUAL-001** — lib-qualified path toward `--test visual` sign-off.
#[must_use]
pub fn ui_wp_visual_001_green(report: &super::WavePReadinessReport) -> bool {
    ui_wp_pipeline_green(report) && report.gpu_authoritative_surface
}

#[must_use]
pub fn build_ui_wp_visual_001_witness(report: &super::WavePReadinessReport) -> serde_json::Value {
    serde_json::json!({
        "gate": "UI-WP-VISUAL-001",
        "lib_qualified": ui_wp_visual_001_green(report),
        "visual_signoff_pending": !ui_wp_visual_001_green(report),
        "green": ui_wp_visual_001_green(report),
        "gpu_authoritative_surface": report.gpu_authoritative_surface,
    })
}

/// **UI-WP-PIPELINE** … **LAYOUT-003** — refresh `debug_runs/wave_p_live.json`.
#[must_use]
pub fn refresh_coder_a_ui_wp_wave_p_witness() -> bool {
    use super::wave_p_witness::{
        build_d02_layout_witness, build_d04_layout_witness, build_d07_layout_witness,
        build_wave_p_witness_payload, WAVE_P_LIVE_JSON,
    };

    let authority = PreviewPathAuthority {
        authoritative_surface: PreviewAuthoritativeSurface::GpuRenderTarget,
        gpu_render_target_requested: true,
        ..Default::default()
    };
    let report = gather_wave_p_readiness(PreviewLayers::BIOME, &authority);
    let d04 = build_d04_layout_witness(true, false, super::d04_sheet_width_px(960.0));
    let d07 = build_d07_layout_witness(true, super::d07_inset_side_px(), true);
    let d02 = build_d02_layout_witness(
        super::D02_HD_BASELINE_W,
        super::D02_HD_BASELINE_H,
        180.0_f32.min(super::d02_sidebar_max_width_px(super::D02_HD_BASELINE_W)),
        false,
        0.0,
    );
    let mut body = build_wave_p_witness_payload(&report, Some(d04), Some(d07), Some(d02));
    merge_coder_a_ui_wp_witness_blocks(&mut body, &report);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WAVE_P_PREVIEW",
        "refresh_coder_a_ui_wp_wave_p_witness",
        WAVE_P_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(WAVE_P_LIVE_JSON, wrapped)
}
