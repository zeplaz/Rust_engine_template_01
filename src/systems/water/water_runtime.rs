//! DF-2 / PLAN-WATER-DEPTH-TRUTH-v1 — runtime projection + unified wet query.
//!
//! Gameplay wet = depth ≥ [`WET_DEPTH_M`]. River/lake/ocean on [`WaterClassRuntime`] are
//! **look labels** derived from the fluid, not separate waters.

use bevy::prelude::*;

use crate::terrain::generation::WaterClass;

use super::static_depth::WorldStaticWaterDepth;
use super::surface_water::SurfaceWaterField;

pub const WET_DEPTH_M: f32 = 0.3;
pub const DRY_DEPTH_M: f32 = 0.05;
pub const DRY_HOLD_S: f32 = 6.0 * 3600.0; // 6 h hysteresis
/// Channel look when speed exceeds this (m/s approx from qx/qy / depth).
pub const CHANNEL_LOOK_SPEED: f32 = 0.15;

/// Presentation look from depth + speed (+ optional ocean/coast hint). Not a gameplay gate.
#[must_use]
pub fn water_look_from_depth_speed(
    depth_m: f32,
    speed: f32,
    coast_ocean_hint: bool,
) -> WaterClass {
    if depth_m < WET_DEPTH_M {
        WaterClass::Dry
    } else if speed > CHANNEL_LOOK_SPEED {
        WaterClass::River
    } else if coast_ocean_hint {
        WaterClass::Ocean
    } else {
        WaterClass::Lake
    }
}

/// Runtime look/class per ROI cell — projected from depth; not the render catalog.
#[derive(Resource, Clone, Debug, Default)]
pub struct WaterClassRuntime {
    pub origin_x: i32,
    pub origin_z: i32,
    pub width: u32,
    pub height: u32,
    pub class: Vec<WaterClass>,
    /// Seconds the cell has been below [`DRY_DEPTH_M`] while previously wet.
    pub dry_timer_s: Vec<f32>,
    pub revision: u64,
}

impl WaterClassRuntime {
    #[must_use]
    pub fn class_at_world(&self, wx: i32, wz: i32) -> Option<WaterClass> {
        if self.width == 0
            || wx < self.origin_x
            || wz < self.origin_z
            || wx >= self.origin_x + self.width as i32
            || wz >= self.origin_z + self.height as i32
        {
            return None;
        }
        let lx = (wx - self.origin_x) as u32;
        let lz = (wz - self.origin_z) as u32;
        let i = (lz as usize) * (self.width as usize) + (lx as usize);
        self.class.get(i).copied()
    }

    #[must_use]
    pub fn is_open_water_world(&self, wx: i32, wz: i32) -> Option<bool> {
        self.class_at_world(wx, wz).map(|c| c.is_open_water())
    }

    pub fn sync_shape_from_field(&mut self, field: &SurfaceWaterField) {
        let n = field.cell_count();
        if self.class.len() != n
            || self.width != field.width
            || self.height != field.height
            || self.origin_x != field.origin_x
            || self.origin_z != field.origin_z
        {
            self.origin_x = field.origin_x;
            self.origin_z = field.origin_z;
            self.width = field.width;
            self.height = field.height;
            self.class = vec![WaterClass::Dry; n];
            self.dry_timer_s = vec![0.0; n];
        }
    }

