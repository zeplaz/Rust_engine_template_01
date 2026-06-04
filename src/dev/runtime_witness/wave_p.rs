//! Wave P preview witness — `debug_runs/wave_p_live.json` (DEV-CONTAIN-004).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::{
    build_d02_layout_witness, build_d04_layout_witness, build_d07_layout_witness,
    build_wave_p_witness_payload, gather_wave_p_readiness, PreviewLayers,
    PreviewPathAuthority, WAVE_P_LIVE_JSON,
};

use super::io::write_enveloped_witness;

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
pub fn commit_wave_p_witness(body: serde_json::Value) -> bool {
    write_enveloped_witness(
        "WAVE_P_PREVIEW",
        "wave_p_witness",
        WAVE_P_LIVE_JSON,
        body,
    )
}

pub fn write_wave_p_witness_system(
    base: Res<State<BaseState>>,
    app: Res<State<crate::engine::AppState>>,
    mut state: ResMut<WavePLiveProofState>,
    authority: Res<PreviewPathAuthority>,
    graph: Option<Res<crate::gui::editor::world_preview::CompositePreviewGraphResource>>,
    preview_ui: Res<crate::gui::editor::world_preview::WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
) {
    let chrome_active = crate::gui::editor::world_preview::world_gen_chrome_may_render(
        app,
        preview_ui.as_ref(),
        world_gen.as_ref(),
    );
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
    let unified = crate::gui::editor::world_preview::world_preview_unified_workspace(preview_ui.as_ref());
    let sheet_w = if preview_ui.last_window_rect.is_some() {
        crate::gui::editor::world_preview::d04_sheet_width_px(
            preview_ui
                .last_window_rect
                .map(|r| r.width())
                .unwrap_or(960.0),
        )
    } else {
        crate::gui::editor::world_preview::d04_sheet_width_px(960.0)
    };
    let d04 = build_d04_layout_witness(unified, world_gen.generator_sheet_open, sheet_w);
    let d07 = build_d07_layout_witness(
        preview_ui.window_open && preview_ui.d07_corner_inset_active,
        if preview_ui.d07_inset_side_px > 0.0 {
            preview_ui.d07_inset_side_px
        } else {
            crate::gui::editor::world_preview::d07_inset_side_px()
        },
        true,
    );
    let (workspace_w, workspace_h) = preview_ui
        .last_window_rect
        .map(|r| (r.width(), r.height()))
        .unwrap_or((
            crate::gui::editor::world_preview::D02_HD_BASELINE_W,
            crate::gui::editor::world_preview::D02_HD_BASELINE_H,
        ));
    let d02 = build_d02_layout_witness(
        workspace_w,
        workspace_h,
        180.0_f32.min(crate::gui::editor::world_preview::d02_sidebar_max_width_px(
            workspace_w,
        )),
        false,
        0.0,
    );
    let body = build_wave_p_witness_payload(&report, Some(d04), Some(d07), Some(d02));
    if commit_wave_p_witness(body) {
        state.written = true;
    }
}
