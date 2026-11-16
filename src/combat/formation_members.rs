//! **UM-05 / MOVE-05** — formation member entities + two-level steering.
//! **UM-06 / MOVE-06** — cost-matrix slot assign.
//! **UM-07 / MOVE-07** — bounding overwatch pairs (member roles only; Formation posture via orders).
//!
//! Dual-cite: MOVE-05 aig3 3.7.2/3.7.4 · MOVE-07 aig3 3.7.10 / 6.4.4 · LEGACY coordinated movement.
//! Anchor tile/posture stay on [`Formation`] via `advance_formations_system` / orders.
//! Members only write their own slot offsets / world tiles / overwatch role.

use bevy::prelude::*;

use super::balance::{FrontBalance, UnitKind};
use super::formation::{CombatClock, Formation};

/// Sim seconds between overwatch role flips while a formation is relocating (UM-07).
pub const OVERWATCH_FLIP_INTERVAL_S: f32 = 2.0;

/// Accumulated phase on the formation anchor; sole writer is [`bounding_overwatch_tick_system`].
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct FormationOverwatchClock {
    pub phase_s: f32,
}

/// Bounding overwatch role for a member (UM-07). Does **not** write [`Formation::posture`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OverwatchRole {
    /// Holding / covering while the pair mate advances.
    #[default]
    Cover,
    /// Advancing under cover of the pair mate.
    Move,
}

/// Child of a formation anchor — occupies a slot relative to [`Formation::tile`].
#[derive(Component, Clone, Debug, PartialEq)]
pub struct FormationMember {
    pub formation: Entity,
    pub slot_id: u8,
    /// Ideal offset from formation tile (tiles).
    pub slot_offset: IVec2,
    /// Current world tile (anchor + offset after steer).
    pub tile: IVec2,
    /// Bounding overwatch role within the formation (UM-07).
    pub overwatch: OverwatchRole,
}

/// How many visible members to spawn for a kind (capped; not 1:1 with toe_personnel).
#[must_use]
pub fn member_count_for_kind(kind: UnitKind) -> u8 {
    match kind {
        UnitKind::Infantry => 4,
        UnitKind::Mechanized | UnitKind::Armor => 3,
        UnitKind::Artillery | UnitKind::AirDefense => 2,
        UnitKind::Recon | UnitKind::Engineer => 3,
    }
}

/// Deterministic slot offsets around the anchor (N/E/S/W then diagonals).
#[must_use]
pub fn slot_offset_for(slot_id: u8) -> IVec2 {
    const RING: [IVec2; 8] = [
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
    ];
    RING[(slot_id as usize) % RING.len()]
}

/// Chebyshev distance cost for assigning a member at `member_tile` to a slot offset from `anchor`.
#[must_use]
pub fn slot_assign_cost(member_tile: IVec2, anchor: IVec2, slot_id: u8) -> i32 {
    let want = anchor + slot_offset_for(slot_id);
    let d = (member_tile - want).abs();
    d.x.max(d.y)
}

/// Greedy deterministic assignment: members (sorted by entity bits) pick lowest-cost free slot.
/// Ties break by lower slot_id. Returns `(entity_bits, slot_id)` pairs — pure helper for UM-06.
#[must_use]
pub fn assign_slots_by_cost(
    anchor: IVec2,
    members: &[(u64, IVec2)],
    slot_count: u8,
) -> Vec<(u64, u8)> {
    let mut order = members.to_vec();
    order.sort_by_key(|(bits, _)| *bits);
    let mut free: Vec<u8> = (0..slot_count).collect();
    let mut out = Vec::with_capacity(order.len());
    for (bits, tile) in order {
        if free.is_empty() {
            break;
        }
        let mut best_i = 0usize;
        let mut best_cost = i32::MAX;
        let mut best_slot = free[0];
        for (i, &slot) in free.iter().enumerate() {
            let c = slot_assign_cost(tile, anchor, slot);
            if c < best_cost || (c == best_cost && slot < best_slot) {
                best_cost = c;
                best_i = i;
                best_slot = slot;
            }
        }
        free.swap_remove(best_i);
        out.push((bits, best_slot));
    }
    out
}

