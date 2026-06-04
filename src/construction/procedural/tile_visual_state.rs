//! TILE-FIX-003 — runtime [`VisualState`] (variant × facing × frame) for atlas UV lookup.

use super::tile_atlas_index::TileAtlasRegistry;
use super::tile_variant_resolver::{resolve_tile_variant, ResolvedTileVariant, TileVariantContext, VariantCatalog};

/// Runtime visual key for iso atlas lookup (buildings, vehicles, props share contract).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualState {
    pub variant_key: String,
    /// Iso facing index `0..render_facings-1` (8-way or 4-way bake grid).
    pub facing: u8,
    /// Animation frame (`0` for static states; fire uses `1..`).
    pub frame: u8,
}

/// Map gameplay quarter-turn placement (`0..4`) to atlas facing index.
#[must_use]
pub fn facing_from_rotation_quarter_turns(
    rotation_quarter_turns: u8,
    render_facings: u8,
    quarter_turn_fallback: bool,
) -> u8 {
    let turns = rotation_quarter_turns % 4;
    match render_facings {
        8 if quarter_turn_fallback => (turns * 2) % 8,
        8 => turns % 8,
        4 => turns % 4,
        n if n > 0 => (turns as u32 * (n as u32 / 4).max(1)) as u8 % n,
        _ => 0,
    }
}

impl VisualState {
    #[must_use]
    pub fn from_resolved(
        resolved: &ResolvedTileVariant,
        rotation_quarter_turns: u8,
        render_facings: u8,
        quarter_turn_fallback: bool,
    ) -> Self {
        let frame = resolved.animation_frame.unwrap_or(0);
        Self {
            variant_key: resolved.variant_key.clone(),
            facing: facing_from_rotation_quarter_turns(
                rotation_quarter_turns,
                render_facings,
                quarter_turn_fallback,
            ),
            frame,
        }
    }
}

impl TileAtlasRegistry {
    /// v2 lookup: `(variant, facing, frame)` → UV. Falls back to v1 `variants` when facing/frame are 0.
    #[must_use]
    pub fn resolve_visual_state_uv(
        &self,
        atlas_id: &str,
        state: &VisualState,
    ) -> Option<[f32; 4]> {
        let entry = self.get(atlas_id)?;
        if let Some(uv) = entry.lookup_uv(&state.variant_key, state.facing, state.frame) {
            return Some(uv);
        }
        if state.facing == 0 && state.frame == 0 {
            return entry.variants.get(&state.variant_key).copied();
        }
        None
    }

    #[must_use]
    pub fn resolve_visual_state_for_site(
        &self,
        atlas_id: &str,
        catalog: &VariantCatalog,
        ctx: TileVariantContext,
        rotation_quarter_turns: u8,
    ) -> Option<VisualState> {
        let entry = self.get(atlas_id)?;
        let resolved = resolve_tile_variant(catalog, ctx, &entry.variants);
        Some(VisualState::from_resolved(
            &resolved,
            rotation_quarter_turns,
            entry.render_facings,
            entry.quarter_turn_fallback,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn quarter_turn_maps_to_facing_8_with_fallback() {
        assert_eq!(facing_from_rotation_quarter_turns(0, 8, true), 0);
        assert_eq!(facing_from_rotation_quarter_turns(1, 8, true), 2);
        assert_eq!(facing_from_rotation_quarter_turns(2, 8, true), 4);
        assert_eq!(facing_from_rotation_quarter_turns(3, 8, true), 6);
    }

    #[test]
    fn visual_state_from_resolver_carries_fire_frame() {
        let resolved = ResolvedTileVariant {
            variant_key: "burning_03".into(),
            animation_frame: Some(3),
        };
        let vs = VisualState::from_resolved(&resolved, 1, 8, true);
        assert_eq!(vs.variant_key, "burning_03");
        assert_eq!(vs.facing, 2);
        assert_eq!(vs.frame, 3);
    }

    #[test]
    fn resolve_visual_state_uv_v2_lookup() {
        use super::super::tile_atlas_index::TileAtlasEntry;
        use crate::construction::procedural::module_index::DevelopmentTier;

        let mut lookups = std::collections::HashMap::new();
        lookups.insert(("clean_day".into(), 2u8, 0u8), [0.25, 0.0, 0.125, 0.5]);
        let entry = TileAtlasEntry {
            atlas_id: "test_v2".into(),
            batch_id: "tile_test".into(),
            assembly_id: String::new(),
            tile_id: "test".into(),
            atlas_png: String::new(),
            atlas_asset: String::new(),
            meta_json: String::new(),
            development_tier: DevelopmentTier::Production,
            style_pack_id: String::new(),
            ship_allowed: true,
            meta_schema_version: 2,
            render_facings: 8,
            quarter_turn_fallback: true,
            variants: HashMap::new(),
            lookups,
        };
        let mut reg = TileAtlasRegistry::default();
        reg.by_atlas_id.insert("test_v2".into(), entry);
        let vs = VisualState {
            variant_key: "clean_day".into(),
            facing: 2,
            frame: 0,
        };
        let uv = reg.resolve_visual_state_uv("test_v2", &vs).expect("uv");
        assert_eq!(uv[0], 0.25);
    }
}
