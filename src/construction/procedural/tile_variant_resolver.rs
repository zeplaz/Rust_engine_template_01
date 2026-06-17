//! PT-4 — sim → tile variant_key resolver (loads [`VariantCatalog`] from RON).

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::Deserialize;

use super::TileAtlasEntry;
use crate::strategic::SiteConstructionPhase;

pub const VARIANT_CATALOG_RON: &str = "assets/configs/buildings/_variant_catalog.ron";
pub const PROCEDURAL_TILES_RUNTIME_LIVE_JSON: &str = "debug_runs/procedural_tiles_runtime_live.json";
const RESOLVER_VERSION: &str = "pt4_v1";

#[derive(Debug, Clone, Deserialize)]
pub struct FireCatalogConfig {
    pub key_prefix: String,
    pub frame_count: u32,
    pub frame_period_ms: u32,
    pub heat_threshold: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DamageBand {
    pub max: f32,
    pub suffix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LightingCompositeRow {
    pub night: bool,
    pub power_on: bool,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConstructionPhaseKeys {
    #[serde(default)]
    pub Planned: String,
    #[serde(default)]
    pub Surveying: String,
    #[serde(default)]
    pub Clearing: String,
    #[serde(default)]
    pub Foundation: String,
    #[serde(default)]
    pub UnderConstruction: String,
    #[serde(default)]
    pub Provisioning: String,
    #[serde(default)]
    pub Operational: String,
    #[serde(default)]
    pub Damaged: String,
    #[serde(default)]
    pub Offline: String,
    #[serde(default)]
    pub Abandoned: String,
}

#[derive(Resource, Debug, Clone, Deserialize)]
pub struct VariantCatalog {
    pub schema_version: u32,
    pub default_fallback_key: String,
    pub canonical_variant_keys: Vec<String>,
    pub fire: FireCatalogConfig,
    pub damage_bands: Vec<DamageBand>,
    pub construction_phase_keys: ConstructionPhaseKeys,
    pub lighting_composite: Vec<LightingCompositeRow>,
    pub damaged_lighting_composite: Vec<LightingCompositeRow>,
    pub ship_minimum_keys: Vec<String>,
}

/// Inputs for deterministic variant resolution (PT-4 / PT-5).
#[derive(Debug, Clone, Copy, Default)]
pub struct TileVariantContext {
    pub phase: SiteConstructionPhase,
    pub damage: f32,
    pub power_on: bool,
    pub night: bool,
    pub fire_heat: f32,
    pub sim_tick: u64,
}

/// Resolver output — variant key + optional fire animation frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTileVariant {
    pub variant_key: String,
    pub animation_frame: Option<u8>,
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

#[must_use]
pub fn load_variant_catalog() -> Option<VariantCatalog> {
    let path = repo_asset_path(VARIANT_CATALOG_RON);
    let text = std::fs::read_to_string(&path).ok()?;
    match ron::from_str(&text) {
        Ok(catalog) => Some(catalog),
        #[cfg(test)]
        Err(err) => {
            eprintln!("VariantCatalog RON parse failed ({path:?}): {err}");
            None
        }
        #[cfg(not(test))]
        Err(_) => None,
    }
}

pub fn init_variant_catalog(mut commands: Commands) {
    match load_variant_catalog() {
        Some(catalog) => {
            commands.insert_resource(catalog);
        }
        None => {
            warn!(target: "tile_variant", "VariantCatalog missing: {VARIANT_CATALOG_RON}");
        }
    }
}

fn key_if_set(raw: &str) -> Option<String> {
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_owned())
    }
}

/// **PT-5-002** — fire animation frame index driven by sim tick (maps from `SimStepStamp::tick`).
#[must_use]
pub fn pt_5_002_fire_frame_tick_witness_green() -> bool {
    let Some(catalog) = load_variant_catalog() else {
        return false;
    };
    let uvs = std::collections::HashMap::from([(
        "burning_00".to_string(),
        [0.0_f32, 0.0, 0.5, 0.5],
    )]);
    let ctx_a = TileVariantContext {
        phase: SiteConstructionPhase::Operational,
        fire_heat: 1.0,
        sim_tick: 0,
        ..Default::default()
    };
    let ctx_b = TileVariantContext {
        sim_tick: catalog.fire.frame_period_ms as u64,
        ..ctx_a
    };
    let a = resolve_tile_variant(&catalog, ctx_a, &uvs);
    let b = resolve_tile_variant(&catalog, ctx_b, &uvs);
    a.animation_frame == Some(0) && b.animation_frame == Some(1)
}

fn fire_frame_index(catalog: &VariantCatalog, sim_tick: u64) -> u8 {
    let period = catalog.fire.frame_period_ms.max(1) as u64;
    let frame = (sim_tick / period) % u64::from(catalog.fire.frame_count.max(1));
    u8::try_from(frame).unwrap_or(0)
}

fn damage_suffix(catalog: &VariantCatalog, damage: f32) -> &str {
    for band in &catalog.damage_bands {
        if damage <= band.max {
            return band.suffix.as_str();
        }
    }
    "ruined"
}

fn pick_from_composite(
    rows: &[LightingCompositeRow],
    night: bool,
    power_on: bool,
) -> Option<String> {
    rows.iter()
        .find(|r| r.night == night && r.power_on == power_on)
        .map(|r| r.key.clone())
}

fn fallback_key(catalog: &VariantCatalog, available: &std::collections::HashMap<String, [f32; 4]>) -> String {
    if available.contains_key(&catalog.default_fallback_key) {
        return catalog.default_fallback_key.clone();
    }
    for key in &catalog.ship_minimum_keys {
        if available.contains_key(key) {
            return key.clone();
        }
    }
    available.keys().next().cloned().unwrap_or_else(|| "clean_day".into())
}

fn clamp_to_available(
    catalog: &VariantCatalog,
    key: &str,
    available: &std::collections::HashMap<String, [f32; 4]>,
) -> String {
    if available.contains_key(key) {
        return key.to_owned();
    }
    fallback_key(catalog, available)
}

/// Deterministic sim → variant_key (see plan § Sim → variant resolver).
#[must_use]
pub fn resolve_tile_variant(
    catalog: &VariantCatalog,
    ctx: TileVariantContext,
    available: &std::collections::HashMap<String, [f32; 4]>,
) -> ResolvedTileVariant {
    if ctx.fire_heat >= catalog.fire.heat_threshold {
        let frame = fire_frame_index(catalog, ctx.sim_tick);
        let key = format!("{}{frame:02}", catalog.fire.key_prefix);
        return ResolvedTileVariant {
            variant_key: clamp_to_available(catalog, &key, available),
            animation_frame: Some(frame),
        };
    }

    let phase_key = match ctx.phase {
        SiteConstructionPhase::Planned => key_if_set(&catalog.construction_phase_keys.Planned),
        SiteConstructionPhase::Surveying => key_if_set(&catalog.construction_phase_keys.Surveying),
        SiteConstructionPhase::Clearing => key_if_set(&catalog.construction_phase_keys.Clearing),
        SiteConstructionPhase::Foundation => key_if_set(&catalog.construction_phase_keys.Foundation),
        SiteConstructionPhase::UnderConstruction => {
            key_if_set(&catalog.construction_phase_keys.UnderConstruction)
        }
        SiteConstructionPhase::Provisioning => key_if_set(&catalog.construction_phase_keys.Provisioning),
        SiteConstructionPhase::Damaged => key_if_set(&catalog.construction_phase_keys.Damaged),
        SiteConstructionPhase::Offline => key_if_set(&catalog.construction_phase_keys.Offline),
        SiteConstructionPhase::Abandoned => key_if_set(&catalog.construction_phase_keys.Abandoned),
        SiteConstructionPhase::Operational => None,
    };
    if let Some(key) = phase_key {
        return ResolvedTileVariant {
            variant_key: clamp_to_available(catalog, &key, available),
            animation_frame: None,
        };
    }

    let suffix = damage_suffix(catalog, ctx.damage);
    let key = if suffix == "clean" {
        pick_from_composite(&catalog.lighting_composite, ctx.night, ctx.power_on)
            .unwrap_or_else(|| catalog.default_fallback_key.clone())
    } else if suffix == "damaged" {
        pick_from_composite(&catalog.damaged_lighting_composite, ctx.night, ctx.power_on)
            .unwrap_or_else(|| "damaged_day".into())
    } else {
        "ruined".into()
    };

    ResolvedTileVariant {
        variant_key: clamp_to_available(catalog, &key, available),
        animation_frame: None,
    }
}

/// Keyframe-pack production atlas → PG-2 mesh spawn should demote (PT-4-003).
/// Uses [`TileAtlasEntry::runtime_stamp_allowed`] (production tier + ship_allowed from index).
#[must_use]
pub fn production_atlas_covers_assembly(entry: &TileAtlasEntry) -> bool {
    entry.runtime_stamp_allowed()
}

/// Production iso atlas demotes PG-2 mesh spawn when true (PT-4-003).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ProceduralTilePrimaryActive(pub bool);

/// **PT-4-004** — night + power_on composite keys resolve from catalog.
#[must_use]
pub fn pt4_power_night_inputs_witness_green() -> bool {
    let Some(catalog) = load_variant_catalog() else {
        return false;
    };
    let uvs = pilot_uv_map();
    let day = resolve_tile_variant(
        &catalog,
        TileVariantContext {
            phase: SiteConstructionPhase::Operational,
            ..Default::default()
        },
        &uvs,
    );
    let night_on = resolve_tile_variant(
        &catalog,
        TileVariantContext {
            phase: SiteConstructionPhase::Operational,
            night: true,
            power_on: true,
            ..Default::default()
        },
        &uvs,
    );
    day.variant_key == "clean_day" && night_on.variant_key == "clean_night_on"
}

/// **PT-4-005** — damage scalar maps to damaged_* lighting composite keys.
#[must_use]
pub fn pt4_damage_variant_witness_green() -> bool {
    let Some(catalog) = load_variant_catalog() else {
        return false;
    };
    let uvs = pilot_uv_map();
    let damaged = resolve_tile_variant(
        &catalog,
        TileVariantContext {
            phase: SiteConstructionPhase::Operational,
            damage: 0.5,
            night: true,
            power_on: true,
            ..Default::default()
        },
        &uvs,
    );
    damaged.variant_key == "damaged_night_on"
}

#[must_use]
pub fn procedural_tiles_runtime_witness_green() -> bool {
    procedural_tiles_runtime_self_check().is_ok()
        && pt4_power_night_inputs_witness_green()
        && pt4_damage_variant_witness_green()
        && crate::gui::map_tile_atlas_stamp::eng_pt_4_001_rowhouse_map_stamp_witness_green()
}

fn procedural_tiles_runtime_self_check() -> Result<(), &'static str> {
    use super::load_variant_catalog;
    use super::resolve_tile_variant;
    use super::TileVariantContext;
    use crate::strategic::SiteConstructionPhase;
    use std::collections::HashMap;

