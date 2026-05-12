//! L0 immediate context — mode, selection, affordances without opening a panel.

use bevy::prelude::*;

use crate::gui::build::{BuildGhostState, BuildPlacementPreview, BuildStripState, ToolContext};
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
) -> String {
    let cycle = tool_help::format_build_cycle_key(bindings);
    let commit = tool_help::format_build_commit_key(bindings);
    let rot = tool_help::format_map_rotate_keys(bindings);
    let overlay = format!(
        "overlays congestion {} / EW {}",
        if policy_routing { "on" } else { "off" },
        if policy_ew { "on" } else { "off" },
    );

    if strip.active == ToolContext::None {
        return format!(
            "CONTEXT — Planning idle · press [{cycle}] to choose build category · map rotate [{rot}] · {overlay}",
        );
    }

    let heading = tool_heading(strip.active);
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

    if preview.report.allows_commit {
        format!(
            "CONTEXT — {heading} · {site} · ok to commit [{commit}] · cycle [{cycle}] · rotate map [{rot}] · {overlay}{warn_note}{projection_note}",
        )
    } else {
        let reason = validation_feedback::primary_validation_message(&preview.report)
            .unwrap_or_else(|| "Placement blocked.".into());
        format!(
            "CONTEXT — {heading} · {site} · blocked: {reason} · [{cycle}] change mode · [{commit}] when valid · {overlay}",
        )
    }
}

pub fn update_developmental_context_strip_system(
    strip: Res<BuildStripState>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    bindings: Res<InputBindings>,
    policy: Res<StrategicOverlayDisplayPolicy>,
    mut q: Query<&mut Text, With<DevelopmentalContextStripLine>>,
) {
    let line = format_developmental_context_line(
        &strip,
        &ghost,
        &preview,
        &bindings,
        policy.apply_routing_congestion,
        policy.apply_ew_denial,
    );
    for mut t in &mut q {
        *t = Text::new(line.clone());
    }
}
