//! WorldGen session chrome — one Bevy UI shell.
//!
//! Replaces the egui workspace (`display_world_preview`, `world_gen_ui_system`) while
//! `AppState::WorldGen` is active. Layer bits, `WorldGenParams` fields, and the preview
//! image handle are the same values the egui panels wrote. Raster, hydrology, and
//! tile sampling stay in their existing systems.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{MouseButton, MouseWheel};
use bevy::prelude::*;
use bevy::ui::widget::ImageNode;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::engine::{
    ux_enter_world_from_world_gen, AppState, BaseState, PauseState, WorldGenChromeLatch,
    WorldGenFlowState, WorldGenState,
};
use crate::gui::editor::world_gen_ui::{
    CancelActiveWorldGenEvent, WorldGenUiState,
};
use crate::gui::map_presentation_fit::{
    compute_map_fit_strict, fit_viewport_to_map, MAP_PANEL_INSET_PX,
};
use crate::gui::resolve_world_preview_texture_source;
use crate::gui::map_view_projection::{
    ensure_viewport_camera_initialized, map_presentation_image_rect,
};
use crate::gui::style::UiPalette;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::tuning_io::{
    load_overlay_prefer_ron, save_overlay_ron, WorldGenTuningOverlay,
};
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, GenerateWorldEvent, WorldGenJobSlot, WorldGenParams,
    WorldGenPhase, WorldGenProgress, WorldMarker, MAX_WORLD_GEN_AXIS, WORLD_GEN_TUNING_JSON_PATH,
    WORLD_GEN_TUNING_RON_PATH,
};
use crate::terrain::generation::world_generator_enhanced::{RegionMethod, TerrainNoiseProfile};
use crate::terrain::material::TagId;
use crate::terrain::world_map_scale::TileExtentPreset;

use super::interaction::{apply_pan, apply_zoom_toward};
use super::layers::PreviewLayers;
use super::preview_render_contract::{PreviewCameraState, PreviewPathAuthority, PreviewRenderMode};
use super::texture_cache::WorldPreviewTexture;
use super::viewport_suggestion::{
    clear_world_preview_viewport_requests, write_world_preview_viewport_request,
};
use super::{d07_inset_side_px, WorldPreviewLifecycle, WorldPreviewRenderTargetRegistry, WorldPreviewUiState};

/// Last tuning I/O line (reload / save / reset). Session chrome only.
#[derive(Resource, Default)]
pub struct WorldGenTuningNote(pub String);

#[derive(Component)]
pub struct WorldGenShellRoot;

#[derive(Component)]
struct WorldGenSheetRoot;

#[derive(Component)]
struct WorldGenPreviewPanel;

#[derive(Component)]
struct WorldGenPreviewImage;

#[derive(Component)]
struct WorldGenCornerInset;

#[derive(Component)]
struct WorldGenMapDim;

#[derive(Component)]
struct WorldGenPlaceholderText;

#[derive(Component)]
struct WorldGenStatusText;

#[derive(Component)]
struct WorldGenFlowText;

#[derive(Component)]
struct WorldGenSeedText;

#[derive(Component)]
struct WorldGenHintText;

#[derive(Component)]
struct WorldGenDebugText;

#[derive(Component)]
struct WorldGenTagList;

#[derive(Component)]
struct WorldGenMobilityText;

#[derive(Component)]
struct WorldGenConfirmRoot;

#[derive(Component, Clone, Copy)]
struct WorldGenLayerButton {
    kind: LayerKind,
}

#[derive(Clone, Copy)]
enum LayerKind {
    Base(PreviewLayers),
    Overlay(PreviewLayers),
    Fire,
    Logistics,
    Construction,
    Ecology,
}

#[derive(Component, Clone, Copy)]
struct WorldGenKnobStep {
    knob: WgKnob,
    dir: i8,
}

#[derive(Component)]
struct WorldGenKnobReadout(WgKnob);

#[derive(Component, Clone, Copy)]
enum WorldGenAction {
    Parameters,
    Follow,
    RegionMethod,
    HeightProfile,
    Island,
    Gpu,
    ZoomOut,
    ZoomIn,
    ZoomOne,
    ZoomFit,
    RandomSeed,
    MediumSmall,
    Standard,
    SyncLand,
    ReloadTuning,
    SaveTuning,
    ResetDefaults,
    GeneratePreview,
    GenerateFull,
    EnterWorld,
    EditMap,
    DiscardWorld,
    DiscardPreview,
    ClosePanel,
    KeepOpen,
    CloseAndCancel,
    Mobility,
}

#[derive(Component)]
struct WorldGenTagToggle(u16);

#[derive(Clone, Copy)]
pub(crate) enum WgKnob {
    Width,
    Height,
    NumRegions,
    RegionIterations,
    StrategicCoupling,
    MoistureBias,
    TemperatureBias,
    MoistureScale,
    TemperatureScale,
    MoistureFreq,
    TemperatureFreq,
    NoiseScale,
    NoiseOctaves,
    Lacunarity,
    Persistence,
    HeightCurve,
    DomainWarp,
    DetailMix,
    WarpScale,
    WarpOctaves,
    WarpSeed,
    DetailScale,
    DetailOctaves,
    DetailSeed,
    WarpFreq,
    WarpZ,
    WarpPhaseX,
    WarpPhaseY,
    WarpDisplacement,
    DetailFreq,
    DetailPersistence,
    Rivers,
    Lakes,
    MountainThreshold,
    IslandFalloff,
    SeaLevel,
    DeepWater,
    ShallowWater,
    Beach,
    MountainHeight,
    GrasslandMoist,
    DesertMoist,
    SwampMoist,
    ForestMoist,
    HotLowlands,
    Tundra,
    SnowPeak,
    /// How many towns a full generation may place. Zero leaves the town book empty.
    Settlements,
}

const KNOBS: &[WgKnob] = &[
    WgKnob::Width,
    WgKnob::Height,
    WgKnob::NumRegions,
    WgKnob::RegionIterations,
    WgKnob::StrategicCoupling,
    WgKnob::MoistureBias,
    WgKnob::TemperatureBias,
    WgKnob::MoistureScale,
    WgKnob::TemperatureScale,
    WgKnob::MoistureFreq,
    WgKnob::TemperatureFreq,
    WgKnob::NoiseScale,
    WgKnob::NoiseOctaves,
    WgKnob::Lacunarity,
    WgKnob::Persistence,
    WgKnob::HeightCurve,
    WgKnob::DomainWarp,
    WgKnob::DetailMix,
    WgKnob::WarpScale,
    WgKnob::WarpOctaves,
    WgKnob::WarpSeed,
    WgKnob::DetailScale,
    WgKnob::DetailOctaves,
    WgKnob::DetailSeed,
    WgKnob::WarpFreq,
    WgKnob::WarpZ,
    WgKnob::WarpPhaseX,
    WgKnob::WarpPhaseY,
    WgKnob::WarpDisplacement,
    WgKnob::DetailFreq,
    WgKnob::DetailPersistence,
    WgKnob::Rivers,
    WgKnob::Lakes,
    WgKnob::MountainThreshold,
    WgKnob::IslandFalloff,
    WgKnob::SeaLevel,
    WgKnob::DeepWater,
    WgKnob::ShallowWater,
    WgKnob::Beach,
    WgKnob::MountainHeight,
    WgKnob::GrasslandMoist,
    WgKnob::DesertMoist,
    WgKnob::SwampMoist,
    WgKnob::ForestMoist,
    WgKnob::HotLowlands,
    WgKnob::Tundra,
    WgKnob::SnowPeak,
    WgKnob::Settlements,
];

