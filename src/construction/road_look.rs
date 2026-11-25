//! RD-9 road and rail look (PLAN-ROADS-v1): how a committed line is drawn — presentation only.
//!
//! - Roads: ribbon width from tier and lanes, colour per tier; a motorway gets a median strip.
//! - State tints over the tier colour: flooded (blue), buried / blocked (rust), congested (amber),
//!   muddy (dark brown, unpaved tiers in wet ground).
//! - Rail: a ballast bed, two rails and sleeper ticks across the bed.
//! - Worn ground (RD-5) tints its tiles by how worn it is, before it becomes a track.
//!
//! Lines are emitted as [`PathLineRequest`]s with a [`PathLook`] (colour + sideways offset +
//! overlay flag); the 2D and 3D path layers draw them unchanged otherwise.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::visual_authority::{PathLineRequest, PathLook, VisualPathKind, ZoneTileRequest};
use crate::gui::UiRgba;
use crate::infrastructure::RoadTier;
use crate::strategic::BuildSiteTile;

/// What a road stretch looks like right now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RoadState {
    pub flooded: bool,
    pub blocked: bool,
    pub congested: bool,
    pub muddy: bool,
    /// Washed out or blast-broken surface (RD-10), short of closed.
    pub damaged: bool,
}

/// Ribbon width (path units, same scale as the old fixed 8) for a tier and lane count.
#[must_use]
pub fn tier_width(tier: RoadTier, lanes: u8) -> f32 {
    let per_lane = match tier {
        RoadTier::Track => 2.0,
        RoadTier::Dirt => 2.6,
        RoadTier::Gravel => 3.0,
        RoadTier::Paved => 3.6,
        RoadTier::Concrete => 3.8,
        RoadTier::Motorway => 3.8,
    };
    per_lane * f32::from(lanes.max(1)) + if tier == RoadTier::Motorway { 2.0 } else { 0.8 }
}

/// Base colour of a tier.
#[must_use]
pub fn tier_color(tier: RoadTier) -> UiRgba {
    match tier {
        RoadTier::Track => UiRgba::from_rgba_unmultiplied(150, 122, 86, 200),
        RoadTier::Dirt => UiRgba::from_rgba_unmultiplied(122, 92, 60, 225),
        RoadTier::Gravel => UiRgba::from_rgba_unmultiplied(150, 144, 128, 230),
        RoadTier::Paved => UiRgba::from_rgba_unmultiplied(58, 60, 66, 235),
        RoadTier::Concrete => UiRgba::from_rgba_unmultiplied(160, 160, 156, 235),
        RoadTier::Motorway => UiRgba::from_rgba_unmultiplied(44, 46, 54, 240),
    }
}

fn mix(a: UiRgba, b: UiRgba, t: f32) -> UiRgba {
    let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    UiRgba { r: m(a.r, b.r), g: m(a.g, b.g), b: m(a.b, b.b), a: a.a.max(b.a) }
}

/// Tier colour with the stretch's state over it (worst state wins).
#[must_use]
pub fn road_color(tier: RoadTier, state: RoadState) -> UiRgba {
    let base = tier_color(tier);
    if state.flooded {
        mix(base, UiRgba::from_rgba_unmultiplied(50, 110, 210, 240), 0.7)
    } else if state.blocked {
        mix(base, UiRgba::from_rgba_unmultiplied(150, 60, 30, 240), 0.7)
    } else if state.congested {
        mix(base, UiRgba::from_rgba_unmultiplied(235, 160, 40, 240), 0.55)
    } else if state.damaged {
        mix(base, UiRgba::from_rgba_unmultiplied(95, 85, 75, 235), 0.6)
    } else if state.muddy {
        mix(base, UiRgba::from_rgba_unmultiplied(70, 50, 30, 235), 0.5)
    } else {
        base
    }
}

fn centre(t: BuildSiteTile) -> Vec3 {
    Vec3::new(t.x as f32 + 0.5, 0.0, t.z as f32 + 0.5)
}

fn line(kind: VisualPathKind, a: Vec3, b: Vec3, width: f32, look: PathLook) -> PathLineRequest {
    PathLineRequest { kind, start: a, end: b, width, valid: true, slope_ok: true, committed: true, look }
}

