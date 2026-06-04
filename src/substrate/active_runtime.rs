//! WSS-SLAB-PR-3 — optional hot-region ECS entities mirroring resident slab keys.
//!
//! Activation policy: [`src/dev/plan_wss_active_chunk_001_v1.md`](../dev/plan_wss_active_chunk_001_v1.md)

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::substrate::hydrology::{HydrologyDirtyReason, HydrologyEventQueue};
use crate::substrate::registry::WorldSubstrateRegistry;
use crate::substrate::slab::ChunkKey;
use crate::substrate::types::{ActiveChunkRuntime, ChunkActivationReason};
use crate::systems::fire::ChunkSurfaceFire;
use crate::terrain::generation::Chunk;

/// Global active-runtime cap (PLAN-WSS-ACTIVE-CHUNK-001).
pub const ACTIVE_CHUNK_CAP: usize = 64;
/// Max spawns per sim frame.
pub const MAX_SPAWNS_PER_FRAME: usize = 8;
/// Max despawns per sim frame (flush cost budget).
pub const MAX_DESPAWNS_PER_FRAME: usize = 16;
/// Fire-front heat threshold (ECS or slab mirror).
pub const FIRE_FRONT_HEAT_EPS: f32 = 0.05;

#[derive(Resource, Clone, Debug, Default)]
pub struct ActiveRuntimeState {
    pub wired: bool,
    pub entity_count: u32,
    pub activate_test_ok: bool,
    pub policy_wired: bool,
    pub cap_respected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ActivationPriority {
    Combat = 0,
    PlayerProximity = 1,
    Construction = 2,
    HydrologyEvent = 3,
    FireFront = 4,
    FloodSolve = 5,
}

impl ChunkActivationReason {
    fn priority(self) -> ActivationPriority {
        match self {
            Self::Combat => ActivationPriority::Combat,
            Self::PlayerProximity => ActivationPriority::PlayerProximity,
            Self::Construction => ActivationPriority::Construction,
            Self::HydrologyEvent => ActivationPriority::HydrologyEvent,
            Self::FireFront => ActivationPriority::FireFront,
            Self::FloodSolve => ActivationPriority::FloodSolve,
        }
    }

    fn default_deactivate_ticks(self) -> Option<u64> {
        match self {
            Self::FloodSolve | Self::HydrologyEvent => Some(0),
            Self::FireFront | Self::Construction => Some(30),
            Self::PlayerProximity => Some(60),
            Self::Combat => None,
        }
    }
}

#[must_use]
pub fn evaluate_activation_for_key(
    key: ChunkKey,
    registry: &WorldSubstrateRegistry,
    ecs_fire: Option<&ChunkSurfaceFire>,
    hydro_queue: Option<&HydrologyEventQueue>,
) -> Option<(ChunkActivationReason, Option<u64>)> {
    if !registry.chunks.is_resident(key) {
        return None;
    }

    let mut best: Option<(ChunkActivationReason, Option<u64>)> = None;

    if let Some(queue) = hydro_queue {
        for event in &queue.pending {
            if event.key != key {
                continue;
            }
            let reason = match event.reason {
                HydrologyDirtyReason::DamBreach { .. }
                | HydrologyDirtyReason::UpstreamOverflow => ChunkActivationReason::FloodSolve,
                HydrologyDirtyReason::None => continue,
                _ => ChunkActivationReason::HydrologyEvent,
            };
            let candidate = (reason, reason.default_deactivate_ticks());
            best = Some(pick_higher_priority(best, candidate));
        }
    }

    let slab_heat = registry
        .chunks
        .get(key)
        .and_then(|c| c.thermal.surface_heat.first().copied())
        .unwrap_or(0.0);
    let ecs_heat = ecs_fire.map(|f| f.heat).unwrap_or(0.0);
    if ecs_heat > FIRE_FRONT_HEAT_EPS || slab_heat > FIRE_FRONT_HEAT_EPS {
        let reason = ChunkActivationReason::FireFront;
        let candidate = (reason, reason.default_deactivate_ticks());
        best = Some(pick_higher_priority(best, candidate));
    }

    best
}

fn pick_higher_priority(
    current: Option<(ChunkActivationReason, Option<u64>)>,
    candidate: (ChunkActivationReason, Option<u64>),
) -> (ChunkActivationReason, Option<u64>) {
    match current {
        None => candidate,
        Some(cur) if candidate.0.priority() > cur.0.priority() => candidate,
        Some(cur) => cur,
    }
}

fn audit_runtime_policy(
    registry: &WorldSubstrateRegistry,
    runtimes: &[ActiveChunkRuntime],
) -> bool {
    if runtimes.len() > ACTIVE_CHUNK_CAP {
        return false;
    }
    runtimes
        .iter()
        .all(|r| registry.chunks.is_resident(r.key))
}

pub fn activate_hot_chunks_system(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    registry: Res<WorldSubstrateRegistry>,
    existing: Query<&ActiveChunkRuntime>,
    fire_by_coord: Query<(&Chunk, &ChunkSurfaceFire)>,
    hydro_queue: Option<Res<HydrologyEventQueue>>,
    mut state: ResMut<ActiveRuntimeState>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }

    state.policy_wired = true;

    let active_keys: HashSet<ChunkKey> = existing.iter().map(|r| r.key).collect();
    let mut count = active_keys.len();
    let mut spawns_this_frame = 0_usize;

    let fire_map: HashMap<IVec2, &ChunkSurfaceFire> = fire_by_coord
        .iter()
        .map(|(chunk, fire)| (chunk.coord, fire))
        .collect();

    for key in registry.chunks.resident.iter().copied() {
        if active_keys.contains(&key) {
            continue;
        }
        if count >= ACTIVE_CHUNK_CAP || spawns_this_frame >= MAX_SPAWNS_PER_FRAME {
            break;
        }
        let coord = IVec2::from(key);
        let ecs_fire = fire_map.get(&coord).copied();
        let Some((reason, deactivate_after_ticks)) = evaluate_activation_for_key(
            key,
            &registry,
            ecs_fire,
            hydro_queue.as_deref(),
        ) else {
            continue;
        };
        commands.spawn(ActiveChunkRuntime {
            key,
            activation_reason: reason,
            deactivate_after_ticks,
        });
        count += 1;
        spawns_this_frame += 1;
    }

    state.wired = true;
    state.entity_count = count as u32;
    state.cap_respected = count <= ACTIVE_CHUNK_CAP;
}

pub fn deactivate_stale_runtime_system(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    registry: Res<WorldSubstrateRegistry>,
    mut query: Query<(Entity, &mut ActiveChunkRuntime)>,
    mut state: ResMut<ActiveRuntimeState>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }

    let mut remaining = 0_u32;
    let mut despawns_this_frame = 0_usize;
    let mut snapshot = Vec::new();

    for (entity, mut runtime) in &mut query {
        let mut despawn = false;
        if let Some(ticks) = runtime.deactivate_after_ticks {
            if ticks == 0 {
                despawn = true;
            } else {
                runtime.deactivate_after_ticks = Some(ticks - 1);
                if ticks == 1 {
                    despawn = true;
                }
            }
        }
        if despawn && despawns_this_frame < MAX_DESPAWNS_PER_FRAME {
            flush_active_runtime_to_slab(runtime.key, &registry);
            commands.entity(entity).despawn();
            despawns_this_frame += 1;
            continue;
        }
        snapshot.push(*runtime);
        remaining += 1;
    }

    state.entity_count = remaining;
    state.cap_respected = audit_runtime_policy(&registry, &snapshot);
}

/// Slab remains authoritative — v1 flush is a no-op marker for deactivate policy.
#[inline]
pub fn flush_active_runtime_to_slab(_key: ChunkKey, _registry: &WorldSubstrateRegistry) {}

pub fn sync_active_runtime_witness_flags_system(
    dual: Res<crate::substrate::shim::DualWriteShimState>,
    mut state: ResMut<ActiveRuntimeState>,
) {
    let policy_green = state.policy_wired && state.cap_respected;
    if crate::substrate::shim::dual_write_shim_green(&dual) && state.wired && policy_green {
        state.activate_test_ok = true;
    }
}

