//! Minimap compositor witness collectors — domain rollups (no disk I/O; writer in `runtime_witness/minimap.rs`).

use bevy::prelude::*;

use crate::gui::{MinimapPresentationSource, MinimapShellState};

use super::pass::{
    minimap_gpu_compositor_env_enabled, MinimapCompositePath, MinimapCompositorState,
};
use super::diagnostics::{diagnostics_json_snapshot, MinimapGpuCompositorDiagnostics};
use super::render_target::MinimapRenderTargetRegistry;

/// UI-P3-001 acceptance rollup (witness A2–A5 + default GPU path).
#[must_use]
pub fn ui_p3_001_minimap_acceptance_green(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
) -> bool {
    let composite_ok = registry.committed_image != Handle::default() && compositor.stamp > 0;
    minimap_gpu_compositor_env_enabled()
        && shell.presentation_source == MinimapPresentationSource::SharedRenderTargetImage
        && composite_ok
        && !compositor.dual_minimap_present
        && compositor.composite_path == MinimapCompositePath::GpuCompute
        && compositor.extent_match_px <= 1.0
}

/// **UI-OH-M3-001** — M2 construction + ecology (**UI-P3-M3-001**); not design M3 (**UI-P3-M4-001**).
#[must_use]
pub fn ui_oh_m3_001_green(compositor: &MinimapCompositorState) -> bool {
    ui_p3_m3_minimap_acceptance_green(compositor)
}

/// **UI-OH-M2-001** — M2 logistics + construction compositor channels (overhaul alias **UI-P3-M2-001** + construction slice).
#[must_use]
pub fn ui_oh_m2_001_green(compositor: &MinimapCompositorState) -> bool {
    ui_w3_m2_001_green(compositor)
}

/// **UI-W3-M3-001** — M2 construction + ecology channels (coder-A alias; not design M4 fog/EW).
#[must_use]
pub fn ui_w3_m3_001_green(compositor: &MinimapCompositorState) -> bool {
    ui_oh_m3_001_green(compositor)
}

/// **UI-W3-M3-001** — compositor M3 channels + GPU minimap acceptance (**UI-P3-001**).
#[must_use]
pub fn ui_w3_m3_001_operational_green(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
) -> bool {
    ui_w3_m3_001_green(compositor)
        && ui_p3_001_minimap_acceptance_green(compositor, registry, shell)
}

/// **UI-W3-M3-001** — Track C: minimap operational + Stage 7 overlay readers (**S7B-M3-001**).
#[must_use]
pub fn ui_w3_m3_001_stage7_operational_green(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
    stage7_overlay_reads_green: bool,
) -> bool {
    ui_w3_m3_001_operational_green(compositor, registry, shell) && stage7_overlay_reads_green
}

/// Headless M3 witness fixture — logistics + construction + ecology + fog/EW + units.
#[must_use]
pub fn fixture_ui_w3_m3_001_compositor(tray: &crate::gui::hud::HudOverlayTrayState) -> MinimapCompositorState {
    fixture_ui_oh_m2_001_compositor(tray)
}

/// UI-P3-M2-TRAY-OPT — tray mask fields match compositor uniform bits.
#[must_use]
pub fn ui_w3_m2_001_green(compositor: &MinimapCompositorState) -> bool {
    compositor.logistics_heat_enabled
        && compositor.logistics_rows > 0
        && compositor.construction_heat_enabled
        && compositor.construction_rows > 0
}

/// Headless witness tray — full M2/M3 mask (not operator Simulation defaults).
#[must_use]
pub fn witness_harness_tray() -> crate::gui::hud::HudOverlayTrayState {
    let mut tray = crate::gui::hud::HudOverlayTrayState::default();
    tray.set_minimap_overlay_mask(crate::gui::minimap_overlay_witness_harness());
    tray
}

/// Headless M2 witness fixture — logistics + construction rows for lib proof refresh.
#[must_use]
pub fn fixture_ui_oh_m2_001_compositor(tray: &crate::gui::hud::HudOverlayTrayState) -> MinimapCompositorState {
    MinimapCompositorState {
        stamp: 4,
        compositor_revision: 2,
        dual_minimap_present: false,
        extent_match_px: 0.0,
        logistics_rows: 2,
        construction_rows: 18,
        ecology_rows: 100,
        fow_rows: 16,
        ew_rows: 12,
        fire_heat_enabled: tray.fire_heat,
        logistics_heat_enabled: tray.logistics_heat,
        construction_heat_enabled: tray.construction_heat,
        ecology_heat_enabled: tray.ecology_heat,
        fow_heat_enabled: true,
        ew_heat_enabled: true,
        units_heat_enabled: true,
        unit_marker_rows: 6,
        replay_scrub_enabled: true,
        composite_path: MinimapCompositePath::GpuCompute,
        ..Default::default()
    }
}

