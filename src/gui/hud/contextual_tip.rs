//! L0 immediate context — mode, selection, affordances without opening a panel.

use bevy::prelude::*;

use crate::construction::{
    BuildGhostState, BuildPlacementMode, BuildPlacementPreview, BuildStripState, ToolContext,
};
use crate::gui::hud::strategic_preview::format_projected_commit_effects;
use crate::gui::hud::tool_help;
use crate::gui::hud::validation_feedback::{self, ValidationSeverity};
use crate::gui::input_bindings::InputBindings;
use crate::strategic::StrategicOverlayDisplayPolicy;

/// Bevy UI text node for the developmental context row (under the operations strip).
#[derive(Component)]
pub struct DevelopmentalContextStripLine;

#[inline]
fn tool_heading(ctx: ToolContext) -> String {
    let strip = ctx.label();
    let archetype = validation_feedback::site_archetype_operational_name(ctx.site_archetype());
    format!("BUILD · {} · {}", strip, archetype)
}

/// Single-line summary: mode, ghost tile, validation gist, keys.
pub fn format_developmental_context_line(
    strip: &BuildStripState,
    ghost: &BuildGhostState,
    preview: &BuildPlacementPreview,
    bindings: &InputBindings,
    policy_routing: bool,
    policy_ew: bool,
    site_stub_overlay_on: bool,
) -> String {
    let cycle = tool_help::format_build_cycle_key(bindings);
    let rot = tool_help::format_map_rotate_keys(bindings);
    let overlay = format!(
        "overlays congestion {} / EW {}",
        if policy_routing { "on" } else { "off" },
        if policy_ew { "on" } else { "off" },
    );
    let site_suffix = if site_stub_overlay_on {
        " · site overlay on"
    } else {
        ""
    };

    if strip.active == ToolContext::None {
        return format!(
            "CONTEXT — Planning idle · press [{cycle}] to choose build category · map rotate [{rot}] · {overlay}",
        );
    }

    let heading = tool_heading(strip.active);
    let is_building_tool = matches!(
        strip.active,
        ToolContext::Industry | ToolContext::Civil | ToolContext::Military
    );

    if is_building_tool {
        let archetype = validation_feedback::site_archetype_operational_name(strip.active.site_archetype());
        let in_adjust = ghost.placement_mode == BuildPlacementMode::Adjust && ghost.origin.is_some();
        if in_adjust {
            let (x, z) = ghost
                .origin
                .map(|t| (t.x, t.z))
                .unwrap_or((0, 0));
            if preview.report.allows_commit {
                return format!(
                    "CONTEXT — BUILD · {archetype} · locked {x},{z} · Ctrl rotate · Shift scale · click to place · Esc cancel{site_suffix}",
                );
            }
            let reason = validation_feedback::primary_validation_message(&preview.report)
                .unwrap_or_else(|| "Placement blocked.".into());
            return format!(
                "CONTEXT — BUILD · {archetype} · locked {x},{z} · blocked: {reason} · Esc cancel{site_suffix}",
            );
        }
        return format!(
            "CONTEXT — BUILD · {} · {} · click map to lock · [{cycle}] category{site_suffix}",
            strip.active.label(),
            archetype,
        );
    }

    let site = ghost
        .origin
        .map(|t| format!("selection {},{}", t.x, t.z))
        .unwrap_or_else(|| "selection — click map".into());

    let projection_note = if preview.report.allows_commit {
        format_projected_commit_effects(&preview.report)
            .map(|s| format!(" · {}", s))
            .unwrap_or_default()
    } else {
        String::new()
    };

    let diags = validation_feedback::diagnostics_from_site_validation(&preview.report);
    let warn_note = diags
        .iter()
        .find(|d| d.severity == ValidationSeverity::Warning)
        .map(|d| format!(" · warning: {}", d.message))
        .unwrap_or_default();

    let commit = tool_help::format_build_commit_key(bindings);
    if preview.report.allows_commit {
        format!(
            "CONTEXT — {heading} · {site} · ok to commit [{commit}] · right-click clears ghost · cycle [{cycle}] · rotate map [{rot}] · {overlay}{warn_note}{projection_note}",
        )
    } else {
        let reason = validation_feedback::primary_validation_message(&preview.report)
            .unwrap_or_else(|| "Placement blocked.".into());
        format!(
            "CONTEXT — {heading} · {site} · blocked: {reason} · [{cycle}] change mode · right-click cancel · {overlay}",
        )
    }
}