fn knob_label(knob: WgKnob) -> &'static str {
    match knob {
        WgKnob::Width => "Width",
        WgKnob::Height => "Height",
        WgKnob::NumRegions => "Number of Regions",
        WgKnob::RegionIterations => "Relaxation Iterations",
        WgKnob::StrategicCoupling => "Strategic field blend",
        WgKnob::MoistureBias => "Moisture bias",
        WgKnob::TemperatureBias => "Temperature bias",
        WgKnob::MoistureScale => "Moisture · patch scale ×",
        WgKnob::TemperatureScale => "Temperature · patch scale ×",
        WgKnob::MoistureFreq => "Moisture · fine detail freq ×",
        WgKnob::TemperatureFreq => "Temperature · fine detail freq ×",
        WgKnob::NoiseScale => "Noise Scale",
        WgKnob::NoiseOctaves => "Noise Octaves",
        WgKnob::Lacunarity => "Lacunarity",
        WgKnob::Persistence => "Persistence",
        WgKnob::HeightCurve => "Height curve",
        WgKnob::DomainWarp => "Domain warp strength",
        WgKnob::DetailMix => "High-frequency detail mix",
        WgKnob::WarpScale => "Warp noise · scale ×",
        WgKnob::WarpOctaves => "Warp octaves",
        WgKnob::WarpSeed => "Warp seed offset",
        WgKnob::DetailScale => "Detail noise · scale ×",
        WgKnob::DetailOctaves => "Detail octaves",
        WgKnob::DetailSeed => "Detail seed offset",
        WgKnob::WarpFreq => "Warp coord frequency",
        WgKnob::WarpZ => "Warp coord Z",
        WgKnob::WarpPhaseX => "Warp phase X",
        WgKnob::WarpPhaseY => "Warp phase Y",
        WgKnob::WarpDisplacement => "Warp displacement",
        WgKnob::DetailFreq => "Detail coord frequency",
        WgKnob::DetailPersistence => "Detail persistence ×",
        WgKnob::Rivers => "Rivers",
        WgKnob::Lakes => "Lakes",
        WgKnob::MountainThreshold => "Mountain Threshold",
        WgKnob::IslandFalloff => "Island Falloff",
        WgKnob::SeaLevel => "Sea level",
        WgKnob::DeepWater => "Deep water height max",
        WgKnob::ShallowWater => "Shallow water height max",
        WgKnob::Beach => "Beach height max",
        WgKnob::MountainHeight => "Mountain height min",
        WgKnob::GrasslandMoist => "Grassland moisture max",
        WgKnob::DesertMoist => "Desert moisture max",
        WgKnob::SwampMoist => "Swamp moisture min",
        WgKnob::ForestMoist => "Forest moisture band max",
        WgKnob::HotLowlands => "Hot lowlands temp min",
        WgKnob::Tundra => "Tundra temp max",
        WgKnob::SnowPeak => "Snow peak temp max",
        WgKnob::Settlements => "Settlements (0 leaves the town book empty)",
    }
}

/// Inclusive ranges copied from the retired egui sliders.
fn knob_bounds(knob: WgKnob) -> Option<(f64, f64)> {
    Some(match knob {
        WgKnob::Width | WgKnob::Height => (128.0, MAX_WORLD_GEN_AXIS as f64),
        WgKnob::NumRegions => (4.0, 64.0),
        WgKnob::RegionIterations => (1.0, 10.0),
        WgKnob::StrategicCoupling => (0.0, 0.35),
        WgKnob::MoistureBias | WgKnob::TemperatureBias => (-0.5, 0.5),
        WgKnob::MoistureScale | WgKnob::TemperatureScale => (0.3, 3.0),
        WgKnob::MoistureFreq | WgKnob::TemperatureFreq => (0.1, 4.0),
        WgKnob::NoiseScale => (0.01, 0.1),
        WgKnob::NoiseOctaves => (1.0, 8.0),
        WgKnob::Lacunarity => (1.2, 3.5),
        WgKnob::Persistence => (0.2, 0.85),
        WgKnob::HeightCurve => (0.35, 2.5),
        WgKnob::DomainWarp => (0.0, 1.5),
        WgKnob::DetailMix => (0.0, 1.0),
        WgKnob::WarpScale => (0.05, 1.0),
        WgKnob::WarpOctaves => (2.0, 12.0),
        WgKnob::DetailScale => (0.5, 8.0),
        WgKnob::DetailOctaves => (1.0, 8.0),
        WgKnob::WarpFreq => (0.02, 0.25),
        WgKnob::WarpZ => (-4.0, 8.0),
        WgKnob::WarpPhaseX | WgKnob::WarpPhaseY => (-50.0, 50.0),
        WgKnob::WarpDisplacement => (1.0, 40.0),
        WgKnob::DetailFreq => (1.0, 12.0),
        WgKnob::DetailPersistence => (0.2, 1.0),
        WgKnob::Rivers => (0.0, 10.0),
        WgKnob::Lakes => (0.0, 5.0),
        WgKnob::MountainThreshold => (0.5, 0.9),
        WgKnob::IslandFalloff => (1.0, 5.0),
        WgKnob::SeaLevel => (0.15, 0.55),
        WgKnob::DeepWater => (0.05, 0.35),
        WgKnob::ShallowWater => (0.2, 0.55),
        WgKnob::Beach => (0.28, 0.48),
        WgKnob::MountainHeight => (0.55, 0.92),
        WgKnob::GrasslandMoist => (0.2, 0.6),
        WgKnob::DesertMoist => (0.1, 0.55),
        WgKnob::SwampMoist => (0.55, 0.95),
        WgKnob::ForestMoist => (0.45, 0.85),
        WgKnob::HotLowlands => (0.45, 0.9),
        WgKnob::Tundra => (0.1, 0.45),
        WgKnob::SnowPeak => (0.05, 0.35),
        WgKnob::Settlements => (0.0, 8.0),
        WgKnob::WarpSeed | WgKnob::DetailSeed => return None,
    })
}

fn knob_integer(knob: WgKnob) -> bool {
    matches!(
        knob,
        WgKnob::Width
            | WgKnob::Height
            | WgKnob::NumRegions
            | WgKnob::RegionIterations
            | WgKnob::NoiseOctaves
            | WgKnob::WarpOctaves
            | WgKnob::DetailOctaves
            | WgKnob::Rivers
            | WgKnob::Lakes
            | WgKnob::Settlements
            | WgKnob::WarpSeed
            | WgKnob::DetailSeed
    )
}

pub(crate) fn step_world_gen_knob(params: &mut WorldGenParams, knob: WgKnob, dir: i8) {
    if dir == 0 {
        return;
    }
    match knob {
        WgKnob::WarpSeed => {
            let next = params.noise_sampling.warp_seed_offset as i128 + dir as i128;
            params.noise_sampling.warp_seed_offset = next.max(0) as u64;
            return;
        }
        WgKnob::DetailSeed => {
            let next = params.noise_sampling.detail_seed_offset as i128 + dir as i128;
            params.noise_sampling.detail_seed_offset = next.max(0) as u64;
            return;
        }
        _ => {}
    }
    let Some((min, max)) = knob_bounds(knob) else {
        return;
    };
    let span = (max - min).max(1e-6);
    let step = if knob_integer(knob) {
        if matches!(knob, WgKnob::Width | WgKnob::Height) {
            32.0
        } else {
            1.0
        }
    } else {
        span / 40.0
    };
    let current = knob_get(params, knob);
    let next = (current + step * f64::from(dir)).clamp(min, max);
    knob_set(params, knob, next);
}

