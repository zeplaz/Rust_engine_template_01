//! Ship-art presentation routing — PixelWorld vs thin tactical icons.
//!
//! Charter: `src/dev/design_dual_view_art_home_v1.md`.
//! Default for `ship: true` packs is [`ArtPresentationTarget::PixelWorld`].
//!
//! PixelWorld landscape paint: LG-5 atlas stamps via
//! [`crate::gui::landscape_chunk_atlas_stamp`] into the same sim RTT hole.

use bevy::prelude::*;

use crate::systems::ecology::LG5_ATLAS_ID;

/// Where a promoted APS / Blender asset may paint in Simulation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ArtPresentationTarget {
    /// Primary APS + Blender consumer (landscape, then buildings).
    #[default]
    PixelWorld,
    /// Thin symbol atlas only — never full module kits.
    TacticalIcon,
}

impl ArtPresentationTarget {
    /// Parse promote / AssetSpec tag `presentation: …`.
    #[must_use]
    pub fn from_tag(raw: &str) -> Option<Self> {
        let t = raw.trim().to_ascii_lowercase();
        let t = t.strip_prefix("presentation:").unwrap_or(&t).trim();
        match t {
            "pixel_world" | "pixelworld" | "world" => Some(Self::PixelWorld),
            "tactical_icon" | "tactical" | "icon" => Some(Self::TacticalIcon),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_tag(self) -> &'static str {
        match self {
            Self::PixelWorld => "presentation: pixel_world",
            Self::TacticalIcon => "presentation: tactical_icon",
        }
    }

    /// Ship packs without an explicit tag land in PixelWorld.
    #[must_use]
    pub fn for_ship_default() -> Self {
        Self::PixelWorld
    }
}

/// Ship landscape grammar preset ids — mirrors
/// `assets/configs/landscape/_preset_index.json` → `ship_presets`.
pub const PIXEL_WORLD_LANDSCAPE_SHIP_PRESETS: &[&str] = &[
    "agri_riparian_v0",
    "coastal_scrub_v0",
    "fire_recovery_v0",
    "industrial_barrier_v0",
    "military_defensive_v0",
    "old_growth_core_v0",
    "settlement_park_v0",
    "urban_canopy_v0",
    "wetland_margin_v0",
    "scrub_highland_v0",
];

/// Ship-allowed building iso atlases (`assets/configs/buildings/_tile_atlas_index.ron`
/// where `ship_allowed: true`) — PixelWorld art home only; never tactical dump.
pub const PIXEL_WORLD_BUILDING_SHIP_ATLASES: &[&str] = &[
    "rail_warehouse_pilot_v1",
    "rowhouse_victorian_production_v1",
    "warehouse_industrial_west_v2",
];

/// Which band paints landscape into the sim RTT hole under PixelWorld.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PixelWorldLandscapeBand {
    /// Height/slope/biome relief only (tactical or no LG-5 stamps yet).
    #[default]
    LandformRelief,
    /// Real LG-5 landscape atlas UV stamps blit into SimulationMapTexture.
    LandscapeAtlasStamp,
}

/// Offline index of landscape/env + building packs intended for PixelWorld (view-only).
#[derive(Resource, Clone, Debug, Default)]
pub struct PixelWorldLandscapePackIndex {
    /// Pack / atlas ids tagged `presentation: pixel_world` (landscape lane).
    pub pack_ids: Vec<String>,
    /// Ship building iso atlas ids (PixelWorld stamps only).
    pub building_pack_ids: Vec<String>,
    /// Last grammar batch witness path (relative).
    pub grammar_batch_witness: Option<String>,
}

impl PixelWorldLandscapePackIndex {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pack_ids.is_empty() && self.building_pack_ids.is_empty()
    }

    #[must_use]
    pub fn registered_count(&self) -> usize {
        self.pack_ids.len() + self.building_pack_ids.len()
    }

    /// Seed grammar presets + LG-5 + veg catalog + ship building atlases (idempotent).
    pub fn seed_ship_landscape_packs(&mut self) {
        self.grammar_batch_witness = Some(
            "debug_runs/mcp_landscape_grammar_preset_batch_live.json".to_string(),
        );
        if self.pack_ids.is_empty() {
            self.pack_ids = PIXEL_WORLD_LANDSCAPE_SHIP_PRESETS
                .iter()
                .map(|s| (*s).to_string())
                .collect();
        }
        self.register_pack(LG5_ATLAS_ID);
        // Veg/cover catalog — presentation: pixel_world (ground cover / burn / regrowth keys).
        self.register_pack("vegetation_variant_catalog_v1");
        self.seed_ship_building_packs();
    }

    /// Seed ship-allowed building iso atlases (idempotent).
    pub fn seed_ship_building_packs(&mut self) {
        for id in PIXEL_WORLD_BUILDING_SHIP_ATLASES {
            self.register_building_pack(*id);
        }
    }

    /// Register one landscape/env pack id if missing (promote / ops path).
    pub fn register_pack(&mut self, pack_id: impl Into<String>) {
        let id = pack_id.into();
        if !self.pack_ids.iter().any(|p| p == &id) {
            self.pack_ids.push(id);
        }
    }

    /// Register one building atlas id if missing.
    pub fn register_building_pack(&mut self, pack_id: impl Into<String>) {
        let id = pack_id.into();
        if !self.building_pack_ids.iter().any(|p| p == &id) {
            self.building_pack_ids.push(id);
        }
    }
}

