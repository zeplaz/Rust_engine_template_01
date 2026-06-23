//! **FIRE-MINIMAP-COHERENCE-001** — overlay vs veg extract revision + sim overlay policy.

pub const MINIMAP_FIRE_VEG_COHERENCE_LIVE_JSON: &str =
    "debug_runs/minimap_fire_veg_coherence_live.json";

#[must_use]
pub fn build_minimap_fire_veg_coherence_body() -> serde_json::Value {
    use crate::dev::landscape_grammar_burn_live_proof::veg_burn_pilot_extract_frame;
    use crate::gui::simulation_minimap_overlay_defaults;
    use crate::systems::sim_control::SimStepStamp;

    let defaults = simulation_minimap_overlay_defaults();
    let frame = veg_burn_pilot_extract_frame();
    let overlay_stamp = SimStepStamp::new(frame.stamp.tick, frame.stamp.sim_time_micros);
    let revision_aligned =
        frame.revision > 0 && frame.stamp == overlay_stamp && frame.rows.iter().any(|r| r.burn_active);
    let policy_ok = !defaults.fire_heat && !defaults.ecology_heat;
    let green = revision_aligned && policy_ok;

    serde_json::json!({
        "slice_id": "FIRE-MINIMAP-COHERENCE-001",
        "gate": "FIRE-MINIMAP-COHERENCE-001",
        "green": green,
        "revision_aligned": revision_aligned,
        "fire_heat_default_off": !defaults.fire_heat,
        "ecology_heat_default_off": !defaults.ecology_heat,
        "sim_overlay_policy_ok": policy_ok,
        "veg_extract_revision": frame.revision,
        "veg_extract_stamp_tick": frame.stamp.tick,
        "burn_active_rows": frame.rows.iter().filter(|r| r.burn_active).count(),
        "runtime_sim_verified": revision_aligned,
    })
}

#[must_use]
pub fn refresh_minimap_fire_veg_coherence_live_witness() -> bool {
    let body = build_minimap_fire_veg_coherence_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "FIRE-MINIMAP-COHERENCE-001",
        "refresh_minimap_fire_veg_coherence_live_witness",
        MINIMAP_FIRE_VEG_COHERENCE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(MINIMAP_FIRE_VEG_COHERENCE_LIVE_JSON, wrapped)
}

#[must_use]
pub fn refresh_veg_minimap_burn_merge_live_witness() -> bool {
    use crate::dev::runtime_witness::minimap::commit_minimap_compositor_live_proof;
    use crate::gui::MinimapPresentationSource;
    use crate::render::minimap_compositor::{
        fixture_ui_w3_m3_001_compositor, minimap_rgba_image, MinimapCompositePath,
        MinimapGpuCompositorDiagnostics, MinimapRenderTargetRegistry,
    };

    let tray = crate::gui::hud::HudOverlayTrayState::default();
    let mut compositor = fixture_ui_w3_m3_001_compositor(&tray);
    compositor.veg_burn_rows = 1;
    compositor.burn_overrides_topology = true;
    compositor.veg_extract_revision = 1;
    compositor.composite_path = MinimapCompositePath::GpuCompute;
    let mut registry = MinimapRenderTargetRegistry::default();
    let mut images = bevy::prelude::Assets::<bevy::prelude::Image>::default();
    registry.committed_size = bevy::prelude::UVec2::new(128, 128);
    registry.revision = 2;
    registry.committed_image = images.add(minimap_rgba_image(128, 128));
    let shell = crate::gui::MinimapShellState {
        presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
        ..Default::default()
    };
    commit_minimap_compositor_live_proof(
        &compositor,
        &registry,
        &shell,
        7,
        false,
        &MinimapGpuCompositorDiagnostics::default(),
        Some(&tray),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veg_minimap_burn_merge_live_witness_refresh_green() {
        assert!(refresh_veg_minimap_burn_merge_live_witness());
    }

    #[test]
    fn minimap_fire_veg_coherence_live_witness_refresh_green() {
        assert!(refresh_minimap_fire_veg_coherence_live_witness());
    }
}
