//! Overlay heat uploads + terrain storage sync for WGSL compute dispatch.

use bevy::asset::RenderAssetUsages;
use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::construction::site_phase_tile_instances::ConstructionPhaseGpuChannel;
use crate::gui::MapViewInstances;
use crate::render::{
    extraction::VegetationExtractFrame, EcologyVisualSnapshot, LogisticsVisualSnapshot,
    SharedOverlayFieldBuffers, TileWorldFallbackState,
};
use crate::render::visual_domain_snapshots::MinimapOperationalSnapshot;
use crate::strategic::{ConstructionPhase, CorridorConstructionBook};
use crate::systems::sim_frame_delta::CommittedSimReplayRing;

const M3_UNIT_MARKER_CAP: usize = 8;

pub const MINIMAP_COMPOSITE_SHADER: &str = "shaders/minimap/minimap_composite.wgsl";

const STORAGE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

#[must_use]
pub fn minimap_storage_rgba_image(width: u32, height: u32, label: &'static str) -> Image {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some(label),
            size,
            dimension: TextureDimension::D2,
            format: STORAGE_FORMAT,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image.data = Some(vec![0; 4 * width as usize * height as usize]);
    image
}

#[derive(Resource, Debug, Default, ExtractResource, Clone)]
pub struct MinimapCompositeHeatTextures {
    pub terrain: Handle<Image>,
    pub fire: Handle<Image>,
    pub logistics: Handle<Image>,
    pub construction: Handle<Image>,
    pub ecology: Handle<Image>,
    pub fow: Handle<Image>,
    pub ew: Handle<Image>,
    pub extent: UVec2,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MinimapCompositeParamsGpu {
    pub fire_heat_enabled: u32,
    pub logistics_heat_enabled: u32,
    pub construction_heat_enabled: u32,
    pub ecology_heat_enabled: u32,
    pub overlay_revision: u64,
    pub logistics_rows: u32,
    pub construction_rows: u32,
    pub ecology_rows: u32,
    pub fow_heat_enabled: u32,
    pub ew_heat_enabled: u32,
    pub fow_rows: u32,
    pub ew_rows: u32,
}

#[derive(Resource, Debug, Default, ExtractResource, Clone)]
pub struct MinimapCompositeDispatch {
    /// Monotonic stamp for render-world dedup (0 = idle). Replaces sticky `pending=true` overflow.
    pub commit_stamp: u64,
    pub terrain: Handle<Image>,
    pub output: Handle<Image>,
    pub params: MinimapCompositeParamsGpu,
}

impl MinimapCompositeDispatch {
    #[inline]
    pub fn has_commit(&self) -> bool {
        self.commit_stamp > 0
    }
}

fn fire_heat_at_chunk(
    coord: IVec2,
    overlay: Option<&SharedOverlayFieldBuffers>,
    map_views: &MapViewInstances,
) -> f32 {
    if !map_views.minimap.overlays.fire_heat {
        return 0.0;
    }
    overlay
        .and_then(|o| o.chunk_fire_heat.get(&coord).copied())
        .unwrap_or(0.0)
}

#[inline]
fn construction_phase_strength(phase: ConstructionPhase, progress: f32) -> f32 {
    match phase {
        ConstructionPhase::Planned => 0.55,
        ConstructionPhase::InProgress => 0.35 + progress.clamp(0.0, 1.0) * 0.65,
        ConstructionPhase::Completed => 0.0,
    }
}

fn fill_construction_heat_from_book(
    out: &mut [u8],
    w: u32,
    h: u32,
    book: Option<&CorridorConstructionBook>,
    enabled: bool,
) -> u32 {
    if !enabled {
        return 0;
    }
    let Some(book) = book else {
        return 0;
    };
    let ww = u64::from(w.max(1));
    let hh = u64::from(h.max(1));
    let mut active = 0u32;
    for row in book.rows.values() {
        if row.phase == ConstructionPhase::Completed {
            continue;
        }
        active = active.saturating_add(1);
        let x = row.edge_id.0 % ww;
        let y = (row.edge_id.0 / ww) % hh;
        let v = (construction_phase_strength(row.phase, row.progress) * 255.0) as u8;
        let i = ((y * ww + x) * 4) as usize;
        if i < out.len() {
            out[i] = out[i].saturating_add(v);
        }
    }
    active
}

fn fill_ecology_heat_from_snapshot(
    out: &mut [u8],
    w: u32,
    h: u32,
    ecology: Option<&EcologyVisualSnapshot>,
    enabled: bool,
) -> u32 {
    if !enabled {
        return 0;
    }
    let Some(eco) = ecology else {
        return 0;
    };
    if eco.chunk_rows.is_empty() {
        return 0;
    }
    let ww = w.max(1);
    let hh = h.max(1);
    for row in &eco.chunk_rows {
        let cx = row.x as u32 % ww;
        let cy = (row.x as u32 / ww) % hh;
        let biomass = (row.y.clamp(0.0, 1.0) * 255.0) as u8;
        let risk = (row.z.clamp(0.0, 1.0) * 255.0) as u8;
        let base = ((cy * ww + cx) * 4) as usize;
        if base + 2 < out.len() {
            out[base + 1] = out[base + 1].saturating_add(biomass);
            out[base + 2] = out[base + 2].saturating_add(risk);
        }
    }
    eco.ecology_chunk_count.max(eco.chunk_rows.len() as u32)
}

/// **VEG-MINIMAP-BURN-MERGE-001** — burn `veg_burn_*` rows override ecology topology tint (Q4a).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct VegEcologyBurnMerge {
    pub veg_burn_rows: u32,
    pub burn_overrides_topology: bool,
}