    let catalog = load_variant_catalog().ok_or("catalog")?;
    let uvs = HashMap::from([
        ("clean_day".into(), [0.0, 0.0, 0.5, 1.0]),
        ("clean_night_on".into(), [0.5, 0.0, 0.5, 1.0]),
        ("under_construction_02".into(), [0.5, 0.5, 0.5, 0.5]),
        ("abandoned".into(), [0.0, 0.0, 0.25, 0.25]),
        ("burning_00".into(), [0.25, 0.0, 0.25, 0.25]),
    ]);
    let cases = [
        (SiteConstructionPhase::Operational, "clean_day"),
        (SiteConstructionPhase::UnderConstruction, "under_construction_02"),
        (SiteConstructionPhase::Abandoned, "abandoned"),
    ];
    if !pt_5_002_fire_frame_tick_witness_green() {
        return Err("pt_5_002_fire_frame_tick");
    }

    for (phase, expected) in cases {
        let resolved = resolve_tile_variant(
            &catalog,
            TileVariantContext {
                phase,
                ..Default::default()
            },
            &uvs,
        );
        if resolved.variant_key != expected {
            return Err("resolver_case");
        }
    }
    Ok(())
}

fn pilot_uv_map() -> HashMap<String, [f32; 4]> {
    HashMap::from([
        ("clean_day".into(), [0.0, 0.0, 0.5, 1.0]),
        ("clean_night_on".into(), [0.5, 0.0, 0.5, 1.0]),
        ("damaged_night_on".into(), [0.0, 0.5, 0.5, 0.5]),
        ("under_construction_02".into(), [0.5, 0.5, 0.5, 0.5]),
        ("abandoned".into(), [0.0, 0.0, 0.25, 0.25]),
        ("burning_00".into(), [0.25, 0.0, 0.25, 0.25]),
        ("burning_01".into(), [0.5, 0.0, 0.25, 0.25]),
    ])
}

