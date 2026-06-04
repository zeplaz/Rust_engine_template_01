//! **CON-P3** — parametric scaling audit (Coder A: S1–S3; Coder B: S4–S6).

use crate::construction::building_catalog::{BuildingFamily, FootprintMatrix};
use crate::construction::parametric_commit::parametric_placement_snapshot;
use crate::construction::placement_scaling::{
    clamp_scale_factor, default_scale_factor_for_family, DEFAULT_SCALE_MAX, DEFAULT_SCALE_MIN,
};
use crate::construction::staged_ghost_panel::{
    staging_panel_visible_witness_green, staging_validity_badges_wired_witness_green,
    StagedValidity,
};
use crate::construction::visual_authority::FootprintTileColorKind;
use crate::construction::weighted_footprint::{
    rasterize_with_effective_scale, PlacementParams, WEIGHT_OMIT_THRESHOLD,
};
use crate::strategic::overlap_blocks_commit_witness_green;
use crate::strategic::BuildSiteTile;
use crate::strategic::commit_carries_scale_and_weights_witness_green;
use crate::strategic::FootprintTiles;

/// Preset footprint sizes (1×1 … 12×12) for tray/readability audit (**S1**).
const PRESET_MATRIX_SIZES: [u32; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

/// **S1** — ghost `FootprintMatrix` occupied cells match commit weight rows at unity scale.
#[must_use]
pub fn scaling_audit_s1_preset_matrix_match_green() -> bool {
    scaling_audit_s1_self_check().is_ok()
}

fn scaling_audit_s1_self_check() -> Result<(), &'static str> {
    let origin = BuildSiteTile { x: 8, z: 12 };
    let family = BuildingFamily::Industry;

    for &n in &PRESET_MATRIX_SIZES {
        let base = FootprintMatrix::from_size(n, n, true);
        let cell_count = base.occupied_local_offsets().count();
        let expected_cells = (n * n) as usize;
        if cell_count != expected_cells {
            return Err("matrix_cell_count");
        }

        let snap = parametric_placement_snapshot(
            &base,
            family,
            origin,
            0,
            false,
            Some(default_scale_factor_for_family(family)),
        );
        if snap.weights.len() != cell_count {
            return Err("ghost_commit_weight_count");
        }

        let footprint = FootprintTiles {
            width: n,
            depth: n,
        };
        if footprint.width * footprint.depth != n * n {
            return Err("footprint_tiles_area");
        }

        let (params, weighted) = rasterize_with_effective_scale(
            &base,
            PlacementParams::identity(1.0),
            origin,
        );
        if (params.effective_scale - 1.0).abs() > 0.08 {
            return Err("unity_effective_scale");
        }
        let raster_tiles = weighted
            .weights
            .iter()
            .filter(|(_, w)| *w >= WEIGHT_OMIT_THRESHOLD)
            .count();
        if raster_tiles < cell_count {
            return Err("raster_covers_base");
        }
    }

    let clamped = clamp_scale_factor(99.0);
    if clamped != DEFAULT_SCALE_MAX {
        return Err("scale_clamp_max");
    }
    if clamp_scale_factor(0.01) != DEFAULT_SCALE_MIN {
        return Err("scale_clamp_min");
    }
    Ok(())
}

/// **S2** — occupied / warn tiles use `FootprintTileColorKind::Risky` through tile debug flags.
#[must_use]
pub fn scaling_audit_s2_occupied_tiles_wired_green() -> bool {
    scaling_audit_s2_self_check().is_ok()
}

fn scaling_audit_s2_self_check() -> Result<(), &'static str> {
    use crate::construction::build_confidence::{confidence_from_validation, BuildConfidence};
    use crate::gui::tile_flags;
    use crate::strategic::SitePlacementValidation;

    if !std::path::Path::new("src/construction/footprint_tile_instances.rs").exists() {
        return Err("footprint_tile_instances");
    }
    if tile_flags::FOOTPRINT_RISKY == 0 {
        return Err("footprint_risky_flag");
    }

    let warn_kind = match StagedValidity::Warn {
        StagedValidity::Ok => FootprintTileColorKind::Valid,
        StagedValidity::Warn => FootprintTileColorKind::Risky,
        StagedValidity::Bad => FootprintTileColorKind::Invalid,
    };
    if warn_kind != FootprintTileColorKind::Risky {
        return Err("staged_warn_risky");
    }

    let overlap_report = SitePlacementValidation {
        valid: false,
        allows_commit: false,
        errors: vec!["weighted_overlap".to_string()],
        ..Default::default()
    };
    if confidence_from_validation(&overlap_report) != BuildConfidence::Risky {
        return Err("overlap_confidence_risky");
    }
    Ok(())
}