fn knob_get(params: &WorldGenParams, knob: WgKnob) -> f64 {
    let b = &params.biome_tuning;
    let n = &params.noise_sampling;
    match knob {
        WgKnob::Width => f64::from(params.width),
        WgKnob::Height => f64::from(params.height),
        WgKnob::NumRegions => f64::from(params.num_regions),
        WgKnob::RegionIterations => f64::from(params.region_iterations),
        WgKnob::StrategicCoupling => f64::from(params.strategic_field_coupling),
        WgKnob::MoistureBias => f64::from(params.moisture_bias),
        WgKnob::TemperatureBias => f64::from(params.temperature_bias),
        WgKnob::MoistureScale => f64::from(n.moisture_noise_scale_mul),
        WgKnob::TemperatureScale => f64::from(n.temperature_noise_scale_mul),
        WgKnob::MoistureFreq => n.moisture_sample_freq_mul,
        WgKnob::TemperatureFreq => n.temperature_sample_freq_mul,
        WgKnob::NoiseScale => f64::from(params.noise_scale),
        WgKnob::NoiseOctaves => f64::from(params.noise_octaves),
        WgKnob::Lacunarity => f64::from(params.noise_lacunarity),
        WgKnob::Persistence => f64::from(params.noise_persistence),
        WgKnob::HeightCurve => f64::from(params.height_curve_exponent),
        WgKnob::DomainWarp => f64::from(params.domain_warp_strength),
        WgKnob::DetailMix => f64::from(params.terrain_detail_mix),
        WgKnob::WarpScale => f64::from(n.warp_noise_scale_mul),
        WgKnob::WarpOctaves => f64::from(n.warp_noise_octaves),
        WgKnob::WarpSeed => n.warp_seed_offset as f64,
        WgKnob::DetailScale => f64::from(n.detail_noise_scale_mul),
        WgKnob::DetailOctaves => f64::from(n.detail_noise_octaves),
        WgKnob::DetailSeed => n.detail_seed_offset as f64,
        WgKnob::WarpFreq => n.warp_coord_frequency_mul,
        WgKnob::WarpZ => n.warp_coord_z,
        WgKnob::WarpPhaseX => n.warp_phase_offset_x,
        WgKnob::WarpPhaseY => n.warp_phase_offset_y,
        WgKnob::WarpDisplacement => n.warp_displacement_scale,
        WgKnob::DetailFreq => n.detail_coord_frequency_mul,
        WgKnob::DetailPersistence => n.detail_persistence_mul,
        WgKnob::Rivers => f64::from(params.river_count),
        WgKnob::Lakes => f64::from(params.lake_count),
        WgKnob::MountainThreshold => f64::from(params.mountain_threshold),
        WgKnob::IslandFalloff => f64::from(params.island_falloff),
        WgKnob::SeaLevel => f64::from(b.sea_level),
        WgKnob::DeepWater => f64::from(b.deep_water_height_max),
        WgKnob::ShallowWater => f64::from(b.shallow_water_height_max),
        WgKnob::Beach => f64::from(b.beach_height_max),
        WgKnob::MountainHeight => f64::from(b.mountain_height_min),
        WgKnob::GrasslandMoist => f64::from(b.grassland_moisture_max),
        WgKnob::DesertMoist => f64::from(b.desert_moisture_max),
        WgKnob::SwampMoist => f64::from(b.swamp_moisture_min),
        WgKnob::ForestMoist => f64::from(b.forest_moisture_max),
        WgKnob::HotLowlands => f64::from(b.hot_lowlands_temperature_min),
        WgKnob::Tundra => f64::from(b.tundra_temperature_max),
        WgKnob::SnowPeak => f64::from(b.snow_peak_temperature_max),
        WgKnob::Settlements => f64::from(params.settlement_count),
    }
}

fn knob_set(params: &mut WorldGenParams, knob: WgKnob, value: f64) {
    let b = &mut params.biome_tuning;
    let n = &mut params.noise_sampling;
    let whole = value.round();
    match knob {
        WgKnob::Width => params.width = whole as u32,
        WgKnob::Height => params.height = whole as u32,
        WgKnob::NumRegions => params.num_regions = whole as u32,
        WgKnob::RegionIterations => params.region_iterations = whole as u32,
        WgKnob::StrategicCoupling => params.strategic_field_coupling = value as f32,
        WgKnob::MoistureBias => params.moisture_bias = value as f32,
        WgKnob::TemperatureBias => params.temperature_bias = value as f32,
        WgKnob::MoistureScale => n.moisture_noise_scale_mul = value as f32,
        WgKnob::TemperatureScale => n.temperature_noise_scale_mul = value as f32,
        WgKnob::MoistureFreq => n.moisture_sample_freq_mul = value,
        WgKnob::TemperatureFreq => n.temperature_sample_freq_mul = value,
        WgKnob::NoiseScale => params.noise_scale = value as f32,
        WgKnob::NoiseOctaves => params.noise_octaves = whole as u32,
        WgKnob::Lacunarity => params.noise_lacunarity = value as f32,
        WgKnob::Persistence => params.noise_persistence = value as f32,
        WgKnob::HeightCurve => params.height_curve_exponent = value as f32,
        WgKnob::DomainWarp => params.domain_warp_strength = value as f32,
        WgKnob::DetailMix => params.terrain_detail_mix = value as f32,
        WgKnob::WarpScale => n.warp_noise_scale_mul = value as f32,
        WgKnob::WarpOctaves => n.warp_noise_octaves = whole as u32,
        WgKnob::WarpSeed => n.warp_seed_offset = whole.max(0.0) as u64,
        WgKnob::DetailScale => n.detail_noise_scale_mul = value as f32,
        WgKnob::DetailOctaves => n.detail_noise_octaves = whole as u32,
        WgKnob::DetailSeed => n.detail_seed_offset = whole.max(0.0) as u64,
        WgKnob::WarpFreq => n.warp_coord_frequency_mul = value,
        WgKnob::WarpZ => n.warp_coord_z = value,
        WgKnob::WarpPhaseX => n.warp_phase_offset_x = value,
        WgKnob::WarpPhaseY => n.warp_phase_offset_y = value,
        WgKnob::WarpDisplacement => n.warp_displacement_scale = value,
        WgKnob::DetailFreq => n.detail_coord_frequency_mul = value,
        WgKnob::DetailPersistence => n.detail_persistence_mul = value,
        WgKnob::Rivers => params.river_count = whole as u32,
        WgKnob::Lakes => params.lake_count = whole as u32,
        WgKnob::MountainThreshold => params.mountain_threshold = value as f32,
        WgKnob::IslandFalloff => params.island_falloff = value as f32,
        WgKnob::SeaLevel => b.sea_level = value as f32,
        WgKnob::DeepWater => b.deep_water_height_max = value as f32,
        WgKnob::ShallowWater => b.shallow_water_height_max = value as f32,
        WgKnob::Beach => b.beach_height_max = value as f32,
        WgKnob::MountainHeight => b.mountain_height_min = value as f32,
        WgKnob::GrasslandMoist => b.grassland_moisture_max = value as f32,
        WgKnob::DesertMoist => b.desert_moisture_max = value as f32,
        WgKnob::SwampMoist => b.swamp_moisture_min = value as f32,
        WgKnob::ForestMoist => b.forest_moisture_max = value as f32,
        WgKnob::HotLowlands => b.hot_lowlands_temperature_min = value as f32,
        WgKnob::Tundra => b.tundra_temperature_max = value as f32,
        WgKnob::SnowPeak => b.snow_peak_temperature_max = value as f32,
        WgKnob::Settlements => params.settlement_count = whole.max(0.0) as u32,
    }
}

fn format_knob(params: &WorldGenParams, knob: WgKnob) -> String {
    let v = knob_get(params, knob);
    if knob_integer(knob) {
        format!("{}  {}", knob_label(knob), v.round() as i64)
    } else {
        format!("{}  {v:.3}", knob_label(knob))
    }
}

