//! Power line segment HP + cut → graph island split (COD-POWER-DAMAGE-SEGMENT-001).

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::infrastructure::utility::{UtilityGraph, UtilityNetworkSnapshot};
use crate::render::{compute_island_partition, PowerLineOverlayState, PowerMapOverlayPresentation};

pub const DEFAULT_SEGMENT_MAX_HP: f32 = 100.0;

#[derive(Clone, Debug, PartialEq)]
pub struct PowerLineSegmentHealth {
    pub link_id: u64,
    pub hp: f32,
    pub max_hp: f32,
    pub destroyed: bool,
}

impl PowerLineSegmentHealth {
    #[must_use]
    pub fn new(link_id: u64) -> Self {
        Self {
            link_id,
            hp: DEFAULT_SEGMENT_MAX_HP,
            max_hp: DEFAULT_SEGMENT_MAX_HP,
            destroyed: false,
        }
    }

    #[must_use]
    pub fn overlay_state(&self) -> PowerLineOverlayState {
        if self.destroyed || self.hp <= 0.0 {
            PowerLineOverlayState::Destroyed
        } else if self.hp < self.max_hp {
            PowerLineOverlayState::Damaged
        } else {
            PowerLineOverlayState::Live
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PowerLineDamageBook {
    pub segments: HashMap<u64, PowerLineSegmentHealth>,
    pub cuts_applied: u32,
}

impl PowerLineDamageBook {
    pub fn ensure_segment(&mut self, link_id: u64) -> &mut PowerLineSegmentHealth {
        self.segments
            .entry(link_id)
            .or_insert_with(|| PowerLineSegmentHealth::new(link_id))
    }

    pub fn damage_segment(&mut self, link_id: u64, amount: f32) -> bool {
        let seg = self.ensure_segment(link_id);
        if seg.destroyed {
            return false;
        }
        seg.hp = (seg.hp - amount.max(0.0)).max(0.0);
        if seg.hp <= 0.0 {
            seg.destroyed = true;
            self.cuts_applied = self.cuts_applied.saturating_add(1);
            return true;
        }
        false
    }

    pub fn cut_segment(&mut self, link_id: u64) -> bool {
        let seg = self.ensure_segment(link_id);
        if seg.destroyed {
            return false;
        }
        seg.hp = 0.0;
        seg.destroyed = true;
        self.cuts_applied = self.cuts_applied.saturating_add(1);
        true
    }

    pub fn restore_segment(&mut self, link_id: u64) {
        let seg = self.ensure_segment(link_id);
        seg.hp = seg.max_hp;
        seg.destroyed = false;
    }

    #[must_use]
    pub fn damaged_link_ids(&self) -> HashSet<u64> {
        self.segments
            .values()
            .filter(|s| !s.destroyed && s.hp < s.max_hp)
            .map(|s| s.link_id)
            .collect()
    }

    #[must_use]
    pub fn destroyed_link_ids(&self) -> HashSet<u64> {
        self.segments
            .values()
            .filter(|s| s.destroyed)
            .map(|s| s.link_id)
            .collect()
    }
}

#[must_use]
pub fn preview_island_offline_from_cut(
    utility: &UtilityGraph,
    snap: &UtilityNetworkSnapshot,
    book: &PowerLineDamageBook,
    link_id: u64,
) -> u32 {
    let mut destroyed = book.destroyed_link_ids();
    destroyed.insert(link_id);
    let damaged = book.damaged_link_ids();
    let (_, _, offline, _) = compute_island_partition(utility, snap, &damaged, &destroyed);
    offline
}

pub fn cut_power_line_segment(book: &mut PowerLineDamageBook, link_id: u64) -> bool {
    book.cut_segment(link_id)
}

pub fn damage_power_line_segment(book: &mut PowerLineDamageBook, link_id: u64, amount: f32) -> bool {
    book.damage_segment(link_id, amount)
}

pub fn sync_power_damage_to_presentation_system(
    book: Res<PowerLineDamageBook>,
    mut presentation: ResMut<PowerMapOverlayPresentation>,
) {
    presentation.damaged_link_ids = book.damaged_link_ids();
    presentation.destroyed_link_ids = book.destroyed_link_ids();
}

pub fn register_power_segments_from_graph_system(
    utility: Option<Res<UtilityGraph>>,
    mut book: ResMut<PowerLineDamageBook>,
) {
    let Some(utility) = utility else {
        return;
    };
    for edge in &utility.power_edges {
        book.ensure_segment(edge.link_id);
    }
}

#[must_use]
pub fn power_damage_segment_witness_green() -> bool {
    let mut book = PowerLineDamageBook::default();
    book.ensure_segment(42);
    book.cut_segment(42) && book.destroyed_link_ids().contains(&42)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::utility::{
        fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot,
    };

    #[test]
    fn cut_segment_marks_destroyed_and_previews_island() {
        let snap = fixture_utility_network_snapshot();
        let graph = hydrate_utility_graph_from_snapshot(&snap);
        let mut book = PowerLineDamageBook::default();
        for edge in &graph.power_edges {
            book.ensure_segment(edge.link_id);
        }
        let link = 11_u64;
        let before = preview_island_offline_from_cut(&graph, &snap, &book, link);
        assert!(book.cut_segment(link));
        assert!(book.destroyed_link_ids().contains(&link));
        let after = preview_island_offline_from_cut(&graph, &snap, &book, link);
        assert!(after >= before);
    }

    #[test]
    fn partial_damage_not_destroyed() {
        let mut book = PowerLineDamageBook::default();
        book.damage_segment(5, 30.0);
        let seg = book.segments.get(&5).unwrap();
        assert!(!seg.destroyed);
        assert_eq!(seg.overlay_state(), PowerLineOverlayState::Damaged);
    }

    #[test]
    fn power_damage_witness_green_lib() {
        assert!(power_damage_segment_witness_green());
    }
}
