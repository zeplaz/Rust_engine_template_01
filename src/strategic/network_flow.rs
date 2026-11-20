//! **Network flow → [`ChunkStrategicOverlay`](super::ChunkStrategicOverlay)** — solvers write only SOA fields;
//! gameplay entities read overlays. U7 extension: [`NetworkDirtyMask`] when [`ChunkNetworkDigest`](super::spatial_network::ChunkNetworkDigest) signatures move.
//!
//! Ref-books **LOOP-11** · LEGACY `game_object_update` (GEA 4e I §8.3.3 Event-Based Updating,
//! gea_v1_e4 p511–512 / gea_v2_e4 p606): chunk diffusion runs only when digest signatures move
//! ([`network_digest_marks_flow_dirty_system`] sets [`NETWORK_DIRTY_FLOW`]);
//! [`network_flow_chunk_local_solver_system`] skips clean chunks and clears the bit after a pass.
//! Do **not** retick flow SOA every frame.

use bevy::prelude::*;

use super::spatial_network::{
    ChunkNetworkDigest, NetworkInsulatedNode, NetworkType, SpatialNetworkGraph,
};
use super::world_field_layers::{FieldLayerUpdateClass, WorldFieldLayerEpoch};
use super::{ChunkStrategicOverlay, InfrastructureGraph};

/// When set, chunk-local network flow should be recomputed (or blended from cold start).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct NetworkDirtyMask {
    pub mask: u8,
}

pub const NETWORK_DIRTY_FLOW: u8 = 1 << 0;
pub const NETWORK_DIRTY_CONNECTIVITY: u8 = 1 << 1;

#[derive(Clone, Copy, Debug, Default)]
pub struct NetworkFlowFieldSample {
    pub power_flow: f32,
    pub logistics_flow: f32,
    pub control_pressure: f32,
    pub visibility: f32,
}

/// Samples overlay flow SOA at world tile **xz** (`IVec2::new(x, z)`). First chunk whose bounds contain the tile wins.
pub fn sample_network_flow_at_world_tile(overlays: &Query<&ChunkStrategicOverlay>, xz: IVec2) -> NetworkFlowFieldSample {
    let p = Vec2::new(xz.x as f32, xz.y as f32);
    for ov in overlays.iter() {
        let Some(i) = cell_index_in_overlay(ov, p) else {
            continue;
        };
        return NetworkFlowFieldSample {
            power_flow: ov.power_flow.get(i).copied().unwrap_or(0.0),
            logistics_flow: ov.logistics_flow.get(i).copied().unwrap_or(0.0),
            control_pressure: ov.control_pressure.get(i).copied().unwrap_or(0.0),
            visibility: ov.visibility.get(i).copied().unwrap_or(0.0),
        };
    }
    NetworkFlowFieldSample::default()
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct NetworkFlowPrevSignatures {
    pub road_signature: u64,
    pub power_signature: u64,
    pub pipe_signature: u64,
    pub connectivity_hash: u64,
    pub flow_hash: u64,
    pub initialized: bool,
}

/// Digest signature tuple used by the LOOP-11 dirty gate (road/power/pipe/connectivity/flow).
#[must_use]
pub fn network_digest_signature_tuple(digest: &ChunkNetworkDigest) -> (u64, u64, u64, u64, u64) {
    (
        digest.road_signature,
        digest.power_signature,
        digest.pipe_signature,
        digest.connectivity_hash,
        digest.flow_hash,
    )
}

/// **LOOP-11 gate:** `true` ⇒ overlay flow SOA should be marked dirty (digest moved / first tick).
#[must_use]
pub fn network_flow_digest_changed(
    prev: &NetworkFlowPrevSignatures,
    digest: &ChunkNetworkDigest,
) -> bool {
    let cur = network_digest_signature_tuple(digest);
    !prev.initialized
        || cur.0 != prev.road_signature
        || cur.1 != prev.power_signature
        || cur.2 != prev.pipe_signature
        || cur.3 != prev.connectivity_hash
        || cur.4 != prev.flow_hash
}

/// **LOOP-11 gate:** `true` ⇒ chunk-local solver should tick this overlay.
#[must_use]
pub fn network_flow_chunk_needs_solve(mask: &NetworkDirtyMask) -> bool {
    mask.mask & NETWORK_DIRTY_FLOW != 0
}

#[inline]
fn cell_index_in_overlay(overlay: &ChunkStrategicOverlay, world_xz: Vec2) -> Option<usize> {
    let cw = overlay.size.x as i32;
    let ch = overlay.size.y as i32;
    if cw <= 0 || ch <= 0 {
        return None;
    }
    let tx = world_xz.x.floor() as i32;
    let tz = world_xz.y.floor() as i32;
    let bx = overlay.chunk_coord.x * cw;
    let bz = overlay.chunk_coord.y * ch;
    let lx = tx - bx;
    let lz = tz - bz;
    if lx < 0 || lz < 0 || lx >= cw || lz >= ch {
        return None;
    }
    Some((lz * cw + lx) as usize)
}

#[inline]
fn inject_at(field: &mut [f32], idx: Option<usize>, amount: f32) {
    if amount <= 0.0 {
        return;
    }
    let Some(i) = idx else {
        return;
    };
    if i < field.len() {
        field[i] = (field[i] + amount).min(1.0);
    }
}

/// Effective visibility for bunkers / trenches: base × layer × insulation × (1 - enemy sensor pressure).
#[inline]
pub fn effective_visibility_sample(
    base_visibility: f32,
    layer: crate::strategic::spatial_network::LayerType,
    insulation_strength: f32,
    enemy_sensor_pressure: f32,
) -> f32 {
    let ins = insulation_strength.clamp(0.0, 1.0);
    let insulation_factor = 1.0 - ins * 0.85;
    let layer_pen = layer.visibility_factor();
    let sensor = enemy_sensor_pressure.clamp(0.0, 1.0);
    (base_visibility * layer_pen * insulation_factor * (1.0 - sensor * 0.7)).clamp(0.0, 1.0)
}

fn diffuse_chunk_field(
    field: &mut [f32],
    width: u32,
    height: u32,
    diffusion: f32,
    decay: f32,
    cap: f32,
) {
    if width == 0 || height == 0 {
        return;
    }
    let w = width as usize;
    let h = height as usize;
    let mut tmp = field.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut sum = 0.0f32;
            let mut cnt = 0.0f32;
            if y > 0 {
                sum += field[i - w];
                cnt += 1.0;
            }
            if y + 1 < h {
                sum += field[i + w];
                cnt += 1.0;
            }
            if x > 0 {
                sum += field[i - 1];
                cnt += 1.0;
            }
            if x + 1 < w {
                sum += field[i + 1];
                cnt += 1.0;
            }
            let neigh = if cnt > 0.0 {
                sum / cnt
            } else {
                field[i]
            };
            let v = field[i] + diffusion * (neigh - field[i]) - decay * field[i];
            tmp[i] = v.clamp(0.0, cap);
        }
    }
    field.copy_from_slice(&tmp);
}

