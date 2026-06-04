//! WSS-POST-SPINE — logistics pressure on slab + weather runbook v2 phase 2 (regional + traction).

use bevy::prelude::*;

use crate::economy::logistics::ThroughputSolverState;
use crate::render::LogisticsVisualSnapshot;
use crate::substrate::atmosphere::{AtmosphereClipmapStack, AtmosphereClipmapWitness};
use crate::substrate::registry::WorldSubstrateRegistry;
use crate::systems::transport::{TransportEdgeDirectory, TransportFieldStore};
use crate::systems::weather::GlobalRenewableWeatherFactors;

pub const WSS_POST_SPINE_GATE: &str = "WSS-POST-SPINE-001";

#[derive(Resource, Clone, Debug, Default)]
pub struct PostSpineWitness {
    pub logistics_pressure_on_slab: bool,
    pub regional_weather_wired: bool,
    pub climate_seed_present: bool,
    pub weather_runbook_phase2_green: bool,
    pub logistics_pressure_sample: f32,
    pub regional_weather_sample: f32,
}

#[must_use]
pub fn clipmap_l2_mean_scalar(stack: &AtmosphereClipmapStack) -> f32 {
    let Some(level) = stack.levels.get(2) else {
        return 0.0;
    };
    if level.smoke_density.is_empty() {
        return 0.0;
    }
    level.smoke_density.iter().sum::<f32>() / level.smoke_density.len() as f32
}

#[must_use]
pub fn mean_slab_congestion(registry: &WorldSubstrateRegistry) -> f32 {
    let mut sum = 0.0_f32;
    let mut n = 0_u32;
    for state in registry.chunks.chunks.values() {
        for &v in &state.dynamic.congestion {
            sum += v;
            n += 1;
        }
    }
    if n == 0 {
        return 0.0;
    }
    sum / n as f32
}

#[must_use]
pub fn climate_seed_present(registry: &WorldSubstrateRegistry) -> bool {
    !registry.chunks.is_empty()
        && registry.chunks.chunks.values().any(|c| {
            c.atmosphere.local.soil_moisture > 0.0
                || c.atmosphere.local.rain_intensity > 0.0
                || c.atmosphere.local.visibility_factor < 1.0
        })
}

/// Max corridor pressure from edge congestion or throughput load/capacity (when congestion unset).
#[must_use]
pub fn max_transport_logistics_pressure(
    fields: &TransportFieldStore,
    directory: Option<&TransportEdgeDirectory>,
    solver: Option<&ThroughputSolverState>,
) -> f32 {
    let from_fields = fields
        .by_edge
        .values()
        .map(|s| s.congestion)
        .fold(0.0_f32, f32::max);
    if from_fields > 1e-5 {
        return from_fields;
    }
    let Some(directory) = directory else {
        return 0.0;
    };
    let Some(solver) = solver else {
        return 0.0;
    };
    directory
        .by_edge
        .keys()
        .filter_map(|id| {
            let idx = id.0 as usize;
            if idx < solver.capacity.len() && solver.capacity[idx] > 0.01 {
                Some((solver.load[idx] / solver.capacity[idx]).clamp(0.0, 1.0))
            } else {
                None
            }
        })
        .fold(0.0_f32, f32::max)
}

/// Mirror transport corridor congestion into resident slab `dynamic.congestion` (W-SIM-4 traction stub).
pub fn mirror_logistics_pressure_to_slab_system(
    fields: Option<Res<TransportFieldStore>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    solver: Option<Res<ThroughputSolverState>>,
    registry: Option<ResMut<WorldSubstrateRegistry>>,
) {
    let (Some(mut registry), Some(fields)) = (registry, fields) else {
        return;
    };
    let max_cong = max_transport_logistics_pressure(
        fields.as_ref(),
        directory.as_deref(),
        solver.as_deref(),
    );
    if max_cong <= 1e-5 {
        return;
    }
    for state in registry.chunks.chunks.values_mut() {
        if state.dynamic.congestion.is_empty() {
            continue;
        }
        for v in &mut state.dynamic.congestion {
            *v = (*v * 0.9 + max_cong * 0.1).clamp(0.0, 1.0);
        }
    }
}

