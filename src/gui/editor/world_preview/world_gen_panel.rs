//! World-generator control surface — grouped parameters, step flow, progress, view toolbar.
//!
//! [`super::world_gen_bevy_shell`] owns the preview raster / viewport placement and spawns
//! empty toolbar + sheet containers; this module fills them (`fill_world_gen_panel`) on the
//! shared widget kit: collapsible parameter
//! groups with sliders + steppers (hold to repeat), a two-step Generate -> Full world flow with
//! progress, disabled-with-reason buttons, wheel scrolling and hover / pressed states.
//!
//! Layout keeps the top-right 224 x 56 px of the window free for the view toggle.

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

use crate::engine::{WorldGenFlowState, WorldGenState};
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::style::widgets::{
    scroll_column_bundle, spawn_button, spawn_note, spawn_stepper, spawn_toggle, ui_button_fired,
    ToggleState, UiButton, UiButtonLabel, UiButtonVariant, UiKit, UiSlider,
    UiTooltip, FONT_BODY, FONT_SMALL, FONT_TITLE, GAP, ROW_H, SECTION_GAP,
};
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::tuning_io::{
    load_overlay_prefer_ron, save_overlay_ron, WorldGenTuningOverlay,
};
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, GenerateWorldEvent, RegionMethod, TerrainNoiseProfile,
    WorldGenJobSlot, WorldGenParams, WorldGenPhase, WorldGenProgress, WorldMarker,
    MAX_WORLD_GEN_AXIS, WORLD_GEN_TUNING_JSON_PATH, WORLD_GEN_TUNING_RON_PATH,
};
use crate::terrain::material::{TagId, TagRegistry};
use crate::terrain::world_map_scale::TileExtentPreset;

use super::layers::PreviewLayers;
use super::preview_render_contract::{PreviewCameraState, PreviewRenderMode};
use super::texture_cache::WorldPreviewTexture;
use super::world_gen_bevy_shell::{WorldGenSheetRoot, WorldGenToolbarRoot};
use super::{WorldPreviewLifecycle, WorldPreviewUiState};

/// Last tuning I/O line (reload / save / reset). Session chrome only.
#[derive(Resource, Default)]
pub struct WorldGenTuningNote(pub String);

#[derive(Clone, Copy)]
enum WgKnob {
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


fn step_world_gen_knob(params: &mut WorldGenParams, knob: WgKnob, dir: i8) {
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


const SHEET_W_PX: f32 = 380.0;
/// Clear zone for the top-right view toggle (196 x 36 at 12 / 12).
const TOP_RIGHT_CLEAR_PX: f32 = 224.0;
const HOLD_DELAY_S: f32 = 0.35;
const HOLD_REPEAT_S: f32 = 0.05;

#[derive(Component)]
struct WgToolbarFilled;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LayerSel {
    Base(u32),
    Overlay(u32),
    Fire,
    Logistics,
    Construction,
    Ecology,
}

/// Every plain button of the panel.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum WgAct {
    Layer(LayerSel),
    Follow,
    Gpu,
    ZoomOut,
    ZoomIn,
    ZoomOne,
    ZoomFit,
    ParamsToggle,
    GeneratePreview,
    GenerateFull,
    EnterWorld,
    EditMap,
    Discard,
    CancelGen,
    Close,
    RandomSeed,
    PresetMedium,
    PresetStandard,
    SyncLand,
    ReloadTuning,
    SaveTuning,
    ResetDefaults,
    RegionMethod,
    HeightProfile,
    Island,
    Mobility,
}

#[derive(Component, Clone, Copy)]
struct WgKnobBtn {
    knob: WgKnob,
    dir: i8,
}

#[derive(Component, Clone, Copy)]
struct WgKnobSlider(WgKnob);

#[derive(Component, Clone, Copy)]
struct WgTag(u16);

#[derive(Component, Clone, Copy)]
struct WgSectionToggle(usize);

#[derive(Component, Clone, Copy)]
struct WgSectionBody(usize);

#[derive(Component)]
struct WgTagList;

#[derive(Component)]
struct WgProgressFill;

#[derive(Component)]
struct WgSheetRoot;

/// Live text slots.
#[derive(Component, Clone, Copy)]
enum WgText {
    Seed,
    Stage,
    Progress,
    Report,
    Hint,
    Mobility,
    RegionMethod,
    HeightProfile,
    Knob(WgKnob),
}

struct Section {
    title: &'static str,
    open_by_default: bool,
}

const SECTIONS: [Section; 8] = [
    Section { title: "Seed & size", open_by_default: true },
    Section { title: "Terrain shape", open_by_default: true },
    Section { title: "Water & settlements", open_by_default: true },
    Section { title: "Climate", open_by_default: false },
    Section { title: "Biome thresholds", open_by_default: false },
    Section { title: "Advanced noise", open_by_default: false },
    Section { title: "Terrain tags & mobility", open_by_default: false },
    Section { title: "Tuning files", open_by_default: false },
];

#[derive(Resource)]
pub struct WorldGenPanelState {
    collapsed: [bool; SECTIONS.len()],
}

impl Default for WorldGenPanelState {
    fn default() -> Self {
        let mut collapsed = [false; SECTIONS.len()];
        for (i, s) in SECTIONS.iter().enumerate() {
            collapsed[i] = !s.open_by_default;
        }
        Self { collapsed }
    }
}

const BASE_LAYERS: [(&str, u32); 7] = [
    ("None", 0),
    ("Height", 1),
    ("Moisture", 2),
    ("Temperature", 3),
    ("Biome", 4),
    ("Ecology", 5),
    ("Regions", 6),
];
const OVERLAY_LAYERS: [(&str, u32); 3] = [("Tags", 0), ("Slope", 1), ("Mobility", 2)];

fn base_bits(i: u32) -> PreviewLayers {
    match i {
        1 => PreviewLayers::HEIGHT,
        2 => PreviewLayers::MOISTURE,
        3 => PreviewLayers::TEMPERATURE,
        4 => PreviewLayers::BIOME,
        5 => PreviewLayers::ECOLOGY,
        6 => PreviewLayers::REGIONS,
        _ => PreviewLayers::empty(),
    }
}

fn overlay_bit(i: u32) -> PreviewLayers {
    match i {
        0 => PreviewLayers::TAG_OVERLAY,
        1 => PreviewLayers::DERIVED_SLOPE_OVERLAY,
        _ => PreviewLayers::MOBILITY_OVERLAY,
    }
}

fn layer_on(view: &crate::gui::map_view::MapViewState, sel: LayerSel) -> bool {
    match sel {
        LayerSel::Base(i) => view.layers.base_bits() == base_bits(i),
        LayerSel::Overlay(i) => view.layers.contains(overlay_bit(i)),
        LayerSel::Fire => view.overlays.fire_heat,
        LayerSel::Logistics => view.overlays.logistics_heat,
        LayerSel::Construction => view.overlays.construction_heat,
        LayerSel::Ecology => view.overlays.ecology_heat,
    }
}

// ---------------------------------------------------------------------------------------------
// Knob table (label / range / get / set). Ranges are the retired egui slider ranges.
// ---------------------------------------------------------------------------------------------

fn knob_label(knob: WgKnob) -> &'static str {
    match knob {
        WgKnob::Width => "Width (tiles)",
        WgKnob::Height => "Height (tiles)",
        WgKnob::NumRegions => "Regions",
        WgKnob::RegionIterations => "Region relaxation",
        WgKnob::StrategicCoupling => "Strategic field blend",
        WgKnob::MoistureBias => "Moisture bias",
        WgKnob::TemperatureBias => "Temperature bias",
        WgKnob::MoistureScale => "Moisture patch scale",
        WgKnob::TemperatureScale => "Temperature patch scale",
        WgKnob::MoistureFreq => "Moisture detail freq",
        WgKnob::TemperatureFreq => "Temperature detail freq",
        WgKnob::NoiseScale => "Noise scale",
        WgKnob::NoiseOctaves => "Noise octaves",
        WgKnob::Lacunarity => "Lacunarity",
        WgKnob::Persistence => "Persistence",
        WgKnob::HeightCurve => "Height curve",
        WgKnob::DomainWarp => "Domain warp",
        WgKnob::DetailMix => "Fine detail mix",
        WgKnob::WarpScale => "Warp noise scale",
        WgKnob::WarpOctaves => "Warp octaves",
        WgKnob::WarpSeed => "Warp seed offset",
        WgKnob::DetailScale => "Detail noise scale",
        WgKnob::DetailOctaves => "Detail octaves",
        WgKnob::DetailSeed => "Detail seed offset",
        WgKnob::WarpFreq => "Warp frequency",
        WgKnob::WarpZ => "Warp Z",
        WgKnob::WarpPhaseX => "Warp phase X",
        WgKnob::WarpPhaseY => "Warp phase Y",
        WgKnob::WarpDisplacement => "Warp displacement",
        WgKnob::DetailFreq => "Detail frequency",
        WgKnob::DetailPersistence => "Detail persistence",
        WgKnob::Rivers => "Rivers",
        WgKnob::Lakes => "Lakes",
        WgKnob::MountainThreshold => "Mountain threshold",
        WgKnob::IslandFalloff => "Island falloff",
        WgKnob::SeaLevel => "Sea level",
        WgKnob::DeepWater => "Deep water max height",
        WgKnob::ShallowWater => "Shallow water max height",
        WgKnob::Beach => "Beach max height",
        WgKnob::MountainHeight => "Mountain min height",
        WgKnob::GrasslandMoist => "Grassland max moisture",
        WgKnob::DesertMoist => "Desert max moisture",
        WgKnob::SwampMoist => "Swamp min moisture",
        WgKnob::ForestMoist => "Forest max moisture",
        WgKnob::HotLowlands => "Hot lowlands min temp",
        WgKnob::Tundra => "Tundra max temp",
        WgKnob::SnowPeak => "Snow peak max temp",
        WgKnob::Settlements => "Settlements (0 = none)",
    }
}