/// If digest signatures moved, mark all strategic overlay chunks for a flow refresh.
pub fn network_digest_marks_flow_dirty_system(
    digest: Res<ChunkNetworkDigest>,
    mut prev: ResMut<NetworkFlowPrevSignatures>,
    mut masks: Query<&mut NetworkDirtyMask, With<ChunkStrategicOverlay>>,
) {
    let changed = network_flow_digest_changed(&prev, &digest);
    if changed {
        for mut m in masks.iter_mut() {
            m.mask |= NETWORK_DIRTY_FLOW;
        }
    }
    let cur = network_digest_signature_tuple(&digest);
    prev.road_signature = cur.0;
    prev.power_signature = cur.1;
    prev.pipe_signature = cur.2;
    prev.connectivity_hash = cur.3;
    prev.flow_hash = cur.4;
    prev.initialized = true;
}

/// Chunk-local diffusion from [`SpatialNetworkGraph`] into overlay flow SOA. Does not touch terrain entities.
/// Runs only when [`NetworkDirtyMask`] has [`NETWORK_DIRTY_FLOW`]; bit is cleared after a successful pass.
pub fn network_flow_chunk_local_solver_system(
    graph: Res<SpatialNetworkGraph>,
    infra: Res<InfrastructureGraph>,
    mut epochs: ResMut<WorldFieldLayerEpoch>,
    mut q: Query<(&mut ChunkStrategicOverlay, &mut NetworkDirtyMask), With<ChunkStrategicOverlay>>,
) {
    let iters = 2u32;
    let mut any_solved = false;
    for (mut overlay, mut mask) in q.iter_mut() {
        // LOOP-11: skip chunks whose digest/graph revision did not mark flow dirty.
        if !network_flow_chunk_needs_solve(&mask) {
            continue;
        }
        let n = overlay.len_cells();
        if n == 0 {
            mask.mask &= !NETWORK_DIRTY_FLOW;
            continue;
        }
        overlay.power_flow.resize(n, 0.0);
        overlay.logistics_flow.resize(n, 0.0);
        overlay.control_pressure.resize(n, 0.0);
        overlay.visibility.resize(n, 0.0);
        for v in overlay.power_flow.iter_mut() {
            *v = 0.0;
        }
        for v in overlay.logistics_flow.iter_mut() {
            *v = 0.0;
        }
        for v in overlay.visibility.iter_mut() {
            *v = 0.0;
        }

        for e in &graph.edges {
            let from_n = infra.nodes.iter().find(|n| n.id == e.from_node);
            let to_n = infra.nodes.iter().find(|n| n.id == e.to_node);
            let (Some(a), Some(b)) = (from_n, to_n) else {
                continue;
            };
            let amt = (e.capacity * (1.0 - e.resistance).max(0.0)).min(1.0);
            let rules = e.network.flow_rules();
            let layer_scale = 1.0 / (1.0 + rules.layer_penalty * e.layer_from.idx() as f32 * 0.15);
            let inject = amt * layer_scale;
            let ia = cell_index_in_overlay(&overlay, a.position);
            let ib = cell_index_in_overlay(&overlay, b.position);

            match e.network {
                NetworkType::Power => {
                    inject_at(&mut overlay.power_flow, ia, inject * 0.5);
                    inject_at(&mut overlay.power_flow, ib, inject * 0.5);
                }
                NetworkType::Fluid => {
                    inject_at(&mut overlay.logistics_flow, ia, inject * 0.45);
                    inject_at(&mut overlay.logistics_flow, ib, inject * 0.45);
                }
                NetworkType::Road | NetworkType::Logistics | NetworkType::MilitarySupply => {
                    inject_at(&mut overlay.logistics_flow, ia, inject * 0.5);
                    inject_at(&mut overlay.logistics_flow, ib, inject * 0.5);
                }
                NetworkType::Data => {
                    inject_at(&mut overlay.visibility, ia, inject * 0.5);
                    inject_at(&mut overlay.visibility, ib, inject * 0.5);
                }
            }
        }

        let pw = overlay.size.x.max(1);
        let ph = overlay.size.y.max(1);
        let power_rules = NetworkType::Power.flow_rules();
        for _ in 0..iters {
            diffuse_chunk_field(
                &mut overlay.power_flow,
                pw,
                ph,
                power_rules.diffusion_rate,
                power_rules.decay,
                power_rules.capacity_limit,
            );
            let log_rules = NetworkType::Logistics.flow_rules();
            diffuse_chunk_field(
                &mut overlay.logistics_flow,
                pw,
                ph,
                log_rules.diffusion_rate,
                log_rules.decay,
                log_rules.capacity_limit,
            );
            let data_rules = NetworkType::Data.flow_rules();
            diffuse_chunk_field(
                &mut overlay.visibility,
                pw,
                ph,
                data_rules.diffusion_rate,
                data_rules.decay,
                data_rules.capacity_limit.max(1.0),
            );
        }

        for i in 0..n {
            let ctrl = overlay.faction_control[i][0].clamp(0.0, 1.0);
            let log = overlay.logistics_flow.get(i).copied().unwrap_or(0.0);
            overlay.control_pressure[i] = (ctrl * 0.5 + log * 0.5).min(1.0);
            let base_vis = overlay.visibility.get(i).copied().unwrap_or(0.0).max(0.02);
            let ew = overlay.ew_denial.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0);
            let recon = overlay.recon_confidence[i][0].clamp(0.0, 1.0);
            overlay.visibility[i] = (base_vis * (1.0 - ew * 0.6) * (0.35 + recon * 0.65)).min(1.0);
        }

        mask.mask &= !NETWORK_DIRTY_FLOW;
        any_solved = true;
    }
    if any_solved {
        epochs.bump(FieldLayerUpdateClass::Slow);
    }
}

