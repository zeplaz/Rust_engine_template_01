//! Tooltip copy for World Generator controls (noise / biomes / hydrology coupling).

pub const RELOAD_JSON: &str =
    "Load `assets/config/world_gen_tuning.json` if present and merge `noise_sampling` + `biome_tuning` into the live sliders.";
pub const SAVE_JSON: &str =
    "Write current noise + biome tuning to the JSON overlay path (does not save width/height/seed/rivers here).";
pub const COPY_DEBUG_PATH: &str = "Copy the last debug report path to the clipboard.";

pub const RESET_DEFAULTS: &str =
    "Restore all world-gen parameters to built-in defaults (including a new random seed and full tag pool).";

pub const WIDTH: &str =
    "World width in tiles for full generation. Preview pass clamps to a smaller max for speed.";
pub const HEIGHT: &str =
    "World height in tiles for full generation. Preview pass clamps to a smaller max for speed.";
pub const RANDOM_SEED: &str =
    "Pick a new random seed. Same seed + same params → same layout; change seed to reshuffle noise.";
pub const SEED_LABEL: &str =
    "RNG seed for Voronoi sites and all noise channels. Locked pairs with the field controls above.";

pub const NUM_REGIONS: &str =
    "How many Voronoi sites to scatter. Tiles attach to the nearest site for ECS hierarchy only; this does not sculpt height.";
pub const REGION_REGULAR: &str = "Standard Euclidean distance to sites — classic Voronoi cells.";
pub const REGION_MANHATTAN: &str = "City-block distance — more square / axis-aligned regions.";
pub const REGION_WEIGHTED: &str = "Sites use distance scaled by per-site weight (when supported by generator).";
pub const REGION_CENTROIDAL: &str = "Lloyd-style relaxation: smoother, more evenly sized cells after iterations.";
pub const REGION_CIRCULAR: &str = "Bias sites toward a circular arena layout (generator-specific).";
pub const REGION_POWER: &str = "Power diagram variant — different cell borders when weights differ.";
pub const REGION_ITERATIONS: &str =
    "Lloyd relaxation passes for Centroidal mode only. More iterations → rounder, more uniform regions.";

pub const PROFILE_FBMPERLIN: &str =
    "Smooth fractional Brownian motion on Perlin — good default continents and soft coasts.";
pub const PROFILE_RIDGED: &str =
    "Ridged multifractal — sharp ridges, ranges, and cliff-like structure.";
pub const PROFILE_BILLOW: &str =
    "Billowy noise — rounded, cloud-like landmasses with gentler valleys.";
pub const PROFILE_HYBRID: &str =
    "Hybrid multifractal — mix of smooth lows and sharper highs.";
pub const PROFILE_FBM_OS: &str =
    "fBm on OpenSimplex — less axis-aligned bias than plain Perlin.";

pub const NOISE_SCALE: &str =
    "Spatial frequency of height noise on tile indices. Higher → smaller landforms and more detail per map.";
pub const NOISE_OCTAVES: &str =
    "Number of stacked octaves. More octaves add small-scale detail but cost more sample work.";
pub const LACUNARITY: &str = "Frequency multiplier between octaves (>1). Higher → proportionally finer detail layers.";
pub const PERSISTENCE: &str =
    "Amplitude falloff per octave (0–1 typical). Lower → smoother, dominant low frequencies.";

pub const POST_SHAPING_SECTION: &str =
    "Runs after base fractal: contrast curve, coordinate warp, and high-frequency detail blend.";
pub const HEIGHT_CURVE: &str =
    "Exponent on normalized height before sea/falloff. >1 flattens lowlands; <1 lifts shallow areas.";
pub const DOMAIN_WARP: &str =
    "Warps (x,y) before sampling height noise. Adds natural coast folds; 0 disables warp.";
pub const DETAIL_MIX: &str =
    "Blend of extra high-frequency fBm into height. Breaks up uniform blobs without changing macro scale.";

pub const MOISTURE_BIAS: &str = "Additive offset on normalized moisture after fBm (shifts wet/dry bands).";
pub const TEMP_BIAS: &str = "Additive offset on normalized temperature after fBm (shifts hot/cold bands).";

pub const WARP_SCALE_MUL: &str =
    "Scales the domain-warp noise relative to world `noise_scale`. Higher → stronger warp at same strength.";
pub const WARP_OCTAVES: &str = "Octaves in the warp fBm — more octaves → finer wrinkle in the warp field.";
pub const WARP_SEED_OFFSET: &str =
    "Added to world seed for warp noise only — decorrelates warp from height without changing main seed.";
pub const DETAIL_SCALE_MUL: &str =
    "Scales the detail fBm versus world `noise_scale`; raises micro relief independent of octaves.";
