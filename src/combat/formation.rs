//! **Formations** (PLAN-FRONT-COMBAT-v1 FC-5): military units that occupy a tile and project presence
//! through the existing `Zone` component (control; artillery also fire), so the control relaxation and
//! the zone pass remain the only writers of the overlay. No second control grid, no third threat writer.
//!
//! Systems here (in order, before the relaxation): clock -> supply -> (resolve, see `resolve.rs`) ->
//! advance -> zone sync. The deployment executor in `orders.rs` is the sole writer of posture/target
//! from orders; `resolve.rs` is the sole writer of strength/morale and break/rally postures.

use std::collections::HashMap;

use bevy::prelude::*;

use super::balance::{FrontBalance, UnitKind};
use super::supply::FrontSupplyCoverage;
use super::theater::TheaterBook;
use crate::strategic::{ChunkStrategicOverlay, Zone, ZoneKind, MAX_STRATEGIC_FACTION_SLOTS};
use crate::systems::sim_control::SimTimeMicros;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Posture {
    #[default]
    Hold,
    /// Relocating; fires at reduced weight (defensive multiplier).
    Move,
    /// Pressing an enemy position; no fortification cover.
    Attack,
    /// Broken: falls back and stays out of orders until it rallies.
    Retreat,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Formation {
    pub faction_slot: u8,
    pub kind: UnitKind,
    /// Fraction of table-of-equipment strength left (0..1).
    pub strength: f32,
    pub morale: f32,
    pub supply: f32,
    pub posture: Posture,
    pub tile: IVec2,
    pub target: Option<IVec2>,
    /// In contact this combat step (set by the resolver).
    pub engaged: bool,
    /// Fractional progress toward the next tile.
    pub move_progress: f32,
}

impl Formation {
    #[must_use]
    pub fn new(faction_slot: u8, kind: UnitKind, tile: IVec2) -> Self {
        Self {
            faction_slot,
            kind,
            strength: 1.0,
            morale: 1.0,
            supply: 1.0,
            posture: Posture::Hold,
            tile,
            target: None,
            engaged: false,
            move_progress: 0.0,
        }
    }
}

/// Artillery's `Zone{Fire}` child.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormationFireZone {
    pub owner: Entity,
}

/// Spawn a formation with its control zone (and a fire zone for artillery).
pub fn spawn_formation(commands: &mut Commands, formation: Formation, balance: &FrontBalance) -> Entity {
    let zone = control_zone(&formation, balance);
    let fire = formation.kind == UnitKind::Artillery;
    let tile = formation.tile;
    let slot = formation.faction_slot;
    let e = commands.spawn((formation, zone)).id();
    if fire {
        commands.spawn(FormationFireZone { owner: e }).insert(Zone {
            kind: ZoneKind::Fire,
            faction_slot: slot,
            center_tile: tile,
            radius_tiles: 0.0,
            intensity: 0.0,
        });
    }
    e
}

fn control_zone(f: &Formation, b: &FrontBalance) -> Zone {
    let r0 = b.metres_to_tiles(b.unit(f.kind).control_radius_m);
    Zone {
        kind: ZoneKind::Control,
        faction_slot: f.faction_slot,
        center_tile: f.tile,
        radius_tiles: (r0 * (0.5 + 0.5 * f.strength)).max(1.0),
        intensity: (f.strength * (0.5 + 0.5 * f.morale)).clamp(0.0, 1.0),
    }
}

/// Frame delta in sim seconds, plus the combat-step accumulator.
#[derive(Resource, Clone, Debug, Default)]
pub struct CombatClock {
    last_micros: Option<u64>,
    /// Sim seconds since the previous frame (0 when paused), capped so a jump cannot teleport units.
    pub dt_s: f32,
    pub(crate) resolve_acc_s: f32,
}

const MAX_FRAME_DT_S: f32 = 5.0;

pub fn advance_combat_clock_system(time: Option<Res<SimTimeMicros>>, mut clock: ResMut<CombatClock>) {
    let Some(time) = time else {
        clock.dt_s = 0.0;
        return;
    };
    clock.dt_s = match clock.last_micros {
        Some(prev) if time.0 >= prev => ((time.0 - prev) as f32 / 1_000_000.0).min(MAX_FRAME_DT_S),
        _ => 0.0,
    };
    clock.last_micros = Some(time.0);
    clock.resolve_acc_s += clock.dt_s;
}

/// Chunk lookup so per-tile reads do not scan every overlay.
pub struct OverlayIndex<'a> {
    by_chunk: HashMap<(i32, i32), &'a ChunkStrategicOverlay>,
}

impl<'a> OverlayIndex<'a> {
    pub fn new(overlays: impl Iterator<Item = &'a ChunkStrategicOverlay>) -> Self {
        Self { by_chunk: overlays.map(|o| ((o.chunk_coord.x, o.chunk_coord.y), o)).collect() }
    }

