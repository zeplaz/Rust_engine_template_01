//! DF-4 — dam placement validation + barrier install into general water.
//!
//! A dam is a hydraulic structure (geology / topography / hydrology). Hydro power is an
//! optional linked plant — not what makes a dam a dam. Construction owns the site;
//! water owns the crest via [`BarrierInstallRequest`].

use bevy::prelude::*;

use crate::strategic::{
    ConstructionSite, SiteArchetype, SiteConstructionPhase, SiteFootprint,
};
use crate::construction::demolish::DestructibleDamageStage;
use crate::systems::water::{
    BarrierInstallRequest, BarrierInstallSpec, SurfaceWaterField, WaterClassRuntime,
    WorldStaticWaterDepth,
};
use bevy::math::UVec2;

/// Structural / hydraulic kind — drives placement gates and barrier profile, not “power tier.”
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum DamKind {
    /// Earth/rock fill; sloping faces; length typically ≫ height.
    #[default]
    Embankment,
    /// Weight-stable concrete/masonry/earth (includes crib fill chambers).
    Gravity,
    /// Arched upstream; needs rock abutments.
    Arch,
    /// Upstream face + downstream buttresses.
    Buttress,
    /// Low channel rise for intake / diversion / small ponds.
    Weir,
    /// Temporary pumped enclosure for construction staging.
    Coffer,
}

/// Crest / span / kind attached when a dam site is committed.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct DamBody {
    pub kind: DamKind,
    pub crest_m: f32,
    pub half_span: u32,
    /// Optional linked hydro plant site — `None` = cooling/flood-control dam only.
    pub hydro_site: Option<Entity>,
    pub barrier_installed: bool,
    /// Aggregate structural integrity 1 = intact → 0 = collapsed (DF-8).
    pub integrity_01: f32,
    pub damage_stage: DestructibleDamageStage,
    /// Per-footprint-tile column integrity (same order as [`SiteFootprint::tiles`]).
    pub column_integrity: Vec<f32>,
}

impl Default for DamBody {
    fn default() -> Self {
        Self {
            kind: DamKind::Embankment,
            crest_m: 40.0,
            half_span: 6,
            hydro_site: None,
            barrier_installed: false,
            integrity_01: 1.0,
            damage_stage: DestructibleDamageStage::Intact,
            column_integrity: Vec::new(),
        }
    }
}

impl DamBody {
    pub fn ensure_columns(&mut self, n: usize) {
        if self.column_integrity.len() != n {
            self.column_integrity = vec![1.0; n];
        }
    }
}

/// Player / scenario multi-channel control on a dam site (DF-4).
/// Turbine bank is only meaningful when [`DamBody::hydro_site`] is `Some`.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct DamControl {
    /// Gated spillway bays.
    pub gate_open01: f32,
    /// Low-level / bottom outlet.
    pub outlet_open01: f32,
    /// Mid-level outlet (augment path).
    pub mid_outlet_open01: f32,
    /// Aggregate effective turbine Q — ignored unless a hydro plant is linked.
    pub turbine_setpoint_m3s: f32,
    pub turbines: [crate::systems::water::DamTurbineUnit;
        crate::systems::water::DAM_TURBINE_UNIT_COUNT],
}

impl Default for DamControl {
    fn default() -> Self {
        Self {
            gate_open01: 0.5,
            outlet_open01: 0.35,
            mid_outlet_open01: 0.0,
            turbine_setpoint_m3s: 0.0,
            turbines: [crate::systems::water::DamTurbineUnit::default();
                crate::systems::water::DAM_TURBINE_UNIT_COUNT],
        }
    }
}

pub const DAM_BANKS_ERROR: &str = "dam_must_span_banks";
pub const DAM_KIND_SITE_ERROR: &str = "dam_kind_site_unsuitable";

/// Span width in tiles (max_x − min_x).
#[must_use]
pub fn dam_span_tiles(tiles: &[IVec2]) -> u32 {
    if tiles.is_empty() {
        return 0;
    }
    let min_x = tiles.iter().map(|t| t.x).min().unwrap_or(0);
    let max_x = tiles.iter().map(|t| t.x).max().unwrap_or(0);
    (max_x - min_x).unsigned_abs().max(1)
}