/// Reassign `slot_id` / `slot_offset` by cost. Does not write [`Formation`].
/// Hysteresis: apply a full reassignment only when total cost drops by ≥2 vs current.
pub fn assign_member_slots_system(
    clock: Res<CombatClock>,
    formations: Query<&Formation>,
    mut members: Query<(Entity, &mut FormationMember)>,
) {
    if clock.dt_s <= 0.0 {
        return;
    }
    let mut by_f: std::collections::BTreeMap<Entity, Vec<(u64, IVec2, u8, Entity)>> =
        std::collections::BTreeMap::new();
    for (e, m) in members.iter() {
        by_f.entry(m.formation)
            .or_default()
            .push((e.to_bits(), m.tile, m.slot_id, e));
    }
    for (form_e, mut rows) in by_f {
        let Ok(f) = formations.get(form_e) else {
            continue;
        };
        let slot_count = member_count_for_kind(f.kind);
        rows.sort_by_key(|(bits, _, _, _)| *bits);
        let input: Vec<(u64, IVec2)> = rows.iter().map(|(b, t, _, _)| (*b, *t)).collect();
        let assigned = assign_slots_by_cost(f.tile, &input, slot_count);
        let mut cur_total = 0i32;
        let mut new_total = 0i32;
        for ((_, tile, cur_slot, _), (_, new_slot)) in rows.iter().zip(assigned.iter()) {
            cur_total += slot_assign_cost(*tile, f.tile, *cur_slot);
            new_total += slot_assign_cost(*tile, f.tile, *new_slot);
        }
        if new_total + 1 >= cur_total {
            continue;
        }
        for ((_, _, _, e), (_, new_slot)) in rows.iter().zip(assigned.iter()) {
            if let Ok((_, mut m)) = members.get_mut(*e) {
                if m.slot_id != *new_slot {
                    m.slot_id = *new_slot;
                    m.slot_offset = slot_offset_for(*new_slot);
                }
            }
        }
    }
}

/// Spawn members for a formation. Does not write [`Formation`] fields.
pub fn spawn_formation_members(
    commands: &mut Commands,
    formation_e: Entity,
    formation: &Formation,
) {
    let n = member_count_for_kind(formation.kind);
    for slot in 0..n {
        let offset = slot_offset_for(slot);
        commands.spawn(FormationMember {
            formation: formation_e,
            slot_id: slot,
            slot_offset: offset,
            tile: formation.tile + offset,
            overwatch: if slot % 2 == 0 {
                OverwatchRole::Move
            } else {
                OverwatchRole::Cover
            },
        });
    }
}

/// Pair members by ascending `slot_id` into (mover, cover) pairs. Odd leftover stays Cover.
#[must_use]
pub fn bounding_overwatch_pairs(slot_ids: &[u8]) -> Vec<(u8, u8)> {
    let mut ids = slot_ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < ids.len() {
        out.push((ids[i], ids[i + 1]));
        i += 2;
    }
    out
}

/// Flip Move/Cover within each pair (phase advance). Pure; deterministic.
#[must_use]
pub fn flip_overwatch_roles(pairs: &[(u8, u8)], roles: &[(u8, OverwatchRole)]) -> Vec<(u8, OverwatchRole)> {
    let mut map: std::collections::BTreeMap<u8, OverwatchRole> =
        roles.iter().copied().collect();
    for &(a, b) in pairs {
        let ra = map.get(&a).copied().unwrap_or(OverwatchRole::Move);
        let rb = map.get(&b).copied().unwrap_or(OverwatchRole::Cover);
        map.insert(a, rb);
        map.insert(b, ra);
    }
    map.into_iter().collect()
}

/// Tick bounding overwatch roles when the formation is relocating (`target` set).
/// Never writes [`Formation`] posture/tile — orders/resolve own those.
pub fn bounding_overwatch_tick_system(
    clock: Res<CombatClock>,
    formations: Query<&Formation>,
    mut clocks: Query<&mut FormationOverwatchClock>,
    mut members: Query<(Entity, &mut FormationMember)>,
) {
    if clock.dt_s <= 0.0 {
        return;
    }
    let mut by_f: std::collections::BTreeMap<Entity, Vec<(u8, OverwatchRole, Entity)>> =
        std::collections::BTreeMap::new();
    for (e, m) in members.iter() {
        by_f.entry(m.formation)
            .or_default()
            .push((m.slot_id, m.overwatch, e));
    }
    for (form_e, rows) in by_f {
        let Ok(f) = formations.get(form_e) else {
            continue;
        };
        // Only phase-flip while the anchor is relocating (orders set target).
        if f.target.is_none() {
            if let Ok(mut ow_clock) = clocks.get_mut(form_e) {
                ow_clock.phase_s = 0.0;
            }
            continue;
        }
        let Ok(mut ow_clock) = clocks.get_mut(form_e) else {
            continue;
        };
        ow_clock.phase_s += clock.dt_s;
        if ow_clock.phase_s < OVERWATCH_FLIP_INTERVAL_S {
            continue;
        }
        ow_clock.phase_s = 0.0;
        let slots: Vec<u8> = rows.iter().map(|(s, _, _)| *s).collect();
        let pairs = bounding_overwatch_pairs(&slots);
        let roles: Vec<(u8, OverwatchRole)> =
            rows.iter().map(|(s, r, _)| (*s, *r)).collect();
        let flipped = flip_overwatch_roles(&pairs, &roles);
        let role_map: std::collections::BTreeMap<u8, OverwatchRole> =
            flipped.into_iter().collect();
        for (slot, _, e) in rows {
            if let Some(want) = role_map.get(&slot).copied() {
                if let Ok((_, mut m)) = members.get_mut(e) {
                    m.overwatch = want;
                }
            }
        }
    }
}