/// PT-4 witness payload — resolver table + gate flags (lib refresh).
#[must_use]
pub fn build_procedural_tiles_runtime_witness_body() -> serde_json::Value {
    let catalog = load_variant_catalog();
    let uvs = pilot_uv_map();
    let mut cases = Vec::new();
    let sample_inputs: [(SiteConstructionPhase, f32, bool, bool, f32, u64, &str); 6] = [
        (SiteConstructionPhase::Operational, 0.0, false, false, 0.0, 0, "clean_day"),
        (
            SiteConstructionPhase::Operational,
            0.0,
            true,
            true,
            0.0,
            0,
            "clean_night_on",
        ),
        (
            SiteConstructionPhase::UnderConstruction,
            0.0,
            false,
            false,
            0.0,
            0,
            "under_construction_02",
        ),
        (SiteConstructionPhase::Abandoned, 0.0, false, false, 0.0, 0, "abandoned"),
        (
            SiteConstructionPhase::Operational,
            0.0,
            true,
            false,
            0.9,
            0,
            "burning_00",
        ),
        (
            SiteConstructionPhase::Operational,
            0.0,
            true,
            false,
            0.9,
            120,
            "burning_01",
        ),
    ];
    let mut smoke_fallback_used = false;
    if let Some(ref cat) = catalog {
        for (phase, damage, night, power_on, fire_heat, sim_tick, expected) in sample_inputs {
            let resolved = resolve_tile_variant(
                cat,
                TileVariantContext {
                    phase,
                    damage,
                    night,
                    power_on,
                    fire_heat,
                    sim_tick,
                },
                &uvs,
            );
            if resolved.variant_key != expected {
                smoke_fallback_used = true;
            }
            cases.push(serde_json::json!({
                "phase": format!("{phase:?}"),
                "damage": damage,
                "night": night,
                "power_on": power_on,
                "fire_heat": fire_heat,
                "sim_tick": sim_tick,
                "variant_key": resolved.variant_key,
                "expected_variant_key": expected,
                "animation_frame": resolved.animation_frame,
                "stamp_applied": true,
            }));
        }
    } else {
        smoke_fallback_used = true;
    }
    let rowhouse_stamp_wired =
        crate::gui::map_tile_atlas_stamp::eng_pt_4_001_rowhouse_map_stamp_witness_green();
    let warehouse_stamp_wired =
        crate::gui::map_tile_atlas_stamp::eng_pt_4_warehouse_map_stamp_witness_green();
    serde_json::json!({
        "gate_id": "TILE-PROD-003",
        "resolver_version": RESOLVER_VERSION,
        "green": procedural_tiles_runtime_witness_green(),
        "catalog_loaded": catalog.is_some(),
        "eng_pt_4_001_rowhouse_map_stamp": rowhouse_stamp_wired,
        "eng_pt_4_002_warehouse_map_stamp": warehouse_stamp_wired,
        "tile_fix_10_warehouse_witness": crate::gui::map_tile_atlas_stamp::tile_fix_10_warehouse_witness_green(),
        "warehouse_v2_index_registered": crate::gui::map_tile_atlas_stamp::warehouse_v2_atlas_index_registered(),
        "power_night_inputs_wired": pt4_power_night_inputs_witness_green(),
        "damage_variant_wired": pt4_damage_variant_witness_green(),
        "smoke_fallback_used": smoke_fallback_used,
        "pg2_suppression": "ProceduralTilePrimaryActive",
        "cases": cases,
    })
}

