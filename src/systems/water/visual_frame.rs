//! DF-3 — sim→view snapshot of surface water for render (no GPU readback).

use bevy::prelude::*;

use super::surface_water::SurfaceWaterField;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Subsampled wet cells for overlay / particle emitters (read-only for render).
#[derive(Resource, Debug, Clone, Default)]
pub struct SurfaceWaterVisualFrame {
    pub revision: u64,
    /// (tile_x, tile_z, depth_m, speed_mps, flow_dir_x, flow_dir_z) — tile space.
    pub samples: Vec<(f32, f32, f32, f32, f32, f32)>,
    pub spillway_flow_m3s: f32,
    pub outlet_flow_m3s: f32,
    pub mid_outlet_flow_m3s: f32,
    pub overtopping_flow_m3s: f32,
    pub turbine_flow_m3s: f32,
    pub breach_flow_m3s: f32,
}

/// Sole writer of [`SurfaceWaterVisualFrame`]. Runs after the surface-water step.
pub fn publish_surface_water_visual_frame_system(
    field: Res<SurfaceWaterField>,
    mut frame: ResMut<SurfaceWaterVisualFrame>,
) {
    if !field.wired {
        if !frame.samples.is_empty() {
            frame.samples.clear();
            frame.revision = frame.revision.wrapping_add(1);
        }
        return;
    }

    let mut next: Vec<(f32, f32, f32, f32, f32, f32)> =
        Vec::with_capacity(frame.samples.capacity());
    // Subsample so overlay stays cheap on large ROIs.
    let step = ((field.width.max(field.height) / 32).max(1)) as usize;
    let dx = field.dx_m.max(TILE_METRES_GAMEPLAY);
    for lz in (0..field.height as usize).step_by(step) {
        for lx in (0..field.width as usize).step_by(step) {
            let i = field.idx(lx as u32, lz as u32);
            let h = field.depth_m.get(i).copied().unwrap_or(0.0);
            // Keep thin sheets / release channels in the foam/overlay sample set.
            if h < 0.04 {
                continue;
            }
            // Tile space — same as WaterSurfaceVisualCatalog river/lake anchors.
            let tx = field.origin_x as f32 + lx as f32 + 0.5;
            let tz = field.origin_z as f32 + lz as f32 + 0.5;
            let qx = field.qx_m3s.get(i).copied().unwrap_or(0.0);
            let qy = field.qy_m3s.get(i).copied().unwrap_or(0.0);
            let cell = (h * dx).max(1e-3);
            let vx = qx / cell;
            let vz = qy / cell;
            let speed = (vx * vx + vz * vz).sqrt();
            let (fdx, fdz) = if speed > 1e-4 {
                (vx / speed, vz / speed)
            } else {
                (1.0, 0.0)
            };
            next.push((tx, tz, h, speed, fdx, fdz));
        }
    }

    let flows_changed = (frame.spillway_flow_m3s - field.spillway_flow_m3s).abs() > 1e-3
        || (frame.outlet_flow_m3s - field.outlet_flow_m3s).abs() > 1e-3
        || (frame.mid_outlet_flow_m3s - field.mid_outlet_flow_m3s).abs() > 1e-3
        || (frame.overtopping_flow_m3s - field.overtopping_flow_m3s).abs() > 1e-3
        || (frame.turbine_flow_m3s - field.turbine_flow_m3s).abs() > 1e-3
        || (frame.breach_flow_m3s - field.breach_flow_m3s).abs() > 1e-3;
    let samples_changed = next.len() != frame.samples.len()
        || next.iter().zip(frame.samples.iter()).any(|(a, b)| {
            (a.0 - b.0).abs() > 0.05
                || (a.1 - b.1).abs() > 0.05
                || (a.2 - b.2).abs() > 0.05
                || (a.3 - b.3).abs() > 0.05
                || (a.4 - b.4).abs() > 0.05
                || (a.5 - b.5).abs() > 0.05
        });

    frame.spillway_flow_m3s = field.spillway_flow_m3s;
    frame.outlet_flow_m3s = field.outlet_flow_m3s;
    frame.mid_outlet_flow_m3s = field.mid_outlet_flow_m3s;
    frame.overtopping_flow_m3s = field.overtopping_flow_m3s;
    frame.turbine_flow_m3s = field.turbine_flow_m3s;
    frame.breach_flow_m3s = field.breach_flow_m3s;
    if samples_changed || flows_changed {
        frame.samples = next;
        frame.revision = frame.revision.wrapping_add(1);
    }
}