/// UI-P3-M2-TRAY-OPT — tray mask fields match compositor uniform bits.
#[must_use]
pub fn ui_p3_m2_tray_opt_green(
    compositor: &MinimapCompositorState,
    tray: &crate::gui::hud::HudOverlayTrayState,
) -> bool {
    compositor.fire_heat_enabled == tray.fire_heat
        && compositor.logistics_heat_enabled == tray.logistics_heat
        && compositor.construction_heat_enabled == tray.construction_heat
        && compositor.ecology_heat_enabled == tray.ecology_heat
}

/// **UI-P3-M4-001** — design **M3** fog-of-war + EW (not **UI-P3-M3-001** M2 construction/ecology).
#[must_use]
pub fn ui_p3_m4_minimap_acceptance_green(compositor: &MinimapCompositorState) -> bool {
    compositor.fow_heat_enabled
        && compositor.ew_heat_enabled
        && compositor.fow_rows > 0
        && compositor.ew_rows > 0
}

/// **UI-P3-M3-001** — design **M2** construction + ecology channels (not design M3 fog/EW).
/// See [`ui_phase3_minimap_track_naming_v1.md`](../../../prompts/guides/ui/ui_phase3_minimap_track_naming_v1.md).
#[must_use]
pub fn ui_p3_m3_minimap_acceptance_green(compositor: &MinimapCompositorState) -> bool {
    compositor.construction_heat_enabled
        && compositor.ecology_heat_enabled
        && (compositor.construction_rows > 0 || compositor.ecology_rows > 0)
}

/// **UI-P3-M3-UNITS-001** — unit aggregation markers on minimap EW channel.
#[must_use]
pub fn ui_p3_m3_units_001_green(compositor: &MinimapCompositorState) -> bool {
    compositor.units_heat_enabled && compositor.unit_marker_rows > 0
}

/// **UI-P3-M3-REPLAY-001** — replay scrub tick when timeline has depth.
#[must_use]
pub fn ui_p3_m3_replay_001_green(compositor: &MinimapCompositorState) -> bool {
    compositor.replay_scrub_enabled
}

/// **UI-P3-M2-CODER-A** — M2 strategic overlays per [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](../../../prompts/guides/ui/ui_phase3_minimap_m2_impl_full_plan_v1.md) and [`minimap_d_m2_signoff_v1.md`](../../dev/minimap_d_m2_signoff_v1.md).
#[must_use]
pub fn ui_p3_m2_minimap_acceptance_green(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
    tray: Option<&crate::gui::hud::HudOverlayTrayState>,
) -> bool {
    ui_p3_001_minimap_acceptance_green(compositor, registry, shell)
        && compositor.logistics_heat_enabled
        && compositor.logistics_rows > 0
        && ui_p3_m3_minimap_acceptance_green(compositor)
        && tray.is_none_or(|t| ui_p3_m2_tray_opt_green(compositor, t))
}

#[must_use]
fn ui_m2_logistics_construction_gate_json(
    gate: &str,
    compositor: &MinimapCompositorState,
) -> serde_json::Value {
    serde_json::json!({
        "gate": gate,
        "green": ui_w3_m2_001_green(compositor),
        "logistics_rows": compositor.logistics_rows,
        "construction_rows": compositor.construction_rows,
        "logistics_heat_enabled": compositor.logistics_heat_enabled,
        "construction_heat_enabled": compositor.construction_heat_enabled,
    })
}

