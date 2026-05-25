//! Control points → road segment previews (polyline chain).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::placement::RoadSegmentPreview;
use super::spline::catmull_rom_chain;
use crate::construction::construction_pipeline::validate_road_segment;

#[inline]
pub fn world_xy_to_tile(p: Vec3) -> BuildSiteTile {
    BuildSiteTile {
        x: p.x.floor().max(0.0) as u32,
        z: p.z.floor().max(0.0) as u32,
    }
}

#[must_use]
pub fn segment_preview_valid(
    start: Vec3,
    end: Vec3,
    params: &WorldGenParams,
) -> bool {
    let head = world_xy_to_tile(start);
    let tail = world_xy_to_tile(end);
    validate_road_segment(head, tail, params).valid
}

/// Rebuild segment list from committed control points plus optional live cursor preview.
pub fn regenerate_road_segments(
    control_points: &[Vec3],
    cursor_world: Option<Vec3>,
    width: f32,
    params: &WorldGenParams,
    use_curved_preview: bool,
) -> Vec<RoadSegmentPreview> {
    let mut chain: Vec<Vec3> = control_points.to_vec();
    if let Some(c) = cursor_world {
        if chain.last().copied().map_or(true, |last| (last - c).length_squared() > 0.01) {
            chain.push(c);
        }
    }
    if chain.len() < 2 {
        return Vec::new();
    }
    let sample_chain = if use_curved_preview && chain.len() >= 3 {
        catmull_rom_chain(&chain, 6)
    } else {
        chain
    };
    sample_chain
        .windows(2)
        .map(|w| {
            let start = w[0];
            let end = w[1];
            RoadSegmentPreview {
                start,
                end,
                width,
                valid: segment_preview_valid(start, end, params),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curved_preview_emits_more_segments_than_polyline() {
        let params = WorldGenParams {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let pts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(8.0, 0.0, 6.0),
            Vec3::new(12.0, 0.0, 6.0),
        ];
        let flat = regenerate_road_segments(&pts, None, 8.0, &params, false);
        let curved = regenerate_road_segments(&pts, None, 8.0, &params, true);
        assert_eq!(flat.len(), 3);
        assert!(curved.len() > flat.len());
    }

    #[test]
    fn polyline_emits_n_minus_one_segments() {
        let params = WorldGenParams {
            width: 64,
            height: 64,
            ..Default::default()
        };
        let pts = vec![
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(5.0, 0.0, 1.0),
            Vec3::new(5.0, 0.0, 8.0),
        ];
        let segs = regenerate_road_segments(&pts, None, 8.0, &params, false);
        assert_eq!(segs.len(), 2);
        assert!(segs.iter().all(|s| s.valid));
    }
}
