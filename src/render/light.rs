//! Local pooled **point-light extraction** layer (Bevy 0.18 / wgpu).
//!
//! [`RequestLocalLight`](crate::render::light::RequestLocalLight) messages (fire path from
//! [`FireVisualFrame`](crate::render::sim_visual_extract::FireVisualFrame)); this plugin **collects**, **scores**,
//! **sorts**, and **truncates** into [`ActiveLights`], then syncs a **fixed pool** of [`PointLight`]
//! entities (no per-frame spawn/despawn).
//!
//! Intended for:
//! - fire lighting via [`crate::render::sim_visual_extract::FireVisualFrame`] (single sim pass, then cluster emit)
//! - explosion / vehicle / emergency lights (direct `RequestLocalLight` writers)
//! - future camera-relative culling; clustering in [`crate::render::lighting`]
//!
//! Avoids:
//! - dynamic `PointLight` churn
//! - archetype fragmentation from light entities
//! - gameplay owning render-facing light state ([`ActiveLights`] is extraction output only)
//!
//! Future roadmap:
//! - tighter budgeting rules per [`LightCategory`]
//! - GPU light buffers / screen-space clustering

use bevy::prelude::*;
use smallvec::SmallVec;

use crate::gui::MainWorldCamera;
use crate::render::extraction::FireVisualFramePlugin;

/// Cap on simultaneous CPU-driven local lights (matches slot entities).
pub const MAX_ACTIVE_LOCAL_LIGHTS: usize = 16;

/// High-level source for budgeting / clustering (`base_fire2_smoke.md` scale-up).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LightCategory {
    Fire,
    Explosion,
    Vehicle,
    Building,
    Emergency,
    #[default]
    Environment,
}

/// Simulation → render **hint** for one logical local light (merged / culled before GPU sync).
#[derive(Message, Clone, Copy, Debug)]
pub struct RequestLocalLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub priority: f32,
    pub category: LightCategory,
    /// Phase offset for [`sync_active_lights_to_point_lights`] flicker (radians scale; see sync).
    pub flicker_phase: f32,
    /// `0` disables flicker modulation.
    pub flicker_strength: f32,
}

/// One **visible** local light after collection (subset of requests, sorted by importance).
#[derive(Debug, Clone, Copy)]
pub struct LightData {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub flicker_phase: f32,
    pub flicker_strength: f32,
    pub category: LightCategory,
}

#[derive(Resource, Debug)]
pub struct ActiveLights {
    /// Prioritized, culled list (len ≤ [`MAX_ACTIVE_LOCAL_LIGHTS`]).
    pub visible: SmallVec<[LightData; MAX_ACTIVE_LOCAL_LIGHTS]>,
}

impl Default for ActiveLights {
    fn default() -> Self {
        Self {
            visible: SmallVec::new(),
        }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalLightExtractSet {
    /// Drain requests, score, sort, fill [`ActiveLights`].
    Collect,
    /// Apply [`ActiveLights`] to pooled [`PointLight`] slot entities.
    Sync,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalLightSlot(pub u8);

#[derive(Resource, Default)]
struct LocalLightSlotsSpawned(bool);

pub struct LocalLightPlugin;

impl Plugin for LocalLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FireVisualFramePlugin)
            .add_plugins(crate::render::extraction::VegetationVisualExtractPlugin)
            .init_resource::<ActiveLights>()
            .init_resource::<LocalLightSlotsSpawned>()
            .add_message::<RequestLocalLight>()
            .configure_sets(
                Update,
                LocalLightExtractSet::Sync.after(LocalLightExtractSet::Collect),
            )
            .add_systems(Startup, spawn_local_light_slots_once)
            .add_systems(
                Update,
                (
                    collect_active_lights.in_set(LocalLightExtractSet::Collect),
                    sync_active_lights_to_point_lights.in_set(LocalLightExtractSet::Sync),
                ),
            );
    }
}

fn spawn_local_light_slots_once(
    mut commands: Commands,
    mut done: ResMut<LocalLightSlotsSpawned>,
    existing: Query<(), With<LocalLightSlot>>,
) {
    if done.0 {
        return;
    }
    if !existing.is_empty() {
        done.0 = true;
        return;
    }
    done.0 = true;
    for i in 0..MAX_ACTIVE_LOCAL_LIGHTS {
        let i = i as u8;
        commands.spawn((
            LocalLightSlot(i),
            Transform::default(),
            Visibility::Inherited,
            PointLight {
                color: Color::WHITE,
                intensity: 0.0,
                range: 1.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Name::new(format!("LocalLightSlot_{i}")),
        ));
    }
}

#[inline]
fn score_request(r: &RequestLocalLight, camera_pos: Vec3) -> f32 {
    let dist2 = r.position.distance_squared(camera_pos);
    let distance_score = 1.0 / (1.0 + dist2 * 0.01);
    let intensity_score = r.intensity * 0.0001;
    r.priority * (distance_score + intensity_score)
}

fn collect_active_lights(
    mut reader: MessageReader<RequestLocalLight>,
    mut active: ResMut<ActiveLights>,
    camera_q: Query<&GlobalTransform, With<MainWorldCamera>>,
) {
    let camera_pos = camera_q
        .iter()
        .next()
        .map(|gt| gt.translation())
        .unwrap_or(Vec3::ZERO);

    let mut scored: Vec<(f32, LightData)> = Vec::new();
    for r in reader.read() {
        let s = score_request(&r, camera_pos);
        scored.push((
            s,
            LightData {
                position: r.position,
                color: r.color,
                intensity: r.intensity,
                range: r.range,
                flicker_phase: r.flicker_phase,
                flicker_strength: r.flicker_strength,
                category: r.category,
            },
        ));
    }

    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    scored.truncate(MAX_ACTIVE_LOCAL_LIGHTS);

    active.visible.clear();
    for (_, ld) in scored {
        active.visible.push(ld);
    }
}

fn sync_active_lights_to_point_lights(
    active: Res<ActiveLights>,
    time: Res<Time>,
    mut q: Query<(&LocalLightSlot, &mut PointLight, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    for (slot, mut pl, mut xf) in &mut q {
        let idx = slot.0 as usize;
        if let Some(ld) = active.visible.get(idx) {
            xf.translation = ld.position;
            pl.color = ld.color;
            let flicker = (t * 7.0 + ld.flicker_phase).sin() * ld.flicker_strength;
            pl.intensity = (ld.intensity * (1.0 + flicker)).max(0.0);
            pl.range = ld.range.max(0.001);
        } else {
            pl.intensity = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_indices_cover_max() {
        assert_eq!(MAX_ACTIVE_LOCAL_LIGHTS, 16);
        assert!(LocalLightSlot(15).0 < MAX_ACTIVE_LOCAL_LIGHTS as u8);
    }

    #[test]
    fn score_prefers_higher_priority_at_same_distance() {
        let cam = Vec3::ZERO;
        let low = RequestLocalLight {
            position: Vec3::new(10.0, 0.0, 0.0),
            color: Color::WHITE,
            intensity: 100_000.0,
            range: 100.0,
            priority: 0.5,
            category: LightCategory::Environment,
            flicker_phase: 0.0,
            flicker_strength: 0.0,
        };
        let high = RequestLocalLight {
            priority: 2.0,
            ..low
        };
        assert!(score_request(&high, cam) > score_request(&low, cam));
    }
}
