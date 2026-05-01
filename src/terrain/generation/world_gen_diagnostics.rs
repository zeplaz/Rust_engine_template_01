//! JSON debug dumps for world generation iteration (timings, height stats, biome counts, hydrology).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bevy::prelude::Resource;
use serde::Serialize;

use super::hydrology::HydrologyResult;

/// Latest run metadata for UI (path to JSON + one-line summary).
#[derive(Resource, Default)]
pub struct WorldGenLastDebugReport {
    pub path: Option<PathBuf>,
    pub summary_one_line: String,
}

#[derive(Serialize)]
pub struct WorldGenDebugReport {
    pub schema_version: u32,
    pub unix_time_secs: u64,
    pub phase: String,
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub num_regions: u32,
    pub region_method: String,
    pub noise_scale: f32,
    pub noise_octaves: u32,
    pub height_noise_profile: String,
    pub island_mode: bool,
    pub island_falloff: f32,
    pub river_count: u32,
    pub lake_count: u32,
    pub timings_ms: TimingsMs,
    pub height_field: HeightFieldStats,
    pub biomes: HashMap<String, u64>,
    pub hydrology: Option<HydroSummary>,
    /// Downsampled height (0–1), stride-based sample across full grid.
    pub height_sample: HeightSampleGrid,
    pub notes_for_iteration: &'static str,
}

#[derive(Serialize)]
pub struct TimingsMs {
    pub regions: f64,
    pub tiling: f64,
    pub hydrology_compute: f64,
    pub total_wall: f64,
}

#[derive(Serialize)]
pub struct HeightFieldStats {
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub p10: f32,
    pub p90: f32,
}

#[derive(Serialize)]
pub struct HydroSummary {
    pub river_traces: usize,
    pub river_mask_cells: u64,
    pub lake_regions: usize,
    pub lake_mask_cells: u64,
    pub max_accumulation: f32,
}

#[derive(Serialize)]
pub struct HeightSampleGrid {
    pub stride: u32,
    pub width: u32,
    pub height: u32,
    /// Row-major: each row is one scanline in +x.
    pub values: Vec<Vec<f32>>,
}

pub struct WorldGenRunTiming {
    pub wall_start: Instant,
    pub regions_ms: f64,
    pub tiling_started: Instant,
    pub tiling_ms: f64,
    pub hydrology_compute_ms: f64,
}

impl WorldGenRunTiming {
    pub fn from_parts(wall_start: Instant, regions_ms: f64, tiling_started: Instant) -> Self {
        Self {
            wall_start,
            regions_ms,
            tiling_started,
            tiling_ms: 0.0,
            hydrology_compute_ms: 0.0,
        }
    }
}

const DEBUG_NOTES: &str = "Regions do not shape height (Voronoi is ECS grouping only). \
Blobby macro shapes → raise noise_scale, add domain_warp_strength / terrain_detail_mix, \
or switch height profile (e.g. Ridged). More biome variety → tune BiomeTuning bands or \
moisture/temperature sampling in Noise channels. Rivers → river_count > 0 and hydrology \
thresholds (HydrologyParams.river_acc_quantile) plus DEM variation; shallow river visuals \
use biome shallow_water_height_max.";

fn height_stats(grid: &[f32]) -> HeightFieldStats {
    if grid.is_empty() {
        return HeightFieldStats {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            p10: 0.0,
            p90: 0.0,
        };
    }
    let mut v = grid.to_vec();
    let n = v.len();
    let min = v.iter().copied().fold(f32::INFINITY, f32::min);
    let max = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mean = v.iter().sum::<f32>() / n as f32;
    v.sort_by(|a, b| a.total_cmp(b));
    let p10 = v[((n as f32 * 0.1).floor() as usize).min(n.saturating_sub(1))];
    let p90 = v[((n as f32 * 0.9).floor() as usize).min(n.saturating_sub(1))];
    HeightFieldStats {
        min,
        max,
        mean,
        p10,
        p90,
    }
}