#[must_use]
pub fn merge_veg_extract_burn_into_ecology_heat(
    out: &mut [u8],
    w: u32,
    h: u32,
    veg: Option<&VegetationExtractFrame>,
    enabled: bool,
    chunk_dims: (u32, u32),
) -> VegEcologyBurnMerge {
    let mut stats = VegEcologyBurnMerge::default();
    if !enabled {
        return stats;
    }
    let Some(veg) = veg else {
        return stats;
    };
    let cw = chunk_dims.0.max(1);
    let ch = chunk_dims.1.max(1);
    for row in &veg.rows {
        if !row.burn_active || !row.variant_key.starts_with("veg_burn_") {
            continue;
        }
        stats.veg_burn_rows = stats.veg_burn_rows.saturating_add(1);
        let px = (row.coord.x.rem_euclid(cw as i32) as u32).min(w.saturating_sub(1));
        let py = (row.coord.y.rem_euclid(ch as i32) as u32).min(h.saturating_sub(1));
        let base = ((py * w + px) * 4) as usize;
        if base + 2 >= out.len() {
            continue;
        }
        let had_topology = out[base + 1] > 0 || out[base + 2] > 0;
        if had_topology {
            stats.burn_overrides_topology = true;
        }
        let burn_strength = ((u16::from(row.frame_index) + 1) * 255 / 8) as u8;
        out[base] = burn_strength.max(out[base]);
        out[base + 1] = 0;
        out[base + 2] = burn_strength;
    }
    stats
}

fn fill_operational_heat_layers(
    fow_out: &mut [u8],
    ew_out: &mut [u8],
    w: u32,
    h: u32,
    operational: Option<&MinimapOperationalSnapshot>,
    fow_enabled: bool,
    ew_enabled: bool,
) -> (u32, u32) {
    let mut fow_rows = 0u32;
    let mut ew_rows = 0u32;
    if !fow_enabled && !ew_enabled {
        return (fow_rows, ew_rows);
    }
    let Some(op) = operational else {
        return (fow_rows, ew_rows);
    };
    if op.chunk_samples.is_empty() {
        return (fow_rows, ew_rows);
    }
    let ww = w.max(1);
    let hh = h.max(1);
    for &(cx, cy, fow_veil, ew) in &op.chunk_samples {
        let x = (cx as u32) % ww;
        let y = (cy as u32) % hh;
        let base = ((y * ww + x) * 4) as usize;
        if fow_enabled && fow_veil > 0.0 && base < fow_out.len() {
            let v = (fow_veil.clamp(0.0, 1.0) * 255.0) as u8;
            fow_out[base] = fow_out[base].saturating_add(v);
            fow_rows = fow_rows.saturating_add(1);
        }
        if ew_enabled && ew > 0.0 && base + 1 < ew_out.len() {
            let v = (ew.clamp(0.0, 1.0) * 255.0) as u8;
            ew_out[base + 1] = ew_out[base + 1].saturating_add(v);
            ew_rows = ew_rows.saturating_add(1);
        }
    }
    (fow_rows, ew_rows)
}