/// The lines one committed road stretch draws as.
#[must_use]
pub fn road_stretch_lines(a: BuildSiteTile, b: BuildSiteTile, tier: RoadTier, lanes: u8, state: RoadState) -> Vec<PathLineRequest> {
    let (pa, pb) = (centre(a), centre(b));
    let mut out = vec![line(
        VisualPathKind::Road,
        pa,
        pb,
        tier_width(tier, lanes),
        PathLook { color: Some(road_color(tier, state)), ..default() },
    )];
    if tier == RoadTier::Motorway {
        // Median strip between the two carriageways.
        out.push(line(
            VisualPathKind::Road,
            pa,
            pb,
            1.0,
            PathLook { color: Some(UiRgba::from_rgba_unmultiplied(70, 110, 60, 240)), overlay: true, ..default() },
        ));
    }
    out
}

/// Ballast colour, rail colour, sleeper colour.
const BALLAST: UiRgba = UiRgba { r: 112, g: 104, b: 96, a: 235 };
const RAIL: UiRgba = UiRgba { r: 40, g: 40, b: 44, a: 255 };
const SLEEPER: UiRgba = UiRgba { r: 92, g: 70, b: 48, a: 245 };
/// Rail gauge offset from the centre line (tile units) and sleeper spacing (tiles).
const RAIL_OFFSET: f32 = 0.11;
const SLEEPER_EVERY: f32 = 0.5;

/// The lines one committed rail stretch draws as: ballast, sleepers, two rails.
#[must_use]
pub fn rail_stretch_lines(a: BuildSiteTile, b: BuildSiteTile, state: RoadState) -> Vec<PathLineRequest> {
    let (pa, pb) = (centre(a), centre(b));
    let ballast = if state.flooded || state.blocked { road_color(RoadTier::Gravel, state) } else { BALLAST };
    let mut out = vec![line(VisualPathKind::Rail, pa, pb, 6.0, PathLook { color: Some(ballast), ..default() })];
    let d = Vec2::new(pb.x - pa.x, pb.z - pa.z);
    let len = d.length();
    if len > 1e-3 {
        let (dir, perp) = (d / len, Vec2::new(-d.y, d.x) / len);
        let n = (len / SLEEPER_EVERY).floor() as usize;
        for i in 0..=n {
            let c = Vec2::new(pa.x, pa.z) + dir * (i as f32 * SLEEPER_EVERY).min(len);
            let (s0, s1) = (c - perp * 0.2, c + perp * 0.2);
            out.push(line(
                VisualPathKind::Rail,
                Vec3::new(s0.x, 0.0, s0.y),
                Vec3::new(s1.x, 0.0, s1.y),
                0.9,
                PathLook { color: Some(SLEEPER), overlay: true, ..default() },
            ));
        }
    }
    for side in [-RAIL_OFFSET, RAIL_OFFSET] {
        out.push(line(VisualPathKind::Rail, pa, pb, 0.5, PathLook { color: Some(RAIL), offset: side, overlay: true }));
    }
    out
}

/// Worn ground (RD-5) as a tint on its tiles, stronger the closer it is to becoming a track.
#[must_use]
pub fn wear_tiles(wear: &std::collections::HashMap<IVec2, f32>, passes_to_form: f32) -> Vec<ZoneTileRequest> {
    let mut v: Vec<(IVec2, f32)> = wear.iter().filter(|(_, w)| **w >= passes_to_form * 0.25).map(|(t, w)| (*t, *w)).collect();
    v.sort_by_key(|(t, _)| (t.x, t.y));
    v.into_iter()
        .map(|(t, w)| {
            let k = (w / passes_to_form).clamp(0.25, 1.0);
            ZoneTileRequest {
                center: Vec3::new(t.x as f32 + 0.5, 0.0, t.y as f32 + 0.5),
                color: UiRgba::from_rgba_unmultiplied(120, 96, 64, (40.0 + 110.0 * k) as u8),
            }
        })
        .collect()
}

/// Road condition inputs (read only; each owner publishes its own).
#[derive(SystemParam)]
pub struct RoadLookReads<'w, 's> {
    directory: Option<Res<'w, crate::systems::transport::TransportEdgeDirectory>>,
    flood: Option<Res<'w, crate::systems::water::FloodRouteBlockBook>>,
    debris: Option<Res<'w, crate::economy::logistics::debris_block::DebrisRouteBlockBook>>,
    flow: Option<Res<'w, crate::economy::logistics::road_flow::RoadFlowBook>>,
    wear: Option<Res<'w, crate::economy::logistics::track_wear::TrackWearBook>>,
    condition: Option<Res<'w, crate::economy::logistics::road_condition::RoadConditionBook>>,
    weather: Query<'w, 's, (&'static crate::terrain::generation::Chunk, &'static crate::systems::weather::ChunkWeather)>,
}

