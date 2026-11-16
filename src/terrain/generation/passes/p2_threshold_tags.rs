//! Pass 2 — threshold tags from scalar fields using [`BiomeTuning`] bands (names → ids via [`TagRegistry`](crate::terrain::material::TagRegistry)).
//!
//! **PCG-04** (aig3 8.3.7 *Altitude Filtering*): [`altitude_filter_elevations`] reshapes the height
//! distribution (percentile stretch) before threshold tagging.

use crate::terrain::biome::BiomeTuning;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::material::{TagRegistry, TagSet};

/// PCG-04: stretch elevations so the `[lo_pct, hi_pct]` band fills `[0,1]` (clamped).
/// Deterministic; in-place. No-op when the band collapses.
pub fn altitude_filter_elevations(elevations: &mut [f32], lo_pct: f32, hi_pct: f32) {
    if elevations.is_empty() {
        return;
    }
    let mut sorted: Vec<f32> = elevations.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let n = sorted.len();
    let lo_i = ((lo_pct.clamp(0.0, 1.0)) * (n.saturating_sub(1) as f32)).round() as usize;
    let hi_i = ((hi_pct.clamp(0.0, 1.0)) * (n.saturating_sub(1) as f32)).round() as usize;
    let (lo_i, hi_i) = if lo_i <= hi_i {
        (lo_i, hi_i)
    } else {
        (hi_i, lo_i)
    };
    let lo = sorted[lo_i];
    let hi = sorted[hi_i];
    let span = hi - lo;
    if span <= 1e-6 {
        return;
    }
    for h in elevations.iter_mut() {
        *h = ((*h - lo) / span).clamp(0.0, 1.0);
    }
}

fn insert_named(
    set: &mut TagSet,
    registry: &TagRegistry,
    name: &str,
    on: bool,
    tag_pool: &TagSet,
) {
    if !on {
        return;
    }
    let Some(id) = registry.tag_id(name) else {
        return;
    };
    if !tag_pool.contains(id) {
        return;
    }
    set.insert(id);
}

/// Threshold tags for one cell — same rules as [`apply_threshold_tags`], shared with world preview fallback.
pub fn threshold_tags_for_scalars(
    h: f32,
    m: f32,
    t: f32,
    tuning: &BiomeTuning,
    tag_registry: &TagRegistry,
    tag_pool: &TagSet,
) -> TagSet {
    let n = &tuning.threshold_tag_names;
    let mut tags = TagSet::default();
    let lowland = h >= tuning.beach_height_max && h < tuning.mountain_height_min;
    insert_named(&mut tags, tag_registry, &n.lowland, lowland, tag_pool);
    insert_named(
        &mut tags,
        tag_registry,
        &n.highland,
        h >= tuning.mountain_height_min,
        tag_pool,
    );
    insert_named(
        &mut tags,
        tag_registry,
        &n.wet,
        m >= tuning.wetland_moist_threshold,
        tag_pool,
    );
    insert_named(
        &mut tags,
        tag_registry,
        &n.dry,
        m <= tuning.desert_moisture_max,
        tag_pool,
    );
    insert_named(
        &mut tags,
        tag_registry,
        &n.hot,
        t >= tuning.hot_lowlands_temperature_min,
        tag_pool,
    );
    insert_named(
        &mut tags,
        tag_registry,
        &n.cold,
        t <= tuning.tundra_temperature_max,
        tag_pool,
    );
    tags
}

/// Writes **baseline** threshold tags into `matrix.tags` (replaces prior contents per cell).
/// Optionally runs [`altitude_filter_elevations`] first when `filter_lo_hi` is `Some((lo, hi))`.
pub fn apply_threshold_tags(
    matrix: &mut ChunkCellMatrix,
    tuning: &BiomeTuning,
    tag_registry: &TagRegistry,
    tag_pool: &TagSet,
) {
    apply_threshold_tags_filtered(matrix, tuning, tag_registry, tag_pool, None);
}

/// Like [`apply_threshold_tags`], with optional PCG-04 altitude filter before banding.
pub fn apply_threshold_tags_filtered(
    matrix: &mut ChunkCellMatrix,
    tuning: &BiomeTuning,
    tag_registry: &TagRegistry,
    tag_pool: &TagSet,
    filter_lo_hi: Option<(f32, f32)>,
) {
    if let Some((lo, hi)) = filter_lo_hi {
        altitude_filter_elevations(&mut matrix.elevation, lo, hi);
    }
    for i in 0..matrix.elevation.len() {
        matrix.tags[i] = threshold_tags_for_scalars(
            matrix.elevation[i],
            matrix.moisture[i],
            matrix.temperature[i],
            tuning,
            tag_registry,
            tag_pool,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;

    #[test]
    fn pass2_threshold_tags_lowland_wet() {
        let tuning = BiomeTuning::default();
        let tag_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/tag_registry.example.json");
        let tag_registry = TagRegistry::load_from_json(tag_path.to_str().unwrap()).unwrap();

        let mut matrix = ChunkCellMatrix::new(UVec2::ONE);
        // Above beach, below mountain → lowland; moisture at wetland threshold → wet
        matrix.elevation[0] = 0.5;
        matrix.moisture[0] = tuning.wetland_moist_threshold + 0.01;
        matrix.temperature[0] = 0.5;

        apply_threshold_tags(&mut matrix, &tuning, &tag_registry, &TagSet::ALL);

        let low_id = tag_registry.tag_id("lowland").unwrap();
        let wet_id = tag_registry.tag_id("wet").unwrap();
        assert!(matrix.tags[0].contains(low_id));
        assert!(matrix.tags[0].contains(wet_id));
    }

    #[test]
    fn pcg04_altitude_filter_stretches_height_band() {
        let mut elev = vec![0.2, 0.25, 0.3, 0.8, 0.85];
        altitude_filter_elevations(&mut elev, 0.0, 1.0);
        assert!((elev[0] - 0.0).abs() < 1e-4);
        assert!((elev[4] - 1.0).abs() < 1e-4);
        assert!(elev[2] > elev[0] && elev[2] < elev[4]);
        let mut flat = vec![0.5; 4];
        altitude_filter_elevations(&mut flat, 0.1, 0.9);
        assert_eq!(flat, vec![0.5; 4], "flat band is a no-op");
    }
}