fn knob_bounds(knob: WgKnob) -> Option<(f64, f64)> {
    Some(match knob {
        WgKnob::Width | WgKnob::Height => (128.0, f64::from(MAX_WORLD_GEN_AXIS)),
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

fn knob_fraction(params: &WorldGenParams, knob: WgKnob) -> f32 {
    match knob_bounds(knob) {
        Some((lo, hi)) => ((knob_get(params, knob) - lo) / (hi - lo).max(1e-9)).clamp(0.0, 1.0) as f32,
        None => 0.0,
    }
}

fn format_knob_value(params: &WorldGenParams, knob: WgKnob) -> String {
    let v = knob_get(params, knob);
    if knob_integer(knob) {
        return format!("{}", v.round() as i64);
    }
    let span = knob_bounds(knob).map_or(1.0, |(lo, hi)| hi - lo);
    if span < 1.0 {
        format!("{v:.3}")
    } else {
        format!("{v:.2}")
    }
}

fn knob_tooltip(knob: WgKnob) -> String {
    match knob_bounds(knob) {
        Some((lo, hi)) => format!(
            "{}\nRange {} to {}. Click the bar to jump, +/- to step (hold to repeat).",
            knob_label(knob),
            lo,
            hi
        ),
        None => format!("{}\nStep with +/-. Changes the noise seed offset.", knob_label(knob)),
    }
}

/// Set a knob from a slider fraction; sizes snap to 32-tile steps.
fn knob_set_fraction(params: &mut WorldGenParams, knob: WgKnob, f: f32) {
    let Some((lo, hi)) = knob_bounds(knob) else { return };
    let mut v = lo + (hi - lo) * f64::from(f);
    if matches!(knob, WgKnob::Width | WgKnob::Height) {
        v = (v / 32.0).round() * 32.0;
    }
    knob_set(params, knob, v.clamp(lo, hi));
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

// ---------------------------------------------------------------------------------------------
// Spawn helpers
// ---------------------------------------------------------------------------------------------

fn knob_stepper(col: &mut ChildSpawnerCommands<'_>, kit: &UiKit, knob: WgKnob, p: &WorldGenParams) {
    spawn_stepper(
        col,
        kit,
        knob_label(knob),
        (WgKnobBtn { knob, dir: -1 }, UiTooltip(knob_tooltip(knob))),
        (WgKnobBtn { knob, dir: 1 }, UiTooltip(knob_tooltip(knob))),
        WgText::Knob(knob),
        &format_knob_value(p, knob),
        (WgKnobSlider(knob), UiTooltip(knob_tooltip(knob))),
        knob_fraction(p, knob),
    );
}

fn act_button<'a>(
    parent: &'a mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    variant: UiButtonVariant,
    act: WgAct,
    tip: &str,
) -> EntityCommands<'a> {
    let mut ec = spawn_button(parent, kit, label, UiButton::new(variant), act);
    ec.insert(UiTooltip(tip.to_string()));
    ec
}

fn grow(mut ec: EntityCommands<'_>) {
    ec.insert(Node {
        flex_grow: 1.0,
        flex_basis: Val::Px(0.0),
        min_height: Val::Px(ROW_H),
        padding: UiRect::axes(Val::Px(GAP), Val::Px(GAP)),
        border: UiRect::all(Val::Px(1.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    });
}

fn row(width_pct: f32) -> Node {
    Node {
        width: Val::Percent(width_pct),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        column_gap: Val::Px(GAP),
        row_gap: Val::Px(GAP),
        ..default()
    }
}

fn group_label(parent: &mut ChildSpawnerCommands<'_>, kit: &UiKit, text: &str) {
    parent.spawn((
        Text::new(text),
        kit.text_font(FONT_SMALL),
        TextColor(kit.palette.bevy_text_muted()),
        Node { margin: UiRect::right(Val::Px(GAP)), ..default() },
    ));
}

fn build_toolbar(bar: &mut ChildSpawnerCommands<'_>, kit: &UiKit) {
    bar.spawn(row(100.0)).with_children(|r| {
        group_label(r, kit, "BASE");
        for (label, i) in BASE_LAYERS {
            spawn_button(r, kit, label, UiButton::new(UiButtonVariant::Tab), WgAct::Layer(LayerSel::Base(i)))
                .insert(UiTooltip(format!("Show {label} as the preview base layer")));
        }
        group_label(r, kit, "OVERLAY");
        for (label, i) in OVERLAY_LAYERS {
            spawn_button(r, kit, label, UiButton::new(UiButtonVariant::Tab), WgAct::Layer(LayerSel::Overlay(i)))
                .insert(UiTooltip(format!("Toggle the {label} overlay")));
        }
    });
    bar.spawn(row(100.0)).with_children(|r| {
        group_label(r, kit, "HEAT");
        for (label, sel) in [
            ("Fire", LayerSel::Fire),
            ("Logistics", LayerSel::Logistics),
            ("Construction", LayerSel::Construction),
            ("Ecology", LayerSel::Ecology),
        ] {
            spawn_button(r, kit, label, UiButton::new(UiButtonVariant::Tab), WgAct::Layer(sel))
                .insert(UiTooltip(format!("Toggle the {label} heat overlay")));
        }
        group_label(r, kit, "VIEW");
        act_button(r, kit, "Follow: free", UiButtonVariant::Normal, WgAct::Follow, "Cycle camera follow mode");
        act_button(r, kit, "GPU target", UiButtonVariant::Normal, WgAct::Gpu, "Render the preview on the GPU (off = CPU raster)");
        act_button(r, kit, "-", UiButtonVariant::Normal, WgAct::ZoomOut, "Zoom out");
        act_button(r, kit, "+", UiButtonVariant::Normal, WgAct::ZoomIn, "Zoom in");
        act_button(r, kit, "1:1", UiButtonVariant::Normal, WgAct::ZoomOne, "Reset zoom to 1:1");
        act_button(r, kit, "Fit", UiButtonVariant::Normal, WgAct::ZoomFit, "Fit the whole map in the preview");
        act_button(r, kit, "Parameters", UiButtonVariant::Normal, WgAct::ParamsToggle, "Show / hide the parameter sheet (Esc closes it)");
    });
}

fn section<'a>(
    body: &'a mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    idx: usize,
    collapsed: bool,
    content: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) {
    body.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(GAP),
        margin: UiRect::top(Val::Px(SECTION_GAP)),
        ..default()
    })
    .with_children(|sec| {
        spawn_button(
            sec,
            kit,
            SECTIONS[idx].title,
            UiButton::new(UiButtonVariant::Tab),
            WgSectionToggle(idx),
        )
        .insert((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(ROW_H),
                padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            UiTooltip("Click to collapse / expand".into()),
        ));
        sec.spawn((
            WgSectionBody(idx),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(SECTION_GAP),
                padding: UiRect::axes(Val::Px(GAP), Val::Px(GAP)),
                display: if collapsed { Display::None } else { Display::Flex },
                ..default()
            },
        ))
        .with_children(content);
    });
}