/// Two-level steer: members chase `formation.tile + slot_offset` (never mutate Formation).
/// While relocating, [`OverwatchRole::Cover`] holds position so the pair mate can advance (UM-07).
pub fn steer_members_to_slots_system(
    clock: Res<CombatClock>,
    mut members: Query<&mut FormationMember>,
    formations: Query<&Formation>,
) {
    if clock.dt_s <= 0.0 {
        return;
    }
    for mut m in &mut members {
        let Ok(f) = formations.get(m.formation) else {
            continue;
        };
        if f.target.is_some() && m.overwatch == OverwatchRole::Cover {
            continue;
        }
        let want = f.tile + m.slot_offset;
        if m.tile == want {
            continue;
        }
        let d = want - m.tile;
        // One tile step toward slot (dominant axis), same family as formation advance.
        m.tile += if d.x.abs() >= d.y.abs() {
            IVec2::new(d.x.signum(), 0)
        } else {
            IVec2::new(0, d.y.signum())
        };
    }
}

/// Despawn members whose formation entity is gone.
pub fn despawn_orphan_members_system(
    mut commands: Commands,
    members: Query<(Entity, &FormationMember)>,
    formations: Query<Entity, With<Formation>>,
) {
    for (e, m) in &members {
        if formations.get(m.formation).is_err() {
            commands.entity(e).despawn();
        }
    }
}

/// Convenience: spawn formation + members.
pub fn spawn_formation_with_members(
    commands: &mut Commands,
    formation: Formation,
    balance: &FrontBalance,
) -> Entity {
    super::formation::spawn_formation(commands, formation, balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::formation::CombatClock;

    #[test]
    fn move05() {
        let mut app = App::new();
        app.init_resource::<CombatClock>()
            .insert_resource(FrontBalance::default())
            .add_systems(
                Update,
                (steer_members_to_slots_system, despawn_orphan_members_system).chain(),
            );
        let balance = app.world().resource::<FrontBalance>().clone();
        let e = {
            let world = app.world_mut();
            let mut commands = world.commands();
            spawn_formation_with_members(
                &mut commands,
                Formation::new(0, UnitKind::Infantry, IVec2::new(10, 10)),
                &balance,
            )
        };
        app.world_mut().flush();
        let n = {
            let mut q = app.world_mut().query::<&FormationMember>();
            q.iter(app.world()).filter(|m| m.formation == e).count()
        };
        assert_eq!(n, member_count_for_kind(UnitKind::Infantry) as usize);

        // Anchor moves (formation authority); members chase slots only.
        app.world_mut().entity_mut(e).get_mut::<Formation>().unwrap().tile = IVec2::new(14, 10);
        app.world_mut().resource_mut::<CombatClock>().dt_s = 1.0;
        for _ in 0..8 {
            app.update();
        }
        let anchor = app.world().entity(e).get::<Formation>().unwrap().tile;
        assert_eq!(anchor, IVec2::new(14, 10));
        let member_tiles: Vec<(u8, IVec2, IVec2)> = {
            let mut q = app.world_mut().query::<&FormationMember>();
            q.iter(app.world())
                .filter(|m| m.formation == e)
                .map(|m| (m.slot_id, m.tile, m.slot_offset))
                .collect()
        };
        for (slot_id, tile, offset) in member_tiles {
            assert_eq!(
                tile,
                anchor + offset,
                "member slot {slot_id} should follow anchor"
            );
        }
    }

    #[test]
    fn move06() {
        let anchor = IVec2::new(10, 10);
        // Two members: one already on slot 0 offset, one on slot 1 offset — swap would be costly.
        let members = [
            (10u64, anchor + slot_offset_for(1)), // nearer slot 1
            (20u64, anchor + slot_offset_for(0)), // nearer slot 0
        ];
        let a = assign_slots_by_cost(anchor, &members, 2);
        let b = assign_slots_by_cost(anchor, &members, 2);
        assert_eq!(a, b, "deterministic");
        assert_eq!(a.len(), 2);
        // Entity 10 → slot 1, entity 20 → slot 0 (cost-optimal, not index order).
        assert_eq!(a[0], (10, 1));
        assert_eq!(a[1], (20, 0));
    }

    #[test]
    fn move07() {
        let pairs = bounding_overwatch_pairs(&[0, 1, 2, 3]);
        assert_eq!(pairs, vec![(0, 1), (2, 3)]);
        let roles = vec![
            (0, OverwatchRole::Move),
            (1, OverwatchRole::Cover),
            (2, OverwatchRole::Move),
            (3, OverwatchRole::Cover),
        ];
        let flipped = flip_overwatch_roles(&pairs, &roles);
        let a = flip_overwatch_roles(&pairs, &roles);
        assert_eq!(flipped, a, "deterministic");
        let map: std::collections::BTreeMap<_, _> = flipped.into_iter().collect();
        assert_eq!(map.get(&0), Some(&OverwatchRole::Cover));
        assert_eq!(map.get(&1), Some(&OverwatchRole::Move));
        // Flip again restores.
        let roles2: Vec<_> = map.iter().map(|(k, v)| (*k, *v)).collect();
        let back = flip_overwatch_roles(&pairs, &roles2);
        let map2: std::collections::BTreeMap<_, _> = back.into_iter().collect();
        assert_eq!(map2.get(&0), Some(&OverwatchRole::Move));
        assert_eq!(map2.get(&1), Some(&OverwatchRole::Cover));
    }
}