/// Kind-specific site suitability after banks already span water.
/// `soft_foundation` = soft/wide valley proxy (true when span is wide and banks soft).
#[must_use]
pub fn dam_kind_site_suitable(
    kind: DamKind,
    tiles: &[IVec2],
    soft_foundation: bool,
) -> bool {
    let span = dam_span_tiles(tiles);
    match kind {
        DamKind::Arch => {
            // Narrow rock canyon — reject soft wide valleys.
            span <= 8 && !soft_foundation
        }
        DamKind::Weir => {
            // Low diversion — short/low spans OK.
            span >= 2 && span <= 14
        }
        DamKind::Coffer => span >= 2 && span <= 20,
        DamKind::Embankment | DamKind::Gravity | DamKind::Buttress => span >= 3,
    }
}

/// Soft foundation proxy: wide span (≥10 tiles) treated as soft/wide valley.
#[must_use]
pub fn dam_soft_foundation_proxy(tiles: &[IVec2]) -> bool {
    dam_span_tiles(tiles) >= 10
}

#[must_use]
pub fn default_crest_m_for_kind(kind: DamKind) -> f32 {
    match kind {
        DamKind::Weir => 8.0,
        DamKind::Coffer => 12.0,
        DamKind::Embankment => 40.0,
        DamKind::Gravity | DamKind::Buttress => 45.0,
        DamKind::Arch => 55.0,
    }
}

/// DF-8: flow-driven erosion on dam columns. Embankment erodes above critical velocity;
/// concrete/gravity only past a higher threshold. Returns mean integrity.
#[must_use]
pub fn erode_dam_columns_by_flow(
    columns: &mut [f32],
    kind: DamKind,
    flow_speed_m_s: f32,
    dt_s: f32,
) -> f32 {
    if columns.is_empty() || dt_s <= 0.0 || flow_speed_m_s <= 0.0 {
        return if columns.is_empty() {
            1.0
        } else {
            columns.iter().copied().sum::<f32>() / columns.len() as f32
        };
    }
    let (v_crit, rate) = match kind {
        DamKind::Embankment | DamKind::Coffer => (0.8_f32, 0.045_f32),
        DamKind::Weir => (1.4, 0.02),
        DamKind::Gravity | DamKind::Buttress => (2.5, 0.008),
        DamKind::Arch => (3.0, 0.005),
    };
    let excess = (flow_speed_m_s - v_crit).max(0.0);
    if excess <= 0.0 {
        return columns.iter().copied().sum::<f32>() / columns.len() as f32;
    }
    let loss = (excess * rate * dt_s).clamp(0.0, 0.2);
    for c in columns.iter_mut() {
        *c = (*c - loss).clamp(0.0, 1.0);
    }
    columns.iter().copied().sum::<f32>() / columns.len() as f32
}

/// Velocity (m/s) of water crossing each dam column, from the water field (DF-8).
///
/// Water crosses a column only when the upstream surface is above that column's crest
/// (overtopping) or the crest has been cleared/lowered by a breach. Head = upstream surface −
/// max(downstream surface, crest); orifice/weir velocity `v = C_d·√(2g·head)`, `C_d` = 0.6.
/// Columns outside the ROI or without water on both faces report 0.
#[must_use]
pub fn dam_face_velocities_m_s(
    field: &crate::systems::water::SurfaceWaterField,
    tiles: &[IVec2],
) -> Vec<f32> {
    const G: f32 = 9.81;
    const CD: f32 = 0.6;
    let at = |x: i32, z: i32| -> Option<usize> {
        field.contains_world(x, z).then(|| {
            field.idx((x - field.origin_x) as u32, (z - field.origin_z) as u32)
        })
    };
    tiles
        .iter()
        .map(|t| {
            // Valley runs +z (upstream) → −z (downstream), as the solver's inflow/outflow edges.
            let (Some(i), Some(u), Some(d)) =
                (at(t.x, t.y), at(t.x, t.y + 1), at(t.x, t.y - 1))
            else {
                return 0.0;
            };
            let crest = field
                .hydraulics
                .barrier_crest_m
                .get(i)
                .copied()
                .unwrap_or(0.0)
                .max(field.bed_m[i]);
            let eta_u = field.surface_at(u);
            let head = eta_u - field.surface_at(d).max(crest);
            if field.depth_m[u] <= 0.05 || head <= 0.0 {
                0.0
            } else {
                CD * (2.0 * G * head).sqrt()
            }
        })
        .collect()
}