fn build_sheet(
    sheet: &mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    p: &WorldGenParams,
    state: &WorldGenPanelState,
) {
    // Title + close.
    sheet
        .spawn(Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|r| {
            r.spawn((
                Text::new("World Generator"),
                kit.text_font(FONT_TITLE),
                TextColor(kit.palette.bevy_primary_text()),
            ));
            act_button(r, kit, "X", UiButtonVariant::Normal, WgAct::Close, "Hide the parameter sheet (Esc)");
        });
    // Stage + progress.
    sheet.spawn((
        WgText::Stage,
        Text::new(""),
        kit.text_font(FONT_SMALL),
        TextColor(kit.palette.bevy_accent_gold()),
        Node { width: Val::Percent(100.0), ..default() },
    ));
    sheet
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(kit.palette.bevy_bg_deep()),
            BorderColor::all(kit.palette.bevy_border_subtle()),
        ))
        .with_children(|track| {
            track.spawn((
                WgProgressFill,
                Node { width: Val::Percent(0.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(kit.palette.bevy_accent_terminal()),
            ));
        });
    sheet.spawn((
        WgText::Progress,
        Text::new(""),
        kit.text_font(FONT_SMALL),
        TextColor(kit.palette.bevy_text_muted()),
        Node { width: Val::Percent(100.0), ..default() },
    ));
    // Flow buttons.
    sheet.spawn(row(100.0)).with_children(|r| {
        grow(act_button(
            r,
            kit,
            "1  Generate preview",
            UiButtonVariant::Primary,
            WgAct::GeneratePreview,
            "Build a small preview terrain from the parameters below (disabled once a full world exists: enter or discard it first)",
        ));
    });
    sheet.spawn(row(100.0)).with_children(|r| {
        grow(act_button(
            r,
            kit,
            "2  Generate full world and enter",
            UiButtonVariant::Primary,
            WgAct::GenerateFull,
            "Needs a preview. Builds the full world from the current parameters; the game starts when it finishes.",
        ));
    });
    sheet.spawn(row(100.0)).with_children(|r| {
        grow(act_button(r, kit, "Enter world", UiButtonVariant::Normal, WgAct::EnterWorld, "Start the game in the generated world"));
        grow(act_button(r, kit, "Edit in map editor", UiButtonVariant::Normal, WgAct::EditMap, "Open the generated world in the map editor"));
    });
    sheet.spawn(row(100.0)).with_children(|r| {
        grow(act_button(r, kit, "Discard", UiButtonVariant::Danger, WgAct::Discard, "Throw away the generated preview / world"));
        grow(act_button(r, kit, "Cancel generation", UiButtonVariant::Danger, WgAct::CancelGen, "Stop the running generation (asks to confirm)"));
    });
    sheet.spawn((
        WgText::Report,
        Text::new(""),
        kit.text_font(FONT_SMALL),
        TextColor(kit.palette.bevy_text_muted()),
        Node { width: Val::Percent(100.0), ..default() },
    ));

    // Scrolling parameter groups.
    sheet
        .spawn((
            scroll_column_bundle(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                min_height: Val::Px(0.0),
                row_gap: Val::Px(GAP),
                padding: UiRect::right(Val::Px(GAP)),
                margin: UiRect::top(Val::Px(GAP)),
                border: UiRect::top(Val::Px(1.0)),
                ..default()
            }),
            BorderColor::all(kit.palette.bevy_border_subtle()),
        ))
        .with_children(|body| {
            let c = &state.collapsed;
            section(body, kit, 0, c[0], |s| {
                s.spawn(row(100.0)).with_children(|r| {
                    r.spawn((
                        WgText::Seed,
                        Text::new("Seed"),
                        kit.text_font(FONT_BODY),
                        TextColor(kit.palette.bevy_primary_text()),
                        Node { flex_grow: 1.0, ..default() },
                    ));
                    act_button(r, kit, "Random seed", UiButtonVariant::Normal, WgAct::RandomSeed, "Roll a new random seed");
                });
                s.spawn(row(100.0)).with_children(|r| {
                    grow(act_button(r, kit, "Medium-small 320x320", UiButtonVariant::Normal, WgAct::PresetMedium, "Set width and height to 320"));
                    grow(act_button(r, kit, "Standard 512x512", UiButtonVariant::Normal, WgAct::PresetStandard, "Set width and height to 512"));
                });
                knob_stepper(s, kit, WgKnob::Width, p);
                knob_stepper(s, kit, WgKnob::Height, p);
            });
            section(body, kit, 1, c[1], |s| {
                knob_stepper(s, kit, WgKnob::NumRegions, p);
                knob_stepper(s, kit, WgKnob::RegionIterations, p);
                s.spawn(row(100.0)).with_children(|r| {
                    grow(act_button(r, kit, "Region method", UiButtonVariant::Normal, WgAct::RegionMethod, "Cycle the region partition method"));
                    grow(act_button(r, kit, "Height profile", UiButtonVariant::Normal, WgAct::HeightProfile, "Cycle the height noise profile"));
                });
                s.spawn((
                    WgText::RegionMethod,
                    Text::new(""),
                    kit.text_font(FONT_SMALL),
                    TextColor(kit.palette.bevy_text_muted()),
                ));
                s.spawn((
                    WgText::HeightProfile,
                    Text::new(""),
                    kit.text_font(FONT_SMALL),
                    TextColor(kit.palette.bevy_text_muted()),
                ));
                for k in [
                    WgKnob::NoiseScale,
                    WgKnob::NoiseOctaves,
                    WgKnob::Lacunarity,
                    WgKnob::Persistence,
                    WgKnob::HeightCurve,
                    WgKnob::DomainWarp,
                    WgKnob::DetailMix,
                    WgKnob::MountainThreshold,
                ] {
                    knob_stepper(s, kit, k, p);
                }
                spawn_toggle(s, kit, "Island mode", p.island_mode, WgAct::Island);
                knob_stepper(s, kit, WgKnob::IslandFalloff, p);
            });
            section(body, kit, 2, c[2], |s| {
                for k in [WgKnob::Rivers, WgKnob::Lakes, WgKnob::Settlements] {
                    knob_stepper(s, kit, k, p);
                }
                act_button(s, kit, "Sync land features", UiButtonVariant::Normal, WgAct::SyncLand, "Recompute rivers / lakes / mountains symbolic features from the current parameters");
            });
            section(body, kit, 3, c[3], |s| {
                for k in [
                    WgKnob::MoistureBias,
                    WgKnob::TemperatureBias,
                    WgKnob::MoistureScale,
                    WgKnob::TemperatureScale,
                    WgKnob::MoistureFreq,
                    WgKnob::TemperatureFreq,
                    WgKnob::StrategicCoupling,
                ] {
                    knob_stepper(s, kit, k, p);
                }
            });
            section(body, kit, 4, c[4], |s| {
                for k in [
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
                ] {
                    knob_stepper(s, kit, k, p);
                }
            });
            section(body, kit, 5, c[5], |s| {
                for k in [
                    WgKnob::WarpScale,
                    WgKnob::WarpOctaves,
                    WgKnob::WarpSeed,
                    WgKnob::WarpFreq,
                    WgKnob::WarpZ,
                    WgKnob::WarpPhaseX,
                    WgKnob::WarpPhaseY,
                    WgKnob::WarpDisplacement,
                    WgKnob::DetailScale,
                    WgKnob::DetailOctaves,
                    WgKnob::DetailSeed,
                    WgKnob::DetailFreq,
                    WgKnob::DetailPersistence,
                ] {
                    knob_stepper(s, kit, k, p);
                }
            });
            section(body, kit, 6, c[6], |s| {
                s.spawn(row(100.0)).with_children(|r| {
                    grow(act_button(r, kit, "Mobility profile", UiButtonVariant::Normal, WgAct::Mobility, "Cycle the mobility profile used by the mobility overlay"));
                });
                s.spawn((
                    WgText::Mobility,
                    Text::new("Mobility: -"),
                    kit.text_font(FONT_SMALL),
                    TextColor(kit.palette.bevy_text_muted()),
                ));
                spawn_note(s, kit, "Terrain tag pool (checked tags may be generated):", true, ());
                s.spawn((
                    WgTagList,
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(GAP),
                        ..default()
                    },
                ));
            });
            section(body, kit, 7, c[7], |s| {
                s.spawn(row(100.0)).with_children(|r| {
                    grow(act_button(r, kit, "Reload", UiButtonVariant::Normal, WgAct::ReloadTuning, "Reload noise / biome tuning from the RON overlay"));
                    grow(act_button(r, kit, "Save RON", UiButtonVariant::Normal, WgAct::SaveTuning, "Save the noise / biome tuning as a RON overlay"));
                });
                act_button(s, kit, "Reset all parameters", UiButtonVariant::Danger, WgAct::ResetDefaults, "Restore built-in defaults (new random seed)");
                s.spawn((
                    WgText::Hint,
                    Text::new(""),
                    kit.text_font(FONT_SMALL),
                    TextColor(kit.palette.bevy_text_muted()),
                    Node { width: Val::Percent(100.0), ..default() },
                ));
            });
        });
}