fn layer_kinds() -> [(&'static str, LayerKind); 14] {
    [
        ("None", LayerKind::Base(PreviewLayers::empty())),
        ("Height", LayerKind::Base(PreviewLayers::HEIGHT)),
        ("Moisture", LayerKind::Base(PreviewLayers::MOISTURE)),
        ("Temperature", LayerKind::Base(PreviewLayers::TEMPERATURE)),
        ("Biome", LayerKind::Base(PreviewLayers::BIOME)),
        ("Ecology", LayerKind::Base(PreviewLayers::ECOLOGY)),
        ("Regions", LayerKind::Base(PreviewLayers::REGIONS)),
        ("Tags", LayerKind::Overlay(PreviewLayers::TAG_OVERLAY)),
        ("Slope", LayerKind::Overlay(PreviewLayers::DERIVED_SLOPE_OVERLAY)),
        ("Mobility", LayerKind::Overlay(PreviewLayers::MOBILITY_OVERLAY)),
        ("Fire heat", LayerKind::Fire),
        ("Logistics heat", LayerKind::Logistics),
        ("Construction heat", LayerKind::Construction),
        ("Ecology heat", LayerKind::Ecology),
    ]
}

pub(crate) fn spawn_world_gen_bevy_shell(
    mut commands: Commands,
    existing: Query<Entity, With<WorldGenShellRoot>>,
    palette: Res<UiPalette>,
) {
    if !existing.is_empty() {
        return;
    }
    let font = TextFont::from_font_size(12.0);
    let fill = palette.bevy_hud_panel_fill();
    let text = palette.bevy_primary_text();
    let muted = palette.bevy_text_muted();
    let border = palette.bevy_border_subtle();
    commands
        .spawn((
            WorldGenShellRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(palette.bevy_backdrop()),
            ZIndex(800),
        ))
        .with_children(|root| {
            root.spawn((
                WorldGenFlowText,
                Text::new("Operational Archive — World Index"),
                font.clone(),
                TextColor(text),
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    ..default()
                },
            ));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            })
            .with_children(|bar| {
                for (label, kind) in layer_kinds() {
                    spawn_btn(bar, label, WorldGenLayerButton { kind }, &font, fill, border, text);
                }
                spawn_btn(bar, "Follow", WorldGenAction::Follow, &font, fill, border, text);
                spawn_btn(bar, "GPU target", WorldGenAction::Gpu, &font, fill, border, text);
                spawn_btn(bar, "Zoom −", WorldGenAction::ZoomOut, &font, fill, border, text);
                spawn_btn(bar, "Zoom +", WorldGenAction::ZoomIn, &font, fill, border, text);
                spawn_btn(bar, "1:1", WorldGenAction::ZoomOne, &font, fill, border, text);
                spawn_btn(bar, "Fit", WorldGenAction::ZoomFit, &font, fill, border, text);
                spawn_btn(bar, "Parameters ▸", WorldGenAction::Parameters, &font, fill, border, text);
            });
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                column_gap: Val::Px(6.0),
                padding: UiRect::horizontal(Val::Px(6.0)),
                ..default()
            })
            .with_children(|body| {
                body.spawn((
                    WorldGenSheetRoot,
                    Node {
                        width: Val::Px(420.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        row_gap: Val::Px(2.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(palette.bevy_bg_elevated()),
                ))
                .with_children(|sheet| {
                    sheet.spawn((
                        Text::new("World Generator — Parameters"),
                        font.clone(),
                        TextColor(text),
                    ));
                    spawn_btn(sheet, "Generate preview", WorldGenAction::GeneratePreview, &font, fill, border, text);
                    spawn_btn(sheet, "Generate full world", WorldGenAction::GenerateFull, &font, fill, border, text);
                    spawn_btn(sheet, "Enter world", WorldGenAction::EnterWorld, &font, fill, border, text);
                    spawn_btn(sheet, "Edit generated world in map editor", WorldGenAction::EditMap, &font, fill, border, text);
                    spawn_btn(sheet, "Discard generated world", WorldGenAction::DiscardWorld, &font, fill, border, text);
                    spawn_btn(sheet, "Discard preview", WorldGenAction::DiscardPreview, &font, fill, border, text);
                    spawn_btn(sheet, "Close panel", WorldGenAction::ClosePanel, &font, fill, border, text);
                    spawn_btn(sheet, "Random Seed", WorldGenAction::RandomSeed, &font, fill, border, text);
                    sheet.spawn((WorldGenSeedText, Text::new("Seed: —"), font.clone(), TextColor(text)));
                    spawn_btn(sheet, "Medium-small (320²)", WorldGenAction::MediumSmall, &font, fill, border, text);
                    spawn_btn(sheet, "Standard (512²)", WorldGenAction::Standard, &font, fill, border, text);
                    spawn_btn(sheet, "Sync land features", WorldGenAction::SyncLand, &font, fill, border, text);
                    spawn_btn(sheet, "Region method", WorldGenAction::RegionMethod, &font, fill, border, text);
                    spawn_btn(sheet, "Height profile", WorldGenAction::HeightProfile, &font, fill, border, text);
                    spawn_btn(sheet, "Island Mode", WorldGenAction::Island, &font, fill, border, text);
                    spawn_btn(sheet, "Reload tuning", WorldGenAction::ReloadTuning, &font, fill, border, text);
                    spawn_btn(sheet, "Save tuning (RON)", WorldGenAction::SaveTuning, &font, fill, border, text);
                    spawn_btn(sheet, "Reset all parameters to defaults", WorldGenAction::ResetDefaults, &font, fill, border, text);
                    sheet.spawn((WorldGenHintText, Text::new(""), font.clone(), TextColor(muted)));
                    sheet.spawn((WorldGenDebugText, Text::new("No report yet"), font.clone(), TextColor(muted)));
                    for knob in KNOBS {
                        sheet.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_btn(row, "−", WorldGenKnobStep { knob: *knob, dir: -1 }, &font, fill, border, text);
                            spawn_btn(row, "+", WorldGenKnobStep { knob: *knob, dir: 1 }, &font, fill, border, text);
                            row.spawn((
                                WorldGenKnobReadout(*knob),
                                Text::new(knob_label(*knob)),
                                font.clone(),
                                TextColor(text),
                            ));
                        });
                    }
                    sheet.spawn((
                        Text::new("Mobility profile"),
                        font.clone(),
                        TextColor(text),
                    ));
                    spawn_btn(sheet, "Mobility profile", WorldGenAction::Mobility, &font, fill, border, text);
                    sheet.spawn((WorldGenMobilityText, Text::new("Mobility: —"), font.clone(), TextColor(muted)));
                    sheet.spawn((
                        Text::new("Terrain tag pool"),
                        font.clone(),
                        TextColor(text),
                    ));
                    sheet.spawn((
                        WorldGenTagList,
                        Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                    ));
                });
                body.spawn((
                    WorldGenPreviewPanel,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        WorldGenPreviewImage,
                        ImageNode::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                    ));
                    panel.spawn((
                        WorldGenCornerInset,
                        ImageNode::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(8.0),
                            bottom: Val::Px(8.0),
                            width: Val::Px(d07_inset_side_px()),
                            height: Val::Px(d07_inset_side_px()),
                            ..default()
                        },
                    ));
                    panel.spawn((
                        WorldGenMapDim,
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 102.0 / 255.0)),
                        Visibility::Hidden,
                    ));
                    panel.spawn((
                        WorldGenPlaceholderText,
                        Text::new("Preview initializing"),
                        font.clone(),
                        TextColor(muted),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(12.0),
                            top: Val::Px(12.0),
                            ..default()
                        },
                    ));
                });
            });
            root.spawn((
                WorldGenStatusText,
                Text::new(""),
                font.clone(),
                TextColor(text),
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    ..default()
                },
            ));
            root.spawn((
                WorldGenConfirmRoot,
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(30.0),
                    top: Val::Percent(40.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(12.0)),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(palette.bevy_bg_elevated()),
                ZIndex(20),
            ))
            .with_children(|confirm| {
                confirm.spawn((
                    Text::new("Generation is still running. Close anyway and discard partial work?"),
                    font.clone(),
                    TextColor(text),
                ));
                spawn_btn(confirm, "Keep open", WorldGenAction::KeepOpen, &font, fill, border, text);
                spawn_btn(confirm, "Close and cancel", WorldGenAction::CloseAndCancel, &font, fill, border, text);
            });
        });
}

fn spawn_btn(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    extra: impl Bundle,
    font: &TextFont,
    fill: Color,
    border: Color,
    text: Color,
) {
    parent
        .spawn((
            Button,
            extra,
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(fill),
            BorderColor::all(border),
        ))
        .with_children(|b| {
            b.spawn((Text::new(label), font.clone(), TextColor(text)));
        });
}

pub(crate) fn despawn_world_gen_bevy_shell(
    mut commands: Commands,
    roots: Query<Entity, With<WorldGenShellRoot>>,
) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

#[derive(Default)]
struct PreviewPointer {
    last: Option<Vec2>,
    tex: (u32, u32),
    tags: usize,
}

#[derive(SystemParam)]
pub(crate) struct ShellSyncA<'w> {
    palette: Res<'w, UiPalette>,
    ui_state: Res<'w, WorldGenUiState>,
    preview_ui: ResMut<'w, WorldPreviewUiState>,
    params: Res<'w, WorldGenParams>,
    progress: Res<'w, WorldGenProgress>,
    flow: Res<'w, State<WorldGenFlowState>>,
    debug: Res<'w, crate::terrain::generation::WorldGenLastDebugReport>,
    hint: Res<'w, WorldGenTuningNote>,
    preview_tex: Res<'w, WorldPreviewTexture>,
    path: Res<'w, PreviewPathAuthority>,
    registry: Res<'w, WorldPreviewRenderTargetRegistry>,
    lifecycle: Res<'w, WorldPreviewLifecycle>,
    views: ResMut<'w, crate::gui::MapViewInstances>,
    preview_cam: ResMut<'w, PreviewCameraState>,
}

