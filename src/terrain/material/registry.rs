//! Material registry + game-specific collision materials (PHYS-04).
//!
//! **Dual-cite (V4-PHYS04 / PHYS-04):**
//! - NEW: gea4v2 14.3.8 Collision Filtering · 14.3.8.1 layers/masks · 14.3.8.3 Game-Specific
//!   Collision Materials — compact `MaterialId` indexes a parallel collision-property table.
//! - LEGACY: `collision_detection` (gea_v2_e4 §14.3 p359) · `physics_collision` (gea_v2_e4 p352) —
//!   filter contacts by category; water/fluid surfaces are not solid (pass-through).
//!
//! Tags on each [`MaterialDef`] drive [`CollisionLayer`] + [`CollisionFilter`] (no rigid-body
//! middleware). Voxel / cell occupation stores [`MaterialId`] only and looks up the same table —
//! one collision-material authority for terrain and voxel surfaces.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};

/// Runtime material variant id (dense index into a loaded registry).
///
/// Also the compact collision-material handle (gea4v2 14.3.8.3): index into
/// [`MaterialRegistry::collision`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaterialId(pub u16);

/// Collision category layer (gea4v2 14.3.8.1) — one primary membership per material.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CollisionLayer {
    Terrain = 0,
    Solid = 1,
    Soft = 2,
    Fluid = 3,
    Vegetation = 4,
    Loose = 5,
    Structure = 6,
    /// Query / ballistic probe layer (not assigned from terrain tags).
    Projectile = 7,
}

impl CollisionLayer {
    pub const COUNT: u32 = 8;

    #[inline]
    pub const fn bit(self) -> u32 {
        1u32 << (self as u8)
    }

    #[inline]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Terrain),
            1 => Some(Self::Solid),
            2 => Some(Self::Soft),
            3 => Some(Self::Fluid),
            4 => Some(Self::Vegetation),
            5 => Some(Self::Loose),
            6 => Some(Self::Structure),
            7 => Some(Self::Projectile),
            _ => None,
        }
    }
}

/// Layer membership + 32-bit collides-with mask (gea4v2 14.3.8.1 `hkpGroupFilter` pattern).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CollisionFilter {
    pub layer: CollisionLayer,
    pub collides_with: u32,
}

impl CollisionFilter {
    /// Symmetric contact gate: each side must include the other's layer bit.
    #[inline]
    pub const fn allows(self, other: Self) -> bool {
        ((self.collides_with & other.layer.bit()) != 0)
            && ((other.collides_with & self.layer.bit()) != 0)
    }
}

/// Compact per-surface collision properties (gea4v2 14.3.8.3), parallel to [`MaterialDef`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionMaterial {
    pub filter: CollisionFilter,
    /// Kinetic friction hint in `[0, 1]` (from tags / `sim.traction_mod`).
    pub friction: f32,
    /// Bounce hint in `[0, 1]` (from hardness / rock tags).
    pub restitution: f32,
    /// False for fluid/water pass-through (book: water surface must not appear solid).
    pub solid_contact: bool,
}

/// Default layer↔layer collides-with masks (one 32-bit word per layer index).
/// Fluid does not generate solid contacts; Projectile hits solid-ish surfaces only.
pub const DEFAULT_LAYER_COLLIDES_WITH: [u32; CollisionLayer::COUNT as usize] = {
    const T: u32 = CollisionLayer::Terrain.bit();
    const S: u32 = CollisionLayer::Solid.bit();
    const O: u32 = CollisionLayer::Soft.bit();
    const V: u32 = CollisionLayer::Vegetation.bit();
    const L: u32 = CollisionLayer::Loose.bit();
    const U: u32 = CollisionLayer::Structure.bit();
    const P: u32 = CollisionLayer::Projectile.bit();
    [
        T | S | O | V | L | U | P, // Terrain
        T | S | O | V | L | U | P, // Solid
        T | S | O | V | L | U | P, // Soft
        0,                         // Fluid — pass-through
        T | S | O | L | U | P,     // Vegetation
        T | S | O | V | L | U | P, // Loose
        T | S | O | V | L | U | P, // Structure
        T | S | O | V | L | U,     // Projectile (no fluid, no self)
    ]
};

