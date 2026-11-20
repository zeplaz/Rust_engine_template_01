//! D-5 — a burning building shows flames and smoke, not just a heat tint.
//!
//! One burning state, one clock: the rows come from [`animate_emitter`], the same animation the
//! vegetation fire uses (flame core on the global loop phase, embers, smoke plume), driven by the
//! sim's [`SiteFireState`] (sole writer `site_fire_exposure_tick`) and the same wall clock
//! ([`FireAnim::time_s`]). Rows are appended into the shared [`WorldFireParticleFrame`] after the
//! fire build; no renderer, buffer or LOD of its own. Read only: nothing writes back to the sim.

use bevy::prelude::*;

use super::emit::fire_particle_scatter_count;
use super::particle_anim::{animate_emitter, FireAnim, FireEmitter};
use crate::render::{ParticleClass, WorldFireParticleFrame};
use crate::strategic::SiteFootprint;
use crate::systems::fire::{SiteFireStage, SiteFireState};

/// Footprint cells that emit per site (large sites are sampled on a stride, deterministic).
pub const SITE_FIRE_MAX_CELLS: usize = 12;
/// Ember slots per emitting cell: a building burns with a few sparks, not a forest's worth.
const SITE_EMBER_SLOTS_MAX: usize = 2;

/// Emitter template for a site's fire state (`xy` is filled per cell). `None` while clear.
///
/// * `Threatened` (exposed): thin smoke only.
/// * `Disrupted` and worse (burning): flames; embers once `Deteriorating`.
/// * Suppressed by rain: smoke only, no open flame.
#[must_use]
pub fn site_fire_emitter(state: &SiteFireState, wind_mps: Vec2) -> Option<FireEmitter> {
    let wash = state.stage.burn_wash01();
    if state.stage == SiteFireStage::Clear {
        return None;
    }
    let burning = !matches!(state.stage, SiteFireStage::Threatened) && !state.suppression;
    let heat = state.heat_mean.max(wash * 0.8).clamp(0.0, 1.0);
    let embers = burning
        && matches!(
            state.stage,
            SiteFireStage::Deteriorating | SiteFireStage::Collapsing
        );
    let slots = if embers {
        fire_particle_scatter_count(heat, 0.0).min(SITE_EMBER_SLOTS_MAX)
    } else {
        0
    };
    Some(FireEmitter {
        xy: Vec2::ZERO,
        heat,
        ember_rate: if embers { heat } else { 0.0 },
        smoke: (0.25 + 0.75 * wash).clamp(0.0, 1.0),
        flame_amp: if burning { 1.0 } else { 0.0 },
        ember_amp: if embers { 1.0 } else { 0.0 },
        smoke_amp: 1.0,
        ember_slots: slots,
        class: ParticleClass::Spark,
        wind_mps,
    })
}

/// Cells of a footprint that emit: every cell up to the cap, else an even stride.
fn emitting_cells(tiles: &[IVec2]) -> impl Iterator<Item = IVec2> + '_ {
    let stride = tiles.len().div_ceil(SITE_FIRE_MAX_CELLS).max(1);
    tiles.iter().copied().step_by(stride)
}

/// Append flame/ember/smoke rows for every exposed or burning site (after the fire build).
pub fn append_site_fire_particles(
    time: Res<Time>,
    climate: Option<Res<crate::render::ClimateVisualAggregate>>,
    sites: Query<(&SiteFootprint, &SiteFireState)>,
    mut frame: ResMut<WorldFireParticleFrame>,
) {
    let anim = FireAnim {
        time_s: time.elapsed_secs(),
        wind_mps: climate.as_deref().map_or(Vec2::ZERO, |c| c.mean_wind_mps),
    };
    let cap = frame.gpu_capacity.max(frame.instances.len());
    let mut rows = Vec::new();
    for (footprint, state) in &sites {
        let Some(template) = site_fire_emitter(state, anim.wind_mps) else {
            continue;
        };
        for tile in emitting_cells(&footprint.tiles) {
            let emitter = FireEmitter {
                xy: tile.as_vec2() + Vec2::splat(0.5),
                ..template
            };
            rows.clear();
            animate_emitter(&emitter, &anim, &mut rows);
            let room = cap.saturating_sub(frame.instances.len());
            rows.truncate(room);
            frame.instances.extend_from_slice(&rows);
            if room == 0 {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::fire_vfx::particle_anim::particle_ground_and_height_m;

    fn state(stage: SiteFireStage, heat: f32, suppression: bool) -> SiteFireState {
        SiteFireState {
            stage,
            heat_mean: heat,
            suppression,
            ..Default::default()
        }
    }

    #[test]
    fn clear_site_emits_nothing() {
        assert!(site_fire_emitter(&state(SiteFireStage::Clear, 0.0, false), Vec2::ZERO).is_none());
    }

    #[test]
    fn exposed_site_smokes_burning_site_flames() {
        let exposed = site_fire_emitter(&state(SiteFireStage::Threatened, 0.1, false), Vec2::ZERO).unwrap();
        assert_eq!(exposed.flame_amp, 0.0);
        assert!(exposed.smoke > 0.0);
        let burning = site_fire_emitter(&state(SiteFireStage::Deteriorating, 0.6, false), Vec2::ZERO).unwrap();
        assert_eq!(burning.flame_amp, 1.0);
        assert!(burning.ember_slots > 0 && burning.ember_amp > 0.0);
        assert!(burning.smoke > exposed.smoke);
        let rained = site_fire_emitter(&state(SiteFireStage::Deteriorating, 0.6, true), Vec2::ZERO).unwrap();
        assert_eq!(rained.flame_amp, 0.0, "rain-suppressed site shows no open flame");
    }

    fn run(stage: SiteFireStage, tiles: Vec<IVec2>, t: f32) -> WorldFireParticleFrame {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<WorldFireParticleFrame>()
            .add_systems(Update, append_site_fire_particles);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_to(std::time::Duration::from_secs_f32(t));
        app.world_mut().spawn((
            SiteFootprint { tiles, layer: crate::strategic::LayerType::Surface },
            state(stage, 0.7, false),
        ));
        app.update();
        app.world_mut().remove_resource::<WorldFireParticleFrame>().unwrap()
    }

    #[test]
    fn burning_site_rows_sit_over_its_footprint_tiles() {
        let tiles = vec![IVec2::new(10, 20), IVec2::new(11, 20)];
        let frame = run(SiteFireStage::Deteriorating, tiles, 3.0);
        assert!(!frame.instances.is_empty());
        let flames = frame
            .instances
            .iter()
            .filter(|r| r.ember_class_radius_smoke.y == ParticleClass::Spark.as_f32())
            .count();
        assert!(flames >= 2, "one flame core per emitting cell");
        for row in &frame.instances {
            let (ground, _h) = particle_ground_and_height_m(row);
            assert!((9.0..13.5).contains(&ground.x) && (19.0..22.5).contains(&ground.y), "{ground:?}");
        }
    }

    #[test]
    fn large_footprint_is_sampled_not_flooded() {
        let tiles: Vec<IVec2> = (0..64).map(|i| IVec2::new(i % 8, i / 8)).collect();
        let frame = run(SiteFireStage::Disrupted, tiles, 3.0);
        let flames = frame
            .instances
            .iter()
            .filter(|r| r.ember_class_radius_smoke.y == ParticleClass::Spark.as_f32())
            .count();
        assert!(flames <= SITE_FIRE_MAX_CELLS, "{flames}");
        assert!(flames >= 1);
    }
}
