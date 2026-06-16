//! BUILD-READ-WORLD-002 — Option A iso building draw scale multiplier (post-commit visual only).

use bevy::prelude::*;

/// Designer table: [`design_build_readability_v1.md`](../../dev/design_build_readability_v1.md) §3b.
pub const DEFAULT_ISO_DRAW_SCALE_MULTIPLIER: f32 = 1.5;
pub const ISO_DRAW_SCALE_MIN: f32 = 1.35;
pub const ISO_DRAW_SCALE_MAX: f32 = 1.75;

/// Post-commit PG-2 module draw scale — does **not** affect ghost pick / tile occupation.
#[derive(Resource, Debug, Clone)]
pub struct ConstructionIsoDrawScale {
    pub multiplier: f32,
}

impl Default for ConstructionIsoDrawScale {
    fn default() -> Self {
        Self {
            multiplier: DEFAULT_ISO_DRAW_SCALE_MULTIPLIER,
        }
    }
}

impl ConstructionIsoDrawScale {
    #[must_use]
    pub fn clamped(multiplier: f32) -> Self {
        Self {
            multiplier: multiplier.clamp(ISO_DRAW_SCALE_MIN, ISO_DRAW_SCALE_MAX),
        }
    }

    #[must_use]
    pub fn visual_scale_vec3(&self) -> Vec3 {
        Vec3::new(self.multiplier, 1.0, self.multiplier)
    }
}

/// BUILD-READ-WORLD-002 witness body (lib self-check + live JSON rollup).
#[must_use]
pub fn build_read_world_002_witness_body() -> serde_json::Value {
    let green = build_read_world_002_witness_green();
    serde_json::json!({
        "gate_id": "BUILD-READ-WORLD-002",
        "lever": "iso_draw_scale_multiplier",
        "iso_draw_scale_multiplier": DEFAULT_ISO_DRAW_SCALE_MULTIPLIER,
        "primary_pct_site_stub": 0.125,
        "green": green,
    })
}

#[must_use]
pub fn build_read_world_002_witness_green() -> bool {
    build_read_world_002_self_check().is_ok()
}

fn build_read_world_002_self_check() -> Result<(), &'static str> {
    let s = ConstructionIsoDrawScale::default();
    if s.multiplier < ISO_DRAW_SCALE_MIN || s.multiplier > ISO_DRAW_SCALE_MAX {
        return Err("multiplier_range");
    }
    let v = s.visual_scale_vec3();
    if (v.x - s.multiplier).abs() > f32::EPSILON || (v.z - s.multiplier).abs() > f32::EPSILON {
        return Err("visual_scale_vec");
    }
    if (v.y - 1.0).abs() > f32::EPSILON {
        return Err("y_unscaled");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_002_witness_green() {
        assert!(build_read_world_002_witness_green());
    }
}