/// Tag-driven layer pick (first match wins; order = specificity).
fn collision_layer_from_tags(tags: &[String]) -> CollisionLayer {
    let has = |name: &str| tags.iter().any(|t| t == name);
    if has("fluid") || has("water") || has("deep_water") || has("shallow_water") {
        CollisionLayer::Fluid
    } else if has("built") {
        CollisionLayer::Structure
    } else if has("vegetation") || has("dense_vegetation") || has("forest_canopy") {
        CollisionLayer::Vegetation
    } else if has("soft") || has("marsh") || has("muddy") || has("waterlogged") {
        CollisionLayer::Soft
    } else if has("sand") || has("loose_surface") || has("snow") {
        CollisionLayer::Loose
    } else if has("rock")
        || has("hard")
        || has("hard_bedrock")
        || has("hard_surface")
        || has("rocky")
        || has("cliff")
    {
        CollisionLayer::Solid
    } else {
        CollisionLayer::Terrain
    }
}

fn collision_material_from_def(def: &MaterialDef) -> CollisionMaterial {
    let mut layer = collision_layer_from_tags(&def.tags);
    if let Some(s) = def
        .properties
        .get("sim.collision_layer")
        .and_then(|v| v.as_str())
    {
        layer = match s {
            "terrain" => CollisionLayer::Terrain,
            "solid" => CollisionLayer::Solid,
            "soft" => CollisionLayer::Soft,
            "fluid" => CollisionLayer::Fluid,
            "vegetation" => CollisionLayer::Vegetation,
            "loose" => CollisionLayer::Loose,
            "structure" => CollisionLayer::Structure,
            "projectile" => CollisionLayer::Projectile,
            _ => layer,
        };
    }

    let mut collides_with = DEFAULT_LAYER_COLLIDES_WITH[layer as usize];
    if let Some(mask) = def
        .properties
        .get("sim.collides_with")
        .and_then(|v| v.as_u64())
    {
        collides_with = mask as u32;
    }

    let traction = def.sim_f32("traction_mod").unwrap_or(match layer {
        CollisionLayer::Fluid => 0.15,
        CollisionLayer::Soft => 0.45,
        CollisionLayer::Loose => 0.55,
        CollisionLayer::Vegetation => 0.7,
        CollisionLayer::Solid | CollisionLayer::Structure => 0.95,
        CollisionLayer::Terrain => 0.8,
        CollisionLayer::Projectile => 0.0,
    });
    let friction = (1.0 - traction * 0.35).clamp(0.05, 1.0);
    let hardness = def.facts_f32("rock_hardness").unwrap_or(match layer {
        CollisionLayer::Solid | CollisionLayer::Structure => 0.85,
        CollisionLayer::Soft | CollisionLayer::Fluid => 0.05,
        CollisionLayer::Loose => 0.2,
        _ => 0.4,
    });
    let restitution = (hardness * 0.55).clamp(0.0, 0.9);
    let solid_contact = layer != CollisionLayer::Fluid
        && def
            .properties
            .get("sim.solid_contact")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

    CollisionMaterial {
        filter: CollisionFilter {
            layer,
            collides_with,
        },
        friction,
        restitution,
        solid_contact,
    }
}

/// One row from `material_registry.json` after load (tag names preserved; intern against [`super::TagRegistry`] separately).
///
/// **`family`:** resolved [`TerrainFamilyId`] from the string in JSON via [`TerrainFamilyRegistry`].
///
/// **`properties`:** opaque JSON. Example registries with **`schema_version` ≥ 2** use **dot-separated namespaces**
/// (`facts.*`, `sim.*`, `render.*`, `gen.*`, `mobility.*`, `build.*`, `warfare.*`) — see `material_tag_rule_system_v1.md` §4.1 under `prompts/designer_questions/terrain_world/`.
/// Optional PHYS-04 overrides: `sim.collision_layer`, `sim.collides_with` (u32), `sim.solid_contact`.
#[derive(Clone, Debug)]
pub struct MaterialDef {
    pub name: String,
    pub family: TerrainFamilyId,
    pub tags: Vec<String>,
    pub properties: serde_json::Value,
    pub preview_color: [u8; 4],
}

