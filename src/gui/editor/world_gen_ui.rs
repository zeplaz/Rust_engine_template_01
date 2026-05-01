use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::tuning_io::{load_overlay, WorldGenTuningOverlay};
use crate::terrain::generation::world_generator_enhanced::{
    GenerateWorldEvent, RegionMethod, TerrainNoiseProfile, WorldGenParams, WORLD_GEN_TUNING_JSON_PATH,
};
use crate::terrain::material::{TagId, TagRegistry};

#[derive(Resource)]
pub struct WorldGenUiState {
    pub visible: bool,
    pub preview_mode: PreviewMode,
    /// Last-selected tag for [`PreviewMode::Tag`].
    pub tag_pick: TagId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    None,
    Height,
    Moisture,
    Temperature,
    Biome,
    Regions,
    Tag(TagId),
}

impl Default for WorldGenUiState {
    fn default() -> Self {
        Self {
            visible: false,
            preview_mode: PreviewMode::None,
            tag_pick: TagId(0),
        }
    }
}

// Toggle event for the world gen UI
#[derive(Message)]
pub struct ToggleWorldGenUiEvent;

// UI System for world generation — EguiPrimaryContextPass, returns Result (bevy_egui 0.39).
pub fn world_gen_ui_system(
    mut contexts: EguiContexts,
    mut world_gen_params: ResMut<WorldGenParams>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    mut generate_event: MessageWriter<GenerateWorldEvent>,
    mut tuning_io_hint: Local<String>,
    handles: Res<TerrainRegistriesHandles>,
    tag_assets: Res<Assets<TagRegistry>>,
    #[cfg(feature = "bevy_tilemap_adapter")] mut tile_layer_vis: Option<
        ResMut<crate::render::tilemap_adapter::TilemapLayerVisibility>,
    >,
) -> Result {
    if !world_gen_ui_state.visible {
        return Ok(());
    }

    egui::Window::new("World Generator")
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.heading("World Generator");
            ui.add_space(10.0);
            
            ui.label(format!("Optional overlay: {}", WORLD_GEN_TUNING_JSON_PATH));
            ui.horizontal(|ui| {
                if ui.button("Reload tuning JSON").clicked() {
                    match load_overlay(WORLD_GEN_TUNING_JSON_PATH)
                    {
                        Ok(Some(o)) => {
                            if let Some(n) = o.noise_sampling {
                                world_gen_params.noise_sampling = n;
                            }
                            if let Some(b) = o.biome_tuning {
                                world_gen_params.biome_tuning = b;
                            }
                            *tuning_io_hint = "Reloaded overlay.".to_string();
                        }
                        Ok(None) => *tuning_io_hint = "No file — using in-memory params only.".to_string(),
                        Err(e) => *tuning_io_hint = format!("Reload error: {e}"),
                    }
                }
                if ui.button("Save tuning JSON").clicked() {
                    let overlay = WorldGenTuningOverlay {
                        noise_sampling: Some(world_gen_params.noise_sampling.clone()),
                        biome_tuning: Some(world_gen_params.biome_tuning.clone()),
                    };
                    match serde_json::to_string_pretty(&overlay) {
                        Ok(s) => match std::fs::write(WORLD_GEN_TUNING_JSON_PATH, s) {
                            Ok(()) => *tuning_io_hint = "Saved overlay.".to_string(),
                            Err(e) => *tuning_io_hint = format!("Write error: {e}"),
                        },
                        Err(e) => *tuning_io_hint = format!("Serialize error: {e}"),
                    }
                }
            });
            if !tuning_io_hint.is_empty() {
                ui.small(&*tuning_io_hint);
            }
            ui.add_space(10.0);
            
            egui::CollapsingHeader::new("General Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut world_gen_params.width, 128..=2048).text("Width"));
                    ui.add(egui::Slider::new(&mut world_gen_params.height, 128..=2048).text("Height"));
                    
                    ui.horizontal(|ui| {
                        if ui.button("Random Seed").clicked() {
                            world_gen_params.seed = rand::random();
                        }
                        ui.label(format!("Seed: {}", world_gen_params.seed));
                    });
                    
                    ui.add_space(5.0);
                });
            
            egui::CollapsingHeader::new("Region Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut world_gen_params.num_regions, 4..=64).text("Number of Regions"));
                    
                    ui.label("Region Method:");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Regular, "Regular Voronoi");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Manhattan, "Manhattan");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Weighted, "Weighted");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Centroidal, "Centroidal");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Circular, "Circular");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Power, "Power");
                    
                    if world_gen_params.region_method == RegionMethod::Centroidal {
                        ui.add(egui::Slider::new(&mut world_gen_params.region_iterations, 1..=10).text("Relaxation Iterations"));
                    }
                    
                    ui.add_space(5.0);
                });
            
            egui::CollapsingHeader::new("Terrain Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("Height noise profile:");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::FbmPerlin, "fBm · Perlin (baseline)");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::RidgedMulti, "Ridged (ranges / cliffs)");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::Billow, "Billow (soft landmasses)");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::HybridMulti, "Hybrid multifractal");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::FbmOpenSimplex, "fBm · OpenSimplex");

                    ui.add(egui::Slider::new(&mut world_gen_params.noise_scale, 0.01..=0.1).text("Noise Scale"));
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_octaves, 1..=8).text("Noise Octaves"));
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_lacunarity, 1.2..=3.5).text("Lacunarity"));
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_persistence, 0.2..=0.85).text("Persistence"));

                    egui::CollapsingHeader::new("Post shaping (naturalistic pass)")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label("Applied after the main fractal: curve land/ocean contrast, warp coasts, add small-scale detail.");
                            ui.add(egui::Slider::new(&mut world_gen_params.height_curve_exponent, 0.35..=2.5).text("Height curve (1 = linear)"));
                            ui.add(egui::Slider::new(&mut world_gen_params.domain_warp_strength, 0.0..=1.5).text("Domain warp strength"));
                            ui.add(egui::Slider::new(&mut world_gen_params.terrain_detail_mix, 0.0..=1.0).text("High-frequency detail mix"));
                        });

                    ui.add(egui::Slider::new(&mut world_gen_params.moisture_bias, -0.5..=0.5).text("Moisture Bias"));
                    ui.add(egui::Slider::new(&mut world_gen_params.temperature_bias, -0.5..=0.5).text("Temperature Bias"));

                    ui.add_space(5.0);
                });
            
            egui::CollapsingHeader::new("Noise channels (advanced)")
                .default_open(false)
                .show(ui, |ui| {
                    let ns = &mut world_gen_params.noise_sampling;
                    ui.add(egui::Slider::new(&mut ns.warp_noise_scale_mul, 0.05..=1.0).text("Warp noise · scale ×"));
                    ui.add(egui::Slider::new(&mut ns.warp_noise_octaves, 2..=12).text("Warp octaves"));
                    ui.horizontal(|ui| {
                        ui.label("Warp seed offset");
                        ui.add(egui::DragValue::new(&mut ns.warp_seed_offset).speed(1.0));
                    });
                    ui.add(egui::Slider::new(&mut ns.detail_noise_scale_mul, 0.5..=8.0).text("Detail noise · scale ×"));
                    ui.add(egui::Slider::new(&mut ns.detail_noise_octaves, 1..=8).text("Detail octaves"));
                    ui.horizontal(|ui| {
                        ui.label("Detail seed offset");
                        ui.add(egui::DragValue::new(&mut ns.detail_seed_offset).speed(1.0));
                    });
                    ui.add(egui::Slider::new(&mut ns.moisture_noise_scale_mul, 0.3..=3.0).text("Moisture fBm · scale ×"));
                    ui.add(egui::Slider::new(&mut ns.temperature_noise_scale_mul, 0.3..=3.0).text("Temperature fBm · scale ×"));
                    ui.add(egui::Slider::new(&mut ns.moisture_sample_freq_mul, 0.1..=4.0).text("Moisture sample freq ×"));
                    ui.add(egui::Slider::new(&mut ns.temperature_sample_freq_mul, 0.1..=4.0).text("Temperature sample freq ×"));
                    ui.add(egui::Slider::new(&mut ns.warp_coord_frequency_mul, 0.02..=0.25).text("Warp coord frequency"));
                    ui.add(egui::Slider::new(&mut ns.warp_coord_z, -4.0..=8.0).text("Warp coord Z"));
                    ui.add(egui::Slider::new(&mut ns.warp_phase_offset_x, -50.0..=50.0).text("Warp phase X"));
                    ui.add(egui::Slider::new(&mut ns.warp_phase_offset_y, -50.0..=50.0).text("Warp phase Y"));
                    ui.add(egui::Slider::new(&mut ns.warp_displacement_scale, 1.0..=40.0).text("Warp displacement (× strength)"));
                    ui.add(egui::Slider::new(&mut ns.detail_coord_frequency_mul, 1.0..=12.0).text("Detail coord frequency"));
                    ui.add(egui::Slider::new(&mut ns.detail_persistence_mul, 0.2..=1.0).text("Detail persistence ×"));
                });
            
            egui::CollapsingHeader::new("Biome generator coupling")
                .default_open(false)
                .show(ui, |ui| {
                    let b = &mut world_gen_params.biome_tuning;
                    ui.label("Thresholds and soft weights used by `classify_biome` on the same height/moisture/temp maps as world gen.");
                    ui.add(egui::Slider::new(&mut b.sea_level, 0.15..=0.55).text("Sea level (soft marine)"));
                    ui.add(egui::Slider::new(&mut b.deep_water_height_max, 0.05..=0.35).text("Deep water height max"));
                    ui.add(egui::Slider::new(&mut b.shallow_water_height_max, 0.2..=0.55).text("Shallow water height max"));
                    ui.add(egui::Slider::new(&mut b.beach_height_max, 0.28..=0.48).text("Beach height max"));
                    ui.add(egui::Slider::new(&mut b.mountain_height_min, 0.55..=0.92).text("Mountain height min"));
                    ui.add(egui::Slider::new(&mut b.grassland_moisture_max, 0.2..=0.6).text("Grassland moisture max"));
                    ui.add(egui::Slider::new(&mut b.desert_moisture_max, 0.1..=0.55).text("Desert moisture max"));
                    ui.add(egui::Slider::new(&mut b.swamp_moisture_min, 0.55..=0.95).text("Swamp moisture min"));
                    ui.add(egui::Slider::new(&mut b.forest_moisture_max, 0.45..=0.85).text("Forest moisture band max"));
                    ui.add(egui::Slider::new(&mut b.hot_lowlands_temperature_min, 0.45..=0.9).text("Hot lowlands temp min"));
                    ui.add(egui::Slider::new(&mut b.tundra_temperature_max, 0.1..=0.45).text("Tundra temp max"));
                    ui.add(egui::Slider::new(&mut b.snow_peak_temperature_max, 0.05..=0.35).text("Snow peak temp max"));
                });
            
            egui::CollapsingHeader::new("Feature Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut world_gen_params.river_count, 0..=10).text("Rivers"));
                    ui.add(egui::Slider::new(&mut world_gen_params.lake_count, 0..=5).text("Lakes"));
                    ui.add(egui::Slider::new(&mut world_gen_params.mountain_threshold, 0.5..=0.9).text("Mountain Threshold"));
                    ui.checkbox(&mut world_gen_params.island_mode, "Island Mode");
                    
                    if world_gen_params.island_mode {
                        ui.add(egui::Slider::new(&mut world_gen_params.island_falloff, 1.0..=5.0).text("Island Falloff"));
                    }
                    
                    ui.add_space(5.0);
                });
            
            ui.add_space(10.0);
            
            ui.heading("Preview Mode");
            ui.horizontal(|ui| {
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::None, "None");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Height, "Height");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Moisture, "Moisture");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Temperature, "Temperature");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Biome, "Biome");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Regions, "Regions");
                let tag_on = matches!(world_gen_ui_state.preview_mode, PreviewMode::Tag(_));
                if ui.selectable_label(tag_on, "Tag").clicked() {
                    world_gen_ui_state.preview_mode =
                        PreviewMode::Tag(world_gen_ui_state.tag_pick);
                }
            });
            if let Some(tag_reg) = tag_assets.get(&handles.tag_registry) {
                ui.horizontal(|ui| {
                    ui.label("Tag overlay:");
                    let sel_name = tag_reg
                        .tags
                        .get(world_gen_ui_state.tag_pick.0 as usize)
                        .map(|t| t.name.as_str())
                        .unwrap_or("—");
                    egui::ComboBox::from_id_salt("tag_overlay_pick")
                        .selected_text(sel_name)
                        .show_ui(ui, |ui| {
                            for (i, t) in tag_reg.tags.iter().enumerate() {
                                let id = TagId(i as u16);
                                if ui
                                    .selectable_value(
                                        &mut world_gen_ui_state.tag_pick,
                                        id,
                                        &t.name,
                                    )
                                    .changed()
                                    && matches!(world_gen_ui_state.preview_mode, PreviewMode::Tag(_))
                                {
                                    world_gen_ui_state.preview_mode = PreviewMode::Tag(id);
                                }
                            }
                        });
                });
            }
            
            #[cfg(feature = "bevy_tilemap_adapter")]
            if let Some(ref mut vis) = tile_layer_vis {
                egui::CollapsingHeader::new("Tilemap layers")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.checkbox(&mut vis.terrain, "Terrain (z=0)");
                        ui.checkbox(&mut vis.overlay, "Overlay / preview (z=10)");
                        ui.checkbox(&mut vis.resources, "Resources (z=20)");
                    });
            }
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("Generate World").clicked() {
                    generate_event.write(GenerateWorldEvent(world_gen_params.clone()));
                }
                
                if ui.button("Close").clicked() {
                    world_gen_ui_state.visible = false;
                }
            });
        });
    Ok(())
}

// System to toggle the world gen UI
pub fn toggle_world_gen_ui_system(
    mut events: MessageReader<ToggleWorldGenUiEvent>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
) {
    for _ in events.read() {
        world_gen_ui_state.visible = !world_gen_ui_state.visible;
    }
}

// Plugin to register all world generation UI components
pub struct WorldGenUiPlugin;

impl Plugin for WorldGenUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenUiState>()
           .add_message::<ToggleWorldGenUiEvent>()
           // Non-egui toggle logic stays in Update; UI rendering in EguiPrimaryContextPass
           .add_systems(Update, toggle_world_gen_ui_system)
           .add_systems(EguiPrimaryContextPass, world_gen_ui_system);
    }
}