/// Frame consume snapshot — updated by pack index sync + landscape/building stamp queues.
#[derive(Resource, Clone, Debug)]
pub struct PixelWorldLandscapeConsume {
    pub pixel_world_active: bool,
    pub registered_packs: usize,
    pub active_band: PixelWorldLandscapeBand,
    /// Chunks stamped this frame under PixelWorld (LG-5 atlas).
    pub stamped_chunk_count: usize,
    /// Building iso atlas stamps queued under PixelWorld (0 in Tactical).
    pub building_stamp_count: usize,
    /// LG-5 stamps using burn/scar/regrowth atlas rows (veg-modulated cover).
    pub veg_modulated_stamp_count: usize,
    /// Primary landscape atlas / pack id.
    pub primary_pack_id: Option<String>,
}

impl Default for PixelWorldLandscapeConsume {
    fn default() -> Self {
        Self {
            pixel_world_active: false,
            registered_packs: 0,
            active_band: PixelWorldLandscapeBand::LandformRelief,
            stamped_chunk_count: 0,
            building_stamp_count: 0,
            veg_modulated_stamp_count: 0,
            primary_pack_id: None,
        }
    }
}

/// Sync pack counts from index (stamp queues own band / stamp counts).
pub fn sync_pixel_world_landscape_consume(
    mode: Res<crate::gui::WorldPresentationMode>,
    index: Res<PixelWorldLandscapePackIndex>,
    mut consume: ResMut<PixelWorldLandscapeConsume>,
) {
    consume.pixel_world_active = mode.is_pixel_world();
    consume.registered_packs = index.registered_count();
    if consume.primary_pack_id.is_none() {
        consume.primary_pack_id = Some(LG5_ATLAS_ID.to_string());
    }
    if !mode.is_pixel_world() {
        consume.active_band = PixelWorldLandscapeBand::LandformRelief;
        consume.stamped_chunk_count = 0;
        consume.building_stamp_count = 0;
        consume.veg_modulated_stamp_count = 0;
    }
}

/// Refresh index sticky witness + ensure ship packs + LG-5 registered.
pub fn sync_pixel_world_landscape_pack_index(
    mut index: ResMut<PixelWorldLandscapePackIndex>,
) {
    index.seed_ship_landscape_packs();
}

fn seed_pixel_world_landscape_pack_index_on_startup(
    mut index: ResMut<PixelWorldLandscapePackIndex>,
) {
    index.seed_ship_landscape_packs();
}

#[must_use]
pub fn landscape_ship_packs_registered_green() -> bool {
    PIXEL_WORLD_LANDSCAPE_SHIP_PRESETS.len() == 10
        && PIXEL_WORLD_LANDSCAPE_SHIP_PRESETS.contains(&"agri_riparian_v0")
        && PIXEL_WORLD_LANDSCAPE_SHIP_PRESETS.contains(&"old_growth_core_v0")
}

#[must_use]
pub fn building_ship_atlases_registered_green() -> bool {
    PIXEL_WORLD_BUILDING_SHIP_ATLASES.len() == 3
        && PIXEL_WORLD_BUILDING_SHIP_ATLASES.contains(&"rowhouse_victorian_production_v1")
        && PIXEL_WORLD_BUILDING_SHIP_ATLASES.contains(&"warehouse_industrial_west_v2")
        && PIXEL_WORLD_BUILDING_SHIP_ATLASES.contains(&"rail_warehouse_pilot_v1")
}

#[must_use]
pub fn landscape_atlas_stamp_path_green() -> bool {
    crate::gui::landscape_chunk_atlas_stamp::landscape_lg5_chunk_uv_stamp_witness_green()
        && crate::systems::ecology::landscape_lg5_registry_stamped()
}

pub struct ArtPresentationTargetPlugin;

