//! MCP G5 module catalog — `assets/configs/buildings/_module_index.ron` (module_id → glb).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

pub const MODULE_INDEX_RON: &str = "assets/configs/buildings/_module_index.ron";
pub const MODULE_INDEX_JSON: &str = "assets/configs/buildings/_module_index.json";

/// Production tier lane for procedural module index rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevelopmentTier {
    Smoke,
    Lod0,
    Production,
}

impl DevelopmentTier {
    #[must_use]
    pub fn parse(explicit: &str, batch_id: &str) -> Self {
        match explicit {
            "production" => Self::Production,
            "lod0" => Self::Lod0,
            "smoke" => Self::Smoke,
            _ if batch_id.starts_with("kit_greybox") || batch_id.starts_with("kit_smoke") => {
                Self::Smoke
            }
            _ if batch_id.starts_with("kit_lod0") => Self::Lod0,
            _ if batch_id.starts_with("kit_production") => Self::Production,
            _ => Self::Smoke,
        }
    }

    #[must_use]
    pub fn is_smoke(self) -> bool {
        self == Self::Smoke
    }

    /// Map/tactical iso stamp — production atlases only ([`super::tile_atlas_index::TileAtlasEntry::runtime_stamp_allowed`]).
    #[must_use]
    pub fn atlas_runtime_stamp_allowed(self) -> bool {
        self == Self::Production
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleIndexFileRon {
    #[serde(default)]
    schema_version: u32,
    entries: Vec<ModuleIndexEntryRon>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleIndexEntryRon {
    module_id: String,
    job_id: String,
    category: String,
    glb: String,
    grid_units: (u32, u32),
    #[serde(default)]
    style_tags: Vec<String>,
    #[serde(default)]
    batch_id: String,
    #[serde(default)]
    development_tier: String,
    #[serde(default)]
    pbr_status: String,
    #[serde(default)]
    stylepack_visible: bool,
    #[serde(default)]
    replaced_by: Option<String>,
    #[serde(default)]
    archetype: String,
    #[serde(default)]
    style_pack: String,
    #[serde(default)]
    snap: String,
    #[serde(default)]
    material_profile: String,
    #[serde(default)]
    palette_family: String,
    #[serde(default)]
    palette_variation_count: u8,
    #[serde(default)]
    default_variation_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleIndexFileJson {
    #[serde(default)]
    schema_version: u32,
    entries: Vec<ModuleIndexEntryJson>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleIndexEntryJson {
    module_id: String,
    job_id: String,
    #[serde(default)]
    category: String,
    glb: String,
    #[serde(default)]
    glb_path: Option<String>,
    grid_units: [u32; 2],
    #[serde(default)]
    style_tags: Vec<String>,
    #[serde(default)]
    batch_id: String,
    #[serde(default)]
    development_tier: String,
    #[serde(default)]
    pbr_status: String,
    #[serde(default)]
    stylepack_visible: bool,
    #[serde(default)]
    replaced_by: Option<String>,
    #[serde(default)]
    archetype: String,
    #[serde(default)]
    style_pack: String,
    #[serde(default)]
    snap: String,
    #[serde(default)]
    material_profile: String,
    #[serde(default)]
    palette_family: String,
    #[serde(default)]
    palette_variation_count: u8,
    #[serde(default)]
    default_variation_id: String,
}

/// One promoted procedural module row from the MCP library index.
#[derive(Debug, Clone)]
pub struct ProceduralModuleEntry {
    pub module_id: String,
    pub job_id: String,
    pub category: String,
    /// Repo-relative path (`assets/models/modules/.../model.glb`).
    pub glb_path: String,
    /// Bevy `AssetServer` path (no `assets/` prefix).
    pub glb_asset: String,
    pub grid_units: (u32, u32),
    pub style_tags: Vec<String>,
    pub batch_id: String,
    pub development_tier: DevelopmentTier,
    pub pbr_status: String,
    pub stylepack_visible: bool,
    pub replaced_by: Option<String>,
    pub archetype: String,
    pub style_pack: String,
    pub snap: String,
    pub material_profile: String,
    pub palette_family: String,
    pub palette_variation_count: u8,
    pub default_variation_id: String,
}

impl ProceduralModuleEntry {
    #[must_use]
    pub fn visible_in_stylepack(&self) -> bool {
        self.stylepack_visible && !self.development_tier.is_smoke()
    }
}

#[derive(Resource, Debug, Default)]
pub struct ProceduralModuleRegistry {
    pub schema_version: u32,
    pub entries: Vec<ProceduralModuleEntry>,
    /// Best-tier row per `module_id` (lod0/production wins over smoke).
    pub by_module_id: HashMap<String, ProceduralModuleEntry>,
    pub by_job_id: HashMap<String, String>,
    pub load_errors: Vec<String>,
}

impl ProceduralModuleRegistry {
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn get(&self, module_id: &str) -> Option<&ProceduralModuleEntry> {
        self.by_module_id.get(module_id)
    }

    #[must_use]
    pub fn glb_path(&self, module_id: &str) -> Option<&str> {
        self.stylepack_entry(module_id)
            .map(|e| e.glb_path.as_str())
            .or_else(|| self.get(module_id).map(|e| e.glb_path.as_str()))
    }

    #[must_use]
    pub fn glb_asset(&self, module_id: &str) -> Option<&str> {
        self.stylepack_glb_asset(module_id)
    }

    /// StylePack / PG-2 path — never returns smoke-tier GLBs.
    #[must_use]
    pub fn stylepack_glb_asset(&self, module_id: &str) -> Option<&str> {
        self.stylepack_entry(module_id)
            .map(|e| e.glb_asset.as_str())
    }

    #[must_use]
    pub fn stylepack_entry(&self, module_id: &str) -> Option<&ProceduralModuleEntry> {
        self.stylepack_entry_for(module_id, None).0
    }

    /// StylePack / PG-2 path — optional `style_pack_id` filters before tier preference.
    #[must_use]
    pub fn stylepack_entry_for(
        &self,
        module_id: &str,
        style_pack_id: Option<&str>,
    ) -> (Option<&ProceduralModuleEntry>, StylePackResolveMeta) {
        let canonical = self.resolve_canonical_module_id(module_id);
        pick_stylepack_entry(
            self.entries
                .iter()
                .filter(|entry| entry.module_id == canonical && entry.visible_in_stylepack()),
            style_pack_id,
        )
    }

    #[must_use]
    pub fn stylepack_glb_asset_for(
        &self,
        module_id: &str,
        style_pack_id: Option<&str>,
    ) -> Option<&str> {
        self.stylepack_entry_for(module_id, style_pack_id)
            .0
            .map(|e| e.glb_asset.as_str())
    }

    /// PG-2 / StylePack path — canonical lod0+ row; never returns smoke-tier entries.
    #[must_use]
    pub fn resolve_module_id_for(
        &self,
        module_id: &str,
        style_pack_id: Option<&str>,
    ) -> (Option<&ProceduralModuleEntry>, StylePackResolveMeta) {
        self.stylepack_entry_for(module_id, style_pack_id)
    }

    /// Legacy tier-only resolve (no style filter) — prefer [`resolve_module_id_for`].
    #[must_use]
    pub fn resolve_module_id(&self, module_id: &str) -> Option<&ProceduralModuleEntry> {
        self.stylepack_entry(module_id)
    }

    /// Follow `replaced_by` from legacy smoke ids to canonical kit inventory ids.
    #[must_use]
    pub fn resolve_canonical_module_id<'a>(&'a self, module_id: &'a str) -> &'a str {
        let mut current = module_id;
        let mut seen = HashSet::new();
        loop {
            let Some(entry) = self.by_module_id.get(current) else {
                return current;
            };
            let Some(ref target) = entry.replaced_by else {
                return current;
            };
            if target == current || !seen.insert(target.clone()) {
                return current;
            }
            current = target;
        }
    }

    #[must_use]
    pub fn modules_for_stylepack(&self) -> impl Iterator<Item = &ProceduralModuleEntry> {
        self.entries
            .iter()
            .filter(|e| e.visible_in_stylepack())
    }

    #[must_use]
    pub fn modules_for_assembly(&self) -> impl Iterator<Item = &ProceduralModuleEntry> {
        self.modules_for_stylepack()
    }

    #[must_use]
    pub fn module_id_for_job(&self, job_id: &str) -> Option<&str> {
        self.by_job_id.get(job_id).map(|s| s.as_str())
    }
}

#[must_use]
fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

#[must_use]
pub fn default_module_index_ron_path() -> PathBuf {
    repo_asset_path(MODULE_INDEX_RON)
}

/// Resolution metadata for style-pack module lookup (**BQ-F2-STYLE-001**).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StylePackResolveMeta {
    pub cross_style_fallback: bool,
}

pub const BQ_F2_STYLE_001_LIVE_JSON: &str = "debug_runs/bq_f2_style_001_live.json";
fn style_pack_matches(entry: &ProceduralModuleEntry, style_pack_id: &str) -> bool {
    entry.style_pack == style_pack_id
}

#[must_use]
fn pick_stylepack_entry<'a>(
    entries: impl Iterator<Item = &'a ProceduralModuleEntry>,
    style_pack_id: Option<&str>,
) -> (Option<&'a ProceduralModuleEntry>, StylePackResolveMeta) {
    let mut tier_best: Option<&ProceduralModuleEntry> = None;
    let mut style_best: Option<&ProceduralModuleEntry> = None;

    for entry in entries {
        tier_best = Some(match tier_best {
            None => entry,
            Some(cur) => prefer_stylepack_tier(entry, cur),
        });
        if style_pack_id.is_some_and(|id| style_pack_matches(entry, id)) {
            style_best = Some(match style_best {
                None => entry,
                Some(cur) => prefer_stylepack_tier(entry, cur),
            });
        }
    }

    if let Some(style_id) = style_pack_id {
        if let Some(best) = style_best {
            return (Some(best), StylePackResolveMeta::default());
        }
        if let Some(fallback) = tier_best {
            warn!(
                target: "procedural_module",
                module_id = %fallback.module_id,
                requested_style = %style_id,
                resolved_style = %fallback.style_pack,
                job_id = %fallback.job_id,
                "stylepack cross-style fallback (BQ-F2)"
            );
            return (
                Some(fallback),
                StylePackResolveMeta {
                    cross_style_fallback: true,
                },
            );
        }
        return (None, StylePackResolveMeta::default());
    }

    (tier_best, StylePackResolveMeta::default())
}

/// PG-2 / assembly prefers **lod0** over production when both are stylepack-visible.
fn prefer_stylepack_tier<'a>(
    candidate: &'a ProceduralModuleEntry,
    current: &'a ProceduralModuleEntry,
) -> &'a ProceduralModuleEntry {
    use DevelopmentTier::{Lod0, Production};
    match (candidate.development_tier, current.development_tier) {
        (Lod0, Production) => candidate,
        (Production, Lod0) => current,
        _ if candidate.development_tier > current.development_tier => candidate,
        _ => current,
    }
}

fn glb_to_asset_path(glb: &str) -> String {
    let trimmed = glb.trim_start_matches('\\').trim_start_matches('/');
    let stripped = trimmed
        .strip_prefix("assets/")
        .or_else(|| trimmed.strip_prefix("assets\\"))
        .unwrap_or(trimmed);
    stripped.replace('\\', "/")
}

fn infer_stylepack_visible(tier: DevelopmentTier, file: bool) -> bool {
    !tier.is_smoke() && (file || true)
}

fn normalize_entry(
    module_id: String,
    job_id: String,
    category: String,
    glb_path: String,
    grid_units: (u32, u32),
    style_tags: Vec<String>,
    batch_id: String,
    development_tier: String,
    pbr_status: String,
    stylepack_visible: bool,
    replaced_by: Option<String>,
    archetype: String,
    style_pack: String,
    snap: String,
    material_profile: String,
    palette_family: String,
    palette_variation_count: u8,
    default_variation_id: String,
) -> ProceduralModuleEntry {
    let tier = DevelopmentTier::parse(&development_tier, &batch_id);
    let glb_asset = glb_to_asset_path(&glb_path);
    ProceduralModuleEntry {
        module_id,
        job_id,
        category,
        glb_path,
        glb_asset,
        grid_units,
        style_tags,
        batch_id,
        development_tier: tier,
        pbr_status,
        stylepack_visible: infer_stylepack_visible(tier, stylepack_visible),
        replaced_by,
        archetype,
        style_pack,
        snap,
        material_profile,
        palette_family,
        palette_variation_count,
        default_variation_id,
    }
}

fn entry_from_ron(raw: ModuleIndexEntryRon) -> ProceduralModuleEntry {
    let glb_path = raw.glb.replace('\\', "/");
    normalize_entry(
        raw.module_id,
        raw.job_id,
        raw.category,
        glb_path,
        raw.grid_units,
        raw.style_tags,
        raw.batch_id,
        raw.development_tier,
        raw.pbr_status,
        raw.stylepack_visible,
        raw.replaced_by,
        raw.archetype,
        raw.style_pack,
        raw.snap,
        raw.material_profile,
        raw.palette_family,
        raw.palette_variation_count,
        raw.default_variation_id,
    )
}

fn entry_from_json(raw: ModuleIndexEntryJson) -> ProceduralModuleEntry {
    let glb_path = raw
        .glb_path
        .filter(|s| !s.is_empty())
        .unwrap_or(raw.glb)
        .replace('\\', "/");
    let grid = (
        raw.grid_units.first().copied().unwrap_or(1),
        raw.grid_units.get(1).copied().unwrap_or(1),
    );
    normalize_entry(
        raw.module_id,
        raw.job_id,
        raw.category,
        glb_path,
        grid,
        raw.style_tags,
        raw.batch_id,
        raw.development_tier,
        raw.pbr_status,
        raw.stylepack_visible,
        raw.replaced_by,
        raw.archetype,
        raw.style_pack,
        raw.snap,
        raw.material_profile,
        raw.palette_family,
        raw.palette_variation_count,
        raw.default_variation_id,
    )
}

fn ingest_entries(registry: &mut ProceduralModuleRegistry, entries: impl Iterator<Item = ProceduralModuleEntry>) {
    for entry in entries {
        let module_id = entry.module_id.clone();
        let job_id = entry.job_id.clone();
        registry.by_job_id.insert(job_id, module_id.clone());
        registry.entries.push(entry.clone());
        match registry.by_module_id.get(&module_id) {
            None => {
                registry.by_module_id.insert(module_id, entry);
            }
            Some(existing) => {
                if entry.development_tier > existing.development_tier {
                    registry.by_module_id.insert(module_id, entry);
                }
            }
        }
    }
}

#[must_use]
pub fn load_procedural_module_registry_from_path(path: &Path) -> ProceduralModuleRegistry {
    let mut registry = ProceduralModuleRegistry::default();
    let Ok(text) = std::fs::read_to_string(path) else {
        registry
            .load_errors
            .push(format!("module index not found: {}", path.display()));
        return registry;
    };

    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        match serde_json::from_str::<ModuleIndexFileJson>(&text) {
            Ok(file) => {
                registry.schema_version = file.schema_version;
                ingest_entries(
                    &mut registry,
                    file.entries.into_iter().map(entry_from_json),
                );
            }
            Err(e) => registry.load_errors.push(format!("JSON parse: {e}")),
        }
    } else {
        match ron::from_str::<ModuleIndexFileRon>(&text) {
            Ok(file) => {
                registry.schema_version = file.schema_version;
                ingest_entries(
                    &mut registry,
                    file.entries.into_iter().map(entry_from_ron),
                );
            }
            Err(e) => registry.load_errors.push(format!("RON parse: {e}")),
        }
    }
    registry
}