/// Refresh `debug_runs/procedural_tiles_runtime_live.json` (PT-4 witness).
#[must_use]
pub fn refresh_procedural_tiles_runtime_live_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_procedural_tiles_runtime_witness_body();
    let wrapped = wrap_debug_run(
        "construction",
        "refresh_procedural_tiles_runtime_live_witness",
        PROCEDURAL_TILES_RUNTIME_LIVE_JSON,
        body,
    );
    write_debug_run_json(PROCEDURAL_TILES_RUNTIME_LIVE_JSON, wrapped)
        && procedural_tiles_runtime_witness_green()
}

/// APS-E5 — atlas domain for landscape LG-5 stamps (distinct from building [`VariantCatalog`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileAtlasDomain {
    Building,
    Landscape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedLandscapeTileVariant {
    pub variant_key: String,
    pub uv: [f32; 4],
    pub atlas_id: String,
}

/// Resolve LG-5 landscape atlas UV for an authored variant_key.
#[must_use]
pub fn resolve_landscape_tile_variant(
    registry: &crate::systems::ecology::LandscapeAtlasRegistry,
    variant_key: &str,
) -> Option<ResolvedLandscapeTileVariant> {
    let entry = registry.lg5_entry()?;
    let uv = entry.resolve_variant_uv(variant_key)?;
    Some(ResolvedLandscapeTileVariant {
        variant_key: variant_key.to_owned(),
        uv,
        atlas_id: entry.atlas_id.clone(),
    })
}