/// Apply bunker / trench insulation to visibility samples (pair [`NetworkInsulatedNode`] + [`SpatialNode`](super::spatial_network::SpatialNode)).
pub fn network_insulation_visibility_post_system(
    mut overlays: Query<&mut ChunkStrategicOverlay>,
    insulated: Query<(&NetworkInsulatedNode, &super::spatial_network::SpatialNode)>,
) {
    for (ins, sn) in insulated.iter() {
        for mut overlay in overlays.iter_mut() {
            let wx = sn.tile.x as f32;
            let wz = sn.tile.z as f32;
            let Some(ci) = cell_index_in_overlay(
                &overlay,
                Vec2::new(wx, wz),
            ) else {
                continue;
            };
            if ci >= overlay.visibility.len() {
                continue;
            }
            let v = overlay.visibility[ci];
            overlay.visibility[ci] = effective_visibility_sample(
                v,
                ins.layer,
                ins.insulation_strength,
                overlay.artillery_danger[ci][0],
            );
        }
    }
}

#[cfg(test)]
mod loop11_revision_gate_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::math::{IVec2, UVec2};
    use bevy::prelude::App;

    #[test]
    fn network_flow_digest_changed_true_until_initialized() {
        let prev = NetworkFlowPrevSignatures::default();
        let digest = ChunkNetworkDigest::default();
        assert!(
            network_flow_digest_changed(&prev, &digest),
            "LOOP-11: first digest tick must dirty"
        );
    }

    #[test]
    fn network_flow_digest_changed_false_when_signatures_match() {
        let digest = ChunkNetworkDigest {
            road_signature: 1,
            power_signature: 2,
            pipe_signature: 3,
            connectivity_hash: 4,
            flow_hash: 5,
            ..Default::default()
        };
        let prev = NetworkFlowPrevSignatures {
            road_signature: 1,
            power_signature: 2,
            pipe_signature: 3,
            connectivity_hash: 4,
            flow_hash: 5,
            initialized: true,
        };
        assert!(
            !network_flow_digest_changed(&prev, &digest),
            "LOOP-11: matching digest must not retick"
        );
    }

    #[test]
    fn network_flow_digest_changed_true_on_flow_hash_bump() {
        let digest = ChunkNetworkDigest {
            flow_hash: 99,
            ..Default::default()
        };
        let prev = NetworkFlowPrevSignatures {
            initialized: true,
            ..Default::default()
        };
        assert!(network_flow_digest_changed(&prev, &digest));
    }

    #[test]
    fn network_flow_chunk_needs_solve_respects_dirty_bit() {
        let clean = NetworkDirtyMask { mask: 0 };
        let dirty = NetworkDirtyMask {
            mask: NETWORK_DIRTY_FLOW,
        };
        assert!(!network_flow_chunk_needs_solve(&clean));
        assert!(network_flow_chunk_needs_solve(&dirty));
    }

    #[test]
    fn network_flow_solver_skips_clean_and_clears_dirty() {
        let mut app = App::new();
        app.init_resource::<SpatialNetworkGraph>()
            .init_resource::<InfrastructureGraph>()
            .init_resource::<crate::strategic::world_field_layers::WorldFieldLayerEpoch>();
        let overlay = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(2, 2));
        let dirty_e = app
            .world_mut()
            .spawn((
                overlay.clone(),
                NetworkDirtyMask {
                    mask: NETWORK_DIRTY_FLOW,
                },
            ))
            .id();
        let clean_e = app
            .world_mut()
            .spawn((overlay, NetworkDirtyMask { mask: 0 }))
            .id();

        app.world_mut()
            .run_system_once(network_flow_chunk_local_solver_system)
            .unwrap();

        let dirty_mask = app.world().entity(dirty_e).get::<NetworkDirtyMask>().unwrap();
        let clean_mask = app.world().entity(clean_e).get::<NetworkDirtyMask>().unwrap();
        assert_eq!(
            dirty_mask.mask & NETWORK_DIRTY_FLOW,
            0,
            "LOOP-11: successful pass clears dirty bit"
        );
        assert_eq!(
            clean_mask.mask, 0,
            "LOOP-11: clean chunk must stay skipped"
        );
    }

    #[test]
    fn network_digest_marks_dirty_only_when_signatures_move() {
        let mut app = App::new();
        app.init_resource::<ChunkNetworkDigest>()
            .init_resource::<NetworkFlowPrevSignatures>();
        let e = app
            .world_mut()
            .spawn((
                ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(1, 1)),
                NetworkDirtyMask { mask: 0 },
            ))
            .id();

        // First tick: uninitialized prev → dirty.
        app.world_mut()
            .run_system_once(network_digest_marks_flow_dirty_system)
            .unwrap();
        assert_ne!(
            app.world().entity(e).get::<NetworkDirtyMask>().unwrap().mask & NETWORK_DIRTY_FLOW,
            0
        );

        // Clear and tick again with same signatures → stay clean.
        app.world_mut()
            .entity_mut(e)
            .get_mut::<NetworkDirtyMask>()
            .unwrap()
            .mask = 0;
        app.world_mut()
            .run_system_once(network_digest_marks_flow_dirty_system)
            .unwrap();
        assert_eq!(
            app.world().entity(e).get::<NetworkDirtyMask>().unwrap().mask & NETWORK_DIRTY_FLOW,
            0,
            "LOOP-11: unchanged digest must not re-dirty"
        );

        // Bump flow_hash → dirty again.
        app.world_mut().resource_mut::<ChunkNetworkDigest>().flow_hash = 42;
        app.world_mut()
            .run_system_once(network_digest_marks_flow_dirty_system)
            .unwrap();
        assert_ne!(
            app.world().entity(e).get::<NetworkDirtyMask>().unwrap().mask & NETWORK_DIRTY_FLOW,
            0,
            "LOOP-11: digest bump must dirty"
        );
    }
}