impl MaterialDef {
    #[inline]
    fn dotted(ns: &str, key: &str) -> String {
        format!("{ns}.{key}")
    }

    pub fn sim_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("sim", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }

    pub fn build_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("build", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }

    pub fn warfare_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("warfare", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }

    pub fn facts_str(&self, key: &str) -> Option<String> {
        self.properties
            .get(&Self::dotted("facts", key))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    pub fn facts_f32(&self, key: &str) -> Option<f32> {
        self.properties
            .get(&Self::dotted("facts", key))
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MaterialDefFile {
    name: String,
    family: String,
    tags: Vec<String>,
    properties: serde_json::Value,
    preview_color: [u8; 4],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MaterialRegistryFile {
    pub schema_version: u32,
    pub materials: Vec<MaterialDefFile>,
}

/// Loaded material table + deterministic name → id map (input row order defines ids).
///
/// [`collision`] is parallel to [`materials`] (same `MaterialId` index) — game-specific
/// collision materials (gea4v2 14.3.8.3), derived from tags at load (sole table writer).
#[derive(Asset, TypePath, Clone, Debug)]
pub struct MaterialRegistry {
    pub schema_version: u32,
    pub materials: Vec<MaterialDef>,
    pub name_to_id: HashMap<String, MaterialId>,
    /// Compact collision props; `collision[id.0 as usize]` for terrain and voxel MaterialId.
    pub collision: Vec<CollisionMaterial>,
}

/// First material definition matching `family` (registry row order — same as `resolve_material` fallback).
pub fn family_default_material_def<'a>(
    registry: &'a MaterialRegistry,
    family: TerrainFamilyId,
) -> Option<&'a MaterialDef> {
    registry.materials.iter().find(|m| m.family == family)
}

/// Schema versions accepted when parsing registry JSON. Bump when breaking; document migration.
pub const SUPPORTED_MATERIAL_REGISTRY_SCHEMA_VERSIONS: &[u32] = &[1, 2];

fn default_family_registry_disk_path() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain/");
    let ron = dir.join("terrain_family_registry.example.ron");
    let json = dir.join("terrain_family_registry.example.json");
    if ron.exists() {
        ron
    } else {
        json
    }
}

impl MaterialRegistry {
    /// Load material registry from **`*.ron`** or **`*.json`**; resolves families via default example registry (`.ron` preferred when present).
    pub fn load_from_json(path: &str) -> std::io::Result<Self> {
        let families = TerrainFamilyRegistry::load_from_path(
            default_family_registry_disk_path()
                .to_str()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "family registry path not utf-8")
                })?,
        )?;
        let file: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(Path::new(path))?;
        Self::from_file(file, &families).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Resolve families using `terrain_family_registry.example.{ron,json}` beside `material_path` when present, else default disk path (RON preferred).
    pub fn load_from_json_with_adjacent_families(material_path: &str) -> std::io::Result<Self> {
        let parent = Path::new(material_path)
            .parent()
            .unwrap_or_else(|| Path::new("."));
        let ron = parent.join("terrain_family_registry.example.ron");
        let json = parent.join("terrain_family_registry.example.json");
        let families_path = if ron.exists() {
            ron
        } else if json.exists() {
            json
        } else {
            default_family_registry_disk_path()
        };
        let families =
            TerrainFamilyRegistry::load_from_path(families_path.to_str().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "family path utf-8")
            })?)?;
        let file: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(Path::new(material_path))?;
        Self::from_file(file, &families).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn from_file(file: MaterialRegistryFile, families: &TerrainFamilyRegistry) -> Result<Self, String> {
        if !SUPPORTED_MATERIAL_REGISTRY_SCHEMA_VERSIONS.contains(&file.schema_version) {
            return Err(format!(
                "Material registry schema_version={} unsupported. Expected one of {:?}. Update assets or run migration.",
                file.schema_version, SUPPORTED_MATERIAL_REGISTRY_SCHEMA_VERSIONS
            ));
        }
        let materials: Vec<MaterialDef> = file
            .materials
            .into_iter()
            .map(|m| {
                let family = families.require_id(&m.family).map_err(|e| {
                    format!("material {:?}: unknown family {:?}: {}", m.name, m.family, e)
                })?;
                Ok(MaterialDef {
                    name: m.name,
                    family,
                    tags: m.tags,
                    properties: m.properties,
                    preview_color: m.preview_color,
                })
            })
            .collect::<Result<_, String>>()?;

        let mut name_to_id = HashMap::new();
        for (i, m) in materials.iter().enumerate() {
            let id = MaterialId(i as u16);
            name_to_id.insert(m.name.clone(), id);
        }
        let collision: Vec<CollisionMaterial> =
            materials.iter().map(collision_material_from_def).collect();
        Ok(Self {
            schema_version: file.schema_version,
            materials,
            name_to_id,
            collision,
        })
    }

    /// Collision material for `id` (compact table lookup — gea4v2 14.3.8.3).
    #[inline]
    pub fn collision_material(&self, id: MaterialId) -> Option<&CollisionMaterial> {
        self.collision.get(id.0 as usize)
    }

    /// True when both materials' layer filters accept the contact (gea4v2 14.3.8.1).
    #[inline]
    pub fn materials_collide(&self, a: MaterialId, b: MaterialId) -> bool {
        match (self.collision_material(a), self.collision_material(b)) {
            (Some(ca), Some(cb)) => ca.filter.allows(cb.filter),
            _ => false,
        }
    }

    /// Solid-contact gate for probes (projectiles / unit feet) vs a surface material.
    #[inline]
    pub fn probe_hits_solid(&self, surface: MaterialId) -> bool {
        self.collision_material(surface)
            .is_some_and(|c| c.solid_contact && c.filter.allows(CollisionFilter {
                layer: CollisionLayer::Projectile,
                collides_with: DEFAULT_LAYER_COLLIDES_WITH[CollisionLayer::Projectile as usize],
            }))
    }
}

