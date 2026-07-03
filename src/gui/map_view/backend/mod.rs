//! Backend texture source selection for map-view consumers.

mod display;
mod preview;
mod resolve;
mod types;

pub use display::{
    minimap_bevy_display_texel_extent, minimap_main_display_uses_gpu_compositor,
    resolve_minimap_bevy_display_handle,
};
pub use preview::resolve_world_preview_texture_source;
pub use resolve::resolve_minimap_texture_source;
pub use types::MapTextureSource;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{MinimapPresentationSource, MinimapShellState};
    use crate::render::{minimap_gpu_compositor_env_enabled, MinimapRenderTargetRegistry, TileWorldFallbackState};
    use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
    use bevy::prelude::*;

    #[test]
    fn bevy_display_falls_back_to_main_terrain_before_compositor_stamp() {
        let shell = MinimapShellState {
            visible: true,
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let mut images = Assets::<Image>::default();
        let main = images.add(Image::default());
        let fallback = TileWorldFallbackState {
            image: main.clone(),
            ..Default::default()
        };
        let mut registry = MinimapRenderTargetRegistry::default();
        registry.committed_image = images.add(Image::default());
        assert_eq!(
            resolve_minimap_bevy_display_handle(&shell, &fallback, &registry, 0),
            main
        );
    }

    #[test]
    fn main_sim_path_gpu_when_compositor_on() {
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let fallback = TileWorldFallbackState::default();
        let mut registry = MinimapRenderTargetRegistry::default();
        let mut images = Assets::<Image>::default();
        registry.committed_image = images.add(Image::default());
        if minimap_gpu_compositor_env_enabled() {
            assert!(matches!(
                resolve_minimap_texture_source(&shell, &fallback, &registry),
                MapTextureSource::GpuRenderTarget(_)
            ));
        }
    }

    #[test]
    fn gpu_display_texel_extent_uses_rt_not_world() {
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        let mut registry = MinimapRenderTargetRegistry::default();
        registry.committed_size = UVec2::new(260, 220);
        let mut images = Assets::<Image>::default();
        registry.committed_image = images.add(Image::default());
        let params = WorldGenParams {
            width: 320,
            height: 320,
            ..Default::default()
        };
        if minimap_gpu_compositor_env_enabled() {
            let ext = minimap_bevy_display_texel_extent(&shell, &registry, 1, &params);
            assert_eq!(ext, UVec2::new(260, 220));
        }
    }
}