#[must_use]
pub fn load_procedural_module_registry() -> ProceduralModuleRegistry {
    let ron_path = default_module_index_ron_path();
    if ron_path.is_file() {
        return load_procedural_module_registry_from_path(&ron_path);
    }
    let json_path = repo_asset_path(MODULE_INDEX_JSON);
    if json_path.is_file() {
        return load_procedural_module_registry_from_path(&json_path);
    }
    let mut registry = ProceduralModuleRegistry::default();
    registry
        .load_errors
        .push("missing _module_index.ron and _module_index.json".into());
    registry
}

pub fn init_procedural_module_registry(mut commands: Commands) {
    let registry = load_procedural_module_registry();
    if !registry.load_errors.is_empty() {
        for err in &registry.load_errors {
            warn!(target: "procedural_module", "{err}");
        }
    } else {
        info!(
            target: "procedural_module",
            "ProceduralModuleRegistry: {} entries ({} stylepack-visible) schema_v{}",
            registry.len(),
            registry.modules_for_stylepack().count(),
            registry.schema_version
        );
    }
    commands.insert_resource(registry);
}

#[must_use]
pub fn build_bq_f2_style_001_witness_body() -> serde_json::Value {
    use crate::render::extraction::{
        assemble_procedural_build_instances, ProceduralModuleSceneCatalog,
    };

    let reg = load_procedural_module_registry();
    let packs = super::load::load_style_pack_registry();
    let table_ok = reg.load_errors.is_empty() && packs.load_errors.is_empty();

    let (victorian_entry, victorian_meta) =
        reg.stylepack_entry_for("wall_brick_1u", Some("style_victorian"));
    let (ungated_entry, _) = reg.stylepack_entry_for("wall_brick_1u", None);
    let victorian_style_ok = victorian_entry
        .is_some_and(|e| e.style_pack == "style_victorian" && !victorian_meta.cross_style_fallback);
    let tier_only_differs = match (victorian_entry, ungated_entry) {
        (Some(v), Some(u)) => v.job_id != u.job_id,
        _ => false,
    };

    let mut cross_style_fallback_count = 0u32;
    if let Some(pack) = packs.get("style_victorian") {
        let request = crate::construction::procedural::ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width: 4,
            depth: 2,
            floors: 2,
            style: crate::construction::procedural::StylePackId("style_victorian".into()),
            seed: 1,
            arch_dna_preset_id: None,
        };
        let grid = crate::construction::procedural::FootprintGrid::from_request(&request);
        let extract = assemble_procedural_build_instances(
            &request,
            pack,
            &grid,
            &reg,
            &ProceduralModuleSceneCatalog::default(),
        );
        cross_style_fallback_count = extract.cross_style_fallback_count;
    }

    let green = table_ok && victorian_style_ok && cross_style_fallback_count == 0;

    serde_json::json!({
        "gate": "BQ-F2-STYLE-001",
        "green": green,
        "table_ok": table_ok,
        "victorian_wall_brick_style_ok": victorian_style_ok,
        "tier_only_picks_different_job": tier_only_differs,
        "cross_style_fallback_count": cross_style_fallback_count,
        "victorian_wall_job_id": victorian_entry.map(|e| e.job_id.clone()),
        "ungated_wall_job_id": ungated_entry.map(|e| e.job_id.clone()),
    })
}