pub const DETAIL_OCTAVES: &str = "Octaves for the detail overlay channel.";
pub const DETAIL_SEED_OFFSET: &str = "Seed tweak for detail noise — change to vary fine grain without touching macro height.";
pub const MOIST_FBMSCALE: &str = "Relative scale of the moisture fBm versus height `noise_scale`.";
pub const TEMP_FBMSCALE: &str = "Relative scale of the temperature fBm versus height `noise_scale`.";
pub const MOIST_FREQ_MUL: &str = "Extra frequency factor inside moisture noise sampling (finer/wetter streaks).";
pub const TEMP_FREQ_MUL: &str = "Extra frequency factor inside temperature noise sampling.";
pub const WARP_COORD_FREQ: &str = "Scales warp sampling coordinates — higher → tighter warp cells.";
pub const WARP_COORD_Z: &str = "Third coordinate into warp noise — slides the warp field without moving the map.";
pub const WARP_PHASE_X: &str = "Phase offset for warp noise along logical X.";
pub const WARP_PHASE_Y: &str = "Phase offset for warp noise along logical Y.";
pub const WARP_DISP_SCALE: &str =
    "Scales how far domain warp displaces world coordinates (pairs with Domain warp strength).";
pub const DETAIL_COORD_FREQ: &str = "Scales detail noise sampling — higher → finer speckle.";
pub const DETAIL_PERSIST_MUL: &str = "Multiplies persistence for the detail fBm only (amplitude per octave).";

pub const BIOME_SECTION: &str =
    "Soft weights + hard height thresholds for `classify_biome`. Matches material / tag passes when aligned.";
pub const SEA_LEVEL_SOFT: &str = "Center of the soft marine weight (normalized height).";
pub const DEEP_WATER_MAX: &str = "Max normalized height classified as deep water.";
pub const SHALLOW_MAX: &str = "Upper bound height for shallow water before beach/coastal classes.";
pub const BEACH_MAX: &str = "Upper bound for beach / strand versus inland soils.";
pub const MOUNTAIN_MIN: &str = "Minimum normalized height to treat as mountain / alpine classes.";
pub const GRASS_MOIST_MAX: &str = "Moisture ceiling for grassland band (versus forest/wetland).";
pub const DESERT_MOIST_MAX: &str = "Moisture below this favors arid classes when hot.";
pub const SWAMP_MOIST_MIN: &str = "Moisture above this can trigger swamp / wetland with suitable temperature.";
pub const FOREST_MOIST_MAX: &str = "Upper moisture edge of temperate forest band.";
pub const HOT_LOWLANDS_MIN: &str = "Temperature above this biases hot lowland / desert responses.";
pub const TUNDRA_TEMP_MAX: &str = "Cold band: below this temperature favors tundra / boreal edges.";
pub const SNOW_PEAK_TEMP_MAX: &str = "Very cold threshold — snow-capped peaks and alpine snow.";

pub const RIVER_COUNT: &str =
    "Target number of river traces from hydrology. 0 skips river routing; needs height variation and valid outlets.";
pub const LAKE_COUNT: &str = "Target number of lake regions from depression fill / masks.";
pub const MOUNTAIN_THRESHOLD_PARAM: &str =
    "Legacy height gate for mountain-related features in some passes (distinct from biome `mountain_height_min`).";
pub const ISLAND_MODE: &str =
    "If on, applies radial falloff so map edges sink toward water — good for standalone islands.";
pub const ISLAND_FALLOFF: &str =
    "Power on edge falloff (with island mode). Higher → steeper drop toward the map rim.";

pub const PREVIEW_NONE: &str = "No false-color overlay on the preview.";
pub const PREVIEW_HEIGHT: &str = "Color tiles by normalized height.";
pub const PREVIEW_MOIST: &str = "Color tiles by moisture channel.";
pub const PREVIEW_TEMP: &str = "Color tiles by temperature channel.";
pub const PREVIEW_BIOME: &str = "Color tiles by classified terrain class.";
pub const PREVIEW_REGIONS: &str = "Tint tiles by Voronoi region index.";
pub const PREVIEW_TAG: &str =
    "Highlight tiles whose material tags intersect the checked tag pool (below).";

pub const TAG_POOL_ENTRY: &str =
    "If checked, this tag may be written in procedural passes; Tag preview only draws tiles carrying these tags.";

pub const TILEMAP_TERRAIN: &str = "Show base terrain tile layer.";
pub const TILEMAP_OVERLAY: &str = "Show overlay / preview layer.";
pub const TILEMAP_RESOURCES: &str = "Show resource layer if present.";

pub const GENERATE_PREVIEW: &str =
    "Build a downscaled fast preview; required before full world generation in the current flow.";
pub const GENERATE_FULL: &str =
    "Spawn the full-size world using current sliders (allowed after a preview exists).";
pub const CLOSE_PANEL: &str = "Hide this window without changing flow state.";
pub const ENTER_WORLD: &str = "Leave the generator and enter simulation with the generated world.";
pub const OPEN_IN_MAP_EDITOR: &str =
    "Open the map editor with this generated world; world-gen flow returns to Idle (TEMP-EGUI shell).";
pub const DISCARD_WORLD: &str = "Despawn the procedural world entity tree and return flow to Idle.";
pub const DISCARD_PREVIEW: &str = "Despawn preview content and return flow to Idle.";

pub const DEBUG_SECTION: &str =
    "After each run, timing + height sample JSON is written under `debug_runs/` for tuning discussions.";