#[derive(SystemParam)]
pub(crate) struct ShellSyncB<'w, 's> {
    viewport_requests: ResMut<'w, crate::gui::ViewportAuthority>,
    viewport_sanity: ResMut<'w, crate::gui::hud::ViewportRectSanity>,
    shell_diag: Option<ResMut<'w, crate::gui::hud::ProductShellDiagnostics>>,
    pending_layout: Res<'w, crate::gui::hud::PendingHudLayoutCommit>,
    handles: Option<Res<'w, TerrainRegistriesHandles>>,
    tag_assets: Option<Res<'w, Assets<crate::terrain::material::TagRegistry>>>,
    mobility_assets: Option<Res<'w, Assets<crate::terrain::mobility::MobilityProfileRegistry>>>,
    // Marker-disjoint: one entity never carries two of these chrome markers.
    // Shared `Visibility` / `Text` writes panic at schedule init (B0001) otherwise.
    shell: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    sheet: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenSheetRoot>,
            Without<WorldGenShellRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    confirm: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenConfirmRoot>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    dim: Query<
        'w,
        's,
        &'static mut Visibility,
        (
            With<WorldGenMapDim>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    image_q: Query<
        'w,
        's,
        (&'static mut ImageNode, &'static mut Node, &'static mut Visibility),
        (
            With<WorldGenPreviewImage>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenCornerInset>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
    inset_q: Query<
        'w,
        's,
        (&'static mut ImageNode, &'static mut Visibility),
        (
            With<WorldGenCornerInset>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPlaceholderText>,
        ),
    >,
}

#[derive(SystemParam)]
pub(crate) struct ShellSyncC<'w, 's> {
    placeholder: Query<
        'w,
        's,
        (&'static mut Text, &'static mut Visibility),
        (
            With<WorldGenPlaceholderText>,
            Without<WorldGenShellRoot>,
            Without<WorldGenSheetRoot>,
            Without<WorldGenConfirmRoot>,
            Without<WorldGenMapDim>,
            Without<WorldGenPreviewImage>,
            Without<WorldGenCornerInset>,
            Without<WorldGenStatusText>,
            Without<WorldGenFlowText>,
            Without<WorldGenSeedText>,
            Without<WorldGenHintText>,
            Without<WorldGenDebugText>,
            Without<WorldGenMobilityText>,
            Without<WorldGenKnobReadout>,
        ),
    >,
    status: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<WorldGenStatusText>,
            Without<WorldGenPlaceholderText>,
            Without<WorldGenFlowText>,
            Without<WorldGenSeedText>,
            Without<WorldGenHintText>,
            Without<WorldGenDebugText>,
            Without<WorldGenMobilityText>,
            Without<WorldGenKnobReadout>,
        ),
    >,
    flow_text: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<WorldGenFlowText>,
            Without<WorldGenPlaceholderText>,
            Without<WorldGenStatusText>,
            Without<WorldGenSeedText>,
            Without<WorldGenHintText>,
            Without<WorldGenDebugText>,
            Without<WorldGenMobilityText>,
            Without<WorldGenKnobReadout>,
        ),
    >,
    seed_text: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<WorldGenSeedText>,
            Without<WorldGenPlaceholderText>,
            Without<WorldGenStatusText>,
            Without<WorldGenFlowText>,
            Without<WorldGenHintText>,
            Without<WorldGenDebugText>,
            Without<WorldGenMobilityText>,
            Without<WorldGenKnobReadout>,
        ),
    >,
    hint_text: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<WorldGenHintText>,
            Without<WorldGenPlaceholderText>,
            Without<WorldGenStatusText>,
            Without<WorldGenFlowText>,
            Without<WorldGenSeedText>,
            Without<WorldGenDebugText>,
            Without<WorldGenMobilityText>,
            Without<WorldGenKnobReadout>,
        ),
    >,
    debug_text: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<WorldGenDebugText>,
            Without<WorldGenPlaceholderText>,
            Without<WorldGenStatusText>,
            Without<WorldGenFlowText>,
            Without<WorldGenSeedText>,
            Without<WorldGenHintText>,
            Without<WorldGenMobilityText>,
            Without<WorldGenKnobReadout>,
        ),
    >,
    mobility_text: Query<
        'w,
        's,
        &'static mut Text,
        (
            With<WorldGenMobilityText>,
            Without<WorldGenPlaceholderText>,
            Without<WorldGenStatusText>,
            Without<WorldGenFlowText>,
            Without<WorldGenSeedText>,
            Without<WorldGenHintText>,
            Without<WorldGenDebugText>,
            Without<WorldGenKnobReadout>,
        ),
    >,
    readouts: Query<
        'w,
        's,
        (&'static WorldGenKnobReadout, &'static mut Text),
        (
            Without<WorldGenPlaceholderText>,
            Without<WorldGenStatusText>,
            Without<WorldGenFlowText>,
            Without<WorldGenSeedText>,
            Without<WorldGenHintText>,
            Without<WorldGenDebugText>,
            Without<WorldGenMobilityText>,
        ),
    >,
    layer_buttons: Query<'w, 's, (&'static WorldGenLayerButton, &'static mut BackgroundColor)>,
    panel_q: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<WorldGenPreviewPanel>>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    wheel: MessageReader<'w, 's, MouseWheel>,
    pointer: Local<'s, PreviewPointer>,
    tag_list: Query<'w, 's, Entity, With<WorldGenTagList>>,
}