/// Map topology kind label → LG-5 variant_key → UV (CDR-B-TILE-RESOLVER-VEG-001).
#[must_use]
pub fn resolve_landscape_tile_from_topology(
    registry: &crate::systems::ecology::LandscapeAtlasRegistry,
    topology_kind: &str,
) -> Option<ResolvedLandscapeTileVariant> {
    let key = crate::systems::ecology::topology_kind_to_variant_key(topology_kind)?;
    resolve_landscape_tile_variant(registry, key)
}

/// Resolve extract-frame `veg_topo_*` or catalog `topology_*` keys on landscape domain.
#[must_use]
pub fn resolve_landscape_tile_from_extract_key(
    registry: &crate::systems::ecology::LandscapeAtlasRegistry,
    extract_variant_key: &str,
) -> Option<ResolvedLandscapeTileVariant> {
    if extract_variant_key.starts_with("veg_topo_") {
        let topo = extract_variant_key.trim_start_matches("veg_topo_");
        let kind = match topo {
            "patch" => "Patch",
            "corridor" => "Corridor",
            "ring" => "Ring",
            "network" => "Network",
            other => {
                let mut s = other.to_owned();
                if let Some(r) = s.get_mut(0..1) {
                    r.make_ascii_uppercase();
                }
                return resolve_landscape_tile_from_topology(registry, &s);
            }
        };
        return resolve_landscape_tile_from_topology(registry, kind);
    }
    resolve_landscape_tile_variant(registry, extract_variant_key)
}