impl RoadLookReads<'_, '_> {
    /// State of the road stretch `a → b` of a `tier` road.
    #[must_use]
    pub fn state(&self, a: BuildSiteTile, b: BuildSiteTile, tier: RoadTier) -> RoadState {
        let dir = self.directory.as_deref();
        let id = dir.and_then(|d| super::find_edge_id_for_segment(d, a, b));
        let flooded = id.is_some_and(|id| crate::economy::logistics::routes::flood_blocks_route_edge(self.flood.as_deref(), dir, id));
        let blocked = id.is_some_and(|id| crate::economy::logistics::routes::debris_blocks_route_edge(self.debris.as_deref(), dir, id));
        let congested = id.is_some_and(|id| self.flow.as_deref().and_then(|f| f.by_edge.get(&id)).is_some_and(|f| f.vc() > 0.8));
        let unpaved = matches!(tier, RoadTier::Track | RoadTier::Dirt | RoadTier::Gravel);
        let muddy = unpaved && {
            let mid = IVec2::new(((a.x + b.x) / 2) as i32, ((a.z + b.z) / 2) as i32);
            let chunk = mid.div_euclid(IVec2::splat(crate::terrain::world_scale_contract::CHUNK_TILES_SIM as i32));
            let wet = crate::infrastructure::road_tier_table().offroad.mud_from_moisture;
            self.weather.iter().any(|(c, w)| c.coord == chunk && w.soil_moisture > wet)
        };
        let condition = self.condition.as_deref();
        let muddy = muddy || id.is_some_and(|id| condition.is_some_and(|c| c.muddy(id)));
        let damaged = id.is_some_and(|id| condition.is_some_and(|c| c.damage(id) >= 0.3));
        RoadState { flooded, blocked, congested, muddy, damaged }
    }

    /// Worn-ground tint tiles.
    #[must_use]
    pub fn wear_tiles(&self) -> Vec<ZoneTileRequest> {
        let to_form = crate::infrastructure::road_tier_table().worn_tracks.passes_to_form;
        self.wear.as_deref().map(|w| wear_tiles(&w.wear, to_form)).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wider_tiers_and_more_lanes_draw_wider_and_each_tier_has_its_colour() {
        assert!(tier_width(RoadTier::Track, 1) < tier_width(RoadTier::Paved, 2));
        assert!(tier_width(RoadTier::Paved, 2) < tier_width(RoadTier::Motorway, 4));
        assert_ne!(tier_color(RoadTier::Gravel), tier_color(RoadTier::Paved));
        assert_eq!(road_stretch_lines(BuildSiteTile { x: 0, z: 0 }, BuildSiteTile { x: 8, z: 0 }, RoadTier::Motorway, 4, RoadState::default()).len(), 2, "motorway median");
    }

    #[test]
    fn states_tint_the_road() {
        let base = road_color(RoadTier::Gravel, RoadState::default());
        for s in [
            RoadState { flooded: true, ..default() },
            RoadState { blocked: true, ..default() },
            RoadState { congested: true, ..default() },
            RoadState { muddy: true, ..default() },
            RoadState { damaged: true, ..default() },
        ] {
            assert_ne!(road_color(RoadTier::Gravel, s), base, "{s:?}");
        }
        let flooded = road_color(RoadTier::Gravel, RoadState { flooded: true, ..default() });
        assert!(flooded.b > flooded.r, "flooded reads blue");
    }

    #[test]
    fn rail_is_ballast_sleepers_and_two_rails() {
        let lines = rail_stretch_lines(BuildSiteTile { x: 0, z: 0 }, BuildSiteTile { x: 4, z: 0 }, RoadState::default());
        let rails = lines.iter().filter(|l| l.look.offset != 0.0).count();
        let sleepers = lines.iter().filter(|l| l.look.overlay && l.look.offset == 0.0).count();
        assert_eq!(rails, 2);
        assert!(sleepers >= 8, "a sleeper every half tile: {sleepers}");
        assert!(!lines[0].look.overlay, "the ballast bed is the base line (bridges hang off it)");
    }

    #[test]
    fn worn_ground_shows_before_it_becomes_a_track() {
        let wear = std::collections::HashMap::from([(IVec2::ZERO, 2.0), (IVec2::ONE, 20.0), (IVec2::X, 40.0)]);
        let tiles = wear_tiles(&wear, 30.0);
        assert_eq!(tiles.len(), 2, "faint wear is not drawn");
        assert!(tiles.iter().any(|t| t.color.a > 140), "nearly a track: strong tint");
    }
}