/// Regional L2 clipmap sample → slab `ChunkWeatherLocal` (weather runbook v2 phase 2).
pub fn sync_regional_weather_from_clipmap_system(
    clipmap: Option<Res<AtmosphereClipmapStack>>,
    registry: Option<ResMut<WorldSubstrateRegistry>>,
) {
    let (Some(clipmap), Some(mut registry)) = (clipmap, registry) else {
        return;
    };
    let regional = clipmap_l2_mean_scalar(&clipmap);
    if regional <= 1e-6 {
        return;
    }
    let rain_target = (regional * 1.15).clamp(0.0, 1.0);
    let fog_target = (regional * 0.85).clamp(0.0, 1.0);
    for state in registry.chunks.chunks.values_mut() {
        let local = &mut state.atmosphere.local;
        local.rain_intensity = local.rain_intensity * 0.92 + rain_target * 0.08;
        local.fog_density = local.fog_density * 0.92 + fog_target * 0.08;
        local.visibility_factor =
            (1.0 - local.fog_density * 0.35 - local.rain_intensity * 0.15).clamp(0.05, 1.0);
    }
}

/// Clipmap L2 cloud proxy → renewables (runbook: wire `GlobalRenewableWeatherFactors` to L2).
pub fn update_global_renewable_weather_from_clipmap_system(
    clipmap: Option<Res<AtmosphereClipmapStack>>,
    out: Option<ResMut<GlobalRenewableWeatherFactors>>,
) {
    let (Some(clipmap), Some(mut out)) = (clipmap, out) else {
        return;
    };
    let cloud = clipmap_l2_mean_scalar(&clipmap).clamp(0.0, 1.0);
    let wind_proxy = clipmap
        .levels
        .first()
        .map(|l0| {
            if l0.smoke_density.is_empty() {
                0.0
            } else {
                l0.smoke_density.iter().copied().fold(0.0_f32, f32::max)
            }
        })
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    out.wind_capacity_factor = (0.12 + wind_proxy * 0.98).clamp(0.05, 1.2);
    out.solar_capacity_factor = (1.0 - cloud * 0.88).clamp(0.05, 1.0);
}

pub fn sync_post_spine_witness_system(
    registry: Option<Res<WorldSubstrateRegistry>>,
    clipmap: Option<Res<AtmosphereClipmapStack>>,
    clipmap_witness: Option<Res<AtmosphereClipmapWitness>>,
    mut witness: ResMut<PostSpineWitness>,
) {
    *witness = compute_post_spine_witness(
        registry.as_deref(),
        clipmap.as_deref(),
        clipmap_witness.as_deref(),
    );
}

/// Pure witness rollup (lib refresh + live sync).
#[must_use]
pub fn compute_post_spine_witness(
    registry: Option<&WorldSubstrateRegistry>,
    clipmap: Option<&AtmosphereClipmapStack>,
    clipmap_witness: Option<&AtmosphereClipmapWitness>,
) -> PostSpineWitness {
    let congestion = registry.map(mean_slab_congestion).unwrap_or(0.0);
    let regional = clipmap.map(clipmap_l2_mean_scalar).unwrap_or(0.0);
    let regional_weather_wired = clipmap_witness
        .map(|w| w.clipmap_advect_wired && w.legacy_atmosphere_field_bridged)
        .unwrap_or(false)
        && clipmap.is_some_and(|c| {
            c.levels.len() >= 3 && !c.levels[2].smoke_density.is_empty()
        });
    let climate_seed_present = registry
        .map(climate_seed_present)
        .unwrap_or(false);
    let logistics_pressure_on_slab = congestion > 0.01;
    let weather_runbook_phase2_green =
        regional_weather_wired && logistics_pressure_on_slab && climate_seed_present;
    PostSpineWitness {
        logistics_pressure_on_slab,
        regional_weather_wired,
        climate_seed_present,
        weather_runbook_phase2_green,
        logistics_pressure_sample: congestion,
        regional_weather_sample: regional,
    }
}