pub(crate) fn sync_world_gen_bevy_shell(
    mut commands: Commands,
    a: ShellSyncA,
    b: ShellSyncB,
    c: ShellSyncC,
) {
    let ShellSyncA {
        palette,
        ui_state,
        mut preview_ui,
        params,
        progress,
        flow,
        debug,
        hint,
        preview_tex,
        path,
        registry,
        lifecycle,
        mut views,
        mut preview_cam,
    } = a;
    let ShellSyncB {
        mut viewport_requests,
        mut viewport_sanity,
        mut shell_diag,
        pending_layout,
        handles,
        tag_assets,
        mobility_assets,
        mut shell,
        mut sheet,
        mut confirm,
        mut dim,
        mut image_q,
        mut inset_q,
    } = b;
    let ShellSyncC {
        mut placeholder,
        mut status,
        mut flow_text,
        mut seed_text,
        mut hint_text,
        mut debug_text,
        mut mobility_text,
        mut readouts,
        mut layer_buttons,
        panel_q,
        windows,
        mouse,
        keys,
        mut wheel,
        mut pointer,
        tag_list,
    } = c;
    let show = preview_ui.window_open || ui_state.visible;
    for mut vis in &mut shell {
        *vis = if show { Visibility::Visible } else { Visibility::Hidden };
    }
    if !show {
        preview_ui.d07_corner_inset_active = false;
        preview_ui.d07_inset_side_px = 0.0;
        clear_world_preview_viewport_requests(&mut viewport_requests);
        return;
    }
    let sheet_open = ui_state.visible && ui_state.generator_sheet_open;
    let vis_sheet = if sheet_open { Visibility::Visible } else { Visibility::Hidden };
    for mut vis in &mut sheet {
        *vis = vis_sheet;
    }
    let confirm_vis = if ui_state.confirm_close_pending {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut vis in &mut confirm {
        *vis = confirm_vis;
    }
    for mut vis in &mut dim {
        *vis = if sheet_open { Visibility::Visible } else { Visibility::Hidden };
    }

    let view = &mut views.world_preview;
    let disp_w = preview_tex.width.max(1);
    let disp_h = preview_tex.height.max(1);
    if pointer.tex != (disp_w, disp_h) {
        pointer.tex = (disp_w, disp_h);
        let panel = if view.viewport_size.x > 0.0 {
            view.viewport_size
        } else {
            Vec2::new(disp_w as f32, disp_h as f32)
        };
        fit_viewport_to_map(view, panel, disp_w as f32, disp_h as f32);
    }
    ensure_viewport_camera_initialized(view, disp_w as f32, disp_h as f32);
    view.zoom = view.zoom.clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);

    let window = windows.iter().next();
    let scale = window.map(|w| w.scale_factor()).unwrap_or(1.0);
    let cursor = window.and_then(|w| w.cursor_position());
    if let Ok((node, xf)) = panel_q.single() {
        if !node.is_empty() {
            let panel_rect = node_logical_rect(node, xf, scale);
            preview_ui.last_viewport_rect = Some(panel_rect);
            preview_ui.last_window_rect = Some(panel_rect);
            write_world_preview_viewport_request(
                &mut viewport_requests,
                panel_rect,
                disp_w,
                disp_h,
                &mut viewport_sanity,
                shell_diag.as_deref_mut(),
                pending_layout.can_emit_layout_capture(),
            );
            let inner = bevy::prelude::Rect::from_corners(
                panel_rect.min + bevy::prelude::Vec2::splat(MAP_PANEL_INSET_PX),
                panel_rect.max - bevy::prelude::Vec2::splat(MAP_PANEL_INSET_PX),
            );
            let fit = compute_map_fit_strict(inner, UVec2::new(disp_w, disp_h), view.fit_mode);
            let gpu_painted = path.authoritative_surface
                == super::preview_render_contract::PreviewAuthoritativeSurface::GpuRenderTarget;
            let image_rect = if gpu_painted {
                fit.image_rect
            } else {
                map_presentation_image_rect(
                    fit.image_rect,
                    view.camera_center,
                    view.zoom,
                    disp_w as f32,
                    disp_h as f32,
                )
            };
            if let Some(cursor) = cursor {
                let p = bevy::prelude::Vec2::new(cursor.x, cursor.y);
                if panel_rect.contains(p) {
                    let scroll: f32 = wheel.read().map(|w| w.y).sum();
                    let zoom_mod = keys.pressed(KeyCode::ControlLeft)
                        || keys.pressed(KeyCode::ControlRight)
                        || keys.pressed(KeyCode::SuperLeft);
                    if zoom_mod && scroll != 0.0 {
                        let factor = 1.0 + scroll * 0.00125;
                        apply_zoom_toward(view, p, image_rect.center(), factor);
                        preview_cam.zoom = view.zoom;
                        preview_cam.center = view.camera_center;
                    }
                    if mouse.pressed(MouseButton::Middle) {
                        if let Some(last) = pointer.last {
                            apply_pan(view, cursor - last);
                            preview_cam.center = view.camera_center;
                        }
                        pointer.last = Some(cursor);
                    } else {
                        pointer.last = None;
                    }
                } else {
                    pointer.last = None;
                }
            }
            let source = resolve_world_preview_texture_source(
                path.as_ref(),
                registry.as_ref(),
                preview_tex.as_ref(),
            );
            let handle = source.handle().clone();
            let ready = handle != Handle::default()
                && lifecycle.phase.allows_egui_present();
            if let Ok((mut image, mut node, mut vis)) = image_q.single_mut() {
                *image = ImageNode::from(handle.clone());
                node.position_type = PositionType::Absolute;
                node.left = Val::Px((image_rect.min.x - panel_rect.min.x).max(0.0));
                node.top = Val::Px((image_rect.min.y - panel_rect.min.y).max(0.0));
                node.width = Val::Px(image_rect.width().max(1.0));
                node.height = Val::Px(image_rect.height().max(1.0));
                *vis = if ready { Visibility::Visible } else { Visibility::Hidden };
            }
            if let Ok((mut inset, mut vis)) = inset_q.single_mut() {
                *inset = ImageNode::from(handle.clone());
                let side = d07_inset_side_px();
                preview_ui.d07_corner_inset_active = ready;
                preview_ui.d07_inset_side_px = if ready { side } else { 0.0 };
                *vis = if ready { Visibility::Visible } else { Visibility::Hidden };
            }
            if let Ok((mut text, mut vis)) = placeholder.single_mut() {
                *text = Text::new(lifecycle.phase.placeholder_label());
                *vis = if ready { Visibility::Hidden } else { Visibility::Visible };
            }
            let _ = gpu_painted;
        }
    }

    if let Ok(mut text) = flow_text.single_mut() {
        let sheet = if ui_state.generator_sheet_open { "Parameters ◂" } else { "Parameters ▸" };
        *text = Text::new(format!(
            "Operational Archive — World Index    Flow: {:?}    {sheet}    {}",
            flow.get(),
            if progress.running {
                format!("{:.0}% {}", progress.fraction * 100.0, progress.label)
            } else {
                String::new()
            }
        ));
    }
    if let Ok(mut text) = seed_text.single_mut() {
        *text = Text::new(format!("Seed: {}", params.seed));
    }
    if let Ok(mut text) = hint_text.single_mut() {
        *text = Text::new(hint.0.clone());
    }
    if let Ok(mut text) = debug_text.single_mut() {
        let line = if debug.summary_one_line.is_empty() {
            "No report yet — generate a world once.".to_string()
        } else {
            debug.summary_one_line.clone()
        };
        *text = Text::new(line);
    }
    if let Ok(mut text) = status.single_mut() {
        *text = Text::new(format!(
            "Tile: {} | {}×{} world | z={:.2} | {}",
            view.hovered_tile
                .map(|t| format!("{} , {}", t.x, t.y))
                .unwrap_or_else(|| "—".to_string()),
            disp_w,
            disp_h,
            view.zoom,
            layer_summary(view.layers),
        ));
    }
    if let Ok(mut text) = mobility_text.single_mut() {
        let label = mobility_label(
            handles.as_deref(),
            mobility_assets.as_deref(),
            ui_state.mobility_profile_index,
        );
        *text = Text::new(label);
    }
    for (readout, mut text) in &mut readouts {
        *text = Text::new(format_knob(params.as_ref(), readout.0));
    }
    let accent = palette.bevy_accent_gold();
    let fill = palette.bevy_hud_panel_fill();
    for (button, mut bg) in &mut layer_buttons {
        let on = layer_on(view, button.kind);
        *bg = BackgroundColor(if on { accent } else { fill });
    }

    if let (Some(handles), Some(tags), Ok(list)) = (
        handles.as_deref(),
        tag_assets.as_deref(),
        tag_list.single(),
    ) {
        if let Some(reg) = tags.get(&handles.tag_registry) {
            if pointer.tags != reg.tags.len() {
                pointer.tags = reg.tags.len();
                commands.entity(list).despawn_related::<Children>();
                let font = TextFont::from_font_size(12.0);
                let fill = palette.bevy_hud_panel_fill();
                let border = palette.bevy_border_subtle();
                let text_c = palette.bevy_primary_text();
                for (i, tag) in reg.tags.iter().enumerate() {
                    let name = tag.name.clone();
                    commands.entity(list).with_children(|row| {
                        spawn_btn(
                            row,
                            &name,
                            WorldGenTagToggle(i as u16),
                            &font,
                            fill,
                            border,
                            text_c,
                        );
                    });
                }
            }
        }
    }
}

fn layer_on(view: &crate::gui::map_view::MapViewState, kind: LayerKind) -> bool {
    match kind {
        LayerKind::Base(base) => view.layers.base_bits() == base,
        LayerKind::Overlay(bit) => view.layers.contains(bit),
        LayerKind::Fire => view.overlays.fire_heat,
        LayerKind::Logistics => view.overlays.logistics_heat,
        LayerKind::Construction => view.overlays.construction_heat,
        LayerKind::Ecology => view.overlays.ecology_heat,
    }
}

fn layer_summary(layers: PreviewLayers) -> String {
    let base = layers.base_bits();
    let mut parts = Vec::new();
    if base.is_empty() {
        parts.push("base: none");
    } else if base.contains(PreviewLayers::REGIONS) {
        parts.push("regions");
    } else if base.contains(PreviewLayers::ECOLOGY) {
        parts.push("ecology");
    } else if base.contains(PreviewLayers::BIOME) {
        parts.push("biome");
    } else if base.contains(PreviewLayers::HEIGHT) {
        parts.push("height");
    } else if base.contains(PreviewLayers::MOISTURE) {
        parts.push("moisture");
    } else if base.contains(PreviewLayers::TEMPERATURE) {
        parts.push("temperature");
    }
    if layers.contains(PreviewLayers::TAG_OVERLAY) {
        parts.push("+tags");
    }
    if layers.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY) {
        parts.push("+slope");
    }
    if layers.contains(PreviewLayers::MOBILITY_OVERLAY) {
        parts.push("+mobility");
    }
    parts.join(" ")
}

fn mobility_label(
    handles: Option<&TerrainRegistriesHandles>,
    assets: Option<&Assets<crate::terrain::mobility::MobilityProfileRegistry>>,
    index: usize,
) -> String {
    let Some((handles, assets)) = handles.zip(assets) else {
        return "Mobility: —".to_string();
    };
    let Some(mob) = assets.get(&handles.mobility_profiles) else {
        return "Mobility: —".to_string();
    };
    if mob.profiles.is_empty() {
        return "Mobility: (none)".to_string();
    }
    let idx = index.min(mob.profiles.len() - 1);
    format!("Mobility: {}", mob.profiles[idx].id.as_str())
}

