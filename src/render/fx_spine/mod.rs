//! RENDER-DIR-RESTRUCTURE-v1 — fire/water VFX spine (mechanical move from `render/`).
//! Named `fx_spine` (not `fire_vfx`) to avoid collision with the pre-existing
//! [`crate::render::fire_vfx`] frontend module (emit/pack/witness — unrelated content).
//! `render/mod.rs` keeps path-preserving shim modules at the old `crate::render::fire_*` /
//! `crate::render::view_fire_projection` / `crate::render::fx_burst_request` /
//! `crate::render::hanabi_embellishment` / `crate::render::water_surface_visual` locations
//! so existing call sites keep resolving.

pub mod fire_chunk_entity_index;
pub mod fire_chunk_runtime;
pub mod fire_streaming;
pub mod fire_smoke_shader_handles;
pub mod fire7_f7_a_exit;
pub mod view_fire_projection;
pub mod fx_burst_request;
pub mod hanabi_embellishment;
pub mod water_surface_visual;
