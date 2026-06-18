//! DES-BUILD-READ-HUD-001 — grammar read copy on sim context strip + placement debug.

use crate::construction::building_definitions::BuildingDefinitionRegistry;
use crate::construction::procedural::{
    generate_with_arch_dna_preset, load_building_grammar_registry, FacilityBindingV1,
};
use crate::construction::{
    human_age_label, human_archetype_label, human_district_label, human_massing_label,
    ActiveBuildTool, BuildGhostState, BuildPlacementMode, BuildPlacementPreview, BuildStripState,
    ToolContext,
};
use crate::gui::hud::validation_feedback;
use crate::gui::input_bindings::InputBindings;

#[derive(Clone, Debug)]
pub struct GrammarReadHudContext {
    pub archetype_id: String,
    pub archetype_label: String,
    pub district_label: String,
    pub massing_label: String,
    pub age_label: String,
}

/// Read-only facility join for debug HUD / placement debug (COD-FACILITY-BINDING-READ-001).
#[must_use]
pub fn resolve_facility_binding_read(
    registry: &BuildingDefinitionRegistry,
    tool: &ActiveBuildTool,
) -> Option<FacilityBindingV1> {
    let intent = tool.building_intent.as_ref()?;
    let catalog_id = intent.catalog_id.as_deref()?;
    let def = registry.get(catalog_id)?;
    let archetype_id = def.grammar_archetype_id.as_deref()?;
    let grammar_registry = load_building_grammar_registry();
    grammar_registry
        .facility_binding_for_archetype(archetype_id)
        .cloned()
}

#[must_use]
pub fn resolve_grammar_read_context(
    tool: &ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
    ghost: &BuildGhostState,
) -> Option<GrammarReadHudContext> {
    let intent = tool.building_intent.as_ref()?;
    let catalog_id = intent.catalog_id.as_deref()?;
    let def = registry.get(catalog_id)?;
    let archetype_id = def.grammar_archetype_id.as_deref()?;
    let district = def
        .district_style
        .as_deref()
        .unwrap_or("industrial_west");
    let seed = ghost
        .origin
        .map(|t| (t.x as u64).wrapping_mul(1_000_003).wrapping_add(t.z as u64))
        .unwrap_or(42);
    let generated = if let Some(preset) = intent.arch_dna_preset_id.as_deref() {
        generate_with_arch_dna_preset(archetype_id, district, seed, Some(preset)).ok()
    } else {
        crate::construction::procedural::generate_building_grammar(archetype_id, district, seed).ok()
    };
    let (massing_id, age_id) = generated
        .as_ref()
        .map(|g| (g.massing_strategy.as_str(), g.age_band.as_str()))
        .unwrap_or(("long_hall", "weathered"));
    Some(GrammarReadHudContext {
        archetype_id: archetype_id.to_string(),
        archetype_label: human_archetype_label(archetype_id),
        district_label: human_district_label(district),
        massing_label: human_massing_label(massing_id),
        age_label: human_age_label(age_id),
    })
}

#[must_use]
pub fn format_grammar_summary_chip(ctx: &GrammarReadHudContext) -> String {
    format!(
        "Style: {} · {} · {}",
        ctx.archetype_label, ctx.massing_label, ctx.age_label
    )
}

#[must_use]
pub fn format_developmental_context_line_with_grammar(
    strip: &BuildStripState,
    ghost: &BuildGhostState,
    preview: &BuildPlacementPreview,
    bindings: &InputBindings,
    policy_routing: bool,
    policy_ew: bool,
    site_stub_overlay_on: bool,
    grammar: Option<&GrammarReadHudContext>,
) -> String {
    let Some(ctx) = grammar else {
        return super::contextual_tip::format_developmental_context_line(
            strip,
            ghost,
            preview,
            bindings,
            policy_routing,
            policy_ew,
            site_stub_overlay_on,
        );
    };

    let is_building_tool = matches!(
        strip.active,
        ToolContext::Industry | ToolContext::Civil | ToolContext::Military
    );
    if !is_building_tool {
        return super::contextual_tip::format_developmental_context_line(
            strip,
            ghost,
            preview,
            bindings,
            policy_routing,
            policy_ew,
            site_stub_overlay_on,
        );
    }

    let site_suffix = if site_stub_overlay_on {
        " · site overlay on"
    } else {
        ""
    };
    let in_adjust = ghost.placement_mode == BuildPlacementMode::Adjust && ghost.origin.is_some();

    if in_adjust {
        let (x, z) = ghost
            .origin
            .map(|t| (t.x, t.z))
            .unwrap_or((0, 0));
        if preview.report.allows_commit {
            return format!(
                "CONTEXT — BUILD · {} · locked {x},{z} · Ctrl rotate · Shift scale · click to place · Esc cancel{site_suffix}",
                ctx.archetype_label,
            );
        }
        let reason = validation_feedback::primary_validation_message(&preview.report)
            .unwrap_or_else(|| "Placement blocked.".into());
        return format!(
            "CONTEXT — BUILD · {} · locked {x},{z} · blocked: {reason} · Esc cancel{site_suffix}",
            ctx.archetype_label,
        );
    }

    format!(
        "CONTEXT — BUILD · {} · {} · click map to lock{site_suffix}",
        ctx.archetype_label, ctx.district_label,
    )
}

/// DES-BUILD-READ-HUD-001 witness — no raw grammar ids in strip copy.
#[must_use]
pub fn des_build_read_hud_001_witness_green() -> bool {
    des_build_read_hud_001_self_check().is_ok()
}

fn des_build_read_hud_001_self_check() -> Result<(), &'static str> {
    use crate::construction::grammar_labels_loaded_green;

    if !grammar_labels_loaded_green() {
        return Err("grammar_labels");
    }
    let ctx = GrammarReadHudContext {
        archetype_id: "RailEdge".into(),
        archetype_label: human_archetype_label("RailEdge"),
        district_label: human_district_label("rail_yard_corridor"),
        massing_label: human_massing_label("l_shape"),
        age_label: human_age_label("weathered"),
    };
    let preview = format!(
        "BUILD · {} · {} · click map to lock",
        ctx.archetype_label, ctx.district_label
    );
    if preview.contains("RailEdge") || preview.contains("rail_yard") || preview.contains("l_shape") {
        return Err("raw_ids_in_strip");
    }
    let chip = format_grammar_summary_chip(&ctx);
    if !chip.starts_with("Style:") {
        return Err("summary_chip");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn des_build_read_hud_001_witness_self_check_green() {
        assert!(des_build_read_hud_001_witness_green());
    }
}
