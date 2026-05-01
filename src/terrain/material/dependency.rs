//! Per-chunk dependency hashes + dirty flags (material unification U7).

use std::hash::{Hash, Hasher};

use bevy::prelude::{Component, IVec2};
use serde::Serialize;

use super::registry::MaterialRegistry;
use super::rules::RuleSet;
use super::tags::TagRegistry;
use crate::terrain::biome::BiomeTuning;
use crate::terrain::generation::terrain_noise::{NoiseSamplingTuning, TerrainNoiseProfile};
use crate::terrain::generation::world_generator_enhanced::{RegionMethod, WorldGenParams};

/// Stable 64-bit hash for any value that implements [`Hash`].
pub fn hash_asset<T: Hash>(asset: &T) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    asset.hash(&mut s);
    s.finish()
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkDependency {
    pub source_noise_id: u64,
    pub registry_hash: u64,
    pub rules_hash: u64,
    pub tags_hash: u64,
    pub tuning_hash: u64,
}

/// Bit *i* set ⇒ pass *(i+1)* must be re-run (`0b00111111` = passes 1–6).
#[derive(Component, Default, Debug)]
pub struct ChunkDirty {
    pub passes: u8,
}

pub const DIRTY_PASS1: u8 = 1 << 0;
pub const DIRTY_PASS2: u8 = 1 << 1;
pub const DIRTY_PASS3: u8 = 1 << 2;
pub const DIRTY_PASS4: u8 = 1 << 3;
pub const DIRTY_PASS5: u8 = 1 << 4;
pub const DIRTY_PASS6: u8 = 1 << 5;
/// Passes 2 through 6 (tag / classify / hydrology / agent / materialize).
pub const DIRTY_PASSES_2_THROUGH_6: u8 = DIRTY_PASS2 | DIRTY_PASS3 | DIRTY_PASS4 | DIRTY_PASS5 | DIRTY_PASS6;
pub const DIRTY_ALL: u8 = 0b0011_1111;

fn combine_u64(a: u64, b: u64) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    a.hash(&mut s);
    b.hash(&mut s);
    s.finish()
}

fn hash_json<T: Serialize>(v: &T) -> u64 {
    let Ok(bytes) = serde_json::to_vec(v) else {
        return 0;
    };
    let mut s = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut s);
    s.finish()
}

#[derive(Serialize)]
struct Pass1Fingerprint<'a> {
    seed: u64,
    noise_scale: f32,
    noise_octaves: u32,
    noise_lacunarity: f32,
    noise_persistence: f32,
    height_noise_profile: TerrainNoiseProfile,
    height_curve_exponent: f32,
    domain_warp_strength: f32,
    terrain_detail_mix: f32,
    moisture_bias: f32,
    temperature_bias: f32,
    noise_sampling: &'a NoiseSamplingTuning,
    island_mode: bool,
    island_falloff: f32,
}

pub fn hash_pass1_bucket(params: &WorldGenParams) -> u64 {
    hash_json(&Pass1Fingerprint {
        seed: params.seed,
        noise_scale: params.noise_scale,
        noise_octaves: params.noise_octaves,
        noise_lacunarity: params.noise_lacunarity,
        noise_persistence: params.noise_persistence,
        height_noise_profile: params.height_noise_profile,
        height_curve_exponent: params.height_curve_exponent,
        domain_warp_strength: params.domain_warp_strength,
        terrain_detail_mix: params.terrain_detail_mix,
        moisture_bias: params.moisture_bias,
        temperature_bias: params.temperature_bias,
        noise_sampling: &params.noise_sampling,
        island_mode: params.island_mode,
        island_falloff: params.island_falloff,
    })
}

#[derive(Serialize)]
struct PassTuningFingerprint<'a> {
    width: u32,
    height: u32,
    num_regions: u32,
    region_method: RegionMethod,
    region_iterations: u32,
    biome_tuning: &'a BiomeTuning,
    river_count: u32,
    lake_count: u32,
    mountain_threshold: f32,
    tag_pool: [u64; 4],
}

pub fn hash_tuning_bucket(params: &WorldGenParams) -> u64 {
    hash_json(&PassTuningFingerprint {
        width: params.width,
        height: params.height,
        num_regions: params.num_regions,
        region_method: params.region_method,
        region_iterations: params.region_iterations,
        biome_tuning: &params.biome_tuning,
        river_count: params.river_count,
        lake_count: params.lake_count,
        mountain_threshold: params.mountain_threshold,
        tag_pool: params.tag_pool.bits(),
    })
}

pub fn source_noise_id_for_chunk(chunk_xy: IVec2, pass1_bucket: u64) -> u64 {
    let cx = chunk_xy.x as i64 as u64;
    let cy = chunk_xy.y as i64 as u64;
    combine_u64(combine_u64(pass1_bucket, cx), cy)
}

pub fn hash_material_registry(reg: &MaterialRegistry) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    reg.schema_version.hash(&mut h);
    reg.materials.len().hash(&mut h);
    for m in &reg.materials {
        m.name.hash(&mut h);
    }
    h.finish()
}

pub fn hash_tag_registry(reg: &TagRegistry) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    reg.schema_version.hash(&mut h);
    reg.tags.len().hash(&mut h);
    for t in &reg.tags {
        t.name.hash(&mut h);
    }
    h.finish()
}

pub fn hash_rule_set(rules: &RuleSet) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    rules.schema_version.hash(&mut h);
    rules.rules.len().hash(&mut h);
    for r in &rules.rules {
        r.result_name.hash(&mut h);
        r.priority.hash(&mut h);
        r.rule_index.hash(&mut h);
        for s in &r.required {
            s.hash(&mut h);
        }
        for s in &r.forbidden {
            s.hash(&mut h);
        }
        r.family_filter.hash(&mut h);
    }
    h.finish()
}

pub fn compute_chunk_dependency(
    chunk_xy: IVec2,
    params: &WorldGenParams,
    reg: &MaterialRegistry,
    rules: &RuleSet,
    tags: &TagRegistry,
) -> ChunkDependency {
    let p1 = hash_pass1_bucket(params);
    ChunkDependency {
        source_noise_id: source_noise_id_for_chunk(chunk_xy, p1),
        registry_hash: hash_material_registry(reg),
        rules_hash: hash_rule_set(rules),
        tags_hash: hash_tag_registry(tags),
        tuning_hash: hash_tuning_bucket(params),
    }
}

/// Returns lowest pass index `0..=5` if any dirty bit set.
pub fn lowest_dirty_pass(mask: u8) -> Option<u32> {
    (0..6).find(|i| mask & (1 << i) != 0).map(|i| i as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Hash)]
    struct Dummy {
        a: u32,
        b: String,
    }

    #[test]
    fn chunk_dependency_hash_stable() {
        let d = Dummy {
            a: 7,
            b: "x".into(),
        };
        let h1 = hash_asset(&d);
        let h2 = hash_asset(&d);
        assert_eq!(h1, h2);
    }
}
