//! Per-surface representation caps (VM-B4) — fire extract and future overlay channels.

use std::collections::HashMap;

use bevy::prelude::*;

use super::ids::ViewSurfaceId;

/// Per-surface caps for representation / extract (one builder; policy differs by surface).
#[derive(Resource, Clone, Debug)]
pub struct PerViewRepresentationPolicy {
    pub fire_instance_cap: HashMap<ViewSurfaceId, usize>,
}

impl Default for PerViewRepresentationPolicy {
    fn default() -> Self {
        let mut fire_instance_cap = HashMap::new();
        fire_instance_cap.insert(ViewSurfaceId::WorldMain, 4096);
        fire_instance_cap.insert(ViewSurfaceId::SimulationMap, 2048);
        fire_instance_cap.insert(ViewSurfaceId::WorldPreview, 256);
        fire_instance_cap.insert(ViewSurfaceId::Minimap, 64);
        fire_instance_cap.insert(ViewSurfaceId::DiagnosticsOverlay, 0);
        fire_instance_cap.insert(ViewSurfaceId::ConstructionPreview, 128);
        Self { fire_instance_cap }
    }
}

impl PerViewRepresentationPolicy {
    #[must_use]
    pub fn fire_cap(&self, id: ViewSurfaceId) -> usize {
        self.fire_instance_cap.get(&id).copied().unwrap_or(512)
    }

    #[inline]
    pub fn fire_cap_for_view_id(&self, id: crate::gui::ViewId) -> usize {
        self.fire_cap(ViewSurfaceId::from_view_id(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimap_fire_cap_lower_than_world_main() {
        let p = PerViewRepresentationPolicy::default();
        assert!(
            p.fire_cap(ViewSurfaceId::Minimap) < p.fire_cap(ViewSurfaceId::WorldMain)
        );
    }
}