fn paint_unit_markers(
    ew_out: &mut [u8],
    w: u32,
    h: u32,
    operational: Option<&MinimapOperationalSnapshot>,
    enabled: bool,
) -> u32 {
    if !enabled {
        return 0;
    }
    let Some(op) = operational else {
        return 0;
    };
    let ww = w.max(1);
    let hh = h.max(1);
    let mut count = 0u32;
    for &(cx, cy) in op.unit_markers.iter().take(M3_UNIT_MARKER_CAP) {
        let x = cx % ww;
        let y = cy % hh;
        let base = ((y * ww + x) * 4) as usize;
        if base + 1 < ew_out.len() {
            ew_out[base + 1] = ew_out[base + 1].saturating_add(200);
            count = count.saturating_add(1);
        }
    }
    count
}

fn paint_replay_scrub(
    fow_out: &mut [u8],
    w: u32,
    h: u32,
    replay: Option<&CommittedSimReplayRing>,
    enabled: bool,
) -> bool {
    if !enabled {
        return false;
    }
    let Some(ring) = replay else {
        return false;
    };
    if ring.stamps.len() < 2 {
        return false;
    }
    let ww = w.max(1);
    let hh = h.max(1);
    let x = (ww * 2 / 3).min(ww.saturating_sub(1));
    for y in 0..hh {
        let base = ((y * ww + x) * 4) as usize;
        if base < fow_out.len() {
            fow_out[base] = fow_out[base].saturating_add(102);
        }
    }
    true
}

fn fill_logistics_heat_from_snapshot(
    out: &mut [u8],
    w: u32,
    h: u32,
    logistics: Option<&LogisticsVisualSnapshot>,
    enabled: bool,
) {
    if !enabled {
        return;
    }
    let Some(log) = logistics else {
        return;
    };
    let ww = w.max(1);
    let hh = h.max(1);
    for &(edge_id, traffic) in &log.edge_rows {
        let x = edge_id % ww;
        let y = (edge_id / ww) % hh;
        let v = (traffic.clamp(0.0, 1.0) * 255.0) as u8;
        let i = ((y * ww + x) * 4 + 1) as usize;
        if i < out.len() {
            out[i] = out[i].saturating_add(v);
        }
    }
}

fn ensure_heat_textures(
    images: &mut Assets<Image>,
    heat: &mut MinimapCompositeHeatTextures,
    extent: UVec2,
) {
    if heat.extent != extent
        || heat.terrain == Handle::default()
        || heat.construction == Handle::default()
        || heat.ecology == Handle::default()
        || heat.fow == Handle::default()
        || heat.ew == Handle::default()
        || images.get(&heat.terrain).is_none()
    {
        heat.terrain = images.add(minimap_storage_rgba_image(extent.x, extent.y, "minimap_terrain_storage"));
        heat.fire = images.add(minimap_storage_rgba_image(extent.x, extent.y, "minimap_fire_heat"));
        heat.logistics =
            images.add(minimap_storage_rgba_image(extent.x, extent.y, "minimap_logistics_heat"));
        heat.construction =
            images.add(minimap_storage_rgba_image(extent.x, extent.y, "minimap_construction_heat"));
        heat.ecology =
            images.add(minimap_storage_rgba_image(extent.x, extent.y, "minimap_ecology_heat"));
        heat.fow = images.add(minimap_storage_rgba_image(extent.x, extent.y, "minimap_fow_veil"));
        heat.ew = images.add(minimap_storage_rgba_image(extent.x, extent.y, "minimap_ew_stress"));
        heat.extent = extent;
    }
}

fn fill_heat_layer(
    out: &mut [u8],
    w: u32,
    h: u32,
    channel: usize,
    sample: impl Fn(u32, u32) -> f32,
) {
    for y in 0..h {
        for x in 0..w {
            let v = (sample(x, y).clamp(0.0, 1.0) * 255.0) as u8;
            let i = ((y * w + x) * 4 + channel as u32) as usize;
            if i < out.len() {
                out[i] = v;
            }
        }
    }
}