    /// `(overlay, cell index)` of a world tile.
    #[must_use]
    pub fn cell(&self, tile: IVec2) -> Option<(&'a ChunkStrategicOverlay, usize)> {
        // Chunk size is uniform per overlay but not known up front: probe by the overlays' own size.
        let any = self.by_chunk.values().next()?;
        let (w, h) = (any.size.x.max(1) as i32, any.size.y.max(1) as i32);
        let ov = self.by_chunk.get(&(tile.x.div_euclid(w), tile.y.div_euclid(h)))?;
        let (lx, ly) = (tile.x.rem_euclid(w) as usize, tile.y.rem_euclid(h) as usize);
        Some((*ov, ly * ov.size.x as usize + lx))
    }
}

/// Supply moves toward what the front's logistics feed delivers: up at `supply_recover_per_s`, down at
/// the burn rate (faster in contact). Feed = delivered stock coverage of the front supply point for the
/// theater side the formation stands in; elsewhere the overlay `logistics_strength` at its tile.
pub fn formation_supply_system(
    clock: Res<CombatClock>,
    balance: Res<FrontBalance>,
    coverage: Res<FrontSupplyCoverage>,
    theaters: Res<TheaterBook>,
    overlays: Query<&ChunkStrategicOverlay>,
    mut formations: Query<&mut Formation>,
) {
    if clock.dt_s <= 0.0 {
        return;
    }
    let idx = OverlayIndex::new(overlays.iter());
    let c = &balance.combat;
    for mut f in &mut formations {
        let slot = f.faction_slot;
        let feed = theaters
            .theaters
            .iter()
            .filter(|t| (t.pair.0 == slot || t.pair.1 == slot) && t.aor_contains(f.tile, theaters.depth_tiles))
            .find_map(|t| coverage.by_point.get(&(t.id, slot)).copied())
            .or_else(|| idx.cell(f.tile).map(|(ov, i)| ov.logistics_strength[i][(slot as usize).min(MAX_STRATEGIC_FACTION_SLOTS - 1)]))
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let burn = balance.combat.supply_burn_per_s * (1.0 + if f.engaged { c.engaged_burn_mult } else { 0.0 });
        let s = f.supply;
        f.supply = if s < feed {
            (s + c.supply_recover_per_s * clock.dt_s).min(feed)
        } else {
            (s - burn * clock.dt_s).max(feed)
        };
    }
}

/// Move formations along the tile grid toward their target (dominant axis first), slowed by overlay
/// mobility cost. Engaged non-retreating formations hold their ground and fight.
pub fn advance_formations_system(
    clock: Res<CombatClock>,
    balance: Res<FrontBalance>,
    overlays: Query<&ChunkStrategicOverlay>,
    mut formations: Query<&mut Formation>,
) {
    if clock.dt_s <= 0.0 {
        return;
    }
    let idx = OverlayIndex::new(overlays.iter());
    for mut f in &mut formations {
        let Some(target) = f.target else { continue };
        if f.engaged && f.posture != Posture::Retreat {
            continue;
        }
        let mobility = idx.cell(f.tile).map_or(0.0, |(ov, i)| ov.mobility_cost[i]);
        let tiles_per_s = balance.unit(f.kind).speed_mps / balance.tile_size_m.max(f32::EPSILON);
        f.move_progress += tiles_per_s * clock.dt_s / (1.0 + balance.combat.mobility_slow_k * mobility);
        while f.move_progress >= 1.0 && f.tile != target {
            f.move_progress -= 1.0;
            let d = target - f.tile;
            f.tile += if d.x.abs() >= d.y.abs() { IVec2::new(d.x.signum(), 0) } else { IVec2::new(0, d.y.signum()) };
        }
        if f.tile == target {
            f.target = None;
            f.move_progress = 0.0;
            if matches!(f.posture, Posture::Move | Posture::Attack) {
                f.posture = Posture::Hold;
            }
        }
    }
}