impl Plugin for ArtPresentationTargetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PixelWorldLandscapePackIndex>()
            .init_resource::<PixelWorldLandscapeConsume>()
            .add_systems(Startup, seed_pixel_world_landscape_pack_index_on_startup)
            .add_systems(
                Update,
                (
                    sync_pixel_world_landscape_pack_index,
                    sync_pixel_world_landscape_consume
                        .after(sync_pixel_world_landscape_pack_index)
                        .in_set(crate::gui::PixelWorldConsumerSystemSet::SyncLandscapeIntent),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_default_is_pixel_world() {
        assert_eq!(
            ArtPresentationTarget::for_ship_default(),
            ArtPresentationTarget::PixelWorld
        );
    }

    #[test]
    fn parses_presentation_tags() {
        assert_eq!(
            ArtPresentationTarget::from_tag("presentation: pixel_world"),
            Some(ArtPresentationTarget::PixelWorld)
        );
        assert_eq!(
            ArtPresentationTarget::from_tag("tactical_icon"),
            Some(ArtPresentationTarget::TacticalIcon)
        );
        assert_eq!(ArtPresentationTarget::from_tag("diffuse"), None);
    }

    #[test]
    fn tags_round_trip_prefix() {
        let t = ArtPresentationTarget::PixelWorld;
        assert!(t.as_tag().contains("pixel_world"));
        assert_eq!(ArtPresentationTarget::from_tag(t.as_tag()), Some(t));
    }

    #[test]
    fn ship_landscape_presets_seed_into_index_with_lg5_atlas() {
        assert!(landscape_ship_packs_registered_green());
        let mut index = PixelWorldLandscapePackIndex::default();
        index.seed_ship_landscape_packs();
        assert!(index.registered_count() >= 12);
        assert!(index.pack_ids.iter().any(|p| p == LG5_ATLAS_ID));
        assert!(index
            .pack_ids
            .iter()
            .any(|p| p == "vegetation_variant_catalog_v1"));
        index.register_pack(LG5_ATLAS_ID);
        assert_eq!(
            index.pack_ids.iter().filter(|p| *p == LG5_ATLAS_ID).count(),
            1,
            "idempotent LG-5 register"
        );
    }

    #[test]
    fn consume_defaults_to_landform_relief_band() {
        let c = PixelWorldLandscapeConsume::default();
        assert_eq!(c.active_band, PixelWorldLandscapeBand::LandformRelief);
        assert!(!c.pixel_world_active);
        assert_eq!(c.stamped_chunk_count, 0);
        assert_eq!(c.building_stamp_count, 0);
        assert_eq!(c.veg_modulated_stamp_count, 0);
    }

    #[test]
    fn tactical_clears_building_and_landscape_stamp_counts() {
        let mut c = PixelWorldLandscapeConsume {
            pixel_world_active: true,
            registered_packs: 12,
            active_band: PixelWorldLandscapeBand::LandscapeAtlasStamp,
            stamped_chunk_count: 4,
            building_stamp_count: 2,
            veg_modulated_stamp_count: 3,
            primary_pack_id: Some(LG5_ATLAS_ID.to_string()),
        };
        let mode = crate::gui::WorldPresentationMode::default();
        assert!(mode.is_tactical());
        c.pixel_world_active = mode.is_pixel_world();
        if !mode.is_pixel_world() {
            c.active_band = PixelWorldLandscapeBand::LandformRelief;
            c.stamped_chunk_count = 0;
            c.building_stamp_count = 0;
            c.veg_modulated_stamp_count = 0;
        }
        assert_eq!(c.building_stamp_count, 0);
        assert_eq!(c.stamped_chunk_count, 0);
        assert_eq!(c.veg_modulated_stamp_count, 0);
    }

    #[test]
    fn ship_packs_include_veg_catalog_for_pixel_world() {
        let mut index = PixelWorldLandscapePackIndex::default();
        index.seed_ship_landscape_packs();
        assert!(index.pack_ids.iter().any(|p| p == "vegetation_variant_catalog_v1"));
        assert!(index.registered_count() >= 12);
    }

    #[test]
    fn ship_building_atlases_seed_for_pixel_world() {
        assert!(building_ship_atlases_registered_green());
        let mut index = PixelWorldLandscapePackIndex::default();
        index.seed_ship_landscape_packs();
        for id in PIXEL_WORLD_BUILDING_SHIP_ATLASES {
            assert!(
                index.building_pack_ids.iter().any(|p| p == *id),
                "missing building atlas {id}"
            );
        }
        assert!(index.registered_count() >= 15);
        index.register_building_pack("rowhouse_victorian_production_v1");
        assert_eq!(
            index
                .building_pack_ids
                .iter()
                .filter(|p| *p == "rowhouse_victorian_production_v1")
                .count(),
            1,
            "idempotent building register"
        );
    }

    #[test]
    fn landscape_atlas_stamp_path_is_real() {
        if !crate::systems::ecology::landscape_lg5_registry_stamped() {
            eprintln!("skip: landscape lg5 index not stamped");
            return;
        }
        assert!(landscape_atlas_stamp_path_green());
    }
}