/// Bevy loader for `*.material_registry.json`.
#[derive(Default, TypePath)]
pub struct MaterialRegistryLoader;

#[derive(Debug)]
pub enum MaterialRegistryLoaderError {
    Io(std::io::Error),
    Json(String),
}

impl fmt::Display for MaterialRegistryLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for MaterialRegistryLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(_) => None,
        }
    }
}

impl From<std::io::Error> for MaterialRegistryLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for MaterialRegistryLoaderError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}

impl AssetLoader for MaterialRegistryLoader {
    type Asset = MaterialRegistry;
    type Settings = ();
    type Error = MaterialRegistryLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let text = std::str::from_utf8(&bytes).map_err(|e| {
            MaterialRegistryLoaderError::Json(format!("UTF-8: {e}"))
        })?;
        let ext = load_context.path().get_full_extension();
        let file: MaterialRegistryFile =
            crate::terrain::registry_serde_path::deserialize_from_str_with_extension_opt(text, ext.as_deref())
                .map_err(|e| MaterialRegistryLoaderError::Json(e.to_string()))?;

        let families = TerrainFamilyRegistry::load_from_path(
            default_family_registry_disk_path().to_str().ok_or_else(|| {
                MaterialRegistryLoaderError::Json("default family registry path not utf-8".into())
            })?,
        )?;