/// Orifice leak intensity from mean integrity (m³/s), zero while nearly intact.
#[must_use]
pub fn leak_flow_from_integrity(integrity_01: f32) -> f32 {
    let loss = (1.0 - integrity_01.clamp(0.0, 1.0)).max(0.0);
    if loss < 0.08 {
        0.0
    } else {
        (loss * loss * 1200.0).clamp(5.0, 2500.0)
    }
}

#[must_use]
fn turbine_availability_caps(integrity_01: f32) -> [f32; crate::systems::water::DAM_TURBINE_UNIT_COUNT] {
    // Soft trip schedule: T3 first, then T2, then T1 as structure degrades.
    if integrity_01 >= 0.85 {
        [1.0, 1.0, 1.0]
    } else if integrity_01 >= 0.65 {
        [1.0, 1.0, 0.0]
    } else if integrity_01 >= 0.45 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 0.0, 0.0]
    }
}

/// When dam integrity falls, trip hydro units offline so the operator must augment
/// release via mid/low outlets (or risk overtop). Writes site [`DamControl`] only
/// (construction must not depend on scenario resources).
pub fn derate_turbines_from_dam_damage_system(mut dams: Query<(&DamBody, &mut DamControl)>) {
    for (body, mut ctrl) in &mut dams {
        if body.hydro_site.is_none() {
            continue;
        }
        let caps = turbine_availability_caps(body.integrity_01);
        for (i, t) in ctrl.turbines.iter_mut().enumerate() {
            t.available01 = t.available01.min(caps[i]);
        }
        ctrl.turbine_setpoint_m3s = ctrl.turbines.iter().map(|t| t.effective_m3s()).sum();
    }
}

/// Progress dam leaks / erosion / collapse → water breach + hydrology DamBreach.
/// Construction domain; water owns crest via [`BarrierBreachRequest`].
pub fn dam_integrity_progress_system(
    mut dams: Query<(
        Entity,
        &mut DamBody,
        &SiteFootprint,
        Option<&mut super::SiteStructuralRoles>,
    )>,
    field: Option<Res<crate::systems::water::SurfaceWaterField>>,
    clock: Res<crate::systems::sim_control::SimStepClock>,
    mut breach: ResMut<crate::systems::water::BarrierBreachRequest>,
    mut hydro_q: Option<ResMut<crate::substrate::hydrology::HydrologyEventQueue>>,
    mut hydro_stats: Option<
        ResMut<crate::substrate::hydrology::HydrologyConstructionCouplingStats>,
    >,
    mut destruction_mask: Option<ResMut<crate::render::pixel_world::DestructionMaskBook>>,
    mut collapsed: Local<std::collections::HashSet<u64>>,
) {
    let dt = clock.step_secs() * clock.due.max(1) as f32;
    if dt <= 0.0 {
        return;
    }
    // DF-8 erosion input: the real velocity of water passing each dam column (overtopping
    // sheet or an open breach), from the water field. Controlled outlet / spillway releases go
    // through the outlet works and never touch the body.
    for (entity, mut body, footprint, roles) in &mut dams {
        if body.damage_stage.is_terminal() {
            continue;
        }
        body.ensure_columns(footprint.tiles.len());
        // STRUCT-ROLE production writer: keep roles parallel to footprint / columns.
        if let Some(mut roles) = roles {
            roles.ensure_dam_span(footprint.tiles.len());
        }
        // A dam erodes only once it is a barrier (crest installed). An unbuilt site has no body
        // for the river to attack — it ran through the footprint and "eroded" the plan.
        let speeds = field
            .as_deref()
            .filter(|_| body.barrier_installed)
            .map(|f| dam_face_velocities_m_s(f, &footprint.tiles))
            .unwrap_or_default();
        if speeds.iter().any(|v| *v > 0.0) && body.column_integrity.len() == speeds.len() {
            let kind = body.kind;
            for (col, v) in body.column_integrity.iter_mut().zip(&speeds) {
                let _ = erode_dam_columns_by_flow(std::slice::from_mut(col), kind, *v, dt);
            }
            let n = body.column_integrity.len().max(1) as f32;
            body.integrity_01 = body.column_integrity.iter().sum::<f32>() / n;
            body.damage_stage = crate::construction::demolish::destructible_stage_from_integrity(
                body.integrity_01,
                crate::construction::demolish::DESTRUCTIBLE_COLLAPSE_INTEGRITY,
            );
        }

        let id = entity.to_bits();
        if body.damage_stage.is_terminal()
            || body.integrity_01
                <= crate::construction::demolish::DESTRUCTIBLE_COLLAPSE_INTEGRITY
        {
            body.damage_stage = DestructibleDamageStage::Collapsed;
            if collapsed.insert(id) {
                // DESTROY-T2: dam collapse → same-frame ground mask.
                if let Some(mask) = destruction_mask.as_mut() {
                    mask.mark_destroyed_tiles(footprint.tiles.iter().copied());
                }
                breach.push(crate::systems::water::BarrierBreachSpec {
                    dam: Some(entity),
                    clear_crest: true,
                    breach_flow_m3s: leak_flow_from_integrity(0.0).max(200.0),
                });
                if let (Some(q), Some(stats)) = (hydro_q.as_mut(), hydro_stats.as_mut()) {
                    if let Some(mid) = footprint.tiles.get(footprint.tiles.len() / 2) {
                        super::hydro_coupling::emit_dam_breach_hydro_dirty(
                            q,
                            stats,
                            id,
                            *mid,
                            UVec2::new(32, 32),
                        );
                    }
                }
            }
        } else {
            let q_leak = leak_flow_from_integrity(body.integrity_01);
            if q_leak > 0.0 {
                breach.push(crate::systems::water::BarrierBreachSpec {
                    dam: Some(entity),
                    clear_crest: false,
                    breach_flow_m3s: q_leak,
                });
            }
        }
    }
}