#[must_use]
pub fn build_minimap_compositor_proof_payload(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
    overlay_revision: u64,
    ui_stress_wrote_sim: bool,
    diagnostics: &MinimapGpuCompositorDiagnostics,
) -> serde_json::Value {
    let presentation_source = match shell.presentation_source {
        MinimapPresentationSource::SharedCpuRaster => "SharedCpuRaster",
        MinimapPresentationSource::SharedRenderTargetImage => "SharedRenderTargetImage",
    };
    let mut body = serde_json::json!({
        "composite_ok": registry.committed_image != Handle::default() && compositor.stamp > 0,
        "stamp": compositor.stamp,
        "extent": {
            "x": registry.committed_size.x,
            "y": registry.committed_size.y,
        },
        "compositor_revision": compositor.compositor_revision,
        "presentation_source": presentation_source,
        "dual_minimap_present": compositor.dual_minimap_present,
        "extent_match_px": compositor.extent_match_px,
        "overlay_revision": overlay_revision,
        "gpu_compositor_env": minimap_gpu_compositor_env_enabled(),
        "rt_bound": registry.committed_image != Handle::default(),
        "ui_stress_wrote_sim": ui_stress_wrote_sim,
        "composite_path": match compositor.composite_path {
            super::pass::MinimapCompositePath::GpuCompute => "GpuCompute",
            super::pass::MinimapCompositePath::CpuBridge => "CpuBridge",
        },
        "logistics_rows": compositor.logistics_rows,
        "construction_rows": compositor.construction_rows,
        "ecology_rows": compositor.ecology_rows,
        "fire_heat_enabled": compositor.fire_heat_enabled,
        "logistics_heat_enabled": compositor.logistics_heat_enabled,
        "construction_heat_enabled": compositor.construction_heat_enabled,
        "ecology_heat_enabled": compositor.ecology_heat_enabled,
        "fow_enabled": compositor.fow_heat_enabled,
        "ew_overlay_enabled": compositor.ew_heat_enabled,
        "fow_rows": compositor.fow_rows,
        "ew_rows": compositor.ew_rows,
        "units_heat_enabled": compositor.units_heat_enabled,
        "unit_marker_rows": compositor.unit_marker_rows,
        "replay_scrub_enabled": compositor.replay_scrub_enabled,
        "veg_burn_rows": compositor.veg_burn_rows,
        "burn_overrides_topology": compositor.burn_overrides_topology,
        "veg_extract_revision": compositor.veg_extract_revision,
        "veg_minimap_burn_merge_green": compositor.veg_burn_rows >= 1
            && compositor.burn_overrides_topology,
        "ui_p3_m3_units_001_green": ui_p3_m3_units_001_green(compositor),
        "ui_p3_m3_replay_001_green": ui_p3_m3_replay_001_green(compositor),
        "gpu_budget": diagnostics_json_snapshot(diagnostics),
        "ui_p3_001_green": ui_p3_001_minimap_acceptance_green(compositor, registry, shell),
        "ui_p3_m4_green": ui_p3_m4_minimap_acceptance_green(compositor),
        "ui_p3_m3_green": ui_p3_m3_minimap_acceptance_green(compositor),
        "ui_p3_m2_green": ui_p3_m2_minimap_acceptance_green(compositor, registry, shell, None),
        "ui_oh_m3_001": {
            "gate": "UI-OH-M3-001",
            "green": ui_oh_m3_001_green(compositor),
            "construction_rows": compositor.construction_rows,
            "ecology_rows": compositor.ecology_rows,
            "construction_heat_enabled": compositor.construction_heat_enabled,
            "ecology_heat_enabled": compositor.ecology_heat_enabled,
        },
    });
    body["ui_oh_m2_001"] =
        ui_m2_logistics_construction_gate_json("UI-OH-M2-001", compositor);
    body["ui_w3_m2_001"] =
        ui_m2_logistics_construction_gate_json("UI-W3-M2-001", compositor);
    body["ui_w3_m3_001"] = serde_json::json!({
        "gate": "UI-W3-M3-001",
        "green": ui_w3_m3_001_green(compositor),
        "operational_green": ui_w3_m3_001_operational_green(compositor, registry, shell),
        "stage7_operational_green": ui_w3_m3_001_stage7_operational_green(
            compositor,
            registry,
            shell,
            crate::dev::stage7_behavioral_witness::stage7_behavioral_live_s7b_m3_green(),
        ),
        "construction_rows": compositor.construction_rows,
        "ecology_rows": compositor.ecology_rows,
        "construction_heat_enabled": compositor.construction_heat_enabled,
        "ecology_heat_enabled": compositor.ecology_heat_enabled,
    });
    body["perf_vis_p1b_gpu_default_001"] = super::pass::perf_vis_p1b_witness_json(
        shell,
        registry,
        compositor,
    );
    let defaults = crate::gui::simulation_minimap_overlay_defaults();
    body["minimap_eco_fire_tray_001"] = serde_json::json!({
        "gate": "MINIMAP-ECO-FIRE-TRAY-001",
        "green": crate::gui::hud::dock_shell::minimap_eco_fire_tray_defaults_green(),
        "fire_heat_default_off": !defaults.fire_heat,
        "ecology_burn_scar_default_on": defaults.ecology_heat,
    });
    body
}

#[must_use]
pub fn build_minimap_compositor_proof_payload_with_tray(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
    overlay_revision: u64,
    ui_stress_wrote_sim: bool,
    diagnostics: &MinimapGpuCompositorDiagnostics,
    tray: Option<&crate::gui::hud::HudOverlayTrayState>,
) -> serde_json::Value {
    let mut body = build_minimap_compositor_proof_payload(
        compositor,
        registry,
        shell,
        overlay_revision,
        ui_stress_wrote_sim,
        diagnostics,
    );
    body["ui_p3_m2_green"] = serde_json::json!(ui_p3_m2_minimap_acceptance_green(
        compositor,
        registry,
        shell,
        tray,
    ));
    if let Some(tray) = tray {
        body["ui_p3_m2_tray_opt_green"] = serde_json::json!(ui_p3_m2_tray_opt_green(compositor, tray));
        body["overlay_tray_minimap_mask"] = serde_json::json!({
            "fire_heat": tray.fire_heat,
            "logistics_heat": tray.logistics_heat,
            "construction_heat": tray.construction_heat,
            "ecology_heat": tray.ecology_heat,
        });
    }
    body
}