pub fn update_developmental_context_strip_system(
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    bindings: Res<InputBindings>,
    policy: Res<StrategicOverlayDisplayPolicy>,
    site_stub: Option<Res<crate::construction::SiteStubOverlayState>>,
    tool: Option<Res<crate::construction::ActiveBuildTool>>,
    registry: Option<Res<crate::construction::BuildingDefinitionRegistry>>,
    mut q: Query<&mut Text, With<DevelopmentalContextStripLine>>,
) {
    let site_stub_overlay_on = site_stub
        .as_deref()
        .map(|s| s.preset_id.is_some())
        .unwrap_or(false);
    let grammar = tool
        .as_deref()
        .zip(registry.as_deref())
        .and_then(|(t, r)| super::grammar_read_hud::resolve_grammar_read_context(t, r, &ghost));
    let line = if let Some(ref ctx) = grammar {
        super::grammar_read_hud::format_developmental_context_line_with_grammar(
            &strip,
            &ghost,
            &preview,
            &bindings,
            policy.apply_routing_congestion,
            policy.apply_ew_denial,
            site_stub_overlay_on,
            Some(ctx),
        )
    } else {
        format_developmental_context_line(
            &strip,
            &ghost,
            &preview,
            &bindings,
            policy.apply_routing_congestion,
            policy.apply_ew_denial,
            site_stub_overlay_on,
        )
    };
    for mut t in &mut q {
        *t = Text::new(line.clone());
    }
}

/// BUILD-READ-DESIGN-002 — context strip wiring witness (copy charter in `design_build_toolbox_hud_v1.md`).
#[must_use]
pub fn build_read_design_002_witness_green() -> bool {
    build_read_design_002_self_check().is_ok()
}

fn build_read_design_002_self_check() -> Result<(), &'static str> {
    use crate::construction::{
        BuildGhostState, BuildPlacementMode, BuildPlacementPreview, BuildStripState, ToolContext,
    };
    use crate::gui::input_bindings::InputBindings;
    use crate::strategic::{BuildSiteTile, StrategicOverlayDisplayPolicy};

    let bindings = InputBindings::default();
    let policy = StrategicOverlayDisplayPolicy::default();
    let mut strip = BuildStripState::default();
    strip.active = ToolContext::Industry;
    let ghost_preview = BuildGhostState::default();
    let preview = BuildPlacementPreview::default();
    let preview_line = format_developmental_context_line(
        &strip,
        &ghost_preview,
        &preview,
        &bindings,
        policy.apply_routing_congestion,
        policy.apply_ew_denial,
        false,
    );
    if !preview_line.contains("click map to lock") {
        return Err("preview_copy");
    }
    if preview_line.contains("shift+click") || preview_line.contains("ok to commit") {
        return Err("legacy_build_copy");
    }

    let ghost_adjust = BuildGhostState {
        origin: Some(BuildSiteTile { x: 4, z: 7 }),
        placement_mode: BuildPlacementMode::Adjust,
        ..Default::default()
    };
    let mut adjust_preview = BuildPlacementPreview::default();
    adjust_preview.report.allows_commit = true;
    let adjust_line = format_developmental_context_line(
        &strip,
        &ghost_adjust,
        &adjust_preview,
        &bindings,
        policy.apply_routing_congestion,
        policy.apply_ew_denial,
        true,
    );
    if !adjust_line.contains("Ctrl rotate") || !adjust_line.contains("Shift scale") {
        return Err("adjust_copy");
    }
    if !adjust_line.contains("site overlay on") {
        return Err("site_overlay_suffix");
    }
    Ok(())
}

#[must_use]
pub fn build_read_design_002_witness_json() -> serde_json::Value {
    serde_json::json!({
        "gate": "BUILD-READ-DESIGN-002",
        "green": build_read_design_002_witness_green(),
        "context_strip_formatter": true,
        "design_copy_wired": build_read_design_002_witness_green(),
        "preview_copy": "click map to lock",
        "adjust_copy": "Ctrl rotate · Shift scale · click to place",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_read_design_002_witness_self_check_green() {
        assert!(build_read_design_002_witness_green());
    }
}