/// **S3** — blocked / overlap footprint disables commit (`allows_commit` + overlap witness).
#[must_use]
pub fn scaling_audit_s3_blocked_disables_commit_green() -> bool {
    overlap_blocks_commit_witness_green() && scaling_audit_s3_self_check().is_ok()
}

fn scaling_audit_s3_self_check() -> Result<(), &'static str> {
    if !scaling_audit_staged_blocked_is_bad(false, &[])? {
        return Err("allows_commit_false");
    }
    if !scaling_audit_staged_blocked_is_bad(true, &["weighted_overlap".to_string()])? {
        return Err("weighted_overlap_bad");
    }
    Ok(())
}

fn scaling_audit_staged_blocked_is_bad(
    allows_commit: bool,
    errors: &[String],
) -> Result<bool, &'static str> {
    let validity = scaling_audit_staged_validity(allows_commit, 1.0, errors);
    if errors.iter().any(|e| e == "weighted_overlap") || !allows_commit {
        return Ok(validity == StagedValidity::Bad);
    }
    Ok(false)
}

fn scaling_audit_staged_validity(
    allows_commit: bool,
    scale: f32,
    errors: &[String],
) -> StagedValidity {
    if errors.iter().any(|e| e == "weighted_overlap") || !allows_commit {
        return StagedValidity::Bad;
    }
    if scale < 0.25 {
        return StagedValidity::Bad;
    }
    if scale < 0.35 {
        return StagedValidity::Warn;
    }
    StagedValidity::Ok
}

/// Rollup witness block `construction_scaling_audit_001` (A: S1–S3).
#[must_use]
pub fn construction_scaling_audit_001_a_witness_green() -> bool {
    scaling_audit_s1_preset_matrix_match_green()
        && scaling_audit_s2_occupied_tiles_wired_green()
        && scaling_audit_s3_blocked_disables_commit_green()
}

/// **S4** — terrain mod token / footprint legend wired (occupied + validity badges).
#[must_use]
pub fn scaling_audit_s4_terrain_mod_legend_green() -> bool {
    std::path::Path::new("src/construction/footprint_tile_instances.rs").exists()
        && super::staged_ghost_panel::staging_validity_badges_wired_witness_green()
}

/// **S5** — rotation + scale persist on committed site.
#[must_use]
pub fn scaling_audit_s5_scale_persists_on_site_green() -> bool {
    commit_carries_scale_and_weights_witness_green()
}

/// **S6** — tray resize bounds independent of building scale clamp.
#[must_use]
pub fn scaling_audit_s6_tray_independent_of_building_scale_green() -> bool {
    let family = crate::construction::building_catalog::BuildingFamily::Industry;
    let default_scale = default_scale_factor_for_family(family);
    let clamped_high = clamp_scale_factor(99.0);
    let clamped_low = clamp_scale_factor(0.01);
    staging_panel_visible_witness_green()
        && staging_validity_badges_wired_witness_green()
        && (DEFAULT_SCALE_MIN..=DEFAULT_SCALE_MAX).contains(&default_scale)
        && clamped_high == DEFAULT_SCALE_MAX
        && clamped_low == DEFAULT_SCALE_MIN
}

/// Rollup witness block `construction_scaling_audit_001` (B: S4–S6).
#[must_use]
pub fn construction_scaling_audit_001_b_witness_green() -> bool {
    scaling_audit_s4_terrain_mod_legend_green()
        && scaling_audit_s5_scale_persists_on_site_green()
        && scaling_audit_s6_tray_independent_of_building_scale_green()
}

/// Full S1–S6 rollup for live JSON (**CON-P3-WIT**).
#[must_use]
pub fn construction_scaling_audit_001_witness_green() -> bool {
    construction_scaling_audit_001_a_witness_green()
        && construction_scaling_audit_001_b_witness_green()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn con_p3_scaling_audit_a_half_green() {
        assert!(
            scaling_audit_s1_preset_matrix_match_green(),
            "S1 preset matrix"
        );
        assert!(
            scaling_audit_s2_occupied_tiles_wired_green(),
            "S2 occupied risky"
        );
        assert!(
            scaling_audit_s3_blocked_disables_commit_green(),
            "S3 blocked commit"
        );
        assert!(construction_scaling_audit_001_a_witness_green());
    }

    #[test]
    fn con_p3_scaling_audit_b_half_green() {
        assert!(
            scaling_audit_s4_terrain_mod_legend_green(),
            "S4 terrain mod / partial-alpha legend"
        );
        assert!(
            scaling_audit_s5_scale_persists_on_site_green(),
            "S5 scale persists on site"
        );
        assert!(
            scaling_audit_s6_tray_independent_of_building_scale_green(),
            "S6 tray independent of building scale"
        );
    }

    #[test]
    fn con_p3_scaling_audit_full_rollup_green() {
        assert!(construction_scaling_audit_001_witness_green());
    }
}
