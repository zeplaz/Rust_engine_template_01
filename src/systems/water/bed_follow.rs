//! DW-1: the solver bed follows terrain deformation (craters, rims, rubble).
//!
//! Terrain owns ground height ([`crate::terrain::ground_deform`]); this system is the water
//! domain's reader of the journal: each applied tile delta is added to the ROI's `bed_m` through
//! [`SurfaceWaterField::apply_bed_delta`]. Depth is untouched (mass kept) so the surface follows.

use bevy::prelude::*;

use super::surface_water::SurfaceWaterField;
use crate::terrain::ground_deform::GroundDeltaJournal;

/// Journal cursor of the water bed.
#[derive(Default)]
pub struct BedFollowCursor {
    seq: u64,
    epoch: u64,
}

pub fn follow_ground_deform_bed_system(
    journal: Res<GroundDeltaJournal>,
    mut field: ResMut<SurfaceWaterField>,
    mut cursor: Local<BedFollowCursor>,
) {
    if cursor.epoch != journal.epoch {
        // World regenerated: a fresh ROI is seeded from the new DEM, nothing to replay.
        // Ops applied after the reset (same frame) are still ours.
        cursor.epoch = journal.epoch;
        cursor.seq = journal.dropped_through;
    }
    if cursor.seq == journal.seq {
        return;
    }
    if !field.wired {
        // Not seeded yet: the seed samples chunk elevation, which already holds the delta.
        cursor.seq = journal.seq;
        return;
    }
    let Some(cells) = journal.since(cursor.seq) else {
        warn!("water bed follow: ground journal overran its cursor; bed may lag the terrain");
        cursor.seq = journal.seq;
        return;
    };
    for c in cells {
        field.apply_bed_delta(c.tile.x, c.tile.y, c.delta_m);
    }
    cursor.seq = journal.seq;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::ground_deform::{
        GroundDeformOp, GroundDeformPlugin, GroundDeformQueue, GroundDeformSet,
    };
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use crate::terrain::generation::WorldGenDenseTerrainCache;

    #[test]
    fn dw1_solver_bed_changes_inside_roi_and_depth_is_kept() {
        let mut app = App::new();
        app.init_resource::<WorldGenParams>();
        app.add_plugins(GroundDeformPlugin);
        let (w, h) = (64u32, 64u32);
        let n = (w * h) as usize;
        app.insert_resource(WorldGenDenseTerrainCache {
            width: w,
            height: h,
            elevation: vec![0.5; n],
            moisture: vec![0.3; n],
            temperature: vec![0.3; n],
            family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; n],
            region_index: vec![0; n],
        });
        let mut f = SurfaceWaterField::default();
        f.allocate(8, 8, 32, 32, vec![600.0; 32 * 32], 0.0);
        f.depth_m.iter_mut().for_each(|d| *d = 1.5);
        app.insert_resource(f);
        app.add_systems(Update, follow_ground_deform_bed_system.after(GroundDeformSet::Drain));
        let at = IVec2::new(20, 20); // inside the ROI (8..40)
        app.world_mut().resource_mut::<GroundDeformQueue>().push(GroundDeformOp::Crater {
            centre_m: Vec2::new((at.x as f32 + 0.5) * 16.0, (at.y as f32 + 0.5) * 16.0),
            radius_m: 40.0,
            depth_m: 6.0,
        });
        app.update();
        let f = app.world().resource::<SurfaceWaterField>();
        let i = f.idx((at.x - 8) as u32, (at.y - 8) as u32);
        assert!(f.bed_m[i] < 600.0 - 3.0, "bed {}", f.bed_m[i]);
        assert!((f.depth_m[i] - 1.5).abs() < 1e-6, "depth kept");
        // A rim cell is raised.
        assert!(f.bed_m.iter().any(|b| *b > 600.0));
        // Outside the ROI: no panic, ROI cells outside the strike untouched.
        assert!((f.bed_m[0] - 600.0).abs() < 1e-6);
    }
}
