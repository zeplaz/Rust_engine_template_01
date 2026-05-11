use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::{BaseState, WorldGenFlowState};
use crate::gui::editor::world_preview::WorldPreviewUiState;
use crate::gui::std_floating;
use crate::gui::style::{
    framed_group, muted_label, path_hint, primary_label, section_heading, strong_body, v_space,
    weak_body, CmdHeadingStyle, UiPalette, UiSpacing, VertSpace,
};
use crate::terrain::generation::tuning_io::{load_overlay_prefer_ron, save_overlay_ron, WorldGenTuningOverlay};
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, GenerateWorldEvent, MAX_WORLD_GEN_AXIS, PREVIEW_WORLD_MAX_AXIS,
    RegionMethod, TerrainNoiseProfile, WorldGenParams, WorldGenPhase, WorldGenProgress, WorldMarker,
    WORLD_GEN_TUNING_JSON_PATH, WORLD_GEN_TUNING_RON_PATH,
};
use crate::terrain::generation::WorldGenLastDebugReport;

// Re-export for `tilemap_adapter` and other call sites.
pub use crate::gui::editor::world_preview::PreviewLayers;

use super::world_gen_hints as hints;

#[inline]
fn tt(response: egui::Response, text: &'static str) -> egui::Response {
    response.on_hover_text(text)
}

#[derive(Resource)]
pub struct WorldGenUiState {
    pub visible: bool,
    pub preview_layers: PreviewLayers,
    /// Index into [`MobilityProfileRegistry::profiles`](crate::terrain::mobility::MobilityProfileRegistry).
    pub mobility_profile_index: usize,
}

impl Default for WorldGenUiState {
    fn default() -> Self {
        Self {
            visible: false,
            preview_layers: PreviewLayers::default(),
            mobility_profile_index: 0,
        }
    }
}

// Toggle event for the world gen UI
#[derive(Message)]
pub struct ToggleWorldGenUiEvent;

