//! RGR-P5-001 — GPU raster/draw pipeline modules (mechanical move from `render/`).
//! `render/mod.rs` keeps path-preserving shim modules at the old `crate::render::gpu_*` /
//! `crate::render::pipelines::terrain_instanced_draw` locations so existing call sites keep resolving.
//!
//! # RND-10 — pipeline state objects: batch by pipeline, specialize once
//!
//! Dual-cite: NEW `RND-10` (gea4v2 11.4.8.4 *Expensive Pipeline State Changes* p123,
//! 11.4.8.5 *Pipeline State Objects* p123-124) · LEGACY keyword `rendering_pipeline`
//! (gea_v2_e4 §11.4 p98; no dedicated `pso`/`pipeline_cache` keyword hit).
//!
//! Contract (Render sub-app, read-only w.r.t. sim):
//! - A PSO is compiled/queued **once**: every `PipelineCache::queue_render_pipeline` /
//!   `queue_compute_pipeline` call lives in a single `init_*pipeline*` system registered on
//!   `RenderStartup`, never in a `Render`-schedule prepare system or a draw/dispatch pass.
//! - Variants are enumerated up front (`[[CachedRenderPipelineId; 4]; 2]` = `[LDR/HDR][msaa 1,2,4,8]`);
//!   the per-frame pass only **selects** a cached id by index, so a state change costs a table
//!   lookup (and ideally one set-pipeline per batch), not a rebuild.
//! - Draw passes batch by pipeline: one instanced draw per pipeline variant (see RND-09 massing
//!   instances, `gpu_indirect_draw`), not one pipeline set per object.
//!
//! Tripwires: tests below scan this directory's sources (no GPU needed).

pub mod gpu_fire_particle_raster;
pub mod gpu_frustum_cull;
pub mod gpu_indirect_draw;
pub mod gpu_particle_draw;
pub mod gpu_tile_debug_draw;
pub mod gpu_water_particle_draw;
pub mod gpu_water_particle_raster;
pub mod gpu_water_surface_draw;
pub mod gpu_water_field_draw;
pub mod terrain_tile_instance;
pub mod terrain_instanced_draw;
pub mod terrain_gpu_bake_spike;
pub mod terrain_gpu_bake_rtt;
pub mod terrain_scorch_field;

// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: GPU passes, draw/raster/compute, overlay draws.
pub mod gpu_spark_compute;
pub mod gpu_instanced_quad;
pub mod gpu_tile_debug_buffer;
pub mod gpu_weather_fire_field;
pub mod overlay_field_buffers;
pub mod power_map_overlay_draw;
pub mod domain_overlay_gpu;
pub mod atmosphere_partial_gpu;
pub mod gpu_particles;
pub mod gpu_water_particles;
pub mod tactical_vector_overlay;
pub mod infrastructure_overlay;
pub mod light;
pub mod core2d_overlay_order;

#[cfg(test)]
mod rnd10_pso_tests {
    //! RND-10 tripwires: PSOs are queued once at `RenderStartup`, never per draw / per frame.
    use std::path::PathBuf;

    const QUEUE_CALLS: [&str; 2] = ["queue_render_pipeline(", "queue_compute_pipeline("];

    /// Non-test, non-comment source of a pipelines file.
    fn production_source(text: &str) -> String {
        let mut out = String::new();
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            // Cut at the trailing `#[cfg(test)] mod …` only (some files gate single items earlier).
            if t == "#[cfg(test)]"
                && lines.get(i + 1).is_some_and(|n| n.trim_start().starts_with("mod "))
            {
                break;
            }
            if t.starts_with("//") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }

    fn pipelines_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/render/pipelines")
    }

    /// `(file name, production source)` for every `.rs` file in this directory except `mod.rs`.
    fn sources() -> Vec<(String, String)> {
        let mut v = Vec::new();
        for entry in std::fs::read_dir(pipelines_dir()).expect("read pipelines dir") {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name == "mod.rs" {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("read source");
            v.push((name, production_source(&text)));
        }
        v.sort();
        v
    }

    /// Name of the fn enclosing byte offset `at` (nearest preceding `fn <ident>`).
    fn enclosing_fn(src: &str, at: usize) -> String {
        let head = &src[..at];
        let pos = head.rfind("fn ").expect("queue call must sit inside a fn");
        head[pos + 3..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }

    fn compact(src: &str) -> String {
        src.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// `(file, enclosing fn)` for every queue call site.
    fn queue_sites() -> Vec<(String, String)> {
        let mut sites = Vec::new();
        for (file, src) in sources() {
            for needle in QUEUE_CALLS {
                for (at, _) in src.match_indices(needle) {
                    sites.push((file.clone(), enclosing_fn(&src, at)));
                }
            }
        }
        sites
    }

    #[test]
    fn rnd10_queue_sites_are_found() {
        // Guard against the scan silently matching nothing (e.g. API rename).
        let sites = queue_sites();
        // Floor tracks current RenderStartup queue sites (scan-not-empty + no silent API rename).
        assert!(sites.len() >= 9, "expected >=9 PSO queue sites, got {}", sites.len());
    }

    #[test]
    fn rnd10_pso_queued_only_in_init_pipeline_system() {
        for (file, func) in queue_sites() {
            assert!(
                func.starts_with("init_") && func.contains("pipeline"),
                "{file}: PSO queued inside `{func}` — must be an `init_*pipeline*` RenderStartup system (RND-10: specialize once)"
            );
        }
    }

    #[test]
    fn rnd10_init_pipeline_system_runs_on_render_startup_only() {
        let all = sources();
        for (file, func) in queue_sites() {
            let (_, src) = all.iter().find(|(f, _)| *f == file).unwrap();
            let c = compact(src);
            assert!(
                c.contains(&format!("add_systems(RenderStartup,{func})")),
                "{file}: `{func}` not registered on RenderStartup"
            );
            // Defn + registration only: no per-frame schedule, no direct call from a pass.
            let refs = c.matches(func.as_str()).count();
            assert_eq!(
                refs, 2,
                "{file}: `{func}` referenced {refs}x (want 2: fn def + RenderStartup registration) — per-frame rebuild?"
            );
        }
    }

    #[test]
    fn rnd10_render_pso_variants_are_a_prebuilt_table() {
        // Raster files enumerate [LDR/HDR][msaa] variants at init; passes only index the table.
        for (file, src) in sources() {
            if !src.contains("queue_render_pipeline(") {
                continue;
            }
            let c = compact(&src);
            assert!(
                c.contains("pipelines:[[CachedRenderPipelineId;4];2]"),
                "{file}: render PSOs must be a prebuilt `[[CachedRenderPipelineId; 4]; 2]` variant table"
            );
            assert!(
                c.contains("pipelines[") || c.contains(".pipelines"),
                "{file}: pass must select a cached PSO by index"
            );
        }
    }

    #[test]
    fn rnd10_no_pso_creation_in_per_frame_passes() {
        // Raw wgpu bypass of PipelineCache would defeat specialize-once + async compile.
        for (file, src) in sources() {
            assert!(
                !src.contains("create_render_pipeline(") && !src.contains("create_compute_pipeline("),
                "{file}: direct device PSO creation bypasses PipelineCache (RND-10)"
            );
        }
    }
}
