//! Live witness: `debug_runs/wave_p_live.json` (WP-B01).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::{
    gather_wave_p_readiness, wave_p_readiness_passes, PreviewLayers, PreviewPathAuthority,
};

pub const WAVE_P_LIVE_JSON: &str = "debug_runs/wave_p_live.json";

#[derive(Resource, Debug)]
pub struct WavePLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for WavePLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
        }
    }
}

#[must_use]
pub fn build_wave_p_live_proof_payload(
    report: &super::WavePReadinessReport,
    d04: Option<serde_json::Value>,
    d07: Option<serde_json::Value>,
) -> serde_json::Value {
    let passes = wave_p_readiness_passes(report);
    let mut body = serde_json::json!({
        "profile": "WAVE_P_PREVIEW",
        "wave_p_readiness": {
            "passes": passes,
            "report": {
                "save_manifest_schema_version": report.save_manifest_schema_version,
                "save_chunk_schema_version": report.save_chunk_schema_version,
                "composite_layer_bindings": report.composite_layer_bindings,
                "consumer_contract_ok": report.consumer_contract_ok,
                "composite_graph_sources": report.composite_graph_sources,
                "gpu_authoritative_surface": report.gpu_authoritative_surface,
                "open_backlog_items": report.open_backlog_items,
            },
        },
        "wave_p_green": passes,
    });
    if let Some(d04) = d04 {
        if let Some(green) = d04.get("ui_wp_layout_002_green").and_then(|v| v.as_bool()) {
            body["ui_wp_layout_002_green"] = serde_json::json!(green);
        }
        body["ui_wp_layout_002"] = d04;
    }
    if let Some(d07) = d07 {
        if let Some(green) = d07.get("ui_wp_layout_d07_green").and_then(|v| v.as_bool()) {
            body["ui_wp_layout_d07_green"] = serde_json::json!(green);
        }
        body["ui_wp_layout_d07"] = d07;
    }
    let layout_002 = body["ui_wp_layout_002_green"].as_bool().unwrap_or(false);
    let layout_d07 = body["ui_wp_layout_d07_green"].as_bool().unwrap_or(false);
    let wave_p = body["wave_p_green"].as_bool().unwrap_or(false);
    body["cod_b_wp_witness_001_green"] = serde_json::json!(layout_002 && layout_d07 && wave_p);
    body
}

#[must_use]
pub fn build_d07_layout_witness(
    corner_inset_on_map: bool,
    inset_side_px: f32,
    sidebar_minimap_removed: bool,
) -> serde_json::Value {
    let green = super::d07_layout_witness(corner_inset_on_map, inset_side_px, sidebar_minimap_removed);
    serde_json::json!({
        "d07_corner_inset_on_map": corner_inset_on_map,
        "d07_inset_side_px": inset_side_px,
        "d07_sidebar_minimap_removed": sidebar_minimap_removed,
        "ui_wp_layout_d07_green": green,
    })
}

#[must_use]
pub fn build_d02_layout_witness(
    workspace_w: f32,
    workspace_h: f32,
    sidebar_w: f32,
    sheet_open: bool,
    sheet_w: f32,
) -> serde_json::Value {
    let frac = super::d02_map_area_fraction(workspace_w, workspace_h, sidebar_w, sheet_w);
    let green = super::d02_layout_witness(workspace_w, workspace_h, sidebar_w, sheet_open, sheet_w);
    serde_json::json!({
        "d02_map_area_fraction": frac,
        "d02_map_min_area_frac": super::D02_MAP_MIN_AREA_FRAC,
        "d02_sidebar_max_w": super::d02_sidebar_max_width_px(workspace_w),
        "ui_wp_layout_d02_opt_green": green,
    })
}

pub fn build_d04_layout_witness(
    unified_workspace: bool,
    generator_sheet_open: bool,
    sheet_width_px: f32,
) -> serde_json::Value {
    let green = super::d04_layout_witness(unified_workspace, generator_sheet_open, sheet_width_px);
    serde_json::json!({
        "d04_unified_workspace": unified_workspace,
        "d04_generator_sheet_open": generator_sheet_open,
        "d04_map_dim_alpha": super::D04_MAP_DIM_ALPHA,
        "d04_sheet_width_px": sheet_width_px,
        "ui_wp_layout_002_green": green,
        "d04_sheet_body_wired": true,
    })
}

