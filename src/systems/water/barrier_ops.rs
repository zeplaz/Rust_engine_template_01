//! General barrier install / breach intents — construction, scenario, or player UI enqueue;
//! water is the sole crest writer.

use bevy::prelude::*;

use super::surface_water::SurfaceWaterField;

/// Pending crest install (after dam commit / SetDamControl / player build complete).
#[derive(Resource, Debug, Default, Clone)]
pub struct BarrierInstallRequest {
    pub pending: Option<BarrierInstallSpec>,
}

#[derive(Debug, Clone)]
pub struct BarrierInstallSpec {
    pub world_z: i32,
    pub crest_m: f32,
    pub half_span: u32,
    pub center_world_x: Option<i32>,
}

/// Pending breach / crest clear (strike, structural failure, scenario destroy).
#[derive(Resource, Debug, Default, Clone)]
pub struct BarrierBreachRequest {
    pub pending: Option<BarrierBreachSpec>,
}

#[derive(Debug, Clone)]
pub struct BarrierBreachSpec {
    pub clear_crest: bool,
    pub breach_flow_m3s: f32,
}

pub fn apply_barrier_install_request_system(
    mut request: ResMut<BarrierInstallRequest>,
    mut field: ResMut<SurfaceWaterField>,
) {
    // Keep pending until the ROI is wired — do not drop the install.
    if request.pending.is_none() {
        return;
    }
    if !field.wired {
        return;
    }
    let Some(spec) = request.pending.take() else {
        return;
    };
    if spec.crest_m <= 0.0 {
        return;
    }
    field.install_barrier_row(
        spec.world_z,
        spec.crest_m,
        spec.half_span.max(1),
        spec.center_world_x,
    );
    info!(
        target: "water::barrier",
        "installed barrier crest_m={} at z={} half_span={}",
        spec.crest_m,
        spec.world_z,
        spec.half_span
    );
}

pub fn apply_barrier_breach_request_system(
    mut request: ResMut<BarrierBreachRequest>,
    mut field: ResMut<SurfaceWaterField>,
) {
    if request.pending.is_none() {
        return;
    }
    if !field.wired {
        return;
    }
    let Some(spec) = request.pending.take() else {
        return;
    };
    field.apply_barrier_breach(spec.clear_crest, spec.breach_flow_m3s);
    info!(
        target: "water::barrier",
        "breach applied clear_crest={} breach_flow_m3s={}",
        spec.clear_crest,
        spec.breach_flow_m3s
    );
}