/// Lib/witness helper — apply mirrored congestion without ECS schedule.
pub fn apply_logistics_pressure_mirror(registry: &mut WorldSubstrateRegistry, max_cong: f32) {
    if max_cong <= 1e-5 {
        return;
    }
    for state in registry.chunks.chunks.values_mut() {
        if state.dynamic.congestion.is_empty() {
            continue;
        }
        for v in &mut state.dynamic.congestion {
            *v = (*v * 0.9 + max_cong * 0.1).clamp(0.0, 1.0);
        }
    }
}

/// Modulate committed logistics overlay rows by slab traction (congestion).
pub fn apply_slab_traction_to_logistics_snapshot(
    registry: Option<Res<WorldSubstrateRegistry>>,
    mut snapshot: ResMut<LogisticsVisualSnapshot>,
) {
    let Some(registry) = registry.as_deref() else {
        return;
    };
    let traction = mean_slab_congestion(registry);
    if traction <= 1e-5 {
        return;
    }
    let factor = (1.0 - traction * 0.35).clamp(0.5, 1.0);
    for (_, load) in &mut snapshot.edge_rows {
        *load *= factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::slab::ChunkKey;
    use crate::substrate::types::WorldChunkState;
    use crate::substrate::UVec2;
    use crate::systems::transport::{EdgeFieldState, TransportEdgeId, TransportFieldStore};

    #[test]
    fn mirror_logistics_pressure_writes_slab_congestion() {
        let mut registry = WorldSubstrateRegistry::default();
        let key = ChunkKey::new(0, 0);
        let cell_count = (UVec2::new(4, 4).x * UVec2::new(4, 4).y) as usize;
        let mut state = WorldChunkState::new_empty(key, cell_count);
        state.dynamic.congestion = vec![0.0; cell_count];
        registry.chunks.insert(key, state);

        let mut fields = TransportFieldStore::default();
        fields.by_edge.insert(
            TransportEdgeId(1),
            EdgeFieldState {
                congestion: 0.55,
                ..Default::default()
            },
        );

        let mut app = App::new();
        app.insert_resource(registry)
            .insert_resource(fields)
            .add_systems(Update, mirror_logistics_pressure_to_slab_system);
        app.update();

        let reg = app.world().resource::<WorldSubstrateRegistry>();
        let mean = mean_slab_congestion(reg);
        assert!(mean > 0.04, "expected slab congestion mirror, got {mean}");
    }

    #[test]
    fn max_transport_pressure_falls_back_to_solver_load_ratio() {
        use crate::economy::logistics::ThroughputSolverState;
        use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId, TransportFieldStore};

        let mut fields = TransportFieldStore::default();
        fields.by_edge.insert(TransportEdgeId(1), Default::default());
        let mut directory = TransportEdgeDirectory::default();
        directory.by_edge.insert(TransportEdgeId(1), Default::default());
        let mut solver = ThroughputSolverState::default();
        solver.ensure_len(2);
        solver.load[1] = 4.5;
        solver.capacity[1] = 10.0;
        let p = max_transport_logistics_pressure(&fields, Some(&directory), Some(&solver));
        assert!((p - 0.45).abs() < 1e-4, "expected solver ratio, got {p}");
    }

    #[test]
    fn post_spine_witness_green_when_wired() {
        let mut registry = WorldSubstrateRegistry::default();
        let key = ChunkKey::new(1, 1);
        let cell_count = 16;
        let mut state = WorldChunkState::new_empty(key, cell_count);
        state.dynamic.congestion = vec![0.2; cell_count];
        state.atmosphere.local.rain_intensity = 0.1;
        registry.chunks.insert(key, state);

        let mut stack = AtmosphereClipmapStack::default();
        if let Some(l2) = stack.levels.get_mut(2) {
            for v in &mut l2.smoke_density {
                *v = 0.25;
            }
        }
        let clip_witness = AtmosphereClipmapWitness {
            clipmap_advect_wired: true,
            legacy_atmosphere_field_bridged: true,
            ..Default::default()
        };
        assert!(climate_seed_present(&registry));
        assert!(clipmap_l2_mean_scalar(&stack) > 0.2);
        assert!(clip_witness.clipmap_advect_wired);
        apply_logistics_pressure_mirror(&mut registry, 0.55);
        let witness = compute_post_spine_witness(
            Some(&registry),
            Some(&stack),
            Some(&clip_witness),
        );
        assert!(witness.weather_runbook_phase2_green);
    }
}