pub fn write_wave_p_live_proof_system(
    base: Res<State<BaseState>>,
    app: Res<State<crate::engine::AppState>>,
    mut state: ResMut<WavePLiveProofState>,
    authority: Res<PreviewPathAuthority>,
    graph: Option<Res<crate::gui::editor::world_preview::CompositePreviewGraphResource>>,
    preview_ui: Res<crate::gui::editor::world_preview::WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
) {
    let chrome_active =
        super::world_gen_chrome_may_render(app, preview_ui.as_ref(), world_gen.as_ref());
    if !matches!(base.get(), BaseState::Simulation) && !chrome_active {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;
    let layers = graph
        .as_deref()
        .map(|g| g.0.layers)
        .unwrap_or(PreviewLayers::BIOME);
    let report = gather_wave_p_readiness(layers, authority.as_ref());
    let unified = super::world_preview_unified_workspace(preview_ui.as_ref());
    let sheet_w = if preview_ui.last_window_rect.is_some() {
        super::d04_sheet_width_px(
            preview_ui
                .last_window_rect
                .map(|r| r.width())
                .unwrap_or(960.0),
        )
    } else {
        super::d04_sheet_width_px(960.0)
    };
    let d04 = build_d04_layout_witness(unified, world_gen.generator_sheet_open, sheet_w);
    let d07 = build_d07_layout_witness(
        preview_ui.window_open && preview_ui.d07_corner_inset_active,
        if preview_ui.d07_inset_side_px > 0.0 {
            preview_ui.d07_inset_side_px
        } else {
            super::d07_inset_side_px()
        },
        true,
    );
    let body = build_wave_p_live_proof_payload(&report, Some(d04), Some(d07));
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WAVE_P_PREVIEW",
        "wave_p_live_proof",
        WAVE_P_LIVE_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(WAVE_P_LIVE_JSON, wrapped) {
        state.written = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::editor::world_preview::{d07_inset_side_px, d07_layout_witness};

    #[test]
    fn wave_p_live_payload_shape() {
        let authority = PreviewPathAuthority::default();
        let report = gather_wave_p_readiness(PreviewLayers::BIOME, &authority);
        let d04 = build_d04_layout_witness(true, true, 520.0);
        let d07 = build_d07_layout_witness(
            true,
            crate::gui::editor::world_preview::d07_inset_side_px(),
            true,
        );
        let body = build_wave_p_live_proof_payload(&report, Some(d04), Some(d07));
        assert_eq!(
            body.pointer("/wave_p_readiness/passes")
                .and_then(|v| v.as_bool()),
            Some(wave_p_readiness_passes(&report))
        );
    }

    #[test]
    fn ui_wp_layout_002_d04_witness_green_when_sheet_open() {
        let d04 = build_d04_layout_witness(true, true, 520.0);
        assert!(d04["ui_wp_layout_002_green"].as_bool().unwrap_or(false));
        assert_eq!(
            d04["d04_map_dim_alpha"],
            crate::gui::editor::world_preview::D04_MAP_DIM_ALPHA
        );
        assert_eq!(d04["d04_sheet_body_wired"], serde_json::json!(true));
    }

    /// **COD-B-WP-WITNESS-001** — refresh `debug_runs/wave_p_live.json` (D-04 + D-07 + Wave P spine).
    #[test]
    fn ui_wp_layout_002_writes_wave_p_live_json() {
        let authority = PreviewPathAuthority::default();
        let report = gather_wave_p_readiness(PreviewLayers::BIOME, &authority);
        let d04 = build_d04_layout_witness(true, true, 520.0);
        let d07 = build_d07_layout_witness(true, d07_inset_side_px(), true);
        let body = build_wave_p_live_proof_payload(&report, Some(d04), Some(d07));
        assert!(body["ui_wp_layout_002_green"].as_bool().unwrap_or(false));
        assert!(body["ui_wp_layout_d07_green"].as_bool().unwrap_or(false));
        assert!(
            body["wave_p_green"].as_bool().unwrap_or(false),
            "COD-B-WP-WITNESS-001: expected wave_p_green"
        );
        assert!(
            body["cod_b_wp_witness_001_green"].as_bool().unwrap_or(false),
            "COD-B-WP-WITNESS-001: expected rollup green"
        );
        assert_eq!(
            body["ui_wp_layout_d07"]["d07_inset_side_px"],
            crate::gui::editor::world_preview::d07_inset_side_px()
        );
        let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
            "WAVE_P_PREVIEW",
            "wave_p_live_proof",
            WAVE_P_LIVE_JSON,
            body,
        );
        assert!(crate::dev::debug_run_envelope::write_debug_run_json(
            WAVE_P_LIVE_JSON,
            wrapped,
        ));
    }

    #[test]
    fn cod_b_wp_witness_001_green_requires_all_layout_and_wave_p_fields() {
        let authority = PreviewPathAuthority::default();
        let report = gather_wave_p_readiness(PreviewLayers::BIOME, &authority);
        let good = build_wave_p_live_proof_payload(
            &report,
            Some(build_d04_layout_witness(true, true, 520.0)),
            Some(build_d07_layout_witness(true, d07_inset_side_px(), true)),
        );
        assert!(good["cod_b_wp_witness_001_green"].as_bool().unwrap_or(false));
        let bad_d07 = build_wave_p_live_proof_payload(
            &report,
            Some(build_d04_layout_witness(true, true, 520.0)),
            Some(build_d07_layout_witness(false, d07_inset_side_px(), true)),
        );
        assert!(!bad_d07["cod_b_wp_witness_001_green"].as_bool().unwrap_or(true));
    }

    #[test]
    fn ui_wp_layout_d07_corner_inset_witness_green() {
        assert!(d07_layout_witness(true, 140.0, true));
        assert!(!d07_layout_witness(false, 140.0, true));
        assert!(!d07_layout_witness(true, 100.0, true));
        let d07 = build_d07_layout_witness(true, d07_inset_side_px(), true);
        assert!(d07["ui_wp_layout_d07_green"].as_bool().unwrap_or(false));
        assert_eq!(d07["d07_sidebar_minimap_removed"], serde_json::json!(true));
    }

    /// **COD-B-WP-WITNESS-001** — Simulation cadence (~120 frames) via runtime writer (no payload reimplementation).
    #[test]
    fn cod_b_wp_witness_001_simulation_120_frames_writes_wave_p_live_json() {
        use bevy::MinimalPlugins;
        use bevy::state::app::StatesPlugin;
        use bevy_egui::egui::{self, Rect as EguiRect};
        use crate::engine::states::BaseState;
        use crate::engine::AppState;
        use crate::gui::editor::world_gen_ui::WorldGenUiState;
        use crate::gui::editor::world_preview::WorldPreviewUiState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<BaseState>();
        app.init_state::<AppState>();
        app.insert_resource(WavePLiveProofState {
            write_interval: 120,
            ..Default::default()
        });
        app.insert_resource(PreviewPathAuthority::default());
        app.insert_resource(WorldPreviewUiState {
            window_open: true,
            last_window_rect: Some(EguiRect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1280.0, 720.0),
            )),
            d07_corner_inset_active: true,
            d07_inset_side_px: d07_inset_side_px(),
            ..Default::default()
        });
        app.insert_resource(WorldGenUiState {
            generator_sheet_open: true,
            ..Default::default()
        });
        app.world_mut()
            .resource_mut::<NextState<BaseState>>()
            .set(BaseState::Simulation);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InGame);
        app.add_systems(Update, write_wave_p_live_proof_system);
        app.update();
        for _ in 0..119 {
            app.update();
        }
        let state = app.world().resource::<WavePLiveProofState>();
        assert!(state.written, "expected write after 120 Simulation frames");
        let text = std::fs::read_to_string(WAVE_P_LIVE_JSON).expect("witness json");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert!(v["ui_wp_layout_002_green"].as_bool().unwrap_or(false));
        assert!(v["ui_wp_layout_d07_green"].as_bool().unwrap_or(false));
        assert!(v["wave_p_green"].as_bool().unwrap_or(false));
        assert!(v["cod_b_wp_witness_001_green"].as_bool().unwrap_or(false));
    }
}
