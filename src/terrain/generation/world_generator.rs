//! Legacy world-generation sketch — **not wired**. Canonical path: `world_generator_enhanced` + `WorldGenToolsPlugin`.
//! Retained for migration trace per `terrain_biome_migration_matrix_v1.md`.

use bevy::prelude::*;

/// No-op plugin preserving module path; do not register in active graphs until ported.
pub struct LegacyWorldGeneratorPlugin;

impl Plugin for LegacyWorldGeneratorPlugin {
    fn build(&self, _app: &mut App) {}
}
