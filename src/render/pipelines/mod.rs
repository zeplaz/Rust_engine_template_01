//! RGR-P5-001 — GPU raster/draw pipeline modules (mechanical move from `render/`).
//! `render/mod.rs` keeps path-preserving shim modules at the old `crate::render::gpu_*` /
//! `crate::render::terrain_instanced_draw` locations so existing call sites keep resolving.

pub mod gpu_fire_particle_raster;
pub mod gpu_indirect_draw;
pub mod gpu_particle_draw;
pub mod gpu_tile_debug_draw;
pub mod gpu_water_particle_draw;
pub mod gpu_water_particle_raster;
pub mod gpu_water_surface_draw;
pub mod terrain_instanced_draw;

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
