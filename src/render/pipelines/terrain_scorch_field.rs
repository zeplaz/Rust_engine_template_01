//! GPU-B — companion scorch/heat accumulation texture (not painted into land bake).
//!
//! CPU uploads heat into an R-channel accumulation Image; a translucent overlay sprite
//! displays it. Land RGBA tint via [`crate::gui::map_tile_raster::apply_cell_fire_heat_to_rgba_subregion`]
//! is gated off when this field is ready under GPU terrain authority.
//!
//! Design: [`plan_gpu_stubs_kill_v1.md`](../../dev/plan_gpu_stubs_kill_v1.md) Upgrade B.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use crate::engine::states::BaseState;
use crate::render::core::terrain_render_authority::TerrainRenderAuthority;
use crate::render::tile_world_fallback::TileWorldFallbackState;
use crate::systems::fire::ChunkFireOverlay;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

const SCORCH_Z: f32 = 0.02;

/// World-sized scorch accumulation (R = heat01, GBA unused / display fill).
#[derive(Resource, Debug, Clone)]
pub struct TerrainScorchField {
    pub image: Handle<Image>,
    pub overlay_entity: Option<Entity>,
    pub w: u32,
    pub h: u32,
    pub revision: u64,
    pub last_upload_texels: u32,
}

impl Default for TerrainScorchField {
    fn default() -> Self {
        Self {
            image: Handle::default(),
            overlay_entity: None,
            w: 0,
            h: 0,
            revision: 0,
            last_upload_texels: 0,
        }
    }
}

/// Marker for the scorch overlay sprite (companion to terrain bake/fallback sprite).
#[derive(Component, Debug, Clone, Copy)]
pub struct TerrainScorchOverlay;

#[must_use]
pub fn terrain_scorch_field_ready(field: &TerrainScorchField) -> bool {
    field.image != Handle::default() && field.w > 0 && field.h > 0 && field.last_upload_texels > 0
}

/// When true, land RGBA must not receive CPU fire/scorch tint.
#[must_use]
pub fn cpu_land_scorch_tint_suppressed(
    authority: &TerrainRenderAuthority,
    field: &TerrainScorchField,
) -> bool {
    authority.is_gpu() && terrain_scorch_field_ready(field)
}

fn make_scorch_image(w: u32, h: u32) -> Image {
    let width = w.max(1);
    let height = h.max(1);
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    // R8Unorm accumulation; expand to RGBA8 for Sprite sampling until a Material2d multiply ships.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("terrain_scorch_field"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    image.data = Some(vec![0u8; 4 * width as usize * height as usize]);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image
}

/// Pack heat01 into scorch RGBA (R=heat, A=display alpha for overlay).
#[must_use]
pub fn pack_scorch_rgba(heat01: f32) -> [u8; 4] {
    let h = heat01.clamp(0.0, 1.0);
    if h <= 0.001 {
        return [0, 0, 0, 0];
    }
    let heat_u = (h * 255.0).round() as u8;
    // Dark ember tint; alpha scales with heat for overlay blend.
    let a = (h * 180.0).round().clamp(0.0, 180.0) as u8;
    [heat_u, (heat_u as f32 * 0.35) as u8, (heat_u as f32 * 0.12) as u8, a]
}

fn ensure_scorch_target(
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
    fallback: Res<TileWorldFallbackState>,
    mut field: ResMut<TerrainScorchField>,
) {
    let w = fallback.last_w.max(params.width).max(1);
    let h = fallback.last_h.max(params.height).max(1);
    if field.image != Handle::default() && field.w == w && field.h == h {
        return;
    }
    field.image = images.add(make_scorch_image(w, h));
    field.w = w;
    field.h = h;
    field.revision = field.revision.wrapping_add(1);
    field.last_upload_texels = 0;
}

