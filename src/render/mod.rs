// Rendering systems
pub mod extraction;
pub mod lighting;
pub mod shaders;
mod tile_world_fallback;
mod visual_perf_budget;
mod fire_vfx;
mod weather_vfx;
mod particle_domains;
mod stage6_virtualization;
pub(crate) mod stage5_full_app_harness;
mod viewport_pipeline;
pub mod view_runtime;
mod mig_a_static;
pub mod minimap_compositor;

#[cfg(feature = "bevy_tilemap_adapter")]
pub mod tilemap_adapter;

// RGR-P5-001 — mechanical move: GPU raster/draw pipelines + render-thread perf probes.
pub(crate) mod pipelines;
pub(crate) mod probes;
// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: registries, formats, GPU lifetime.
pub(crate) mod core;
// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: fire/water VFX spine (renamed from plan's
// "fire_vfx" to "fx_spine" to avoid collision with the pre-existing render::fire_vfx frontend).
pub(crate) mod fx_spine;
// RENDER-DIR-RESTRUCTURE-v1 — mechanical move: proof/witness/CI-matrix surfaces.
pub(crate) mod witness;

// Path-preserving root shims removed (ES-8-3 / CLN-SHIM-API-BATCH):
// Callers + api.rs retargeted to pipelines:: / core:: / fx_spine:: / probes:: / witness:: / extraction::.

pub mod api;
pub use api::*;