fn downsample_grid(grid: &[f32], w: u32, h: u32, out_w: u32, out_h: u32) -> HeightSampleGrid {
    if w == 0 || h == 0 {
        return HeightSampleGrid {
            stride: 0,
            width: 0,
            height: 0,
            values: vec![],
        };
    }
    let ow = out_w.max(1);
    let oh = out_h.max(1);
    let sx = w as f32 / ow as f32;
    let sy = h as f32 / oh as f32;
    let mut rows = Vec::with_capacity(oh as usize);
    for oy in 0..oh {
        let mut row = Vec::with_capacity(ow as usize);
        let y = ((oy as f32 + 0.5) * sy).floor() as u32;
        let y = y.min(h.saturating_sub(1));
        for ox in 0..ow {
            let x = ((ox as f32 + 0.5) * sx).floor() as u32;
            let x = x.min(w.saturating_sub(1));
            let i = (y * w + x) as usize;
            row.push(grid.get(i).copied().unwrap_or(0.0));
        }
        rows.push(row);
    }
    let stride_guess_x = (w + ow - 1) / ow;
    HeightSampleGrid {
        stride: stride_guess_x,
        width: ow,
        height: oh,
        values: rows,
    }
}

fn summarize_hydro(h: &HydrologyResult) -> HydroSummary {
    let river_mask_cells = h.river_mask.iter().filter(|x| **x).count() as u64;
    let lake_mask_cells = h.lake_mask.iter().filter(|x| **x).count() as u64;
    let max_accumulation = h
        .accumulation
        .iter()
        .copied()
        .fold(0.0_f32, |a, b| a.max(b));
    HydroSummary {
        river_traces: h.rivers.len(),
        river_mask_cells,
        lake_regions: h.lakes.len(),
        lake_mask_cells,
        max_accumulation,
    }
}

/// Writes `debug_runs/world_gen_<unix>_<phase>.json` under the crate root.
pub fn write_world_gen_debug_report(
    phase: &str,
    seed: u64,
    num_regions: u32,
    region_method: &str,
    noise_scale: f32,
    noise_octaves: u32,
    height_noise_profile: &str,
    island_mode: bool,
    island_falloff: f32,
    river_count: u32,
    lake_count: u32,
    width: u32,
    height: u32,
    timing: &WorldGenRunTiming,
    height_grid: &[f32],
    biome_counts: &HashMap<String, u64>,
    hydro: Option<&HydrologyResult>,
) -> std::io::Result<(PathBuf, WorldGenDebugReport)> {
    let unix_time_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let total_wall = timing.wall_start.elapsed().as_secs_f64() * 1000.0;

    let report = WorldGenDebugReport {
        schema_version: 1,
        unix_time_secs,
        phase: phase.to_string(),
        width,
        height,
        seed,
        num_regions,
        region_method: region_method.to_string(),
        noise_scale,
        noise_octaves,
        height_noise_profile: height_noise_profile.to_string(),
        island_mode,
        island_falloff,
        river_count,
        lake_count,
        timings_ms: TimingsMs {
            regions: timing.regions_ms,
            tiling: timing.tiling_ms,
            hydrology_compute: timing.hydrology_compute_ms,
            total_wall,
        },
        height_field: height_stats(height_grid),
        biomes: biome_counts.clone(),
        hydrology: hydro.map(summarize_hydro),
        height_sample: downsample_grid(height_grid, width, height, 48, 48),
        notes_for_iteration: DEBUG_NOTES,
    };

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("debug_runs");
    std::fs::create_dir_all(&dir)?;
    let safe_phase: String = phase
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let name = format!("world_gen_{unix_time_secs}_{safe_phase}.json");
    let path = dir.join(name);
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(&path, json)?;
    Ok((path, report))
}

pub fn summary_line(path: &Path, report: &WorldGenDebugReport) -> String {
    format!(
        "{}×{} {} | total {:.0}ms (regions {:.0} | tiling {:.0} | hydro {:.0}) | {}",
        report.width,
        report.height,
        report.phase,
        report.timings_ms.total_wall,
        report.timings_ms.regions,
        report.timings_ms.tiling,
        report.timings_ms.hydrology_compute,
        path.display()
    )
}