/// Dam must cover ≥1 water tile and both banks (non-water at span ends).
#[must_use]
pub fn dam_spans_banks(
    tiles: &[IVec2],
    is_water: impl Fn(i32, i32) -> bool,
) -> bool {
    if tiles.len() < 3 {
        return false;
    }
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut water = 0_u32;
    for t in tiles {
        min_x = min_x.min(t.x);
        max_x = max_x.max(t.x);
        if is_water(t.x, t.y) {
            water += 1;
        }
    }
    if water == 0 || max_x <= min_x {
        return false;
    }
    // Ends of the span should sit on dry banks (or at least not both wet).
    let left_wet = tiles
        .iter()
        .filter(|t| t.x == min_x)
        .any(|t| is_water(t.x, t.y));
    let right_wet = tiles
        .iter()
        .filter(|t| t.x == max_x)
        .any(|t| is_water(t.x, t.y));
    water >= 1 && !(left_wet && right_wet)
}

/// Ensure a live [`SurfaceWaterRoiRequest`] exists for player-built dams (not scenario-only).
/// Without this, barrier install waits forever because the ROI was never seeded outside DamFlood.
pub fn ensure_dam_surface_water_roi_system(
    dams: Query<(&ConstructionSite, &SiteFootprint, &DamBody)>,
    field: Option<Res<SurfaceWaterField>>,
    mut request: ResMut<crate::systems::water::SurfaceWaterRoiRequest>,
) {
    if request.pending.is_some() {
        return;
    }
    if field.as_ref().is_some_and(|f| f.wired) {
        return;
    }
    let mut origin_x = i32::MAX;
    let mut origin_z = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_z = i32::MIN;
    let mut river_x = None;
    let mut any = false;
    for (site, footprint, body) in &dams {
        if site.archetype != SiteArchetype::Dam {
            continue;
        }
        if site.phase != SiteConstructionPhase::Operational {
            continue;
        }
        if body.barrier_installed {
            continue;
        }
        for t in &footprint.tiles {
            any = true;
            origin_x = origin_x.min(t.x);
            origin_z = origin_z.min(t.y);
            max_x = max_x.max(t.x);
            max_z = max_z.max(t.y);
        }
        if river_x.is_none() {
            if let Some(mid) = footprint.tiles.get(footprint.tiles.len() / 2) {
                river_x = Some(mid.x);
            }
        }
    }
    if !any {
        return;
    }
    // Pad corridor upstream/downstream so reservoir + release sheet are visible.
    let pad_x = 24_i32;
    let pad_z = 40_i32;
    origin_x -= pad_x;
    origin_z -= pad_z;
    max_x += pad_x;
    max_z += pad_z;
    let width = (max_x - origin_x + 1).max(32) as u32;
    let height = (max_z - origin_z + 1).max(48) as u32;
    request.pending = Some(crate::systems::water::SurfaceWaterRoiSpec {
        origin_x,
        origin_z,
        width,
        height,
        inflow_m3s: 120.0,
        min_release_m3s: 12.0,
        river_center_x: river_x,
        river_half_width: 3,
        river_seed_depth_m: 3.0,
        barrier_world_z: None,
        barrier_crest_m: 0.0,
        barrier_half_span: 6,
        synthetic_valley_if_no_dem: false,
    });
    info!(
        target: "construction::dam",
        "player dam → SurfaceWaterRoiRequest {}x{} at ({},{})",
        width, height, origin_x, origin_z
    );
}

