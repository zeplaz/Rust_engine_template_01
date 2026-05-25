//! Pending blueprint queue — defers world mutation until approval + confirm.

use bevy::prelude::*;

use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype};

use super::build_tool_authority::ZoneTool;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingEntryKind {
    BuildSite,
    ZonePaint(ZoneTool),
    Demolish,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingBuildBlueprint {
    pub kind: PendingEntryKind,
    pub label: String,
    pub archetype: SiteArchetype,
    pub origin: BuildSiteTile,
    pub footprint: FootprintTiles,
    pub layer: LayerType,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
    pub approved: bool,
    pub catalog_id: Option<String>,
}

#[derive(Resource, Default, Debug)]
pub struct PendingConstructionQueue {
    pub entries: Vec<PendingBuildBlueprint>,
}

impl PendingConstructionQueue {
    pub fn push(&mut self, entry: PendingBuildBlueprint) {
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn clear_unapproved(&mut self) {
        self.entries.retain(|entry| entry.approved);
    }

    pub fn approve_all(&mut self) {
        for entry in &mut self.entries {
            entry.approved = true;
        }
    }

    pub fn toggle_approval(&mut self, index: usize) {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.approved = !entry.approved;
        }
    }

    pub fn remove_at(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
    }

    pub fn pending_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.approved).count()
    }

    pub fn drain_approved(&mut self) -> Vec<PendingBuildBlueprint> {
        let mut out = Vec::new();
        self.entries.retain(|entry| {
            if entry.approved {
                out.push(entry.clone());
                false
            } else {
                true
            }
        });
        out
    }

    pub fn approve_matching_archetype(&mut self, archetype: SiteArchetype) {
        for entry in &mut self.entries {
            if entry.archetype == archetype {
                entry.approved = true;
            }
        }
    }

    pub fn remove_matching_archetype(&mut self, archetype: SiteArchetype) {
        self.entries.retain(|entry| entry.archetype != archetype);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site_entry(label: &str, approved: bool) -> PendingBuildBlueprint {
        PendingBuildBlueprint {
            kind: PendingEntryKind::BuildSite,
            label: label.into(),
            archetype: SiteArchetype::Factory,
            origin: BuildSiteTile { x: 0, z: 0 },
            footprint: FootprintTiles {
                width: 1,
                depth: 1,
            },
            layer: LayerType::Surface,
            rotation_quarter_turns: 0,
            mirror_x: false,
            approved,
            catalog_id: None,
        }
    }

    #[test]
    fn pending_queue_drains_only_approved() {
        let mut q = PendingConstructionQueue::default();
        q.push(site_entry("a", false));
        q.push(site_entry("b", true));
        let drained: Vec<_> = q.drain_approved();
        assert_eq!(drained.len(), 1);
        assert_eq!(q.entries.len(), 1);
        assert_eq!(q.pending_count(), 1);
    }

    #[test]
    fn pending_queue_clears_unapproved_only() {
        let mut q = PendingConstructionQueue::default();
        q.push(site_entry("keep", true));
        q.push(site_entry("drop", false));
        q.clear_unapproved();
        assert_eq!(q.entries.len(), 1);
        assert_eq!(q.entries[0].label, "keep");
    }
}
