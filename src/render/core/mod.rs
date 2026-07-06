//! RENDER-DIR-RESTRUCTURE-v1 — registries, formats, GPU lifetime modules (mechanical move from `render/`).
//! `render/mod.rs` keeps path-preserving shim modules at the old `crate::render::gpu_*` /
//! `crate::render::terrain_*` / `crate::render::per_view_residency` locations so existing call sites
//! keep resolving.

pub mod gpu_buffer_registry;
pub mod gpu_bind_group_registry;
pub mod gpu_packed_formats;
pub mod gpu_surface_teardown;
pub mod gpu_representation_metrics;
pub mod terrain_material_atlas;
pub mod terrain_render_authority;
pub mod per_view_residency;
