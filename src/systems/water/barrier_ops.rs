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
    /// DF-4.4 crest profile: the dam's own footprint tiles (world XY). When non-empty these are
    /// the barrier cells and `world_z` / `half_span` / `center_world_x` are ignored.
    pub tiles: Vec<IVec2>,
    pub world_z: i32,
    pub crest_m: f32,
    pub half_span: u32,
    pub center_world_x: Option<i32>,
    /// Dam body that raised the install (`None` = scenario / script). Lets a later breach name
    /// one of several dams.
    pub dam: Option<Entity>,
}

/// DW-3: per-column crest updates from dam damage (construction publishes, water applies).
/// Latest fraction per tile wins.
#[derive(Resource, Debug, Default, Clone)]
pub struct BarrierColumnRequest {
    pub pending: Vec<(IVec2, f32)>,
}

impl BarrierColumnRequest {
    pub fn push(&mut self, tile: IVec2, frac: f32) {
        if let Some(e) = self.pending.iter_mut().find(|(t, _)| *t == tile) {
            e.1 = frac;
        } else {
            self.pending.push((tile, frac));
        }
    }
}

/// Apply queued per-column crests once the barrier exists (kept until the field is wired).
pub fn apply_barrier_column_request_system(
    mut request: ResMut<BarrierColumnRequest>,
    mut field: ResMut<SurfaceWaterField>,
) {
    if request.pending.is_empty() || !field.wired || field.hydraulics.barrier_height_m <= 0.0 {
        return;
    }
    let cols = std::mem::take(&mut request.pending);
    field.set_barrier_column_fractions(&cols);
}

/// DW-4: structure walls for the water solver (construction publishes, water applies).
/// `(tile, height above ground m)`; 0 clears. Latest per tile wins.
#[derive(Resource, Debug, Default, Clone)]
pub struct WaterObstacleRequest {
    pub pending: Vec<(IVec2, f32)>,
}

impl WaterObstacleRequest {
    pub fn push(&mut self, tile: IVec2, height_m: f32) {
        if let Some(e) = self.pending.iter_mut().find(|(t, _)| *t == tile) {
            e.1 = height_m;
        } else {
            self.pending.push((tile, height_m));
        }
    }
}

/// Apply queued structure walls once the field is wired (kept until then).
pub fn apply_water_obstacle_request_system(
    mut request: ResMut<WaterObstacleRequest>,
    mut field: ResMut<SurfaceWaterField>,
) {
    if request.pending.is_empty() || !field.wired {
        return;
    }
    let cells = std::mem::take(&mut request.pending);
    field.set_obstacles(&cells);
}