// UI System for world generation — EguiPrimaryContextPass, returns Result (bevy_egui 0.39).
pub fn world_gen_ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut world_gen_params: ResMut<WorldGenParams>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    mut generate_event: MessageWriter<GenerateWorldEvent>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
    mut next_base: ResMut<NextState<BaseState>>,
    flow: Res<State<WorldGenFlowState>>,
    world_roots: Query<Entity, With<WorldMarker>>,
    progress: Res<WorldGenProgress>,
    last_debug: Res<WorldGenLastDebugReport>,
    mut tuning_io_hint: Local<String>,
    mut world_preview_ui: ResMut<WorldPreviewUiState>,
    palette: Res<UiPalette>,
    spacing: Res<UiSpacing>,
    #[cfg(feature = "bevy_tilemap_adapter")] mut tile_layer_vis: Option<
        ResMut<crate::render::tilemap_adapter::TilemapLayerVisibility>,
    >,
) -> Result {
    if !world_gen_ui_state.visible {
        return Ok(());
    }

    std_floating(egui::Window::new("World Generator"))
        .default_size(egui::vec2(520.0, 640.0))
        .collapsible(true)
        .show(contexts.ctx_mut()?, |ui| {
            let pal: &UiPalette = &*palette;
            let sp: &UiSpacing = &*spacing;
            section_heading(ui, pal, CmdHeadingStyle::Gt, "World Generator");
            weak_body(ui, pal, format!("Flow: {:?}", flow.get()));
            if progress.running {
                ui.add(egui::ProgressBar::new(progress.fraction).show_percentage());
                strong_body(ui, pal, progress.label.as_str());
                muted_label(
                    ui,
                    pal,
                    "Height field is sampled in parallel (CPU); tiles spawn in batches so the window keeps redrawing. Rivers/lakes remain the heaviest tail pass.",
                );
                v_space(ui, sp, VertSpace::Sm);
            }
            if matches!(*flow.get(), WorldGenFlowState::PreviewReady | WorldGenFlowState::FullReady)
            {
                muted_label(
                    ui,
                    pal,
                    "Review the preview / map, then run full generation, then confirm before entering simulation.",
                );
                ui.checkbox(&mut world_preview_ui.window_open, "Show World Preview window");
                muted_label(
                    ui,
                    pal,
                    "Preview layers and tag pool: use the World Preview panel (base combo + overlay toggles).",
                );
                v_space(ui, sp, VertSpace::Sm);
            } else if matches!(*flow.get(), WorldGenFlowState::NewWorldSetup) {
                muted_label(
                    ui,
                    pal,
                    "Adjust parameters, then generate a preview. Full world runs only after a preview exists.",
                );
            }
            v_space(ui, sp, VertSpace::Md);

            egui::ScrollArea::vertical()
                .id_salt("world_gen_main_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
            primary_label(
                ui,
                pal,
                format!(
                    "Optional overlay: {} (else {})",
                    WORLD_GEN_TUNING_RON_PATH, WORLD_GEN_TUNING_JSON_PATH
                ),
            );
            ui.horizontal(|ui| {
                if tt(ui.button("Reload tuning"), hints::RELOAD_TUNING).clicked() {
                    match load_overlay_prefer_ron(WORLD_GEN_TUNING_RON_PATH, WORLD_GEN_TUNING_JSON_PATH)
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
                if tt(ui.button("Save tuning (RON)"), hints::SAVE_TUNING_RON).clicked() {
                    let overlay = WorldGenTuningOverlay {
                        noise_sampling: Some(world_gen_params.noise_sampling.clone()),
                        biome_tuning: Some(world_gen_params.biome_tuning.clone()),
                    };
                    match save_overlay_ron(WORLD_GEN_TUNING_RON_PATH, &overlay) {
                        Ok(()) => *tuning_io_hint = "Saved RON overlay.".to_string(),
                        Err(e) => *tuning_io_hint = format!("Write error: {e}"),
                    }
                }
            });
            if !tuning_io_hint.is_empty() {
                muted_label(ui, pal, tuning_io_hint.as_str());
            }
            ui.horizontal(|ui| {
                if tt(
                    ui.button("Reset all parameters to defaults"),
                    hints::RESET_DEFAULTS,
                )
                .clicked()
                {
                    *world_gen_params = WorldGenParams::default();
                    *tuning_io_hint =
                        "Restored built-in defaults (new random seed).".to_string();
                }
            });
            v_space(ui, sp, VertSpace::Sm);

            egui::CollapsingHeader::new("Iteration / debug report")
                .default_open(false)
                .show(ui, |ui| {
                    muted_label(
                        ui,
                        pal,
                        "After each preview or full run, a JSON file is written under `debug_runs/` in the repo \
                         (timings, height stats, downsampled height sample, biome counts, hydrology summary). \
                         Share that file when describing “change X → looks like Y”.",
                    )
                    .on_hover_text(hints::DEBUG_SECTION);
                    v_space(ui, sp, VertSpace::Sm);
                    if last_debug.path.is_none() && last_debug.summary_one_line.is_empty() {
                        weak_body(ui, pal, "No report yet — generate a world once.");
                    } else {
                        muted_label(ui, pal, last_debug.summary_one_line.as_str());
                        if let Some(ref p) = last_debug.path {
                            ui.monospace(p.display().to_string());
                            ui.horizontal(|ui| {
                                if tt(ui.button("Copy path"), hints::COPY_DEBUG_PATH).clicked() {
                                    ui.ctx()
                                        .copy_text(p.to_string_lossy().to_string());
                                }
                            });
                        }
                    }
                });

            v_space(ui, sp, VertSpace::Md);
            
            egui::CollapsingHeader::new("General Settings")
                .default_open(true)
                .show(ui, |ui| {
                    framed_group(ui, pal, |ui| {
                        section_heading(ui, pal, CmdHeadingStyle::Gt, "Chunk Settings");
                        path_hint(ui, pal, "/assets/scenarios/test.ron");
                        v_space(ui, sp, VertSpace::Sm);
                        tt(
                            ui.add(
                                egui::Slider::new(&mut world_gen_params.width, 128..=MAX_WORLD_GEN_AXIS)
                                    .text("Width"),
                            ),
                            hints::WIDTH,
                        );
                        tt(
                            ui.add(
                                egui::Slider::new(&mut world_gen_params.height, 128..=MAX_WORLD_GEN_AXIS)
                                    .text("Height"),
                            ),
                            hints::HEIGHT,
                        );
                        muted_label(
                            ui,
                            pal,
                            format!(
                                "Preview pass uses a downscaled grid (max {} per side) for speed; full generation uses these sliders up to {}×{} tiles. Use World Preview zoom to inspect.",
                                PREVIEW_WORLD_MAX_AXIS, MAX_WORLD_GEN_AXIS, MAX_WORLD_GEN_AXIS
                            ),
                        );

                        ui.horizontal(|ui| {
                            if tt(ui.button("Random Seed"), hints::RANDOM_SEED).clicked() {
                                world_gen_params.seed = rand::random();
                            }
                            tt(
                                primary_label(ui, pal, format!("Seed: {}", world_gen_params.seed)),
                                hints::SEED_LABEL,
                            );
                        });

                        v_space(ui, sp, VertSpace::Xs);
                    });
                });
            
            egui::CollapsingHeader::new("Region Settings")
                .default_open(true)
                .show(ui, |ui| {
                    muted_label(
                        ui,
                        pal,
                        "Regions group tiles under Voronoi sites (strategic partitioning — see \
                         `prompts/guides/voronoi_polygon_worlds_notes.md.md`). \
                         They do not set height; optional **strategic semantics** below bias moisture/temperature slightly.",
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.num_regions, 4..=64).text("Number of Regions")),
                        hints::NUM_REGIONS,
                    );
                    
                    primary_label(ui, pal, "Region Method:");
                    tt(
                        ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Regular, "Regular Voronoi"),
                        hints::REGION_REGULAR,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Manhattan, "Manhattan"),
                        hints::REGION_MANHATTAN,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Weighted, "Weighted"),
                        hints::REGION_WEIGHTED,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Centroidal, "Centroidal"),
                        hints::REGION_CENTROIDAL,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Circular, "Circular"),
                        hints::REGION_CIRCULAR,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Power, "Power"),
                        hints::REGION_POWER,
                    );
                    
                    if world_gen_params.region_method == RegionMethod::Centroidal {
                        tt(
                            ui.add(
                                egui::Slider::new(&mut world_gen_params.region_iterations, 1..=10)
                                    .text("Relaxation Iterations"),
                            ),
                            hints::REGION_ITERATIONS,
                        );
                    }

                    muted_label(
                        ui,
                        pal,
                        "Climate nudge from Voronoi sites: see **Climate variation** section below.",
                    )
                    .on_hover_text(hints::STRATEGIC_FIELD_COUPLING);

                    v_space(ui, sp, VertSpace::Xs);
                });

            egui::CollapsingHeader::new("Climate variation & pseudo–weather bands")
                .default_open(true)
                .show(ui, |ui| {
                    section_heading(ui, pal, CmdHeadingStyle::None, "Moisture / temperature fields");
                    muted_label(ui, pal, hints::CLIMATE_VARIATION_SECTION);
                    muted_label(ui, pal, hints::CLIMATE_QUICK_TIP).on_hover_text(
                        "Operational fronts & pressure systems live in the weather ECS later; world-gen noise here shapes believable *static* climate diversity.",
                    );
                    v_space(ui, sp, VertSpace::Sm);
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.moisture_bias, -0.5..=0.5).text("Moisture bias (global wet/dry shift)")),
                        hints::MOISTURE_BIAS,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.temperature_bias, -0.5..=0.5).text("Temperature bias (global hot/cold shift)")),
                        hints::TEMP_BIAS,
                    );
                    v_space(ui, sp, VertSpace::Xs);
                    primary_label(ui, pal, "Patch size & streakiness (pass-1 fBm):");
                    let ns = &mut world_gen_params.noise_sampling;
                    tt(
                        ui.add(egui::Slider::new(&mut ns.moisture_noise_scale_mul, 0.3..=3.0).text("Moisture · patch scale ×")),
                        hints::MOIST_FBMSCALE,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.temperature_noise_scale_mul, 0.3..=3.0).text("Temperature · patch scale ×")),
                        hints::TEMP_FBMSCALE,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.moisture_sample_freq_mul, 0.1..=4.0).text("Moisture · fine detail / fronts freq ×")),
                        hints::MOIST_FREQ_MUL,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.temperature_sample_freq_mul, 0.1..=4.0).text("Temperature · fine detail / fronts freq ×")),
                        hints::TEMP_FREQ_MUL,
                    );
                    v_space(ui, sp, VertSpace::Sm);
                    egui::CollapsingHeader::new("Voronoi → climate provinces")
                        .default_open(false)
                        .show(ui, |ui| {
                        muted_label(
                            ui,
                            pal,
                            "Region sites nudge moisture/temperature before biomes — continent-scale *zones*.",
                        );
                        tt(
                            ui.add(
                                egui::Slider::new(&mut world_gen_params.strategic_field_coupling, 0.0..=0.35)
                                    .text("Strategic field blend (regions → moisture/temp)"),
                            ),
                            hints::STRATEGIC_FIELD_COUPLING,
                        );
                    });
                    egui::CollapsingHeader::new("Rim falloff (islands)")
                        .default_open(false)
                        .show(ui, |ui| {
                        muted_label(
                            ui,
                            pal,
                            "Use **Feature Settings** → Island mode + falloff for edge sink toward water.",
                        );
                    });
                });
            
            egui::CollapsingHeader::new("Terrain Settings (height)")
                .default_open(true)
                .show(ui, |ui| {
                    primary_label(ui, pal, "Height noise profile:");
                    tt(
                        ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::FbmPerlin, "fBm · Perlin (baseline)"),
                        hints::PROFILE_FBMPERLIN,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::RidgedMulti, "Ridged (ranges / cliffs)"),
                        hints::PROFILE_RIDGED,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::Billow, "Billow (soft landmasses)"),
                        hints::PROFILE_BILLOW,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::HybridMulti, "Hybrid multifractal"),
                        hints::PROFILE_HYBRID,
                    );
                    tt(
                        ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::FbmOpenSimplex, "fBm · OpenSimplex"),
                        hints::PROFILE_FBM_OS,
                    );

                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.noise_scale, 0.01..=0.1).text("Noise Scale")),
                        hints::NOISE_SCALE,
                    );
                    muted_label(
                        ui,
                        pal,
                        "Higher scale = more hills/coast detail at the same map size (less one big smooth mass). \
                         Try ~0.04–0.07 on 512² if the default feels too blobby.",
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.noise_octaves, 1..=8).text("Noise Octaves")),
                        hints::NOISE_OCTAVES,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.noise_lacunarity, 1.2..=3.5).text("Lacunarity")),
                        hints::LACUNARITY,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.noise_persistence, 0.2..=0.85).text("Persistence")),
                        hints::PERSISTENCE,
                    );

                    egui::CollapsingHeader::new("Post shaping (naturalistic pass)")
                        .default_open(false)
                        .show(ui, |ui| {
                            primary_label(
                                ui,
                                pal,
                                "Applied after the main fractal: curve land/ocean contrast, warp coasts, add small-scale detail.",
                            )
                            .on_hover_text(hints::POST_SHAPING_SECTION);
                            tt(
                                ui.add(egui::Slider::new(&mut world_gen_params.height_curve_exponent, 0.35..=2.5).text("Height curve (1 = linear)")),
                                hints::HEIGHT_CURVE,
                            );
                            tt(
                                ui.add(egui::Slider::new(&mut world_gen_params.domain_warp_strength, 0.0..=1.5).text("Domain warp strength")),
                                hints::DOMAIN_WARP,
                            );
                            tt(
                                ui.add(egui::Slider::new(&mut world_gen_params.terrain_detail_mix, 0.0..=1.0).text("High-frequency detail mix")),
                                hints::DETAIL_MIX,
                            );
                        });

                    weak_body(
                        ui,
                        pal,
                        "Moisture / temperature biases and fBm knobs live under **Climate variation** — this block is **elevation** only.",
                    );

                    v_space(ui, sp, VertSpace::Xs);
                });
            
            egui::CollapsingHeader::new("Noise channels (warp, detail, coords)")
                .default_open(false)
                .show(ui, |ui| {
                    weak_body(
                        ui,
                        pal,
                        "Moisture/temperature **scale & freq** are in **Climate variation**. Here: domain warp, detail fBm, and coordinate transforms.",
                    );
                    v_space(ui, sp, VertSpace::Xs);
                    let ns = &mut world_gen_params.noise_sampling;
                    tt(
                        ui.add(egui::Slider::new(&mut ns.warp_noise_scale_mul, 0.05..=1.0).text("Warp noise · scale ×")),
                        hints::WARP_SCALE_MUL,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.warp_noise_octaves, 2..=12).text("Warp octaves")),
                        hints::WARP_OCTAVES,
                    );
                    ui.horizontal(|ui| {
                        primary_label(ui, pal, "Warp seed offset");
                        tt(
                            ui.add(egui::DragValue::new(&mut ns.warp_seed_offset).speed(1.0)),
                            hints::WARP_SEED_OFFSET,
                        );
                    });
                    tt(
                        ui.add(egui::Slider::new(&mut ns.detail_noise_scale_mul, 0.5..=8.0).text("Detail noise · scale ×")),
                        hints::DETAIL_SCALE_MUL,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.detail_noise_octaves, 1..=8).text("Detail octaves")),
                        hints::DETAIL_OCTAVES,
                    );
                    ui.horizontal(|ui| {
                        primary_label(ui, pal, "Detail seed offset");
                        tt(
                            ui.add(egui::DragValue::new(&mut ns.detail_seed_offset).speed(1.0)),
                            hints::DETAIL_SEED_OFFSET,
                        );
                    });
                    tt(
                        ui.add(egui::Slider::new(&mut ns.warp_coord_frequency_mul, 0.02..=0.25).text("Warp coord frequency")),
                        hints::WARP_COORD_FREQ,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.warp_coord_z, -4.0..=8.0).text("Warp coord Z")),
                        hints::WARP_COORD_Z,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.warp_phase_offset_x, -50.0..=50.0).text("Warp phase X")),
                        hints::WARP_PHASE_X,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.warp_phase_offset_y, -50.0..=50.0).text("Warp phase Y")),
                        hints::WARP_PHASE_Y,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.warp_displacement_scale, 1.0..=40.0).text("Warp displacement (× strength)")),
                        hints::WARP_DISP_SCALE,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.detail_coord_frequency_mul, 1.0..=12.0).text("Detail coord frequency")),
                        hints::DETAIL_COORD_FREQ,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut ns.detail_persistence_mul, 0.2..=1.0).text("Detail persistence ×")),
                        hints::DETAIL_PERSIST_MUL,
                    );
                });
            
            egui::CollapsingHeader::new("Biomes & climate zones (thresholds)")
                .default_open(true)
                .show(ui, |ui| {
                    let b = &mut world_gen_params.biome_tuning;
                    muted_label(ui, pal, "Where pass-1 **height / moisture / temperature** become named biomes. Tighter temp cuts → sharper polar/tropical contrast; moisture bands control wet belts vs arid interiors.")
                        .on_hover_text(hints::BIOME_SECTION);
                    tt(
                        ui.add(egui::Slider::new(&mut b.sea_level, 0.15..=0.55).text("Sea level (soft marine)")),
                        hints::SEA_LEVEL_SOFT,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.deep_water_height_max, 0.05..=0.35).text("Deep water height max")),
                        hints::DEEP_WATER_MAX,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.shallow_water_height_max, 0.2..=0.55).text("Shallow water height max")),
                        hints::SHALLOW_MAX,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.beach_height_max, 0.28..=0.48).text("Beach height max")),
                        hints::BEACH_MAX,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.mountain_height_min, 0.55..=0.92).text("Mountain height min")),
                        hints::MOUNTAIN_MIN,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.grassland_moisture_max, 0.2..=0.6).text("Grassland moisture max")),
                        hints::GRASS_MOIST_MAX,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.desert_moisture_max, 0.1..=0.55).text("Desert moisture max")),
                        hints::DESERT_MOIST_MAX,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.swamp_moisture_min, 0.55..=0.95).text("Swamp moisture min")),
                        hints::SWAMP_MOIST_MIN,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.forest_moisture_max, 0.45..=0.85).text("Forest moisture band max")),
                        hints::FOREST_MOIST_MAX,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.hot_lowlands_temperature_min, 0.45..=0.9).text("Hot lowlands temp min")),
                        hints::HOT_LOWLANDS_MIN,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.tundra_temperature_max, 0.1..=0.45).text("Tundra temp max")),
                        hints::TUNDRA_TEMP_MAX,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut b.snow_peak_temperature_max, 0.05..=0.35).text("Snow peak temp max")),
                        hints::SNOW_PEAK_TEMP_MAX,
                    );
                });
            
            egui::CollapsingHeader::new("Feature Settings")
                .default_open(true)
                .show(ui, |ui| {
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.river_count, 0..=10).text("Rivers")),
                        hints::RIVER_COUNT,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.lake_count, 0..=5).text("Lakes")),
                        hints::LAKE_COUNT,
                    );
                    tt(
                        ui.add(egui::Slider::new(&mut world_gen_params.mountain_threshold, 0.5..=0.9).text("Mountain Threshold")),
                        hints::MOUNTAIN_THRESHOLD_PARAM,
                    );
                    tt(
                        ui.checkbox(&mut world_gen_params.island_mode, "Island Mode"),
                        hints::ISLAND_MODE,
                    );
                    
                    if world_gen_params.island_mode {
                        tt(
                            ui.add(egui::Slider::new(&mut world_gen_params.island_falloff, 1.0..=5.0).text("Island Falloff")),
                            hints::ISLAND_FALLOFF,
                        );
                    }
                    
                    v_space(ui, sp, VertSpace::Xs);
                });
            
            #[cfg(feature = "bevy_tilemap_adapter")]
            if let Some(ref mut vis) = tile_layer_vis {
                egui::CollapsingHeader::new("Tilemap layers")
                    .default_open(true)
                    .show(ui, |ui| {
                        tt(ui.checkbox(&mut vis.terrain, "Terrain (z=0)"), hints::TILEMAP_TERRAIN);
                        tt(ui.checkbox(&mut vis.overlay, "Overlay / preview (z=10)"), hints::TILEMAP_OVERLAY);
                        tt(ui.checkbox(&mut vis.resources, "Resources (z=20)"), hints::TILEMAP_RESOURCES);
                    });
            }
            
            v_space(ui, sp, VertSpace::Lg);
            v_space(ui, sp, VertSpace::Sm);

            let busy = progress.running;

            if matches!(*flow.get(), WorldGenFlowState::FullReady) {
                ui.horizontal(|ui| {
                    if tt(
                        ui.add_enabled(!busy, egui::Button::new("Enter world")),
                        hints::ENTER_WORLD,
                    )
                    .clicked()
                    {
                        NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
                        NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                        world_gen_ui_state.visible = false;
                    }
                    if tt(
                        ui.add_enabled(!busy, egui::Button::new("Edit generated world in map editor")),
                        hints::EDIT_GENERATED_IN_MAP_EDITOR,
                    )
                    .clicked()
                    {
                        NextState::set_if_neq(&mut *next_base, BaseState::Editor);
                        NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                        world_gen_ui_state.visible = false;
                    }
                    if tt(
                        ui.add_enabled(!busy, egui::Button::new("Discard generated world")),
                        hints::DISCARD_WORLD,
                    )
                    .clicked()
                    {
                        despawn_generated_world_entities(&mut commands, &world_roots);
                        NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                        world_gen_ui_state.visible = false;
                    }
                });
                v_space(ui, sp, VertSpace::Md);
            } else if matches!(*flow.get(), WorldGenFlowState::PreviewReady) {
                if tt(
                    ui.add_enabled(!busy, egui::Button::new("Discard preview")),
                    hints::DISCARD_PREVIEW,
                )
                .clicked()
                {
                    despawn_generated_world_entities(&mut commands, &world_roots);
                    NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                    world_gen_ui_state.visible = false;
                }
                v_space(ui, sp, VertSpace::Md);
            }

            ui.horizontal(|ui| {
                if tt(
                    ui.add_enabled(!busy, egui::Button::new("Generate preview")),
                    hints::GENERATE_PREVIEW,
                )
                .clicked()
                {
                    generate_event.write(GenerateWorldEvent {
                        params: world_gen_params.clone(),
                        phase: WorldGenPhase::Preview,
                    });
                }
                let can_full =
                    matches!(*flow.get(), WorldGenFlowState::PreviewReady) && !busy;
                let full_btn = tt(
                    ui.add_enabled(can_full, egui::Button::new("Generate full world")),
                    hints::GENERATE_FULL,
                );
                if full_btn.clicked() {
                    generate_event.write(GenerateWorldEvent {
                        params: world_gen_params.clone(),
                        phase: WorldGenPhase::Full,
                    });
                }

                if tt(ui.button("Close panel"), hints::CLOSE_PANEL).clicked() {
                    world_gen_ui_state.visible = false;
                }
            });
                }); // world_gen_main_scroll
        });
    Ok(())
}

