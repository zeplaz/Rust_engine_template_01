//! RTT-SPRITE-TRACE — temporary render-world probe for the tactical terrain sprite void.
//!
//! Active only with `RTT_SPRITE_TRACE=1`. Answers, from inside the `RenderApp` (invisible to
//! main-world witnesses), every 60th render frame:
//! 1. does `RenderAssets<GpuImage>` hold the tile-fallback terrain image (GPU residency)?
//! 2. was the terrain sprite extracted this frame (`ExtractedSprites` entry with that asset id)?
//! 3. how many `Transparent2d` phase items exist per view (is anything queued at all)?

use bevy::asset::AssetId;
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::ViewSortedRenderPhases;
use bevy::render::texture::GpuImage;
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSystems};
use bevy::sprite_render::ExtractedSprites;

#[must_use]
pub fn rtt_sprite_trace_enabled() -> bool {
    std::env::var("RTT_SPRITE_TRACE").is_ok()
}

/// Render-world copy of the main-world terrain image id (+ render-frame counter).
#[derive(Resource, Default)]
struct RttSpriteTraceTarget {
    image: Option<AssetId<Image>>,
    sprite_entity: Option<Entity>,
    main_world_camera: Option<Entity>,
    frame: u64,
}

fn extract_rtt_sprite_trace_target(
    state: Extract<Option<Res<crate::render::tile_world_fallback::TileWorldFallbackState>>>,
    main_cam: Extract<
        Query<Entity, With<crate::gui::tactical::map_camera::MainWorldCamera>>,
    >,
    mut target: ResMut<RttSpriteTraceTarget>,
) {
    target.image = state
        .as_ref()
        .filter(|s| s.image != Handle::default())
        .map(|s| s.image.id());
    target.sprite_entity = state.as_ref().and_then(|s| s.sprite_entity);
    target.main_world_camera = main_cam.iter().next();
    target.frame = target.frame.wrapping_add(1);
}

fn log_rtt_sprite_trace(
    target: Res<RttSpriteTraceTarget>,
    extracted: Option<Res<ExtractedSprites>>,
    gpu_images: Option<Res<RenderAssets<GpuImage>>>,
    phases: Option<Res<ViewSortedRenderPhases<Transparent2d>>>,
) {
    if target.frame % 60 != 0 {
        return;
    }
    let Some(id) = target.image else {
        info!("RTT_SPRITE_TRACE frame={} terrain_image=absent", target.frame);
        return;
    };
    let gpu_image_present = gpu_images.as_ref().is_some_and(|g| g.get(id).is_some());
    let (extracted_total, extracted_matching) = extracted
        .as_ref()
        .map(|e| {
            (
                e.sprites.len(),
                e.sprites.iter().filter(|s| s.image_handle_id == id).count(),
            )
        })
        .unwrap_or((0, 0));
    let transparent2d: Vec<String> = phases
        .as_ref()
        .map(|p| {
            p.0.iter()
                .map(|(view, phase)| {
                    let is_main_cam = target
                        .main_world_camera
                        .is_some_and(|cam| view.main_entity.id() == cam);
                    let has_terrain = target.sprite_entity.is_some_and(|sprite| {
                        phase
                            .items
                            .keys()
                            .any(|(_, main_ent)| main_ent.id() == sprite)
                    });
                    format!(
                        "view={:?} main_world_cam={} items={} has_terrain_sprite={}",
                        view.main_entity.id(),
                        is_main_cam,
                        phase.items.len(),
                        has_terrain
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    info!(
        "RTT_SPRITE_TRACE frame={} gpu_image_present={} extracted_sprites_total={} extracted_matching_terrain={} views={:?}",
        target.frame, gpu_image_present, extracted_total, extracted_matching, transparent2d
    );
}

pub struct RttSpriteTracePlugin;

impl Plugin for RttSpriteTracePlugin {
    fn build(&self, app: &mut App) {
        if !rtt_sprite_trace_enabled() {
            return;
        }
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            warn!("RTT_SPRITE_TRACE set but RenderApp absent — probe inactive");
            return;
        };
        render_app
            .init_resource::<RttSpriteTraceTarget>()
            .add_systems(ExtractSchedule, extract_rtt_sprite_trace_target)
            .add_systems(Render, log_rtt_sprite_trace.after(RenderSystems::Queue));
        info!("RTT_SPRITE_TRACE probe active (logs every 60th render frame)");
    }
}
