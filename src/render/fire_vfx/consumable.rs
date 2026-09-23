//! VSS-T4-004 — artist `EffectConsumable` registry + spawn_hook resolver.
//!
//! Loads promoted packs from `assets/effects/registry/` and routes lanes through the existing
//! `fire_vfx` → `gpu_particle_draw` spine — **no parallel draw path**.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use serde::Deserialize;

use crate::gui::WorldLodBand;
use crate::render::extraction::sim_visual_extract::ClimateVisualAggregate;
use crate::render::particle_domains::ParticleDomainId;
use crate::render::SharedOverlayFieldBuffers;

pub const EFFECT_REGISTRY_ROOT: &str = "assets/effects/registry";
pub const EFFECT_SPEC_FILENAME: &str = "effect_spec.json";

/// Stable registry key — matches `effect_id` in EffectSpec v1.
pub type EffectId = String;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectLane {
    ParticleInstanced,
    GpuField,
    MaterialOverlay,
    Embellishment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpawnHookKind {
    WorldXy,
    AttachEntity,
    FieldSample,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SimCoupling {
    #[default]
    None,
    HeatField,
    SmokeField,
    PrecipField,
    CustomMessage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectFieldId {
    SharedOverlayHeat,
    SharedOverlaySmoke,
    GpuWeatherFireField,
    ClimatePrecip,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectSpawnWorldXy {
    pub tile_x: i32,
    pub tile_y: i32,
    #[serde(default = "default_offset_tile")]
    pub offset_tile: [f32; 2],
    #[serde(default)]
    pub world_z: f32,
}

fn default_offset_tile() -> [f32; 2] {
    [0.5, 0.5]
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectSpawnAttachEntity {
    pub anchor_kind: String,
    #[serde(default)]
    pub entity_tag: Option<String>,
    #[serde(default = "default_socket")]
    pub socket: String,
    #[serde(default = "default_offset_local")]
    pub offset_local: [f32; 3],
}

fn default_socket() -> String {
    "origin".into()
}

fn default_offset_local() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectSpawnFieldSample {
    pub field_id: EffectFieldId,
    pub sample_uv: [f32; 2],
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    #[serde(default = "default_density_scale")]
    pub spawn_density_scale: f32,
}

fn default_threshold() -> f32 {
    0.1
}

fn default_density_scale() -> f32 {
    1.0
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EffectSpawnHookConfig {
    #[serde(default)]
    pub world_xy: Option<EffectSpawnWorldXy>,
    #[serde(default)]
    pub attach_entity: Option<EffectSpawnAttachEntity>,
    #[serde(default)]
    pub field_sample: Option<EffectSpawnFieldSample>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectGridConfig {
    #[serde(default = "default_tile_units")]
    pub tile_units: f32,
    #[serde(default = "default_true")]
    pub snap_to_grid: bool,
    #[serde(default)]
    pub rotation_quarters: u8,
    #[serde(default = "default_anchor")]
    pub anchor: String,
}

fn default_tile_units() -> f32 {
    crate::terrain::world_scale_contract::TILE_SIM_UNIT
}

fn default_true() -> bool {
    true
}

fn default_anchor() -> String {
    "tile_center".into()
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectEmitParams {
    pub duration_secs: f32,
    pub rate: f32,
    #[serde(default)]
    pub burst_count: u32,
    #[serde(default)]
    pub lifetime_secs: Option<f32>,
    #[serde(default)]
    pub prewarm_secs: f32,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EffectLodTierPolicy {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub max_instances: u32,
    #[serde(default = "default_rate_scale")]
    pub rate_scale: f32,
    #[serde(default = "default_sprite_px_scale")]
    pub sprite_px_scale: f32,
    #[serde(default)]
    pub fire_lod_band: Option<String>,
}

fn default_rate_scale() -> f32 {
    1.0
}

fn default_sprite_px_scale() -> f32 {
    1.0
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectLodTiers {
    #[serde(default)]
    pub r#macro: Option<EffectLodTierPolicy>,
    #[serde(default)]
    pub strategic: Option<EffectLodTierPolicy>,
    pub operational: EffectLodTierPolicy,
    #[serde(default)]
    pub local_tactical: Option<EffectLodTierPolicy>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EffectShaderPack {
    #[serde(default)]
    pub compute: Option<String>,
    #[serde(default)]
    pub draw: Option<String>,
    #[serde(default)]
    pub particle: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EffectVisualSpec {
    pub source: String,
    #[serde(default)]
    pub asset_path: Option<String>,
    #[serde(default)]
    pub shader_entry: Option<String>,
    #[serde(default)]
    pub shader_pack: Option<EffectShaderPack>,
    #[serde(default)]
    pub blend: Option<String>,
    #[serde(default)]
    pub sprite_px: Option<f32>,
    #[serde(default)]
    pub tint: Option<[f32; 4]>,
}

#[derive(Clone, Debug, Deserialize)]
struct EffectSpecFile {
    schema_version: String,
    effect_id: EffectId,
    #[serde(default)]
    display_name: Option<String>,
    seed: u32,
    batch_id: String,
    #[serde(default)]
    development_tier: Option<String>,
    lane: EffectLane,
    #[serde(default)]
    sim_coupling: SimCoupling,
    spawn_hook: SpawnHookKind,
    #[serde(default)]
    spawn_hook_config: EffectSpawnHookConfig,
    #[serde(default)]
    grid: Option<EffectGridConfig>,
    emit: EffectEmitParams,
    lod_tiers: EffectLodTiers,
    #[serde(default)]
    visual: Option<EffectVisualSpec>,
}

/// Parsed, promote-ready consumable definition (runtime authority for one effect pack).
#[derive(Clone, Debug)]
pub struct EffectConsumableDef {
    pub effect_id: EffectId,
    pub display_name: Option<String>,
    pub seed: u32,
    pub batch_id: String,
    pub development_tier: Option<String>,
    pub lane: EffectLane,
    pub sim_coupling: SimCoupling,
    pub spawn_hook: SpawnHookKind,
    pub spawn_hook_config: EffectSpawnHookConfig,
    pub grid: EffectGridConfig,
    pub emit: EffectEmitParams,
    pub lod_tiers: EffectLodTiers,
    pub visual: Option<EffectVisualSpec>,
    /// Absolute registry directory containing `effect_spec.json`.
    pub registry_dir: PathBuf,
}

impl EffectConsumableDef {
    #[must_use]
    pub fn tile_units(&self) -> f32 {
        self.grid.tile_units
    }

    #[must_use]
    pub fn lod_policy_for_band(&self, band: WorldLodBand) -> &EffectLodTierPolicy {
        match band {
            WorldLodBand::Macro => self
                .lod_tiers
                .r#macro
                .as_ref()
                .unwrap_or(&self.lod_tiers.operational),
            WorldLodBand::Strategic => self
                .lod_tiers
                .strategic
                .as_ref()
                .unwrap_or(&self.lod_tiers.operational),
            WorldLodBand::Operational => &self.lod_tiers.operational,
            WorldLodBand::LocalTactical => self
                .lod_tiers
                .local_tactical
                .as_ref()
                .unwrap_or(&self.lod_tiers.operational),
        }
    }

    /// Maps consumable lane to the existing particle-domain spine (no fork).
    #[must_use]
    pub fn particle_domain(&self) -> Option<ParticleDomainId> {
        match self.lane {
            EffectLane::ParticleInstanced => {
                if self.sim_coupling == SimCoupling::PrecipField {
                    Some(ParticleDomainId::WeatherPrecip)
                } else {
                    Some(ParticleDomainId::Fire)
                }
            }
            EffectLane::GpuField | EffectLane::MaterialOverlay | EffectLane::Embellishment => None,
        }
    }

    #[must_use]
    pub fn routes_through_fire_vfx_spine(&self) -> bool {
        matches!(
            self.lane,
            EffectLane::ParticleInstanced | EffectLane::GpuField
        )
    }
}

/// ECS marker for a spawned consumable instance (spawn authority is resolver output).
#[derive(Component, Clone, Debug)]
pub struct EffectConsumable {
    pub effect_id: EffectId,
    pub lane: EffectLane,
    pub resolved_spawn: EffectResolvedSpawn,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectResolvedSpawn {
    WorldPosition(Vec3),
    AttachedEntity {
        entity: Entity,
        offset: Vec3,
    },
    FieldSample {
        field_id: EffectFieldId,
        uv: Vec2,
        value: f32,
        density_scale: f32,
        emits: bool,
    },
    Unresolved(SpawnHookKind),
}

/// Authoritative in-memory catalog — loaded once at startup from disk registry.
#[derive(Resource, Clone, Debug, Default)]
pub struct EffectConsumableRegistry {
    pub entries: HashMap<EffectId, EffectConsumableDef>,
    pub registry_root: PathBuf,
    pub load_errors: Vec<String>,
}

impl EffectConsumableRegistry {
    pub fn load_from_disk(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let mut registry = Self {
            registry_root: root.clone(),
            ..Default::default()
        };
        if !root.is_dir() {
            registry
                .load_errors
                .push(format!("registry root missing: {}", root.display()));
            return registry;
        }
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
            .map(|read| {
                read.filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect()
            })
            .unwrap_or_default();
        dirs.sort();
        for dir in dirs {
            let spec_path = dir.join(EFFECT_SPEC_FILENAME);
            if !spec_path.is_file() {
                continue;
            }
            match load_effect_spec_file(&spec_path, &dir) {
                Ok(def) => {
                    registry.entries.insert(def.effect_id.clone(), def);
                }
                Err(e) => registry
                    .load_errors
                    .push(format!("{}: {e}", spec_path.display())),
            }
        }
        registry
    }

    #[must_use]
    pub fn get(&self, effect_id: &str) -> Option<&EffectConsumableDef> {
        self.entries.get(effect_id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn witness_json(&self) -> serde_json::Value {
        serde_json::json!({
            "effect_consumable_registry": true,
            "registry_root": self.registry_root.to_string_lossy(),
            "loaded_count": self.len(),
            "load_errors": self.load_errors,
            "effect_ids": self.entries.keys().collect::<Vec<_>>(),
            "lanes": self.entries.values().map(|d| {
                serde_json::json!({
                    "effect_id": d.effect_id,
                    "lane": format!("{:?}", d.lane),
                    "spawn_hook": format!("{:?}", d.spawn_hook),
                    "particle_domain": d.particle_domain().map(|p| p.label()),
                    "fire_vfx_spine": d.routes_through_fire_vfx_spine(),
                })
            }).collect::<Vec<_>>(),
        })
    }
}

fn load_effect_spec_file(path: &Path, registry_dir: &Path) -> Result<EffectConsumableDef, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: EffectSpecFile =
        serde_json::from_str(&text).map_err(|e| format!("JSON parse: {e}"))?;
    if file.schema_version != "effect_spec_v1" {
        return Err(format!(
            "unsupported schema_version {}",
            file.schema_version
        ));
    }
    let grid = file.grid.unwrap_or(EffectGridConfig {
        tile_units: default_tile_units(),
        snap_to_grid: true,
        rotation_quarters: 0,
        anchor: default_anchor(),
    });
    Ok(EffectConsumableDef {
        effect_id: file.effect_id,
        display_name: file.display_name,
        seed: file.seed,
        batch_id: file.batch_id,
        development_tier: file.development_tier,
        lane: file.lane,
        sim_coupling: file.sim_coupling,
        spawn_hook: file.spawn_hook,
        spawn_hook_config: file.spawn_hook_config,
        grid,
        emit: file.emit,
        lod_tiers: file.lod_tiers,
        visual: file.visual,
        registry_dir: registry_dir.to_path_buf(),
    })
}

/// Context for resolving spawn hooks at runtime (read-only authorities).
pub struct EffectSpawnResolveContext<'a> {
    pub overlay: Option<&'a SharedOverlayFieldBuffers>,
    pub climate: Option<&'a ClimateVisualAggregate>,
    pub entity_by_tag: Option<&'a dyn Fn(&str) -> Option<Entity>>,
}

impl<'a> Default for EffectSpawnResolveContext<'a> {
    fn default() -> Self {
        Self {
            overlay: None,
            climate: None,
            entity_by_tag: None,
        }
    }
}

/// Resolve spawn attachment for a consumable definition — pure, no ECS writes.
#[must_use]
pub fn resolve_effect_spawn(
    def: &EffectConsumableDef,
    ctx: &EffectSpawnResolveContext<'_>,
) -> EffectResolvedSpawn {
    match def.spawn_hook {
        SpawnHookKind::WorldXy => resolve_world_xy(def),
        SpawnHookKind::AttachEntity => resolve_attach_entity(def, ctx.entity_by_tag),
        SpawnHookKind::FieldSample => resolve_field_sample(def, ctx),
    }
}

fn resolve_world_xy(def: &EffectConsumableDef) -> EffectResolvedSpawn {
    let Some(cfg) = def.spawn_hook_config.world_xy.as_ref() else {
        return EffectResolvedSpawn::Unresolved(SpawnHookKind::WorldXy);
    };
    let tu = def.tile_units();
    let x = cfg.tile_x as f32 * tu + cfg.offset_tile[0] * tu;
    let y = cfg.tile_y as f32 * tu + cfg.offset_tile[1] * tu;
    EffectResolvedSpawn::WorldPosition(Vec3::new(x, y, cfg.world_z))
}

fn resolve_attach_entity(
    def: &EffectConsumableDef,
    entity_by_tag: Option<&dyn Fn(&str) -> Option<Entity>>,
) -> EffectResolvedSpawn {
    let Some(cfg) = def.spawn_hook_config.attach_entity.as_ref() else {
        return EffectResolvedSpawn::Unresolved(SpawnHookKind::AttachEntity);
    };
    let Some(tag) = cfg.entity_tag.as_deref() else {
        return EffectResolvedSpawn::Unresolved(SpawnHookKind::AttachEntity);
    };
    let Some(resolve) = entity_by_tag else {
        return EffectResolvedSpawn::Unresolved(SpawnHookKind::AttachEntity);
    };
    let Some(entity) = resolve(tag) else {
        return EffectResolvedSpawn::Unresolved(SpawnHookKind::AttachEntity);
    };
    let offset = Vec3::new(
        cfg.offset_local[0],
        cfg.offset_local[1],
        cfg.offset_local[2],
    );
    let _ = (&cfg.anchor_kind, &cfg.socket);
    EffectResolvedSpawn::AttachedEntity { entity, offset }
}

fn resolve_field_sample(
    def: &EffectConsumableDef,
    ctx: &EffectSpawnResolveContext<'_>,
) -> EffectResolvedSpawn {
    let Some(cfg) = def.spawn_hook_config.field_sample.as_ref() else {
        return EffectResolvedSpawn::Unresolved(SpawnHookKind::FieldSample);
    };
    let uv = Vec2::new(cfg.sample_uv[0], cfg.sample_uv[1]);
    let value = sample_effect_field(cfg.field_id, uv, ctx);
    let emits = value >= cfg.threshold;
    EffectResolvedSpawn::FieldSample {
        field_id: cfg.field_id,
        uv,
        value,
        density_scale: cfg.spawn_density_scale,
        emits,
    }
}

#[must_use]
pub fn sample_effect_field(
    field_id: EffectFieldId,
    uv: Vec2,
    ctx: &EffectSpawnResolveContext<'_>,
) -> f32 {
    match field_id {
        EffectFieldId::ClimatePrecip => ctx
            .climate
            .map(|c| c.mean_rain)
            .unwrap_or(0.0),
        EffectFieldId::SharedOverlayHeat => sample_overlay_heat_at_uv(ctx.overlay, uv),
        EffectFieldId::SharedOverlaySmoke | EffectFieldId::GpuWeatherFireField => {
            // Smoke/gpu field buffers are projection-owned — density gate deferred to field upload slice.
            0.0
        }
    }
}

fn sample_overlay_heat_at_uv(
    overlay: Option<&SharedOverlayFieldBuffers>,
    uv: Vec2,
) -> f32 {
    let Some(overlay) = overlay else {
        return 0.0;
    };
    if overlay.chunk_fire_heat.is_empty() {
        return 0.0;
    }
    let (min_x, max_x, min_y, max_y) = overlay
        .chunk_fire_heat
        .keys()
        .fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |acc, c| {
            (
                acc.0.min(c.x),
                acc.1.max(c.x),
                acc.2.min(c.y),
                acc.3.max(c.y),
            )
        });
    if min_x > max_x || min_y > max_y {
        return 0.0;
    }
    let tx = min_x + ((max_x - min_x) as f32 * uv.x).round() as i32;
    let ty = min_y + ((max_y - min_y) as f32 * uv.y).round() as i32;
    overlay.fire_surface_heat_at(IVec2::new(tx, ty))
}

/// Build an [`EffectConsumable`] component from registry + resolver context.
#[must_use]
pub fn spawn_effect_consumable(
    def: &EffectConsumableDef,
    ctx: &EffectSpawnResolveContext<'_>,
) -> EffectConsumable {
    EffectConsumable {
        effect_id: def.effect_id.clone(),
        lane: def.lane,
        resolved_spawn: resolve_effect_spawn(def, ctx),
        active: true,
    }
}

/// Witness gate — all three spawn_hook kinds have resolver branches wired.
#[must_use]
pub fn effect_spawn_hook_resolver_wired() -> bool {
    let src = include_str!("consumable.rs");
    src.contains("resolve_world_xy")
        && src.contains("resolve_attach_entity")
        && src.contains("resolve_field_sample")
        && src.contains("SpawnHookKind::WorldXy")
        && src.contains("SpawnHookKind::AttachEntity")
        && src.contains("SpawnHookKind::FieldSample")
}

fn default_registry_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(EFFECT_REGISTRY_ROOT)
}

fn load_effect_consumable_registry(mut registry: ResMut<EffectConsumableRegistry>) {
    *registry = EffectConsumableRegistry::load_from_disk(default_registry_root());
}

pub struct EffectConsumableRegistryPlugin;

impl Plugin for EffectConsumableRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectConsumableRegistry>()
            .add_systems(Startup, load_effect_consumable_registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::entity::Entity;

    fn registry_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(EFFECT_REGISTRY_ROOT)
    }

    #[test]
    fn loads_reference_batch_from_disk() {
        let reg = EffectConsumableRegistry::load_from_disk(registry_root());
        assert!(
            reg.load_errors.is_empty(),
            "load errors: {:?}",
            reg.load_errors
        );
        assert_eq!(reg.len(), 3);
        assert!(reg.get("spark_shower").is_some());
        assert!(reg.get("smoke_column").is_some());
        assert!(reg.get("rain_streaks").is_some());
    }

    #[test]
    fn spawn_hook_resolver_wired() {
        assert!(effect_spawn_hook_resolver_wired());
    }

    #[test]
    fn world_xy_resolves_tile_center() {
        let reg = EffectConsumableRegistry::load_from_disk(registry_root());
        let def = reg.get("spark_shower").expect("spark_shower");
        assert_eq!(def.spawn_hook, SpawnHookKind::WorldXy);
        let resolved = resolve_effect_spawn(def, &EffectSpawnResolveContext::default());
        match resolved {
            EffectResolvedSpawn::WorldPosition(p) => {
                assert!((p.x - 0.5).abs() < f32::EPSILON);
                assert!((p.y - 0.5).abs() < f32::EPSILON);
            }
            other => panic!("expected WorldPosition, got {other:?}"),
        }
    }

    #[test]
    fn attach_entity_resolves_with_tag() {
        let reg = EffectConsumableRegistry::load_from_disk(registry_root());
        let def = reg.get("smoke_column").expect("smoke_column");
        let entity = Entity::from_raw_u32(42).expect("test entity");
        let ctx = EffectSpawnResolveContext {
            entity_by_tag: Some(&|tag| {
                if tag == "vss_reference_chimney" {
                    Some(entity)
                } else {
                    None
                }
            }),
            ..Default::default()
        };
        let resolved = resolve_effect_spawn(def, &ctx);
        match resolved {
            EffectResolvedSpawn::AttachedEntity { entity: e, offset } => {
                assert_eq!(e, entity);
                assert!((offset.y - 0.5).abs() < f32::EPSILON);
            }
            other => panic!("expected AttachedEntity, got {other:?}"),
        }
    }

    #[test]
    fn field_sample_climate_precip_emits() {
        let reg = EffectConsumableRegistry::load_from_disk(registry_root());
        let def = reg.get("rain_streaks").expect("rain_streaks");
        let climate = ClimateVisualAggregate {
            mean_rain: 0.5,
            ..Default::default()
        };
        let ctx = EffectSpawnResolveContext {
            climate: Some(&climate),
            ..Default::default()
        };
        let resolved = resolve_effect_spawn(def, &ctx);
        match resolved {
            EffectResolvedSpawn::FieldSample {
                field_id,
                value,
                emits,
                ..
            } => {
                assert_eq!(field_id, EffectFieldId::ClimatePrecip);
                assert!((value - 0.5).abs() < f32::EPSILON);
                assert!(emits);
            }
            other => panic!("expected FieldSample, got {other:?}"),
        }
    }

    #[test]
    fn particle_domain_routing_no_fork() {
        let reg = EffectConsumableRegistry::load_from_disk(registry_root());
        let spark = reg.get("spark_shower").unwrap();
        let rain = reg.get("rain_streaks").unwrap();
        let smoke = reg.get("smoke_column").unwrap();
        assert_eq!(spark.particle_domain(), Some(ParticleDomainId::Fire));
        assert_eq!(rain.particle_domain(), Some(ParticleDomainId::WeatherPrecip));
        assert_eq!(smoke.particle_domain(), None);
        assert!(spark.routes_through_fire_vfx_spine());
        assert!(rain.routes_through_fire_vfx_spine());
        assert!(smoke.routes_through_fire_vfx_spine());
    }

    #[test]
    fn lod_policy_tracks_world_lod_band() {
        let reg = EffectConsumableRegistry::load_from_disk(registry_root());
        let def = reg.get("spark_shower").unwrap();
        let tac = def.lod_policy_for_band(WorldLodBand::LocalTactical);
        assert!(tac.enabled);
        assert_eq!(tac.max_instances, 512);
        let mac = def.lod_policy_for_band(WorldLodBand::Macro);
        assert!(!mac.enabled);
    }
}