fn node_logical_rect(node: &ComputedNode, xf: &UiGlobalTransform, scale: f32) -> bevy::prelude::Rect {
    let scale = scale.max(1e-6);
    let half = node.size() * 0.5;
    let corners = [
        Vec2::new(-half.x, -half.y),
        Vec2::new(half.x, -half.y),
        Vec2::new(half.x, half.y),
        Vec2::new(-half.x, half.y),
    ];
    let mut pmin = Vec2::splat(f32::INFINITY);
    let mut pmax = Vec2::splat(f32::NEG_INFINITY);
    for corner in corners {
        let p = (*xf) * corner;
        pmin = pmin.min(p);
        pmax = pmax.max(p);
    }
    pmin /= scale;
    pmax /= scale;
    bevy::prelude::Rect::from_corners(bevy::prelude::Vec2::new(pmin.x, pmin.y), bevy::prelude::Vec2::new(pmax.x, pmax.y))
}

#[derive(SystemParam)]
pub(crate) struct ShellActA<'w> {
    ui_state: ResMut<'w, WorldGenUiState>,
    preview_ui: ResMut<'w, WorldPreviewUiState>,
    params: ResMut<'w, WorldGenParams>,
    views: ResMut<'w, crate::gui::MapViewInstances>,
    preview_cam: ResMut<'w, PreviewCameraState>,
    hint: ResMut<'w, WorldGenTuningNote>,
    next_flow: ResMut<'w, NextState<WorldGenFlowState>>,
    next_base: ResMut<'w, NextState<BaseState>>,
    next_app: ResMut<'w, NextState<AppState>>,
    next_wg: ResMut<'w, NextState<WorldGenState>>,
    next_pause: ResMut<'w, NextState<PauseState>>,
    latch: ResMut<'w, WorldGenChromeLatch>,
    lifecycle: ResMut<'w, WorldPreviewLifecycle>,
    flow: Res<'w, State<WorldGenFlowState>>,
    generate: MessageWriter<'w, GenerateWorldEvent>,
    cancel: MessageWriter<'w, CancelActiveWorldGenEvent>,
}

#[derive(SystemParam)]
pub(crate) struct ShellActB<'w, 's> {
    progress: Res<'w, WorldGenProgress>,
    job: Res<'w, WorldGenJobSlot>,
    roots: Query<'w, 's, Entity, With<WorldMarker>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    handles: Option<Res<'w, TerrainRegistriesHandles>>,
    mobility_assets: Option<Res<'w, Assets<crate::terrain::mobility::MobilityProfileRegistry>>>,
    preview_tex: Res<'w, WorldPreviewTexture>,
    layers: Query<'w, 's, (&'static Interaction, &'static WorldGenLayerButton), Changed<Interaction>>,
    knobs: Query<'w, 's, (&'static Interaction, &'static WorldGenKnobStep), Changed<Interaction>>,
    actions: Query<'w, 's, (&'static Interaction, &'static WorldGenAction), Changed<Interaction>>,
    tags: Query<'w, 's, (&'static Interaction, &'static WorldGenTagToggle), Changed<Interaction>>,
}

pub(crate) fn world_gen_bevy_shell_actions(
    mut commands: Commands,
    a: ShellActA,
    b: ShellActB,
) {
    let ShellActA {
        mut ui_state,
        mut preview_ui,
        mut params,
        mut views,
        mut preview_cam,
        mut hint,
        mut next_flow,
        mut next_base,
        mut next_app,
        mut next_wg,
        mut next_pause,
        mut latch,
        mut lifecycle,
        flow,
        mut generate,
        mut cancel,
    } = a;
    let ShellActB {
        progress,
        job,
        roots,
        keys,
        handles,
        mobility_assets,
        preview_tex,
        layers,
        knobs,
        actions,
        tags,
    } = b;
    if keys.just_pressed(KeyCode::Escape) && ui_state.generator_sheet_open {
        ui_state.generator_sheet_open = false;
    }
    let view = &mut views.world_preview;
    for (interaction, button) in &layers {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match button.kind {
            LayerKind::Base(base) => view.layers.replace_base(base),
            LayerKind::Overlay(bit) => view.layers.toggle(bit),
            LayerKind::Fire => view.overlays.fire_heat = !view.overlays.fire_heat,
            LayerKind::Logistics => view.overlays.logistics_heat = !view.overlays.logistics_heat,
            LayerKind::Construction => {
                view.overlays.construction_heat = !view.overlays.construction_heat
            }
            LayerKind::Ecology => view.overlays.ecology_heat = !view.overlays.ecology_heat,
        }
        view.bump_revision();
        ui_state.preview_layers = view.layers;
    }
    for (interaction, step) in &knobs {
        if *interaction != Interaction::Pressed {
            continue;
        }
        step_world_gen_knob(params.as_mut(), step.knob, step.dir);
    }
    for (interaction, tag) in &tags {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let id = TagId(tag.0);
        if params.tag_pool.contains(id) {
            params.tag_pool.remove(id);
        } else {
            params.tag_pool.insert(id);
        }
    }
    let busy = progress.running || job.is_busy();
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            WorldGenAction::Parameters => {
                ui_state.generator_sheet_open = !ui_state.generator_sheet_open;
            }
            WorldGenAction::Follow => {
                view.follow_mode = match view.follow_mode {
                    crate::gui::MinimapFollowMode::Free => crate::gui::MinimapFollowMode::FollowCamera,
                    crate::gui::MinimapFollowMode::FollowCamera => {
                        crate::gui::MinimapFollowMode::FollowBookmark
                    }
                    crate::gui::MinimapFollowMode::FollowBookmark => crate::gui::MinimapFollowMode::Free,
                };
                view.bump_revision();
            }
            WorldGenAction::RegionMethod => {
                params.region_method = next_region(params.region_method);
            }
            WorldGenAction::HeightProfile => {
                params.height_noise_profile = next_profile(params.height_noise_profile);
            }
            WorldGenAction::Island => params.island_mode = !params.island_mode,
            WorldGenAction::Gpu => {
                preview_cam.mode = if preview_cam.mode == PreviewRenderMode::GpuRenderTarget {
                    PreviewRenderMode::CpuRaster
                } else {
                    PreviewRenderMode::GpuRenderTarget
                };
            }
            WorldGenAction::ZoomOut => {
                view.zoom = (view.zoom / 1.1).clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
                preview_cam.zoom = view.zoom;
            }
            WorldGenAction::ZoomIn => {
                view.zoom = (view.zoom * 1.1).clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
                preview_cam.zoom = view.zoom;
            }
            WorldGenAction::ZoomOne => {
                view.zoom = 1.0;
                preview_cam.zoom = 1.0;
            }
            WorldGenAction::ZoomFit => {
                view.reset_camera_for_map(preview_tex.width as f32, preview_tex.height as f32);
                view.zoom = 1.0;
                preview_cam.zoom = 1.0;
                preview_cam.center = view.camera_center;
            }
            WorldGenAction::RandomSeed => params.seed = rand::random(),
            WorldGenAction::MediumSmall => {
                params.apply_tile_extent_preset(TileExtentPreset::MediumSmall);
            }
            WorldGenAction::Standard => {
                params.apply_tile_extent_preset(TileExtentPreset::Standard);
            }
            WorldGenAction::SyncLand => params.recompute_symbolic_land_features(),
            WorldGenAction::ReloadTuning => match load_overlay_prefer_ron(
                WORLD_GEN_TUNING_RON_PATH,
                WORLD_GEN_TUNING_JSON_PATH,
            ) {
                Ok(Some(overlay)) => {
                    if let Some(n) = overlay.noise_sampling {
                        params.noise_sampling = n;
                    }
                    if let Some(b) = overlay.biome_tuning {
                        params.biome_tuning = b;
                    }
                    hint.0 = "Reloaded overlay.".to_string();
                }
                Ok(None) => hint.0 = "No file — using in-memory params only.".to_string(),
                Err(err) => hint.0 = format!("Reload error: {err}"),
            },
            WorldGenAction::SaveTuning => {
                let overlay = WorldGenTuningOverlay {
                    noise_sampling: Some(params.noise_sampling.clone()),
                    biome_tuning: Some(params.biome_tuning.clone()),
                };
                hint.0 = match save_overlay_ron(WORLD_GEN_TUNING_RON_PATH, &overlay) {
                    Ok(()) => "Saved RON overlay.".to_string(),
                    Err(err) => format!("Write error: {err}"),
                };
            }
            WorldGenAction::ResetDefaults => {
                *params = WorldGenParams::default();
                hint.0 = "Restored built-in defaults (new random seed).".to_string();
            }
            WorldGenAction::GeneratePreview if !busy => {
                if matches!(*flow.get(), WorldGenFlowState::Idle) {
                    NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::NewWorldSetup);
                }
                preview_ui.window_open = true;
                generate.write(GenerateWorldEvent {
                    params: params.clone(),
                    phase: WorldGenPhase::Preview,
                });
            }
            WorldGenAction::GenerateFull
                if matches!(*flow.get(), WorldGenFlowState::PreviewReady) && !busy =>
            {
                preview_ui.window_open = true;
                generate.write(GenerateWorldEvent {
                    params: params.clone(),
                    phase: WorldGenPhase::Full,
                });
            }
            WorldGenAction::EnterWorld
                if matches!(*flow.get(), WorldGenFlowState::FullReady) && !busy =>
            {
                ux_enter_world_from_world_gen(
                    &mut latch,
                    &mut next_app,
                    &mut next_wg,
                    &mut next_pause,
                    &mut next_base,
                    &mut ui_state,
                    &mut preview_ui,
                    &mut lifecycle,
                );
            }
            WorldGenAction::EditMap
                if matches!(*flow.get(), WorldGenFlowState::FullReady) && !busy =>
            {
                NextState::set_if_neq(&mut *next_base, BaseState::Editor);
                NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                ui_state.visible = false;
            }
            WorldGenAction::DiscardWorld
                if matches!(*flow.get(), WorldGenFlowState::FullReady) && !busy =>
            {
                despawn_generated_world_entities(&mut commands, &roots);
                NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                ui_state.visible = false;
            }
            WorldGenAction::DiscardPreview
                if matches!(*flow.get(), WorldGenFlowState::PreviewReady) && !busy =>
            {
                despawn_generated_world_entities(&mut commands, &roots);
                NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                ui_state.visible = false;
            }
            WorldGenAction::ClosePanel => {
                if busy {
                    ui_state.confirm_close_pending = true;
                } else {
                    ui_state.generator_sheet_open = false;
                }
            }
            WorldGenAction::KeepOpen => ui_state.confirm_close_pending = false,
            WorldGenAction::CloseAndCancel => {
                cancel.write(CancelActiveWorldGenEvent);
                preview_ui.window_open = false;
                ui_state.visible = false;
                ui_state.generator_sheet_open = false;
                ui_state.confirm_close_pending = false;
            }
            WorldGenAction::GeneratePreview
            | WorldGenAction::GenerateFull
            | WorldGenAction::EnterWorld
            | WorldGenAction::EditMap
            | WorldGenAction::DiscardWorld
            | WorldGenAction::DiscardPreview => {}
            WorldGenAction::Mobility => {
                if let (Some(handles), Some(assets)) = (handles.as_deref(), mobility_assets.as_deref())
                {
                    if let Some(mob) = assets.get(&handles.mobility_profiles) {
                        if !mob.profiles.is_empty() {
                            ui_state.mobility_profile_index =
                                (ui_state.mobility_profile_index + 1) % mob.profiles.len();
                        }
                    }
                }
            }
        }
    }
    ui_state.preview_layers = view.layers;
}