// ---------------------------------------------------------------------------------------------
// Fill the shell's toolbar + sheet containers
// ---------------------------------------------------------------------------------------------

/// Build the toolbar and parameter sheet into the containers the shell spawned.
fn fill_world_gen_panel(
    mut commands: Commands,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    params: Res<WorldGenParams>,
    state: Res<WorldGenPanelState>,
    toolbars: Query<Entity, (With<WorldGenToolbarRoot>, Without<WgToolbarFilled>)>,
    sheets: Query<Entity, (With<WorldGenSheetRoot>, Without<WgSheetRoot>)>,
) {
    let kit = UiKit::new(&palette, mono.0.clone());
    for toolbar in &toolbars {
        commands.entity(toolbar).insert((
            WgToolbarFilled,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(GAP),
                padding: UiRect {
                    left: Val::Px(SECTION_GAP),
                    right: Val::Px(TOP_RIGHT_CLEAR_PX),
                    top: Val::Px(SECTION_GAP),
                    bottom: Val::Px(GAP),
                },
                ..default()
            },
        ));
        commands.entity(toolbar).with_children(|bar| build_toolbar(bar, &kit));
    }
    for sheet in &sheets {
        commands.entity(sheet).insert((
            WgSheetRoot,
            Node {
                width: Val::Px(SHEET_W_PX),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(GAP),
                padding: UiRect::all(Val::Px(SECTION_GAP)),
                border: UiRect::all(Val::Px(1.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_border_subtle()),
            FocusPolicy::Block,
        ));
        commands
            .entity(sheet)
            .with_children(|s| build_sheet(s, &kit, &params, &state));
    }
}

// ---------------------------------------------------------------------------------------------
// Live state
// ---------------------------------------------------------------------------------------------

fn busy(progress: &WorldGenProgress, job: &WorldGenJobSlot) -> bool {
    progress.running || job.is_busy()
}

fn stage_text(flow: WorldGenFlowState, busy: bool) -> &'static str {
    if busy {
        return "Generating ...";
    }
    match flow {
        WorldGenFlowState::PreviewReady => {
            "Step 2 of 2: preview ready. Tweak parameters and regenerate, or generate the full world."
        }
        WorldGenFlowState::FullReady => "World ready. Enter it, or edit it in the map editor.",
        _ => "Step 1 of 2: set parameters, then generate a preview.",
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct PanelSyncRes<'w> {
    params: Res<'w, WorldGenParams>,
    progress: Res<'w, WorldGenProgress>,
    job: Res<'w, WorldGenJobSlot>,
    flow: Res<'w, State<WorldGenFlowState>>,
    ui_state: Res<'w, WorldGenUiState>,
    views: Res<'w, crate::gui::MapViewInstances>,
    cam: Res<'w, PreviewCameraState>,
    hint: Res<'w, WorldGenTuningNote>,
    debug: Res<'w, crate::terrain::generation::WorldGenLastDebugReport>,
    panel: Res<'w, WorldGenPanelState>,
    handles: Option<Res<'w, TerrainRegistriesHandles>>,
    tags: Option<Res<'w, Assets<TagRegistry>>>,
    mobility: Option<Res<'w, Assets<crate::terrain::mobility::MobilityProfileRegistry>>>,
}

fn sync_world_gen_panel(
    mut commands: Commands,
    res: PanelSyncRes,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut texts: Query<(&WgText, &mut Text), Without<UiButtonLabel>>,
    mut labels: Query<&mut Text, With<UiButtonLabel>>,
    mut buttons: Query<(&WgAct, &mut UiButton, &Children), Without<WgSectionToggle>>,
    mut section_btns: Query<(&WgSectionToggle, &Children), Without<WgAct>>,
    mut section_bodies: Query<(&WgSectionBody, &mut Node), Without<WgProgressFill>>,
    mut fill: Query<&mut Node, (With<WgProgressFill>, Without<WgSectionBody>)>,
    mut sliders: Query<(&WgKnobSlider, &mut UiSlider)>,
    mut island: Query<(&WgAct, &mut ToggleState), Without<WgTag>>,
    mut tag_toggles: Query<(&WgTag, &mut ToggleState), Without<WgAct>>,
    tag_list: Query<Entity, With<WgTagList>>,
    mut built_tags: Local<usize>,
) {
    let p = &*res.params;
    let flow = *res.flow.get();
    let is_busy = busy(&res.progress, &res.job);
    let preview_ready = flow == WorldGenFlowState::PreviewReady;
    let full_ready = flow == WorldGenFlowState::FullReady;

    let mobility_label = || -> String {
        let Some((h, a)) = res.handles.as_deref().zip(res.mobility.as_deref()) else {
            return "Mobility: -".into();
        };
        let Some(m) = a.get(&h.mobility_profiles) else { return "Mobility: -".into() };
        if m.profiles.is_empty() {
            return "Mobility: (none)".into();
        }
        let i = res.ui_state.mobility_profile_index.min(m.profiles.len() - 1);
        format!("Mobility: {}", m.profiles[i].id.as_str())
    };

    for (slot, mut text) in &mut texts {
        let next = match slot {
            WgText::Seed => format!("Seed {}", p.seed),
            WgText::Stage => stage_text(flow, is_busy).to_string(),
            WgText::Progress => {
                if res.progress.running {
                    format!("{:.0}%  {}", res.progress.fraction * 100.0, res.progress.label)
                } else {
                    String::new()
                }
            }
            WgText::Report => {
                if res.debug.summary_one_line.is_empty() {
                    String::new()
                } else {
                    res.debug.summary_one_line.clone()
                }
            }
            WgText::Hint => res.hint.0.clone(),
            WgText::Mobility => mobility_label(),
            WgText::RegionMethod => format!("Region method: {:?}", p.region_method),
            WgText::HeightProfile => format!("Height profile: {:?}", p.height_noise_profile),
            WgText::Knob(k) => format_knob_value(p, *k),
        };
        if text.0 != next {
            text.0 = next;
        }
    }

    for mut node in &mut fill {
        let pct = if res.progress.running { res.progress.fraction.clamp(0.0, 1.0) * 100.0 } else { 0.0 };
        let w = Val::Percent(pct);
        if node.width != w {
            node.width = w;
        }
    }

    let view = &res.views.world_preview;
    let gpu = res.cam.mode == PreviewRenderMode::GpuRenderTarget;
    for (act, mut btn, kids) in &mut buttons {
        let (selected, disabled, label): (bool, bool, Option<String>) = match *act {
            WgAct::Layer(sel) => (layer_on(view, sel), false, None),
            WgAct::Follow => (
                view.follow_mode != crate::gui::MinimapFollowMode::Free,
                false,
                Some(
                    match view.follow_mode {
                        crate::gui::MinimapFollowMode::Free => "Follow: free",
                        crate::gui::MinimapFollowMode::FollowCamera => "Follow: camera",
                        crate::gui::MinimapFollowMode::FollowBookmark => "Follow: bookmark",
                    }
                    .to_string(),
                ),
            ),
            WgAct::Gpu => (gpu, false, None),
            WgAct::ParamsToggle => (res.ui_state.generator_sheet_open, false, None),
            WgAct::GeneratePreview => (
                false,
                is_busy || full_ready,
                Some(
                    if preview_ready {
                        "1  Regenerate preview"
                    } else {
                        "1  Generate preview"
                    }
                    .to_string(),
                ),
            ),
            WgAct::GenerateFull => (false, !(preview_ready && !is_busy), None),
            WgAct::EnterWorld | WgAct::EditMap => (false, !(full_ready && !is_busy), None),
            WgAct::Discard => (false, !((preview_ready || full_ready) && !is_busy), None),
            WgAct::CancelGen => (false, !is_busy, None),
            _ => continue,
        };
        btn.set(selected, disabled);
        if let Some(label) = label {
            for kid in kids.iter() {
                if let Ok(mut t) = labels.get_mut(kid) {
                    if t.0 != label {
                        t.0 = label.clone();
                    }
                }
            }
        }
    }

    for (sec, kids) in &mut section_btns {
        let collapsed = res.panel.collapsed[sec.0];
        let title = format!("{} {}", if collapsed { "[+]" } else { "[-]" }, SECTIONS[sec.0].title);
        for kid in kids.iter() {
            if let Ok(mut t) = labels.get_mut(kid) {
                if t.0 != title {
                    t.0 = title.clone();
                }
            }
        }
    }
    for (sec, mut node) in &mut section_bodies {
        let want = if res.panel.collapsed[sec.0] { Display::None } else { Display::Flex };
        if node.display != want {
            node.display = want;
        }
    }
    for (knob, mut slider) in &mut sliders {
        let f = knob_fraction(p, knob.0);
        if (slider.fraction - f).abs() > 1e-4 {
            slider.fraction = f;
        }
    }
    for (act, mut state) in &mut island {
        if *act == WgAct::Island && state.0 != p.island_mode {
            state.0 = p.island_mode;
        }
    }
    for (tag, mut state) in &mut tag_toggles {
        let on = p.tag_pool.contains(TagId(tag.0));
        if state.0 != on {
            state.0 = on;
        }
    }

    // Tag pool list: built once when the registry is available.
    if let (Some(handles), Some(tags), Ok(list)) =
        (res.handles.as_deref(), res.tags.as_deref(), tag_list.single())
    {
        if let Some(reg) = tags.get(&handles.tag_registry) {
            if *built_tags != reg.tags.len() {
                *built_tags = reg.tags.len();
                let kit = UiKit::new(&palette, mono.0.clone());
                commands.entity(list).despawn_related::<Children>();
                commands.entity(list).with_children(|col| {
                    for (i, tag) in reg.tags.iter().enumerate() {
                        spawn_toggle(
                            col,
                            &kit,
                            &tag.name,
                            p.tag_pool.contains(TagId(i as u16)),
                            WgTag(i as u16),
                        );
                    }
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------------------------

#[derive(bevy::ecs::system::SystemParam)]
struct FlowActRes<'w> {
    ui_state: ResMut<'w, WorldGenUiState>,
    preview_ui: ResMut<'w, WorldPreviewUiState>,
    params: Res<'w, WorldGenParams>,
    progress: Res<'w, WorldGenProgress>,
    job: Res<'w, WorldGenJobSlot>,
    flow: Res<'w, State<WorldGenFlowState>>,
    next_flow: ResMut<'w, NextState<WorldGenFlowState>>,
    next_base: ResMut<'w, NextState<crate::engine::BaseState>>,
    next_app: ResMut<'w, NextState<crate::engine::AppState>>,
    next_wg: ResMut<'w, NextState<WorldGenState>>,
    next_pause: ResMut<'w, NextState<crate::engine::PauseState>>,
    latch: ResMut<'w, crate::engine::WorldGenChromeLatch>,
    lifecycle: ResMut<'w, WorldPreviewLifecycle>,
    generate: MessageWriter<'w, GenerateWorldEvent>,
}

/// Generate / enter / discard / close.
fn world_gen_panel_flow_actions(
    mut commands: Commands,
    q: Query<(&Interaction, &UiButton, &WgAct), Changed<Interaction>>,
    mut res: FlowActRes,
    roots: Query<Entity, With<WorldMarker>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let is_busy = busy(&res.progress, &res.job);
    let flow = *res.flow.get();
    for (interaction, btn, act) in &q {
        if !ui_button_fired(interaction, btn) {
            continue;
        }
        match *act {
            WgAct::GeneratePreview if !is_busy && flow != WorldGenFlowState::FullReady => {
                if matches!(flow, WorldGenFlowState::Idle) {
                    NextState::set_if_neq(&mut res.next_flow, WorldGenFlowState::NewWorldSetup);
                }
                res.preview_ui.window_open = true;
                res.generate.write(GenerateWorldEvent {
                    params: (*res.params).clone(),
                    phase: WorldGenPhase::Preview,
                });
            }
            WgAct::GenerateFull if !is_busy && flow == WorldGenFlowState::PreviewReady => {
                res.preview_ui.window_open = true;
                res.generate.write(GenerateWorldEvent {
                    params: (*res.params).clone(),
                    phase: WorldGenPhase::Full,
                });
            }
            WgAct::EnterWorld if !is_busy && flow == WorldGenFlowState::FullReady => {
                let r = &mut res;
                crate::engine::ux_enter_world_from_world_gen(
                    &mut r.latch,
                    &mut r.next_app,
                    &mut r.next_wg,
                    &mut r.next_pause,
                    &mut r.next_base,
                    &mut r.ui_state,
                    &mut r.preview_ui,
                    &mut r.lifecycle,
                );
            }
            WgAct::EditMap if !is_busy && flow == WorldGenFlowState::FullReady => {
                NextState::set_if_neq(&mut res.next_base, crate::engine::BaseState::Editor);
                NextState::set_if_neq(&mut res.next_flow, WorldGenFlowState::Idle);
                res.ui_state.visible = false;
            }
            WgAct::Discard
                if !is_busy
                    && matches!(flow, WorldGenFlowState::PreviewReady | WorldGenFlowState::FullReady) =>
            {
                despawn_generated_world_entities(&mut commands, &roots);
                // Back to parameter setup with the sheet still open (no blank screen).
                NextState::set_if_neq(&mut res.next_flow, WorldGenFlowState::NewWorldSetup);
            }
            WgAct::CancelGen if is_busy => res.ui_state.confirm_close_pending = true,
            WgAct::Close => {
                if is_busy {
                    res.ui_state.confirm_close_pending = true;
                } else {
                    res.ui_state.generator_sheet_open = false;
                }
            }
            _ => {}
        }
    }
    // Esc closing the sheet is owned by the shell; Enter must never start a generation.
    let _ = keys;
}

/// Layer / zoom / follow / sheet toggle.
fn world_gen_panel_view_actions(
    q: Query<(&Interaction, &UiButton, &WgAct), Changed<Interaction>>,
    mut views: ResMut<crate::gui::MapViewInstances>,
    mut cam: ResMut<PreviewCameraState>,
    mut ui_state: ResMut<WorldGenUiState>,
    tex: Res<WorldPreviewTexture>,
) {
    let view = &mut views.world_preview;
    for (interaction, btn, act) in &q {
        if !ui_button_fired(interaction, btn) {
            continue;
        }
        match *act {
            WgAct::Layer(sel) => {
                match sel {
                    LayerSel::Base(i) => view.layers.replace_base(base_bits(i)),
                    LayerSel::Overlay(i) => view.layers.toggle(overlay_bit(i)),
                    LayerSel::Fire => view.overlays.fire_heat = !view.overlays.fire_heat,
                    LayerSel::Logistics => view.overlays.logistics_heat = !view.overlays.logistics_heat,
                    LayerSel::Construction => {
                        view.overlays.construction_heat = !view.overlays.construction_heat;
                    }
                    LayerSel::Ecology => view.overlays.ecology_heat = !view.overlays.ecology_heat,
                }
                view.bump_revision();
                ui_state.preview_layers = view.layers;
            }
            WgAct::Follow => {
                use crate::gui::MinimapFollowMode as M;
                view.follow_mode = match view.follow_mode {
                    M::Free => M::FollowCamera,
                    M::FollowCamera => M::FollowBookmark,
                    M::FollowBookmark => M::Free,
                };
                view.bump_revision();
            }
            WgAct::Gpu => {
                cam.mode = if cam.mode == PreviewRenderMode::GpuRenderTarget {
                    PreviewRenderMode::CpuRaster
                } else {
                    PreviewRenderMode::GpuRenderTarget
                };
            }
            WgAct::ZoomOut => {
                view.zoom = (view.zoom / 1.1).clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
                cam.zoom = view.zoom;
            }
            WgAct::ZoomIn => {
                view.zoom = (view.zoom * 1.1).clamp(PreviewLayers::ZOOM_MIN, PreviewLayers::ZOOM_MAX);
                cam.zoom = view.zoom;
            }
            WgAct::ZoomOne => {
                view.zoom = 1.0;
                cam.zoom = 1.0;
            }
            WgAct::ZoomFit => {
                view.reset_camera_for_map(tex.width as f32, tex.height as f32);
                view.zoom = 1.0;
                cam.zoom = 1.0;
                cam.center = view.camera_center;
            }
            WgAct::ParamsToggle => ui_state.generator_sheet_open = !ui_state.generator_sheet_open,
            _ => {}
        }
    }
}

#[derive(Default)]
struct HoldStep {
    held: Option<Entity>,
    timer: f32,
}

/// Parameters: steppers (hold to repeat), sliders, cycles, presets, tuning files, tags, sections.
fn world_gen_panel_param_actions(
    time: Res<Time>,
    acts: Query<(&Interaction, &UiButton, &WgAct), Changed<Interaction>>,
    knob_btns: Query<(Entity, &Interaction, &WgKnobBtn)>,
    sliders: Query<(&WgKnobSlider, &Interaction, &RelativeCursorPosition)>,
    tag_btns: Query<(&Interaction, &WgTag), Changed<Interaction>>,
    section_btns: Query<(&Interaction, &WgSectionToggle), Changed<Interaction>>,
    mut params: ResMut<WorldGenParams>,
    mut hint: ResMut<WorldGenTuningNote>,
    mut panel: ResMut<WorldGenPanelState>,
    mut ui_state: ResMut<WorldGenUiState>,
    handles: Option<Res<TerrainRegistriesHandles>>,
    mobility: Option<Res<Assets<crate::terrain::mobility::MobilityProfileRegistry>>>,
    mut hold: Local<HoldStep>,
) {
    // Steppers with hold-to-repeat.
    let pressed = knob_btns.iter().find(|(_, i, _)| **i == Interaction::Pressed);
    match pressed {
        Some((entity, _, btn)) => {
            if hold.held != Some(entity) {
                hold.held = Some(entity);
                hold.timer = -HOLD_DELAY_S;
                step_world_gen_knob(&mut params, btn.knob, btn.dir);
            } else {
                hold.timer += time.delta_secs();
                while hold.timer >= HOLD_REPEAT_S {
                    hold.timer -= HOLD_REPEAT_S;
                    step_world_gen_knob(&mut params, btn.knob, btn.dir);
                }
            }
        }
        None => hold.held = None,
    }
    // Slider drags.
    for (knob, interaction, cursor) in &sliders {
        if let Some(f) = crate::gui::style::widgets::ui_slider_drag(interaction, cursor) {
            knob_set_fraction(&mut params, knob.0, f);
        }
    }
    for (interaction, tag) in &tag_btns {
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
    for (interaction, sec) in &section_btns {
        if *interaction == Interaction::Pressed {
            panel.collapsed[sec.0] = !panel.collapsed[sec.0];
        }
    }
    for (interaction, btn, act) in &acts {
        if !ui_button_fired(interaction, btn) {
            continue;
        }
        match *act {
            WgAct::RandomSeed => params.seed = rand::random(),
            WgAct::PresetMedium => params.apply_tile_extent_preset(TileExtentPreset::MediumSmall),
            WgAct::PresetStandard => params.apply_tile_extent_preset(TileExtentPreset::Standard),
            WgAct::SyncLand => params.recompute_symbolic_land_features(),
            WgAct::RegionMethod => params.region_method = next_region(params.region_method),
            WgAct::HeightProfile => {
                params.height_noise_profile = next_profile(params.height_noise_profile);
            }
            WgAct::Island => params.island_mode = !params.island_mode,
            WgAct::ReloadTuning => {
                hint.0 = match load_overlay_prefer_ron(WORLD_GEN_TUNING_RON_PATH, WORLD_GEN_TUNING_JSON_PATH) {
                    Ok(Some(overlay)) => {
                        if let Some(n) = overlay.noise_sampling {
                            params.noise_sampling = n;
                        }
                        if let Some(b) = overlay.biome_tuning {
                            params.biome_tuning = b;
                        }
                        "Reloaded overlay.".to_string()
                    }
                    Ok(None) => "No tuning file - using in-memory parameters only.".to_string(),
                    Err(err) => format!("Reload error: {err}"),
                };
            }
            WgAct::SaveTuning => {
                let overlay = WorldGenTuningOverlay {
                    noise_sampling: Some(params.noise_sampling.clone()),
                    biome_tuning: Some(params.biome_tuning.clone()),
                };
                hint.0 = match save_overlay_ron(WORLD_GEN_TUNING_RON_PATH, &overlay) {
                    Ok(()) => "Saved RON overlay.".to_string(),
                    Err(err) => format!("Write error: {err}"),
                };
            }
            WgAct::ResetDefaults => {
                *params = WorldGenParams::default();
                hint.0 = "Restored built-in defaults (new random seed).".to_string();
            }
            WgAct::Mobility => {
                if let (Some(h), Some(a)) = (handles.as_deref(), mobility.as_deref()) {
                    if let Some(m) = a.get(&h.mobility_profiles) {
                        if !m.profiles.is_empty() {
                            ui_state.mobility_profile_index =
                                (ui_state.mobility_profile_index + 1) % m.profiles.len();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Register on the world-gen session. Order: shell spawn -> fill -> sync -> actions.
pub(crate) fn register_world_gen_panel(app: &mut App) {
    use crate::engine::AppState;
    app.init_resource::<WorldGenPanelState>().add_systems(
        Update,
        (
            fill_world_gen_panel,
            sync_world_gen_panel,
            world_gen_panel_flow_actions,
            world_gen_panel_view_actions,
            world_gen_panel_param_actions,
        )
            .chain()
            .run_if(in_state(AppState::WorldGen))
            .in_set(crate::gui::ViewRepresentationSystemSet::UiCollect),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::style::widgets::test_support::init_ok;

    #[test]
    fn panel_systems_have_disjoint_queries() {
        init_ok(fill_world_gen_panel);
        init_ok(sync_world_gen_panel);
        init_ok(world_gen_panel_flow_actions);
        init_ok(world_gen_panel_view_actions);
        init_ok(world_gen_panel_param_actions);
    }

    #[test]
    fn width_step_stays_inside_slider_range() {
        let mut params = WorldGenParams::default();
        params.width = 128;
        step_world_gen_knob(&mut params, WgKnob::Width, -1);
        assert_eq!(params.width, 128);
        step_world_gen_knob(&mut params, WgKnob::Width, 1);
        assert!(params.width > 128 && params.width <= MAX_WORLD_GEN_AXIS);
        params.width = MAX_WORLD_GEN_AXIS;
        step_world_gen_knob(&mut params, WgKnob::Width, 1);
        assert_eq!(params.width, MAX_WORLD_GEN_AXIS);
    }

    #[test]
    fn slider_fraction_round_trips_and_sizes_snap() {
        let mut p = WorldGenParams::default();
        knob_set_fraction(&mut p, WgKnob::Width, 0.5);
        assert_eq!(p.width % 32, 0);
        assert!(p.width >= 128 && p.width <= MAX_WORLD_GEN_AXIS);
        knob_set_fraction(&mut p, WgKnob::SeaLevel, 1.0);
        assert!((knob_get(&p, WgKnob::SeaLevel) - 0.55).abs() < 1e-4);
        assert!((knob_fraction(&p, WgKnob::SeaLevel) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn every_ranged_knob_has_label_and_nonempty_range() {
        for k in [
            WgKnob::Width,
            WgKnob::NumRegions,
            WgKnob::SnowPeak,
            WgKnob::Settlements,
            WgKnob::DetailPersistence,
        ] {
            let (lo, hi) = knob_bounds(k).expect("bounds");
            assert!(hi > lo);
            assert!(!knob_label(k).is_empty());
        }
        assert!(knob_bounds(WgKnob::WarpSeed).is_none());
    }
}