/// Upload fire + logistics heat R8 into storage textures (no terrain composite on CPU).
pub fn upload_minimap_heat_textures(
    images: &mut Assets<Image>,
    heat: &mut MinimapCompositeHeatTextures,
    overlay: Option<&SharedOverlayFieldBuffers>,
    logistics: Option<&LogisticsVisualSnapshot>,
    construction_book: Option<&CorridorConstructionBook>,
    ecology: Option<&EcologyVisualSnapshot>,
    operational: Option<&MinimapOperationalSnapshot>,
    construction_channel: Option<&ConstructionPhaseGpuChannel>,
    replay: Option<&CommittedSimReplayRing>,
    veg_extract: Option<&VegetationExtractFrame>,
    map_views: &MapViewInstances,
    fallback: &TileWorldFallbackState,
    extent: UVec2,
) -> (
    bool,
    u32,
    u32,
    u32,
    u32,
    u32,
    u32,
    bool,
    VegEcologyBurnMerge,
) {
    ensure_heat_textures(images, heat, extent);
    let w = extent.x;
    let h = extent.y;
    let chunk_count = (fallback.last_w.max(1), fallback.last_h.max(1));

    let mut fire_buf = vec![0u8; (w * h * 4) as usize];
    fill_heat_layer(&mut fire_buf, w, h, 0, |x, y| {
        let cx = (x * chunk_count.0 / w.max(1)) as i32;
        let cy = (y * chunk_count.1 / h.max(1)) as i32;
        fire_heat_at_chunk(IVec2::new(cx, cy), overlay, map_views)
    });

    let mut log_buf = vec![0u8; (w * h * 4) as usize];
    fill_logistics_heat_from_snapshot(
        &mut log_buf,
        w,
        h,
        logistics,
        map_views.minimap.overlays.logistics_heat,
    );
    let logistics_rows = logistics
        .map(|l| l.active_overlay_rows)
        .unwrap_or(0);

    let mut construction_buf = vec![0u8; (w * h * 4) as usize];
    let mut construction_rows = fill_construction_heat_from_book(
        &mut construction_buf,
        w,
        h,
        construction_book,
        map_views.minimap.overlays.construction_heat,
    );
    if let Some(channel) = construction_channel {
        if channel.active {
            construction_rows = construction_rows.max(channel.instance_count);
        }
    }

    let mut ecology_buf = vec![0u8; (w * h * 4) as usize];
    let ecology_rows = fill_ecology_heat_from_snapshot(
        &mut ecology_buf,
        w,
        h,
        ecology,
        map_views.minimap.overlays.ecology_heat,
    );
    let veg_merge = merge_veg_extract_burn_into_ecology_heat(
        &mut ecology_buf,
        w,
        h,
        veg_extract,
        map_views.minimap.overlays.ecology_heat,
        chunk_count,
    );

    if let Some(img) = images.get_mut(&heat.fire) {
        img.data = Some(fire_buf);
    }
    if let Some(img) = images.get_mut(&heat.logistics) {
        img.data = Some(log_buf);
    }
    if let Some(img) = images.get_mut(&heat.construction) {
        img.data = Some(construction_buf);
    }
    if let Some(img) = images.get_mut(&heat.ecology) {
        img.data = Some(ecology_buf);
    }

    let mut fow_buf = vec![0u8; (w * h * 4) as usize];
    let mut ew_buf = vec![0u8; (w * h * 4) as usize];
    let (fow_rows, ew_rows) = fill_operational_heat_layers(
        &mut fow_buf,
        &mut ew_buf,
        w,
        h,
        operational,
        map_views.minimap.overlays.fow,
        map_views.minimap.overlays.ew,
    );
    let unit_marker_rows = paint_unit_markers(
        &mut ew_buf,
        w,
        h,
        operational,
        map_views.minimap.overlays.units,
    );
    let replay_scrub_enabled = paint_replay_scrub(
        &mut fow_buf,
        w,
        h,
        replay,
        map_views.minimap.overlays.replay_scrub,
    );
    if let Some(img) = images.get_mut(&heat.fow) {
        img.data = Some(fow_buf);
    }
    if let Some(img) = images.get_mut(&heat.ew) {
        img.data = Some(ew_buf);
    }
    (
        true,
        logistics_rows,
        construction_rows,
        ecology_rows,
        fow_rows,
        ew_rows,
        unit_marker_rows,
        replay_scrub_enabled,
        veg_merge,
    )
}

