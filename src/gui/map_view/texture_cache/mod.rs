//! Unified egui texture binding lifecycle for map-view consumers.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiTextureHandle};

use super::presentation::MapViewInstanceId;
use super::resolved::ResolvedMapViewFrame;

#[derive(Clone, Debug, Default)]
pub struct MapViewTextureBinding {
    handle: Option<Handle<Image>>,
    texture_id: Option<egui::TextureId>,
    revision: u64,
    pub rebinds_frame: u32,
    pub uploads_frame: u32,
    pub stale_cache_frame: u32,
    pub rebinds_total: u64,
}

impl MapViewTextureBinding {
    pub fn clear(&mut self) {
        self.handle = None;
        self.texture_id = None;
        self.revision = 0;
    }

    pub fn begin_frame(&mut self) {
        self.rebinds_frame = 0;
        self.uploads_frame = 0;
        self.stale_cache_frame = 0;
    }

    #[must_use]
    pub fn is_bound(&self) -> bool {
        self.texture_id.is_some() && self.handle.is_some()
    }

    pub fn resolve(
        &mut self,
        contexts: &mut EguiContexts,
        frame: &ResolvedMapViewFrame,
        interaction_frozen: bool,
    ) -> Option<egui::TextureId> {
        let handle = frame.texture_source.handle().clone();
        if handle == Handle::default() {
            self.clear();
            return None;
        }
        let revision = frame.texture_revision_key();
        if let (Some(cached_handle), Some(tex_id)) = (&self.handle, self.texture_id) {
            if *cached_handle == handle {
                if self.revision == revision {
                    return Some(tex_id);
                }
                if interaction_frozen {
                    return Some(tex_id);
                }
                // Same image handle, new revision (extent/overlay/present) — rebind without
                // returning None (avoids placeholder ↔ texture flash in world preview).
                let tex_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));
                self.texture_id = Some(tex_id);
                self.revision = revision;
                self.rebinds_frame = self.rebinds_frame.saturating_add(1);
                self.rebinds_total = self.rebinds_total.wrapping_add(1);
                return Some(tex_id);
            }
        }
        if interaction_frozen {
            return self.texture_id;
        }
        let tex_id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));
        self.handle = Some(handle);
        self.texture_id = Some(tex_id);
        self.revision = revision;
        self.rebinds_frame = self.rebinds_frame.saturating_add(1);
        self.uploads_frame = self.uploads_frame.saturating_add(1);
        self.rebinds_total = self.rebinds_total.wrapping_add(1);
        Some(tex_id)
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MapViewTextureCache {
    world_preview: MapViewTextureBinding,
    minimap: MapViewTextureBinding,
}

impl MapViewTextureCache {
    pub fn begin_frame(&mut self) {
        self.world_preview.begin_frame();
        self.minimap.begin_frame();
    }

    pub fn binding_mut(&mut self, id: MapViewInstanceId) -> &mut MapViewTextureBinding {
        match id {
            MapViewInstanceId::Minimap => &mut self.minimap,
            MapViewInstanceId::WorldPreview => &mut self.world_preview,
            _ => &mut self.world_preview,
        }
    }

    pub fn binding(&self, id: MapViewInstanceId) -> &MapViewTextureBinding {
        match id {
            MapViewInstanceId::Minimap => &self.minimap,
            MapViewInstanceId::WorldPreview => &self.world_preview,
            _ => &self.world_preview,
        }
    }
}

pub fn reset_map_view_texture_frame(mut cache: ResMut<MapViewTextureCache>) {
    cache.begin_frame();
}