// System to toggle the world gen UI
pub fn toggle_world_gen_ui_system(
    mut events: MessageReader<ToggleWorldGenUiEvent>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    flow: Res<State<WorldGenFlowState>>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
) {
    for _ in events.read() {
        let will_show = !world_gen_ui_state.visible;
        if will_show && matches!(*flow.get(), WorldGenFlowState::Idle) {
            #[cfg(not(debug_assertions))]
            {
                warn!("Use Main Menu → New World to open the world generator.");
                continue;
            }
            #[cfg(debug_assertions)]
            NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::NewWorldSetup);
        }
        world_gen_ui_state.visible = !world_gen_ui_state.visible;
    }
}

fn open_world_gen_panel_on_flow_setup(mut world_gen_ui_state: ResMut<WorldGenUiState>) {
    world_gen_ui_state.visible = true;
}

// Plugin to register all world generation UI components
pub struct WorldGenUiPlugin;

impl Plugin for WorldGenUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenUiState>()
            .add_message::<ToggleWorldGenUiEvent>()
            .add_systems(
                OnEnter(WorldGenFlowState::NewWorldSetup),
                open_world_gen_panel_on_flow_setup,
            )
            // Non-egui toggle logic stays in Update; UI rendering in EguiPrimaryContextPass
            .add_systems(Update, toggle_world_gen_ui_system)
            .add_systems(EguiPrimaryContextPass, world_gen_ui_system);
    }
}