#[must_use]
pub fn landscape_tile_resolver_witness_green() -> bool {
    use crate::systems::ecology::load_landscape_atlas_registry;
    let registry = load_landscape_atlas_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    ["Patch", "Corridor", "Ring"]
        .iter()
        .all(|kind| resolve_landscape_tile_from_topology(&registry, kind).is_some())
        && resolve_landscape_tile_from_extract_key(&registry, "veg_topo_patch").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn pilot_uvs() -> HashMap<String, [f32; 4]> {
        HashMap::from([
            ("clean_day".into(), [0.0, 0.0, 0.5, 1.0]),
            ("clean_night_on".into(), [0.5, 0.0, 0.5, 1.0]),
            ("damaged_night_on".into(), [0.0, 0.5, 0.5, 0.5]),
            ("under_construction_02".into(), [0.5, 0.5, 0.5, 0.5]),
            ("abandoned".into(), [0.0, 0.0, 0.25, 0.25]),
            ("burning_00".into(), [0.25, 0.0, 0.25, 0.25]),
            ("burning_01".into(), [0.5, 0.0, 0.25, 0.25]),
        ])
    }

    #[test]
    fn variant_catalog_loads_from_ron() {
        let catalog = load_variant_catalog().expect("catalog");
        assert_eq!(catalog.default_fallback_key, "clean_day");
        assert!(catalog.fire.frame_count >= 8);
    }

    #[test]
    fn resolver_operational_day_clean() {
        let catalog = load_variant_catalog().expect("catalog");
        let uvs = pilot_uvs();
        let resolved = resolve_tile_variant(
            &catalog,
            TileVariantContext {
                phase: SiteConstructionPhase::Operational,
                ..Default::default()
            },
            &uvs,
        );
        assert_eq!(resolved.variant_key, "clean_day");
        assert!(resolved.animation_frame.is_none());
    }

    #[test]
    fn pt4_damage_variant_maps_to_damaged_night_on() {
        assert!(super::pt4_damage_variant_witness_green());
    }

    #[test]
    fn pt4_power_night_inputs_witness_green_lib() {
        assert!(super::pt4_power_night_inputs_witness_green());
    }

    #[test]
    fn resolver_night_power_on() {
        let catalog = load_variant_catalog().expect("catalog");
        let uvs = pilot_uvs();
        let resolved = resolve_tile_variant(
            &catalog,
            TileVariantContext {
                phase: SiteConstructionPhase::Operational,
                night: true,
                power_on: true,
                ..Default::default()
            },
            &uvs,
        );
        assert_eq!(resolved.variant_key, "clean_night_on");
    }

    #[test]
    fn resolver_fire_frame_advances() {
        let catalog = load_variant_catalog().expect("catalog");
        let uvs = pilot_uvs();
        let a = resolve_tile_variant(
            &catalog,
            TileVariantContext {
                phase: SiteConstructionPhase::Operational,
                fire_heat: 0.9,
                sim_tick: 0,
                ..Default::default()
            },
            &uvs,
        );
        let b = resolve_tile_variant(
            &catalog,
            TileVariantContext {
                phase: SiteConstructionPhase::Operational,
                fire_heat: 0.9,
                sim_tick: catalog.fire.frame_period_ms as u64,
                ..Default::default()
            },
            &uvs,
        );
        assert!(a.variant_key.starts_with("burning_"));
        assert_eq!(a.animation_frame, Some(0));
        assert_eq!(b.animation_frame, Some(1));
    }

    #[test]
    fn landscape_tile_resolver_maps_topology_and_extract_keys() {
        use crate::systems::ecology::load_landscape_atlas_registry;
        let registry = load_landscape_atlas_registry();
        if !registry.lg5_entry().is_some_and(|e| e.chunk_stamp_allowed()) {
            return;
        }
        assert!(super::landscape_tile_resolver_witness_green());
        let patch = super::resolve_landscape_tile_from_extract_key(&registry, "veg_topo_patch")
            .expect("veg_topo_patch");
        assert!(patch.variant_key.starts_with("topology_"));
    }

    #[test]
    fn runtime_witness_green() {
        assert!(super::procedural_tiles_runtime_witness_green());
    }

    #[test]
    fn refresh_procedural_tiles_runtime_live_witness_writes_json() {
        assert!(super::refresh_procedural_tiles_runtime_live_witness());
    }

    #[test]
    fn production_atlas_covers_keyframe_pack_entry() {
        use super::production_atlas_covers_assembly;
        use crate::construction::procedural::module_index::DevelopmentTier;
        use crate::construction::procedural::TileAtlasEntry;
        use std::collections::HashMap;

        let production = TileAtlasEntry {
            atlas_id: "rowhouse_victorian_production_v1".into(),
            batch_id: "tile_rowhouse_victorian_production_v1".into(),
            assembly_id: "victorian_4x3_s42_a7cb".into(),
            tile_id: "rowhouse_victorian".into(),
            atlas_png: "assets/textures/buildings_iso/production/rowhouse_victorian_production_v1_atlas.png".into(),
            atlas_asset: "textures/buildings_iso/production/rowhouse_victorian_production_v1_atlas.png".into(),
            meta_json: "assets/staging/tiles/tile_rowhouse_victorian_production_v1/atlas_meta.json".into(),
            development_tier: DevelopmentTier::Production,
            style_pack_id: "style_victorian".into(),
            ship_allowed: true,
            variants: HashMap::from([("clean_day".into(), [0.0, 0.0, 0.5, 1.0])]),
            meta_schema_version: 1,
            render_facings: 8,
            quarter_turn_fallback: true,
            lookups: HashMap::new(),
        };
        assert!(production.runtime_stamp_allowed());
        assert!(production_atlas_covers_assembly(&production));

        let lod0 = TileAtlasEntry {
            development_tier: DevelopmentTier::Lod0,
            ship_allowed: true,
            ..production.clone()
        };
        assert!(!lod0.runtime_stamp_allowed());
        assert!(!production_atlas_covers_assembly(&lod0));
    }
}