    /// Project depth → look class with hysteresis.
    pub fn project_from_field(
        &mut self,
        field: &SurfaceWaterField,
        base: Option<&[WaterClass]>,
        dt_s: f32,
    ) {
        if !field.wired {
            return;
        }
        self.sync_shape_from_field(field);
        let n = field.cell_count();
        let dx = field.dx_m.max(1e-3);
        for i in 0..n {
            let depth = field.depth_m[i];
            let base_c = base.and_then(|b| b.get(i).copied()).unwrap_or(WaterClass::Dry);
            let prev = self.class[i];
            if depth >= WET_DEPTH_M {
                self.dry_timer_s[i] = 0.0;
                let qx = field.qx_m3s.get(i).copied().unwrap_or(0.0).abs();
                let qy = field.qy_m3s.get(i).copied().unwrap_or(0.0).abs();
                let speed = (qx + qy) / (depth * dx).max(1e-3);
                let coast = matches!(base_c, WaterClass::Ocean);
                self.class[i] = water_look_from_depth_speed(depth, speed, coast);
            } else if depth <= DRY_DEPTH_M {
                if prev.is_open_water() || prev == WaterClass::Wetland {
                    self.dry_timer_s[i] += dt_s;
                    if self.dry_timer_s[i] >= DRY_HOLD_S {
                        self.class[i] = if base_c.is_open_water() {
                            base_c
                        } else {
                            WaterClass::Dry
                        };
                    }
                } else {
                    self.class[i] = base_c;
                    self.dry_timer_s[i] = 0.0;
                }
            } else {
                self.dry_timer_s[i] = 0.0;
                if !prev.is_open_water() {
                    self.class[i] = WaterClass::Wetland;
                }
            }
        }
        self.revision = self.revision.saturating_add(1);
    }
}

/// Wet test over explicit sources (WM-4). Systems use [`super::WaterTileProbe`]; this stays for
/// pure helpers that hold plain references.
///
/// Wired field: its live depth decides, and a tile off the map is dry. Not seeded yet: the static
/// stamp the field will be seeded from, then `fallback` (the render catalog). `_runtime` is
/// ignored: the look labels are a projection of the field, never a second wet source.
#[must_use]
pub fn tile_is_wet(
    field: Option<&SurfaceWaterField>,
    _runtime: Option<&WaterClassRuntime>,
    static_depth: Option<&WorldStaticWaterDepth>,
    wx: i32,
    wz: i32,
    fallback: impl FnOnce(i32, i32) -> bool,
) -> bool {
    if let Some(f) = field.filter(|f| f.wired) {
        return f.depth_at_world(wx, wz).is_some_and(|d| d >= WET_DEPTH_M);
    }
    if let Some(st) = static_depth {
        if let Some(wet) = st.is_wet_world(wx, wz) {
            return wet;
        }
    }
    fallback(wx, wz)
}

/// Legacy name — same as [`tile_is_wet`] without field/static (catalog/runtime only).
#[must_use]
pub fn tile_is_water_prefer_runtime(
    runtime: Option<&WaterClassRuntime>,
    wx: i32,
    wz: i32,
    fallback: impl FnOnce(i32, i32) -> bool,
) -> bool {
    tile_is_wet(None, runtime, None, wx, wz, fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_runtime_hysteresis_holds_wet() {
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 4, 4, vec![0.0; 16], 0.0);
        field.depth_m[5] = 1.0;
        let mut rt = WaterClassRuntime::default();
        rt.project_from_field(&field, None, 1.0);
        assert!(rt.class[5].is_open_water());
        field.depth_m[5] = 0.0;
        rt.project_from_field(&field, None, 10.0);
        assert!(rt.class[5].is_open_water(), "still wet during hysteresis");
        rt.project_from_field(&field, None, DRY_HOLD_S);
        assert_eq!(rt.class[5], WaterClass::Dry);
    }

    #[test]
    fn dried_field_cell_beats_wet_static_stamp() {
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 2, 2, vec![0.0; 4], 0.0);
        let stamp = WorldStaticWaterDepth { width: 2, height: 2, depth_m: vec![3.0; 4], stamp: 0 };
        assert!(!tile_is_wet(Some(&field), None, Some(&stamp), 0, 0, |_, _| true));
        assert!(tile_is_wet(None, None, Some(&stamp), 0, 0, |_, _| false), "unseeded: static stamp");
    }

    #[test]
    fn tile_is_wet_prefers_field_depth() {
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, 4, 4, vec![0.0; 16], 0.0);
        field.depth_m[0] = 1.0;
        assert!(tile_is_wet(
            Some(&field),
            None,
            None,
            0,
            0,
            |_, _| false
        ));
        assert!(!tile_is_wet(
            Some(&field),
            None,
            None,
            1,
            0,
            |_, _| false
        ));
    }
}
