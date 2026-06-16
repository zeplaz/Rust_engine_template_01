//! [`SimEffectQueue`] — tick dedupe + single enqueue writer surface.

use std::collections::HashSet;

use bevy::prelude::*;

use super::event::SimEffectEvent;

#[derive(Resource, Debug, Default, Clone)]
pub struct SimEffectQueue {
    pub pending: Vec<SimEffectEvent>,
    pub pushed_total: u64,
    pub dedupe_rejected: u64,
    pub drained_total: u64,
    pub last_drain_count: u32,
    pub last_drain_us: u64,
    tick_dedupe: HashSet<(u8, i64, u64, u8)>,
}

impl SimEffectQueue {
    pub fn clear_tick_dedupe(&mut self) {
        self.tick_dedupe.clear();
    }

    /// Returns `false` when tick dedupe rejects a duplicate enqueue.
    pub fn push(&mut self, event: SimEffectEvent) -> bool {
        let Some(key) = dedupe_key(&event) else {
            self.pending.push(event);
            self.pushed_total = self.pushed_total.saturating_add(1);
            return true;
        };
        if !self.tick_dedupe.insert(key) {
            self.dedupe_rejected = self.dedupe_rejected.saturating_add(1);
            return false;
        }
        self.pending.push(event);
        self.pushed_total = self.pushed_total.saturating_add(1);
        true
    }
}

fn dedupe_key(event: &SimEffectEvent) -> Option<(u8, i64, u64, u8)> {
    let tag = event.kind.dedupe_tag();
    match &event.kind {
        super::event::SimEffectKind::IgniteCells { cells } => {
            let first = cells.first()?;
            Some((
                tag,
                i64::from(first.0.chunk.x) << 32 | i64::from(first.0.chunk.y),
                u64::from(first.0.cell_index),
                0,
            ))
        }
        super::event::SimEffectKind::LightningStrike { chunk, .. } => Some((
            tag,
            i64::from(chunk.x) << 32 | i64::from(chunk.y),
            0,
            0,
        )),
        super::event::SimEffectKind::HydroDirty(ev) => Some((
            tag,
            i64::from(ev.key.x) << 32 | i64::from(ev.key.y),
            ev.structure_id,
            ev.reason.dedupe_tag(),
        )),
        super::event::SimEffectKind::StructureHeat { chunk, .. } => Some((
            tag,
            i64::from(chunk.x) << 32 | i64::from(chunk.y),
            0,
            0,
        )),
        super::event::SimEffectKind::LandscapeDisturbance { chunk, harvest } => Some((
            tag,
            i64::from(chunk.x) << 32 | i64::from(chunk.y),
            u64::from(*harvest),
            0,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::effects::event::{SimEffectKind, SimEffectSource};
    use crate::substrate::ChunkKey;
    use crate::substrate::hydrology::HydrologyDirtyReason;

    #[test]
    fn dedupe_rejects_duplicate_lightning_strike() {
        let mut q = SimEffectQueue::default();
        let ev = SimEffectEvent {
            source: SimEffectSource::Lightning,
            cause_id: "CAUSE-lightning-1".into(),
            parent_effect_id: None,
            kind: SimEffectKind::LightningStrike {
                chunk: IVec2::new(2, 3),
                cell_indices: vec![0, 1],
                spark: 0.2,
            },
        };
        assert!(q.push(ev.clone()));
        assert!(!q.push(ev));
        assert_eq!(q.dedupe_rejected, 1);
        assert_eq!(q.pending.len(), 1);
    }

    #[test]
    fn hydro_dedupe_matches_structure_id() {
        let mut q = SimEffectQueue::default();
        let base = SimEffectEvent {
            source: SimEffectSource::Construction,
            cause_id: "CAUSE-con-9".into(),
            parent_effect_id: None,
            kind: SimEffectKind::HydroDirty(crate::substrate::hydrology::HydrologyDirtyEvent {
                key: ChunkKey::new(1, 1),
                reason: HydrologyDirtyReason::ConstructionComplete { structure_id: 9 },
                structure_id: 9,
                affected_cells: vec![0],
            }),
        };
        assert!(q.push(base.clone()));
        assert!(!q.push(base));
    }
}