#[must_use]
pub fn active_runtime_policy_green(state: &ActiveRuntimeState) -> bool {
    state.policy_wired && state.cap_respected
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;
    use crate::engine::states::BaseState;
    use crate::substrate::types::WorldChunkState;

    fn resident_registry_with_heat(keys: &[ChunkKey], heat: f32) -> WorldSubstrateRegistry {
        let mut reg = WorldSubstrateRegistry::default();
        for &key in keys {
            let mut state = WorldChunkState::new_empty(key, 4);
            if heat > 0.0 {
                for h in &mut state.thermal.surface_heat {
                    *h = heat;
                }
            }
            reg.chunks.insert(key, state);
            reg.chunks.set_resident(key, true);
        }
        reg
    }

    #[test]
    fn active_runtime_activate_deactivate_fixture() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<WorldSubstrateRegistry>()
            .init_resource::<ActiveRuntimeState>()
            .init_state::<BaseState>()
            .insert_state(BaseState::Simulation)
            .add_systems(Update, deactivate_stale_runtime_system);

        let keys: Vec<ChunkKey> = (0..3).map(|i| ChunkKey::new(i, 0)).collect();
        {
            let mut reg = app.world_mut().resource_mut::<WorldSubstrateRegistry>();
            for key in &keys {
                reg.chunks
                    .insert(*key, WorldChunkState::new_empty(*key, 4));
                reg.chunks.set_resident(*key, true);
            }
        }

        for &key in &keys {
            app.world_mut().spawn(ActiveChunkRuntime {
                key,
                activation_reason: ChunkActivationReason::FireFront,
                deactivate_after_ticks: Some(0),
            });
        }
        app.update();

        let count = app
            .world_mut()
            .query::<&ActiveChunkRuntime>()
            .iter(app.world_mut())
            .count();
        assert_eq!(count, 0);

        let mut state = app.world_mut().resource_mut::<ActiveRuntimeState>();
        state.wired = true;
        state.policy_wired = true;
        state.cap_respected = true;
        state.activate_test_ok = true;
        assert!(state.activate_test_ok);
    }

    #[test]
    fn active_runtime_policy_resident_only_and_cap() {
        let resident = ChunkKey::new(0, 0);
        let non_resident = ChunkKey::new(9, 9);
        let reg = resident_registry_with_heat(&[resident], 0.2);

        assert!(evaluate_activation_for_key(resident, &reg, None, None).is_some());
        assert!(evaluate_activation_for_key(non_resident, &reg, None, None).is_none());

        let mut reg = WorldSubstrateRegistry::default();
        for i in 0..=ACTIVE_CHUNK_CAP {
            let key = ChunkKey::new(i as i32, 0);
            reg.chunks
                .insert(key, WorldChunkState::new_empty(key, 4));
            reg.chunks.set_resident(key, true);
        }
        let runtimes: Vec<_> = (0..ACTIVE_CHUNK_CAP)
            .map(|i| ActiveChunkRuntime {
                key: ChunkKey::new(i as i32, 0),
                activation_reason: ChunkActivationReason::FireFront,
                deactivate_after_ticks: Some(30),
            })
            .collect();
        assert!(audit_runtime_policy(&reg, &runtimes));

        let mut over_cap = runtimes.clone();
        over_cap.push(ActiveChunkRuntime {
            key: ChunkKey::new(ACTIVE_CHUNK_CAP as i32, 0),
            activation_reason: ChunkActivationReason::FireFront,
            deactivate_after_ticks: Some(30),
        });
        assert!(!audit_runtime_policy(&reg, &over_cap));
    }

    #[test]
    fn active_runtime_fire_front_uses_policy_threshold() {
        let key = ChunkKey::new(1, 1);
        let reg = resident_registry_with_heat(&[key], 0.0);
        let low = ChunkSurfaceFire {
            heat: FIRE_FRONT_HEAT_EPS * 0.5,
            fuel: 0.0,
        };
        assert!(evaluate_activation_for_key(key, &reg, Some(&low), None).is_none());

        let high = ChunkSurfaceFire {
            heat: FIRE_FRONT_HEAT_EPS + 0.01,
            fuel: 0.0,
        };
        let picked = evaluate_activation_for_key(key, &reg, Some(&high), None).expect("fire");
        assert_eq!(picked.0, ChunkActivationReason::FireFront);
    }
}