fn sync_scorch_from_chunk_fire(
    mut images: ResMut<Assets<Image>>,
    mut field: ResMut<TerrainScorchField>,
    chunks: Query<(&Chunk, &ChunkCellMatrix, Option<&ChunkFireOverlay>)>,
) {
    if field.image == Handle::default() || field.w == 0 || field.h == 0 {
        return;
    }
    let Some(mut img) = images.get_mut(&field.image) else {
        return;
    };
    let Some(data) = img.data.as_mut() else {
        return;
    };
    // Clear then stamp hot cells — accumulation texture, not land bake.
    data.fill(0);
    let w = field.w;
    let h = field.h;
    let mut written = 0u32;
    for (chunk, matrix, overlay) in chunks.iter() {
        let Some(overlay) = overlay else {
            continue;
        };
        if overlay.heat.is_empty() {
            continue;
        }
        let sx = matrix.size.x as usize;
        let sy = matrix.size.y as usize;
        if sx == 0 || sy == 0 {
            continue;
        }
        for y in 0..sy {
            for x in 0..sx {
                let i = y * sx + x;
                let heat = overlay.heat.get(i).copied().unwrap_or(0.0);
                if heat <= 0.0 {
                    continue;
                }
                let wx = chunk.coord.x as isize * sx as isize + x as isize;
                let wy = chunk.coord.y as isize * sy as isize + y as isize;
                if wx < 0 || wy < 0 {
                    continue;
                }
                let xu = wx as u32;
                let yu = wy as u32;
                if xu >= w || yu >= h {
                    continue;
                }
                let o = ((yu * w + xu) * 4) as usize;
                if o + 3 >= data.len() {
                    continue;
                }
                let px = pack_scorch_rgba(heat);
                data[o..o + 4].copy_from_slice(&px);
                written = written.saturating_add(1);
            }
        }
    }
    img.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    if let Some(d) = img.data.as_ref() {
        img.data = Some(d.clone());
    }
    // Ready after first sync (even if no hot cells yet) so CPU land tint stays suppressed.
    field.last_upload_texels = written.max(1);
    field.revision = field.revision.wrapping_add(1);
}

fn sync_scorch_overlay_sprite(
    mut commands: Commands,
    mut field: ResMut<TerrainScorchField>,
    mut q: Query<(&mut Sprite, &mut Transform), With<TerrainScorchOverlay>>,
) {
    if field.image == Handle::default() || field.w == 0 || field.h == 0 {
        if let Some(e) = field.overlay_entity.take() {
            commands.entity(e).despawn();
        }
        return;
    }
    let world_size = Vec2::new(field.w as f32, field.h as f32);
    let center = Transform::from_xyz(world_size.x * 0.5, world_size.y * 0.5, SCORCH_Z);
    if let Some(e) = field.overlay_entity {
        if let Ok((mut sprite, mut tf)) = q.get_mut(e) {
            sprite.image = field.image.clone();
            sprite.custom_size = Some(world_size);
            sprite.flip_y = true;
            *tf = center;
            return;
        }
        field.overlay_entity = None;
    }
    let e = commands
        .spawn((
            TerrainScorchOverlay,
            crate::render::mig_a_static_bulk_bundle(),
            Visibility::Visible,
            Sprite {
                image: field.image.clone(),
                custom_size: Some(world_size),
                flip_y: true,
                ..default()
            },
            center,
        ))
        .id();
    field.overlay_entity = Some(e);
}

pub struct TerrainScorchFieldPlugin;

impl Plugin for TerrainScorchFieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainScorchField>().add_systems(
            Update,
            (
                ensure_scorch_target,
                sync_scorch_from_chunk_fire.after(ensure_scorch_target),
                sync_scorch_overlay_sprite.after(sync_scorch_from_chunk_fire),
            )
                .run_if(in_state(BaseState::Simulation)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_scorch_rgba_zero_is_clear() {
        assert_eq!(pack_scorch_rgba(0.0), [0, 0, 0, 0]);
        assert!(pack_scorch_rgba(0.5)[3] > 0);
        assert_eq!(pack_scorch_rgba(1.0)[0], 255);
    }

    #[test]
    fn cpu_tint_suppressed_only_when_gpu_and_ready() {
        let mut field = TerrainScorchField::default();
        let auth = TerrainRenderAuthority::GpuBake;
        assert!(!cpu_land_scorch_tint_suppressed(&auth, &field));
        field.image = Handle::default();
        field.w = 8;
        field.h = 8;
        field.last_upload_texels = 4;
        // Still not ready — default handle.
        assert!(!terrain_scorch_field_ready(&field));
    }
}