        MaterialRegistry::from_file(file, &families).map_err(MaterialRegistryLoaderError::Json)
    }

    fn extensions(&self) -> &[&str] {
        &["material_registry.json", "material_registry.ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_material_registry_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/material_registry.example.json")
    }

    fn families() -> TerrainFamilyRegistry {
        TerrainFamilyRegistry::load_from_path(
            default_family_registry_disk_path().to_str().unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn material_registry_loads_example() {
        let path = example_material_registry_path();
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        assert_eq!(reg.schema_version, 2);
        assert!(reg.materials.len() >= 4);
        assert_eq!(reg.name_to_id.get("loam_wet"), Some(&MaterialId(2)));
    }

    #[test]
    fn sim_f32_reads_namespaced_key() {
        let path = example_material_registry_path();
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        let loam = &reg.materials[reg.name_to_id["loam_wet"].0 as usize];
        assert!((loam.sim_f32("traction_mod").unwrap() - 0.8).abs() < 1e-4);
        assert!((loam.sim_f32("water_retention").unwrap() - 0.9).abs() < 1e-4);
    }

    #[test]
    fn material_registry_rejects_unknown_schema_version() {
        let raw = r#"{"schema_version":99,"materials":[]}"#;
        let file: MaterialRegistryFile = serde_json::from_str(raw).unwrap();
        let fam = families();
        assert!(MaterialRegistry::from_file(file, &fam).is_err());
    }

    #[test]
    fn material_registry_loader_extension() {
        let loader = MaterialRegistryLoader::default();
        assert!(loader.extensions().contains(&"material_registry.json"));
        assert!(loader.extensions().contains(&"material_registry.ron"));
    }

    #[test]
    fn material_example_json_round_trips_ron_wire_format() {
        let path = example_material_registry_path();
        let s = std::fs::read_to_string(&path).unwrap();
        let file: MaterialRegistryFile = serde_json::from_str(&s).unwrap();
        let cfg = ron::ser::PrettyConfig::new().depth_limit(64).indentor("    ".into());
        let ron_s = ron::ser::to_string_pretty(&file, cfg).unwrap();
        let file2: MaterialRegistryFile = ron::de::from_str(&ron_s).unwrap();
        assert_eq!(
            serde_json::to_value(&file).unwrap(),
            serde_json::to_value(&file2).unwrap()
        );
    }

    #[test]
    fn material_minimal_ron_loads_via_path() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/material_registry_minimal.example.ron");
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        assert_eq!(reg.schema_version, 2);
        assert_eq!(reg.materials.len(), 1);
        assert!(reg.name_to_id.contains_key("water_deep"));
    }

    #[test]
    fn material_full_example_json_and_ron_decode_to_same_file() {
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain");
        let j: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(
                &dir.join("material_registry.example.json"),
            )
            .unwrap();
        let r: MaterialRegistryFile =
            crate::terrain::registry_serde_path::read_to_deserializable(
                &dir.join("material_registry.example.ron"),
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(&j).unwrap(),
            serde_json::to_value(&r).unwrap()
        );
    }

    #[test]
    fn material_full_example_json_and_ron_load_same_registry() {
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/config/terrain");
        let a =
            MaterialRegistry::load_from_json(dir.join("material_registry.example.json").to_str().unwrap())
                .unwrap();
        let b =
            MaterialRegistry::load_from_json(dir.join("material_registry.example.ron").to_str().unwrap())
                .unwrap();
        assert_eq!(a.schema_version, b.schema_version);
        assert_eq!(a.materials.len(), b.materials.len());
        assert_eq!(a.name_to_id, b.name_to_id);
        assert_eq!(a.collision.len(), b.collision.len());
        for (ma, mb) in a.materials.iter().zip(b.materials.iter()) {
            assert_eq!(ma.name, mb.name);
            assert_eq!(ma.family, mb.family);
            assert_eq!(ma.tags, mb.tags);
            assert_eq!(ma.properties, mb.properties);
            assert_eq!(ma.preview_color, mb.preview_color);
        }
        for (ca, cb) in a.collision.iter().zip(b.collision.iter()) {
            assert_eq!(ca.filter, cb.filter);
            assert_eq!(ca.solid_contact, cb.solid_contact);
        }
    }

    #[test]
    #[ignore = "Regenerates material_registry.example.ron from JSON. Run: cargo test emit_material_registry_example_ron_fixture -- --ignored --nocapture"]
    fn emit_material_registry_example_ron_fixture() {
        let path = example_material_registry_path();
        let s = std::fs::read_to_string(&path).unwrap();
        let file: MaterialRegistryFile = serde_json::from_str(&s).unwrap();
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/material_registry.example.ron");
        let cfg = ron::ser::PrettyConfig::new().depth_limit(64).indentor("    ".into());
        let ron_s = ron::ser::to_string_pretty(&file, cfg).unwrap();
        std::fs::write(&out, format!("{}\n", ron_s.trim_end())).unwrap();
    }

    // --- PHYS-04 / V4-PHYS04 tripwires ---

    #[test]
    fn phys04_tags_drive_layers_and_filters() {
        let path = example_material_registry_path();
        let reg = MaterialRegistry::load_from_json(path.to_str().unwrap()).unwrap();
        assert_eq!(reg.collision.len(), reg.materials.len());

        let water = reg.name_to_id["water_deep"];
        let basalt = reg.name_to_id["basalt_dense"];
        let marsh = reg.name_to_id["silt_marsh"];
        let turf = reg.name_to_id["turf_forest"];
        let sand = reg.name_to_id["sand_dry"];

        assert_eq!(
            reg.collision_material(water).unwrap().filter.layer,
            CollisionLayer::Fluid
        );
        assert!(!reg.collision_material(water).unwrap().solid_contact);
        assert_eq!(
            reg.collision_material(basalt).unwrap().filter.layer,
            CollisionLayer::Solid
        );
        assert_eq!(
            reg.collision_material(marsh).unwrap().filter.layer,
            CollisionLayer::Soft
        );
        assert_eq!(
            reg.collision_material(turf).unwrap().filter.layer,
            CollisionLayer::Vegetation
        );
        assert_eq!(
            reg.collision_material(sand).unwrap().filter.layer,
            CollisionLayer::Loose
        );

        // Book water example: fluid surface is not solid to probes.
        assert!(!reg.probe_hits_solid(water));
        assert!(reg.probe_hits_solid(basalt));
        assert!(reg.probe_hits_solid(marsh));

        // Symmetric layer mask: solid↔solid ok; projectile↔fluid rejected.
        assert!(reg.materials_collide(basalt, basalt));
        let projectile_filter = CollisionFilter {
            layer: CollisionLayer::Projectile,
            collides_with: DEFAULT_LAYER_COLLIDES_WITH[CollisionLayer::Projectile as usize],
        };
        assert!(!reg
            .collision_material(water)
            .unwrap()
            .filter
            .allows(projectile_filter));
        assert!(reg
            .collision_material(basalt)
            .unwrap()
            .filter
            .allows(projectile_filter));
    }

    #[test]
    fn phys04_property_override_and_compact_id_index() {
        let fam = families();
        let file = MaterialRegistryFile {
            schema_version: 2,
            materials: vec![MaterialDefFile {
                name: "override_fluidish".into(),
                family: "Grassland".into(),
                tags: vec!["soil".into()],
                properties: serde_json::json!({
                    "sim.collision_layer": "fluid",
                    "sim.collides_with": 0u64,
                    "sim.solid_contact": false
                }),
                preview_color: [1, 2, 3, 255],
            }],
        };
        let reg = MaterialRegistry::from_file(file, &fam).unwrap();
        let id = MaterialId(0);
        let cm = reg.collision_material(id).unwrap();
        assert_eq!(cm.filter.layer, CollisionLayer::Fluid);
        assert_eq!(cm.filter.collides_with, 0);
        assert!(!cm.solid_contact);
        // Compact index: MaterialId is the table handle (voxel cells store only this).
        assert_eq!(reg.collision.len(), 1);
        assert!(std::ptr::eq(
            reg.collision_material(id).unwrap(),
            &reg.collision[0]
        ));
    }

    #[test]
    fn phys04_default_layer_mask_table_is_sized() {
        assert_eq!(
            DEFAULT_LAYER_COLLIDES_WITH.len(),
            CollisionLayer::COUNT as usize
        );
        assert_eq!(DEFAULT_LAYER_COLLIDES_WITH[CollisionLayer::Fluid as usize], 0);
        assert_ne!(
            DEFAULT_LAYER_COLLIDES_WITH[CollisionLayer::Solid as usize] & CollisionLayer::Projectile.bit(),
            0
        );
    }
}