/// Mirror formation state into its `Zone`s (the zone pass / relaxation read these).
pub fn sync_formation_zones_system(
    mut commands: Commands,
    balance: Res<FrontBalance>,
    mut controls: Query<(&Formation, &mut Zone)>,
    mut fires: Query<(Entity, &FormationFireZone, &mut Zone), Without<Formation>>,
    formations: Query<&Formation>,
) {
    for (f, mut z) in &mut controls {
        let want = control_zone(f, &balance);
        if *z != want {
            *z = want;
        }
    }
    for (fire_entity, fz, mut z) in &mut fires {
        let Ok(f) = formations.get(fz.owner) else {
            // Owner died: its fire zone goes with it.
            if let Ok(mut e) = commands.get_entity(fire_entity) {
                e.despawn();
            }
            continue;
        };
        let range_tiles = balance.metres_to_tiles(balance.unit(f.kind).range_m) * balance.combat.fire_zone_range_fraction;
        z.center_tile = f.tile;
        z.faction_slot = f.faction_slot;
        z.radius_tiles = range_tiles.max(1.0);
        // No ammunition, no fire.
        z.intensity = (f.strength * f.morale * f.supply).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        derive_theaters_system, CombatPlugin, DeploymentKind, DeploymentOrder, DeploymentOrderQueue,
    };
    use crate::strategic::{derive_frontline_from_control_system, FrontlineState, StrategicFieldPipeline};

    const W: u32 = 64;

    /// Headless world: one 64x1 overlay row, slot 0 owns x < 32, slot 1 owns x >= 32, full supply.
    fn world(supply: [f32; 2]) -> App {
        let mut app = App::new();
        app.init_resource::<FrontlineState>()
            .insert_resource(SimTimeMicros(0))
            .add_plugins(CombatPlugin)
            .add_systems(Update, derive_frontline_from_control_system.in_set(StrategicFieldPipeline::ZoneAndReadModel));
        let mut ov = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(W, 1));
        for x in 0..W as usize {
            ov.faction_control[x][usize::from(x >= 32)] = 0.8;
            ov.logistics_strength[x][0] = supply[0];
            ov.logistics_strength[x][1] = supply[1];
        }
        app.world_mut().spawn(ov);
        app
    }

    fn tick(app: &mut App, secs: u32) {
        for _ in 0..secs {
            app.world_mut().resource_mut::<SimTimeMicros>().0 += 1_000_000;
            app.update();
        }
    }

    fn spawn(app: &mut App, f: Formation) -> Entity {
        let b = app.world().resource::<FrontBalance>().clone();
        let mut cmds = app.world_mut().commands();
        let e = spawn_formation(&mut cmds, f, &b);
        app.world_mut().flush();
        e
    }

    fn owner_at(app: &mut App, x: usize) -> Option<usize> {
        let mut q = app.world_mut().query::<&ChunkStrategicOverlay>();
        let ov = q.single(app.world()).unwrap();
        let c = &ov.faction_control[x];
        (c[0].max(c[1]) >= 0.25).then(|| usize::from(c[1] > c[0]))
    }

    #[test]
    fn deployment_order_moves_a_formation_to_its_target() {
        let mut app = world([1.0, 1.0]);
        let e = spawn(&mut app, Formation::new(0, UnitKind::Mechanized, IVec2::new(5, 0)));
        let order = DeploymentOrder { formation: e, kind: DeploymentKind::Move(IVec2::new(15, 0)), issued_by: 0 };
        app.world_mut().resource_mut::<DeploymentOrderQueue>().enqueue(order).unwrap();
        tick(&mut app, 120);
        let f = app.world().entity(e).get::<Formation>().unwrap();
        assert_eq!(f.tile, IVec2::new(15, 0));
        assert_eq!(f.posture, Posture::Hold, "arrived");
        // Wrong faction cannot command it.
        let bad = DeploymentOrder { formation: e, kind: DeploymentKind::Hold, issued_by: 1 };
        app.world_mut().resource_mut::<DeploymentOrderQueue>().enqueue(bad).unwrap();
        tick(&mut app, 1);
        let out = app.world().resource::<crate::combat::DeploymentOutcomes>();
        assert_eq!(out.last[0].1, Err(crate::combat::DeploymentReject::NotOwner));
    }

    #[test]
    fn supplied_three_to_one_attack_wins_ground() {
        let mut app = world([1.0, 1.0]);
        let mut attackers = Vec::new();
        for x in [20, 21, 22] {
            attackers.push(spawn(&mut app, Formation::new(0, UnitKind::Mechanized, IVec2::new(x, 0))));
        }
        let defender = spawn(&mut app, Formation::new(1, UnitKind::Infantry, IVec2::new(44, 0)));
        assert_eq!(owner_at(&mut app, 36), Some(1));
        for a in &attackers {
            let o = DeploymentOrder { formation: *a, kind: DeploymentKind::Attack(IVec2::new(44, 0)), issued_by: 0 };
            app.world_mut().resource_mut::<DeploymentOrderQueue>().enqueue(o).unwrap();
        }
        tick(&mut app, 1500);
        let defender_gone = app.world().get_entity(defender).is_err()
            || app.world().entity(defender).get::<Formation>().is_some_and(|f| f.posture == Posture::Retreat);
        assert!(defender_gone, "defender destroyed or broken");
        assert_eq!(owner_at(&mut app, 36), Some(0), "ground changed hands");
    }
}