#[must_use]
pub fn bq_f2_style_001_witness_green() -> bool {
    build_bq_f2_style_001_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_bq_f2_style_001_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_f2_style_001_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-F2-STYLE-001",
        "refresh_bq_f2_style_001_witness",
        BQ_F2_STYLE_001_LIVE_JSON,
        body,
    );
    write_debug_run_json(BQ_F2_STYLE_001_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_kit_greybox_module_index() {
        let reg = load_procedural_module_registry();
        assert!(
            reg.load_errors.is_empty(),
            "errors: {:?}",
            reg.load_errors
        );
        assert!(reg.len() >= 10);
        let wall = reg.get("wall_concrete_2u").expect("wall_concrete_2u");
        assert_eq!(wall.job_id, "wall_concrete_2u_production_run001");
        assert!(wall.glb_path.contains("wall_concrete_2u_production_run001"));
        assert_eq!(wall.palette_family, "palette_industrial_west");
        assert!(wall.glb_asset.starts_with("models/modules/"));
        assert_eq!(
            reg.module_id_for_job("wall_concrete_2u_lod0_run001"),
            Some("wall_concrete_2u")
        );
    }

    #[test]
    fn smoke_excluded_from_stylepack_iterator() {
        let reg = load_procedural_module_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        let stylepack_ids: HashSet<_> = reg
            .modules_for_stylepack()
            .map(|e| e.module_id.as_str())
            .collect();
        assert!(!stylepack_ids.contains("door_shop_1u"));
        for entry in reg.modules_for_stylepack() {
            assert!(!entry.development_tier.is_smoke());
            assert!(entry.stylepack_visible);
        }
    }

    #[test]
    fn lod0_included_in_stylepack_iterator() {
        let reg = load_procedural_module_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        let stylepack_ids: HashSet<_> = reg
            .modules_for_stylepack()
            .map(|e| e.module_id.as_str())
            .collect();
        assert!(stylepack_ids.contains("wall_brick_1u"));
        assert!(stylepack_ids.contains("door_residential"));
        assert!(stylepack_ids.contains("win_single_1u"));
    }

    #[test]
    fn wall_brick_1u_resolves_to_lod0_job() {
        let reg = load_procedural_module_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        let entry = reg
            .stylepack_entry("wall_brick_1u")
            .expect("lod0 wall_brick_1u");
        assert_eq!(entry.job_id, "wall_brick_1u_lod0_run001");
        assert_eq!(entry.development_tier, DevelopmentTier::Lod0);
        let asset = reg
            .stylepack_glb_asset("wall_brick_1u")
            .expect("lod0 glb asset");
        assert!(asset.contains("wall_brick_1u_lod0_run001"));
    }

    #[test]
    fn resolve_canonical_follows_replaced_by_alias() {
        let reg = load_procedural_module_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        assert_eq!(
            reg.resolve_canonical_module_id("door_residential_1u"),
            "door_residential"
        );
        assert_eq!(
            reg.resolve_canonical_module_id("window_single_1u"),
            "win_single_1u"
        );
        let asset = reg
            .stylepack_glb_asset("door_residential_1u")
            .expect("lod0 via alias");
        assert!(asset.contains("door_residential_lod0_run001"));
    }

    #[test]
    fn smoke_only_module_returns_none_for_stylepack_glb() {
        let reg = load_procedural_module_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        assert!(reg.stylepack_glb_asset("corner_brick_outer").is_none());
        assert!(reg.resolve_module_id("corner_brick_outer").is_none());
    }

    #[test]
    fn resolve_module_id_alias_for_lod0() {
        let reg = load_procedural_module_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        let entry = reg
            .resolve_module_id("wall_brick_1u")
            .expect("resolve_module_id lod0");
        assert_eq!(entry.development_tier, DevelopmentTier::Lod0);
    }

    #[test]
    fn bq_f2_victorian_wall_brick_never_picks_industrial_lod0() {
        let reg = load_procedural_module_registry();
        assert!(reg.load_errors.is_empty(), "{:?}", reg.load_errors);
        let (entry, meta) = reg.stylepack_entry_for("wall_brick_1u", Some("style_victorian"));
        let entry = entry.expect("victorian wall_brick");
        assert_eq!(entry.style_pack, "style_victorian");
        assert!(!meta.cross_style_fallback);
        assert_eq!(entry.development_tier, DevelopmentTier::Production);
        let (ungated, _) = reg.stylepack_entry_for("wall_brick_1u", None);
        assert_eq!(
            ungated.map(|e| e.style_pack.as_str()),
            Some("style_industrial_west")
        );
    }

    #[test]
    fn bq_f2_style_001_witness_green_lib() {
        assert!(super::bq_f2_style_001_witness_green());
    }

    #[test]
    fn bq_f2_style_001_live_witness_refresh_green() {
        assert!(super::refresh_bq_f2_style_001_witness());
    }
}