/// When a dam becomes operational (and the ROI is wired), enqueue crest install.
/// Re-checks every frame until installed so late ROI seed still attaches the barrier.
pub fn install_barrier_from_operational_dam_system(
    mut q: Query<(&ConstructionSite, &SiteFootprint, &mut DamBody)>,
    mut request: ResMut<BarrierInstallRequest>,
    field: Option<Res<SurfaceWaterField>>,
) {
    if request.pending.is_some() {
        return;
    }
    for (site, footprint, mut body) in &mut q {
        if site.archetype != SiteArchetype::Dam {
            continue;
        }
        if site.phase != SiteConstructionPhase::Operational {
            continue;
        }
        if body.barrier_installed {
            continue;
        }
        if field.as_ref().is_none_or(|f| !f.wired) {
            continue;
        }
        let Some(mid) = footprint.tiles.get(footprint.tiles.len() / 2) else {
            continue;
        };
        let half = body.half_span.max(
            footprint
                .tiles
                .iter()
                .map(|t| (t.x - mid.x).unsigned_abs())
                .max()
                .unwrap_or(1),
        );
        // DF-4.4: the barrier is exactly the dam's footprint (a single row + per-kind half span
        // missed footprint rows, so the river ran through the dam and eroded it).
        request.pending = Some(BarrierInstallSpec {
            tiles: footprint.tiles.clone(),
            world_z: mid.y,
            crest_m: body.crest_m.max(1.0),
            half_span: half.max(1),
            center_world_x: Some(mid.x),
        });
        body.barrier_installed = true;
        info!(
            target: "construction::dam",
            "Dam site operational → BarrierInstallRequest z={} crest_m={}",
            mid.y,
            body.crest_m
        );
    }
}

/// Attach [`DamBody`] + [`DamControl`] on dam commits (catalog / archetype).
pub fn attach_dam_body_on_commit_system(
    mut commands: Commands,
    registry: Option<Res<crate::construction::building_definitions::BuildingDefinitionRegistry>>,
    q: Query<
        (Entity, &ConstructionSite, Option<&crate::economy::activation::BuildingDefinitionRef>),
        (Added<ConstructionSite>, Without<DamBody>),
    >,
) {
    for (e, site, def_ref) in &q {
        if site.archetype != SiteArchetype::Dam {
            continue;
        }
        let catalog_id = def_ref.map(|d| d.catalog_id.as_str());
        let def = catalog_id.and_then(|id| registry.as_ref().and_then(|r| r.get(id)));
        let kind = def
            .and_then(|d| d.dam_kind.as_deref())
            .map(|k| dam_kind_from_catalog_id(Some(k)))
            .unwrap_or_else(|| dam_kind_from_catalog_id(catalog_id));
        let crest = def
            .and_then(|d| d.crest_m)
            .unwrap_or_else(|| default_crest_m_for_kind(kind));
        let mut body = DamBody {
            kind,
            crest_m: crest.max(1.0),
            ..DamBody::default()
        };
        body.half_span = match kind {
            DamKind::Weir => 3,
            DamKind::Arch => 3,
            DamKind::Coffer => 4,
            _ => 6,
        };
        // STRUCT-ROLE: dam span tags (ends Foundation, mid Column) — filled once footprint known.
        commands.entity(e).insert((
            body,
            DamControl::default(),
            super::SiteStructuralRoles::default(),
        ));
    }
}

