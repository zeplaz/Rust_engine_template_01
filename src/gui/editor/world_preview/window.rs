//! WorldGen preview painting no longer acquires an egui context.
//!
//! The session shell is [`super::world_gen_bevy_shell`]. Raster, sampling, GPU
//! targets, and the preview image handle are unchanged.

#![allow(dead_code)]