/// Pending breach / crest clear (strike, structural failure, scenario destroy).
///
/// Queue, not a slot: several producers (blast execute, dam integrity progress, scenario)
/// may write in the same tick. Water drains and merges them, so a leak queued first can
/// never shadow a later full collapse.
#[derive(Resource, Debug, Default, Clone)]
pub struct BarrierBreachRequest {
    pending: Vec<BarrierBreachSpec>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarrierBreachSpec {
    /// Dam body that raised the request (`None` = scenario / script).
    pub dam: Option<Entity>,
    pub clear_crest: bool,
    pub breach_flow_m3s: f32,
}

impl BarrierBreachRequest {
    /// Fold into the pending spec as it arrives: leaking dams push every tick and the field
    /// may stay unwired for a while, so the queue must stay bounded (≤ 1 entry).
    pub fn push(&mut self, spec: BarrierBreachSpec) {
        match self.pending.first() {
            Some(prev) => {
                let merged = merge_breach_specs(&[prev.clone(), spec]);
                self.pending.clear();
                self.pending.extend(merged);
            }
            None => self.pending.push(spec),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Take every queued spec and fold them into the one the field applies.
    pub fn drain_merged(&mut self) -> Option<BarrierBreachSpec> {
        let merged = merge_breach_specs(&self.pending);
        self.pending.clear();
        merged
    }
}

/// Fold same-tick breach specs: any crest clear wins, flow is the max (two writers about
/// one dam must not double-count). The water field holds a single barrier, so `dam`
/// reports the crest-clearing source if any, else the strongest leak.
#[must_use]
pub fn merge_breach_specs(specs: &[BarrierBreachSpec]) -> Option<BarrierBreachSpec> {
    let first = specs.first()?;
    let clear_crest = specs.iter().any(|s| s.clear_crest);
    let breach_flow_m3s = specs
        .iter()
        .map(|s| s.breach_flow_m3s.max(0.0))
        .fold(0.0_f32, f32::max);
    let dam = specs
        .iter()
        .find(|s| s.clear_crest)
        .or_else(|| {
            specs
                .iter()
                .max_by(|a, b| a.breach_flow_m3s.total_cmp(&b.breach_flow_m3s))
        })
        .unwrap_or(first)
        .dam;
    Some(BarrierBreachSpec {
        dam,
        clear_crest,
        breach_flow_m3s,
    })
}

pub fn apply_barrier_install_request_system(
    mut request: ResMut<BarrierInstallRequest>,
    mut field: ResMut<SurfaceWaterField>,
) {
    // Keep pending until the field is wired — do not drop the install.
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
    if spec.tiles.is_empty() {
        field.install_barrier_row(
            spec.world_z,
            spec.crest_m,
            spec.half_span.max(1),
            spec.center_world_x,
        );
    } else {
        field.install_barrier_tiles_for(&spec.tiles, spec.crest_m, spec.dam);
    }
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
    // Keep queued until the field is wired — do not drop the breach.
    if request.is_empty() || !field.wired {
        return;
    }
    let Some(spec) = request.drain_merged() else {
        return;
    };
    field.apply_barrier_breach_for(spec.dam, spec.clear_crest, spec.breach_flow_m3s);
    info!(
        target: "water::barrier",
        "breach applied clear_crest={} breach_flow_m3s={} dam={:?}",
        spec.clear_crest,
        spec.breach_flow_m3s,
        spec.dam
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DW-3: one broken column is a notch — water passes there, the intact columns still hold.
    #[test]
    fn dw3_broken_column_passes_water_only_through_its_notch() {
        let run = |break_col: bool| {
            let mut f = SurfaceWaterField::default();
            f.allocate(0, 0, 9, 9, vec![100.0; 81], 0.0);
            let tiles: Vec<IVec2> = (0..9).map(|x| IVec2::new(x, 4)).collect();
            f.install_barrier_tiles(&tiles, 10.0);
            for z in 5..9 {
                for x in 0..9 {
                    let i = f.idx(x, z);
                    f.depth_m[i] = 5.0;
                }
            }
            f.recompute_mass();
            if break_col {
                assert!(f.set_barrier_column_fractions(&[(IVec2::new(4, 4), 0.0)]));
                assert!(f.hydraulics.barrier_damaged);
            }
            // The 9x9 map drains through its open edges within seconds once the water is moving,
            // so the claim is about what ever passed the dam (peak depth below it), not the last
            // frame.
            let (mut notch, mut side) = (0.0_f32, 0.0_f32);
            for _ in 0..40 {
                f.step_virtual_pipes_sub(0.5);
                notch = notch.max(f.depth_m[f.idx(4, 3)]);
                side = side.max(f.depth_m[f.idx(0, 3)]);
            }
            (notch, side)
        };
        let (intact_mid, _) = run(false);
        let (notch, side) = run(true);
        assert!(intact_mid < 1e-3, "an intact dam holds: {intact_mid}");
        assert!(notch > 0.05, "water pours through the broken column: {notch}");
        assert!(notch > side, "the notch carries more than the side: {notch} vs {side}");
    }

    fn spec(dam: Option<Entity>, clear_crest: bool, flow: f32) -> BarrierBreachSpec {
        BarrierBreachSpec {
            dam,
            clear_crest,
            breach_flow_m3s: flow,
        }
    }

    #[test]
    fn leak_queued_first_does_not_shadow_collapse() {
        let a = Entity::from_bits(1 << 32 | 7);
        let b = Entity::from_bits(1 << 32 | 9);
        let mut req = BarrierBreachRequest::default();
        req.push(spec(Some(a), false, 40.0));
        req.push(spec(Some(b), true, 200.0));
        let merged = req.drain_merged().expect("merged");
        assert!(merged.clear_crest);
        assert_eq!(merged.breach_flow_m3s, 200.0);
        assert_eq!(merged.dam, Some(b));
        assert!(req.is_empty());
    }

    #[test]
    fn same_dam_leaks_take_max_not_sum() {
        let a = Entity::from_bits(1 << 32 | 7);
        let merged =
            merge_breach_specs(&[spec(Some(a), false, 30.0), spec(Some(a), false, 50.0)])
                .expect("merged");
        assert!(!merged.clear_crest);
        assert_eq!(merged.breach_flow_m3s, 50.0);
    }

    #[test]
    fn empty_queue_merges_to_none() {
        assert!(merge_breach_specs(&[]).is_none());
    }
}