/// Mirror per-dam [`DamControl`] into general [`BarrierControlIntent`] (water owns solve).
/// Deterministic: highest entity bits among installed dams wins.
pub fn mirror_dam_control_to_barrier_system(
    dams: Query<(Entity, &DamBody, &DamControl)>,
    mut barrier: ResMut<crate::systems::water::BarrierControlIntent>,
) {
    let mut best: Option<(u64, &DamControl, bool)> = None;
    for (e, body, ctrl) in &dams {
        if !body.barrier_installed && body.integrity_01 >= 0.99 {
            // Still allow control before install so scenario can pre-set gates.
        }
        let id = e.to_bits();
        let has_hydro = body.hydro_site.is_some();
        if best.as_ref().is_none_or(|(b, _, _)| id >= *b) {
            best = Some((id, ctrl, has_hydro));
        }
    }
    let Some((_, ctrl, has_hydro)) = best else {
        return;
    };
    barrier.gate_open01 = ctrl.gate_open01.clamp(0.0, 1.0);
    barrier.outlet_open01 = ctrl.outlet_open01.clamp(0.0, 1.0);
    barrier.mid_outlet_open01 = ctrl.mid_outlet_open01.clamp(0.0, 1.0);
    if has_hydro {
        barrier.turbines = ctrl.turbines;
        barrier.sync_turbine_aggregate();
    } else {
        barrier.turbines = [crate::systems::water::DamTurbineUnit::default();
            crate::systems::water::DAM_TURBINE_UNIT_COUNT];
        barrier.turbine_flow_setpoint_m3s = 0.0;
    }
    barrier.prefer_level_pool = barrier.combined_outlet_open01() <= 0.2
        && barrier.gate_open01 <= 0.2
        && barrier.effective_turbine_m3s() < 1.0;
}

/// Infer kind from catalog id / display tokens (embankment default).
#[must_use]
pub fn dam_kind_from_catalog_id(catalog_id: Option<&str>) -> DamKind {
    let Some(id) = catalog_id.map(str::to_ascii_lowercase) else {
        return DamKind::Embankment;
    };
    if id.contains("arch") {
        DamKind::Arch
    } else if id.contains("buttress") {
        DamKind::Buttress
    } else if id.contains("weir") {
        DamKind::Weir
    } else if id.contains("coffer") {
        DamKind::Coffer
    } else if id.contains("gravity") || id.contains("crib") {
        DamKind::Gravity
    } else {
        DamKind::Embankment
    }
}