/// Copy authoritative terrain raster into storage-readable texture (raw bytes, no overlay blend).
pub fn sync_minimap_terrain_storage(
    images: &mut Assets<Image>,
    heat: &mut MinimapCompositeHeatTextures,
    terrain_src: &Handle<Image>,
    extent: UVec2,
) -> bool {
    ensure_heat_textures(images, heat, extent);
    let Some(src) = images.get(terrain_src) else {
        return false;
    };
    let Some(src_data) = src.data.clone() else {
        return false;
    };
    let sw = src.width();
    let sh = src.height();
    let dw = extent.x;
    let dh = extent.y;
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return false;
    }
    let out = if sw == dw && sh == dh {
        src_data
    } else {
        downsample_nearest_rgba(&src_data, sw, sh, dw, dh)
    };
    if out.len() != (dw * dh * 4) as usize {
        return false;
    }
    if let Some(dst) = images.get_mut(&heat.terrain) {
        dst.data = Some(out);
        true
    } else {
        false
    }
}

fn downsample_nearest_rgba(src: &[u8], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for y in 0..dh {
        for x in 0..dw {
            let sx = x * sw / dw.max(1);
            let sy = y * sh / dh.max(1);
            let si = ((sy * sw + sx) * 4) as usize;
            let di = ((y * dw + x) * 4) as usize;
            if si + 3 < src.len() && di + 3 < out.len() {
                out[di..di + 4].copy_from_slice(&src[si..si + 4]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_image_has_storage_binding_usage() {
        let img = minimap_storage_rgba_image(4, 4, "test");
        assert!(img
            .texture_descriptor
            .usage
            .contains(TextureUsages::STORAGE_BINDING));
    }

    #[test]
    fn logistics_heat_upload_maps_edge_rows() {
        let mut images = Assets::<Image>::default();
        let mut heat = MinimapCompositeHeatTextures::default();
        let mut map_views = MapViewInstances::default();
        map_views.minimap.overlays.logistics_heat = true;
        let fallback = TileWorldFallbackState {
            last_w: 64,
            last_h: 64,
            ..Default::default()
        };
        let logistics = LogisticsVisualSnapshot {
            active_overlay_rows: 2,
            edge_rows: vec![(3, 0.8), (7, 0.5)],
            ..Default::default()
        };
        let (ok, _, _, _, _, _, _, _, _) = upload_minimap_heat_textures(
            &mut images,
            &mut heat,
            None,
            Some(&logistics),
            None,
            None,
            None,
            None,
            None,
            None,
            &map_views,
            &fallback,
            UVec2::new(32, 32),
        );
        assert!(ok);
        let img = images.get(&heat.logistics).expect("logistics texture");
        let data = img.data.as_ref().expect("pixel data");
        assert!(data.iter().any(|&b| b > 0));
    }

    #[test]
    fn fire_and_logistics_toggles_are_independent() {
        let mut map_views = MapViewInstances::default();
        map_views.minimap.overlays.fire_heat = false;
        map_views.minimap.overlays.logistics_heat = true;
        let mut log_buf = vec![0u8; 32 * 32 * 4];
        fill_logistics_heat_from_snapshot(
            &mut log_buf,
            32,
            32,
            Some(&LogisticsVisualSnapshot {
                edge_rows: vec![(5, 1.0)],
                ..Default::default()
            }),
            map_views.minimap.overlays.logistics_heat,
        );
        assert!(log_buf.iter().any(|&b| b > 0));
        let fire = fire_heat_at_chunk(IVec2::ZERO, None, &map_views);
        assert_eq!(fire, 0.0);
    }

    #[test]
    fn construction_and_ecology_heat_upload_when_enabled() {
        let mut images = Assets::<Image>::default();
        let mut heat = MinimapCompositeHeatTextures::default();
        let mut map_views = MapViewInstances::default();
        map_views.minimap.overlays = crate::gui::simulation_minimap_overlay_defaults();
        map_views.minimap.overlays.ecology_heat = true;
        let fallback = TileWorldFallbackState {
            last_w: 64,
            last_h: 64,
            ..Default::default()
        };
        let mut book = CorridorConstructionBook::default();
        book.plan_edge(crate::systems::transport::TransportEdgeId(4));
        let ecology = EcologyVisualSnapshot {
            ecology_chunk_count: 2,
            chunk_rows: vec![
                bevy::math::Vec4::new(1.0, 0.6, 0.2, 0.0),
                bevy::math::Vec4::new(2.0, 0.4, 0.5, 0.0),
            ],
            ..Default::default()
        };
        let mut operational = MinimapOperationalSnapshot::default();
        use crate::strategic::{LogisticsEdge, LogisticsGraph, LogisticsNode, LogisticsNodeId};
        use crate::terrain::ChunkCellKey;
        let mut graph = LogisticsGraph::default();
        graph.nodes = vec![
            LogisticsNode {
                id: LogisticsNodeId(0),
                throughput: 1.0,
                stockpile: 0.0,
                anchor: Some(ChunkCellKey::new(IVec2::new(1, 2), 0)),
            },
            LogisticsNode {
                id: LogisticsNodeId(1),
                throughput: 1.0,
                stockpile: 0.0,
                anchor: Some(ChunkCellKey::new(IVec2::new(3, 2), 0)),
            },
        ];
        graph.edges.push(LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            transport_edge: Some(crate::systems::transport::TransportEdgeId(1)),
            capacity: 1.0,
            disruption: 0.0,
            traversal_cost: 1.0,
        });
        crate::render::fill_minimap_unit_markers_from_logistics(
            Some(&graph),
            None,
            None,
            None,
            [].into_iter(),
            &mut operational.unit_markers,
        );
        let (ok, _, construction_rows, ecology_rows, _, _, unit_rows, replay_on, veg_merge) =
            upload_minimap_heat_textures(
                &mut images,
                &mut heat,
                None,
                None,
                Some(&book),
                Some(&ecology),
                Some(&operational),
                None,
                None,
                None,
                &map_views,
                &fallback,
                UVec2::new(32, 32),
            );
        let _ = veg_merge;
        assert!(unit_rows > 0);
        let _ = replay_on;
        assert!(ok);
        assert!(construction_rows > 0);
        assert!(ecology_rows > 0);
        let construction = images.get(&heat.construction).expect("construction tex");
        assert!(construction.data.as_ref().is_some_and(|d| d.iter().any(|&b| b > 0)));
        let eco = images.get(&heat.ecology).expect("ecology tex");
        assert!(eco.data.as_ref().is_some_and(|d| d.iter().any(|&b| b > 0)));
    }

    #[test]
    fn veg_burn_merge_overrides_ecology_topology_tint() {
        use crate::dev::landscape_grammar_burn_live_proof::veg_burn_pilot_extract_frame;
        use crate::systems::ecology::LG1_PILOT_CHUNK;

        let mut images = Assets::<Image>::default();
        let mut heat = MinimapCompositeHeatTextures::default();
        let mut map_views = MapViewInstances::default();
        map_views.minimap.overlays.ecology_heat = true;
        let fallback = TileWorldFallbackState {
            last_w: 64,
            last_h: 64,
            ..Default::default()
        };
        let ecology = EcologyVisualSnapshot {
            ecology_chunk_count: 1,
            chunk_rows: vec![bevy::math::Vec4::new(
                LG1_PILOT_CHUNK.x as f32,
                0.8,
                0.3,
                0.0,
            )],
            ..Default::default()
        };
        let veg = veg_burn_pilot_extract_frame();
        let (ok, _, _, _, _, _, _, _, merge) = upload_minimap_heat_textures(
            &mut images,
            &mut heat,
            None,
            None,
            None,
            Some(&ecology),
            None,
            None,
            None,
            Some(&veg),
            &map_views,
            &fallback,
            UVec2::new(32, 32),
        );
        assert!(ok);
        assert!(merge.veg_burn_rows >= 1, "expected veg_burn rows");
        assert!(merge.burn_overrides_topology, "burn must override topology tint");
    }
}