fn next_region(method: RegionMethod) -> RegionMethod {
    match method {
        RegionMethod::Regular => RegionMethod::Manhattan,
        RegionMethod::Manhattan => RegionMethod::Weighted,
        RegionMethod::Weighted => RegionMethod::Centroidal,
        RegionMethod::Centroidal => RegionMethod::Circular,
        RegionMethod::Circular => RegionMethod::Power,
        RegionMethod::Power => RegionMethod::Regular,
    }
}

fn next_profile(profile: TerrainNoiseProfile) -> TerrainNoiseProfile {
    match profile {
        TerrainNoiseProfile::FbmPerlin => TerrainNoiseProfile::RidgedMulti,
        TerrainNoiseProfile::RidgedMulti => TerrainNoiseProfile::Billow,
        TerrainNoiseProfile::Billow => TerrainNoiseProfile::HybridMulti,
        TerrainNoiseProfile::HybridMulti => TerrainNoiseProfile::FbmOpenSimplex,
        TerrainNoiseProfile::FbmOpenSimplex => TerrainNoiseProfile::FbmPerlin,
    }
}

#[cfg(test)]
mod wgen_tests {
    use super::*;

    #[test]
    fn wgen_width_step_stays_inside_egui_slider_range() {
        let mut params = WorldGenParams::default();
        params.width = 128;
        step_world_gen_knob(&mut params, WgKnob::Width, -1);
        assert_eq!(params.width, 128, "width must not drop below the egui minimum");
        step_world_gen_knob(&mut params, WgKnob::Width, 1);
        assert!(params.width > 128 && params.width <= MAX_WORLD_GEN_AXIS);
        params.width = MAX_WORLD_GEN_AXIS;
        step_world_gen_knob(&mut params, WgKnob::Width, 1);
        assert_eq!(params.width, MAX_WORLD_GEN_AXIS);
    }

    #[test]
    fn wgen_base_layer_replace_does_not_touch_height_math() {
        let mut layers = PreviewLayers::HEIGHT;
        layers.replace_base(PreviewLayers::BIOME);
        assert!(layers.contains(PreviewLayers::BIOME));
        assert!(!layers.contains(PreviewLayers::HEIGHT));
    }

    #[test]
    fn wgen_world_gen_session_does_not_schedule_egui_draw() {
        let plugin = include_str!("plugin.rs").replace("\r\n", "\n");
        assert!(
            !plugin.contains("RetiredEgui"),
            "world preview plugin must not schedule an egui pass"
        );
        let ui = include_str!("../world_gen_ui.rs");
        assert!(
            !ui.contains("ctx_mut"),
            "canonical world-gen UI must not acquire an egui context"
        );
        assert!(
            !ui.contains("RetiredEgui"),
            "canonical world-gen UI must not schedule the egui pass"
        );
        let shell = include_str!("world_gen_bevy_shell.rs");
        let shell_body = shell.split("#[cfg(test)]").next().unwrap_or(shell);
        assert!(
            !shell_body.contains("ctx_mut"),
            "WorldGen Bevy shell must not acquire an egui context"
        );
        assert!(shell_body.contains("WorldGenShellRoot"));
        assert!(shell_body.contains("struct WorldGenPreviewImage"));
        let theme = include_str!("../../style/theme.rs");
        let theme_fn = theme
            .split("fn apply_egui_theme_system")
            .nth(1)
            .expect("theme system");
        let theme_fn = theme_fn.split("\nfn ").next().unwrap_or(theme_fn);
        let world_gen_at = theme_fn
            .find("AppState::WorldGen")
            .expect("theme must see WorldGen");
        let ctx_at = theme_fn.find("ctx_mut").expect("theme still serves the editor");
        assert!(
            world_gen_at < ctx_at,
            "WorldGen must return before theme ctx_mut"
        );
        // Product / map-editor shells stay off during WorldGen (no egui finalize stubs).
        use crate::engine::AppState;
        use crate::engine::states::BaseState;
        use crate::gui::ui_gates::{map_editor_chrome_states_active, product_shell_states_active};
        assert!(
            !product_shell_states_active(BaseState::Editor, AppState::WorldGen),
            "product shell must stay off during WorldGen"
        );
        assert!(
            !map_editor_chrome_states_active(BaseState::Editor, AppState::WorldGen),
            "map editor chrome must stay off during WorldGen"
        );
        let sub = include_str!("../../../bevysubengines/world_generator_plugin.rs");
        let gate = sub
            .split("fn subengine_world_gen_editor_only")
            .nth(1)
            .expect("subengine gate");
        let gate = gate.split("\nfn ").next().unwrap_or(gate);
        assert!(
            gate.contains("AppState::WorldGen"),
            "duplicate subengine window must stay off during WorldGen"
        );
    }

    /// Schedule init of `sync_world_gen_bevy_shell` builds the same query pair
    /// that panics with B0001 when `Visibility` writers are not marker-disjoint.
    #[test]
    fn wgen_shell_sync_visibility_queries_are_disjoint() {
        use bevy::ecs::schedule::Schedule;
        use bevy::ecs::world::World;

        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(sync_world_gen_bevy_shell);
        schedule
            .initialize(&mut world)
            .expect("sync_world_gen_bevy_shell schedule must build");
    }
}