/// Pure helper for placement preview (depth-first wet test).
#[must_use]
pub fn dam_spans_banks_with_runtime(
    tiles: &[IVec2],
    field: Option<&SurfaceWaterField>,
    runtime: Option<&WaterClassRuntime>,
    static_depth: Option<&WorldStaticWaterDepth>,
    catalog_fallback: impl Fn(i32, i32) -> bool,
) -> bool {
    dam_spans_banks(tiles, |x, z| {
        crate::systems::water::tile_is_wet(field, runtime, static_depth, x, z, &catalog_fallback)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5×5 ROI on a 300 m valley floor (real terrain metres), dam row at z=2.
    fn dam_field(up_depth: f32) -> crate::systems::water::SurfaceWaterField {
        let mut f = crate::systems::water::SurfaceWaterField::default();
        f.allocate(0, 0, 5, 5, vec![300.0; 25], 0.0);
        f.install_barrier_row(2, 10.0, 1, Some(2));
        for z in 3..5 {
            for x in 0..5 {
                let i = f.idx(x, z);
                f.depth_m[i] = up_depth;
            }
        }
        f
    }

    /// CLOSE-D2: crest is the dam height above the span's bed (not an absolute elevation).
    #[test]
    fn close_d2_crest_is_height_above_bed() {
        let f = dam_field(0.0);
        let i = f.idx(2, 2);
        assert!((f.hydraulics.barrier_crest_m[i] - 310.0).abs() < 1e-3);
    }

    /// CLOSE-D2: a full reservoir below the crest does not erode the dam body.
    #[test]
    fn close_d2_no_erosion_below_crest() {
        let f = dam_field(8.0);
        let v = dam_face_velocities_m_s(&f, &[IVec2::new(2, 2)]);
        assert_eq!(v, vec![0.0]);
    }

    /// CLOSE-D2: overtopping gives a real face velocity that erodes an embankment.
    #[test]
    fn close_d2_overtopping_erodes_embankment() {
        let f = dam_field(12.0); // 2 m over a 10 m crest
        let v = dam_face_velocities_m_s(&f, &[IVec2::new(2, 2)])[0];
        assert!((v - 0.6 * (2.0 * 9.81 * 2.0_f32).sqrt()).abs() < 1e-3);
        let mut cols = [1.0_f32];
        let mean = erode_dam_columns_by_flow(&mut cols, DamKind::Embankment, v, 10.0);
        assert!(mean < 1.0);
    }

    #[test]
    fn dam_spans_banks_validation_requires_water_and_dry_ends() {
        let tiles = vec![
            IVec2::new(0, 5),
            IVec2::new(1, 5),
            IVec2::new(2, 5),
            IVec2::new(3, 5),
            IVec2::new(4, 5),
        ];
        let is_water = |x: i32, _z: i32| x == 2 || x == 3;
        assert!(dam_spans_banks(&tiles, is_water));
        let all_dry = |_x: i32, _z: i32| false;
        assert!(!dam_spans_banks(&tiles, all_dry));
        let all_wet = |_x: i32, _z: i32| true;
        assert!(!dam_spans_banks(&tiles, all_wet));
    }

    #[test]
    fn dam_kind_site_suitability_arch_rejects_wide_soft() {
        let narrow: Vec<IVec2> = (0..=5).map(|x| IVec2::new(x, 0)).collect();
        assert!(dam_kind_site_suitable(DamKind::Arch, &narrow, false));
        let wide: Vec<IVec2> = (0..=12).map(|x| IVec2::new(x, 0)).collect();
        assert!(!dam_kind_site_suitable(DamKind::Arch, &wide, true));
        assert!(dam_kind_site_suitable(DamKind::Embankment, &wide, true));
        let weir: Vec<IVec2> = (0..=4).map(|x| IVec2::new(x, 0)).collect();
        assert!(dam_kind_site_suitable(DamKind::Weir, &weir, false));
    }

    #[test]
    fn dam_barrier_profile_spillway_orifice_within_2pct() {
        use crate::systems::water::BarrierHydraulics;
        let h = BarrierHydraulics {
            barrier_crest_m: vec![40.0; 8],
            spillway_length_m: 40.0,
            outlet_area_m2: 8.0,
            mid_outlet_area_m2: 4.0,
            turbine_design_m3s: 0.0,
            min_release_m3s: 0.0,
            spillway_c: 1.7,
            orifice_cd: 0.6,
        };
        let head = 2.0_f32;
        let gate = 0.5_f32;
        let outlet = 0.35_f32;
        let q_spill = h.spillway_flow_m3s(head, gate);
        let q_orifice = h.outlet_flow_m3s(head, outlet);
        let expect_spill = 1.7 * 40.0 * gate * head.powf(1.5);
        let expect_orifice = 0.6 * 8.0 * outlet * (2.0 * crate::systems::water::G_MPS2 * head).sqrt();
        assert!(
            (q_spill - expect_spill).abs() / expect_spill <= 0.02,
            "spillway ±2%: got {q_spill} expect {expect_spill}"
        );
        assert!(
            (q_orifice - expect_orifice).abs() / expect_orifice <= 0.02,
            "orifice ±2%: got {q_orifice} expect {expect_orifice}"
        );
        // Operational dam crest install feeds the same hydraulics profile.
        let crest_m = 40.0_f32;
        assert!((h.barrier_crest_m.iter().copied().sum::<f32>() / 8.0 - crest_m).abs() < 1e-3);
    }

    #[test]
    fn leak_grows_by_erosion() {
        let mut cols = vec![1.0_f32; 4];
        let mean0 = erode_dam_columns_by_flow(&mut cols, DamKind::Embankment, 0.2, 10.0);
        assert!((mean0 - 1.0).abs() < 1e-4, "below critical velocity: no erosion");
        let mean1 = erode_dam_columns_by_flow(&mut cols, DamKind::Embankment, 2.5, 30.0);
        assert!(mean1 < mean0, "embankment erodes above critical velocity");
        let q0 = leak_flow_from_integrity(mean0);
        let q1 = leak_flow_from_integrity(mean1);
        assert!(q1 > q0, "leak orifice grows as integrity falls");
        let mut concrete = vec![1.0_f32; 4];
        let c_mean = erode_dam_columns_by_flow(&mut concrete, DamKind::Gravity, 2.5, 30.0);
        assert!(
            c_mean > mean1,
            "gravity concrete resists more than embankment at same flow"
        );
    }
}
