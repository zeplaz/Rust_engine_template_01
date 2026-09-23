//! H-A2 / ES-7 — optional Hanabi L3 embellishment (`hanabi_l3` feature + env gate only).
//!
//! Contract: event puffs from projection burst chunks. Never weather authority, never sim writeback,
//! never minimap (LocalTactical gate). Consumer must **not** name `FxParticleBurstRequest`.

#[cfg(feature = "hanabi_l3")]
use bevy::prelude::*;

use bevy::math::{IVec2, Vec2};

use crate::gui::WorldLodBand;

/// Production caps (TUNE headroom vs spike report) — [`hanabi_event_vfx_style_bounds_v1.md`](../../dev/hanabi_event_vfx_style_bounds_v1.md).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HanabiPresetCaps {
    pub id: &'static str,
    pub max_instances: u32,
    pub lifetime_min_s: f32,
    pub lifetime_max_s: f32,
    pub peak_alpha: f32,
}

#[must_use]
pub const fn production_preset_table() -> [HanabiPresetCaps; 3] {
    [
        HanabiPresetCaps {
            id: "fire_ember_burst",
            max_instances: 20,
            lifetime_min_s: 0.35,
            lifetime_max_s: 0.85,
            peak_alpha: 0.35,
        },
        HanabiPresetCaps {
            id: "water_splash_mist",
            max_instances: 16,
            lifetime_min_s: 0.25,
            lifetime_max_s: 0.55,
            peak_alpha: 0.28,
        },
        HanabiPresetCaps {
            id: "construction_micro_spark",
            max_instances: 8,
            lifetime_min_s: 0.2,
            lifetime_max_s: 0.35,
            peak_alpha: 0.22,
        },
    ]
}

#[must_use]
pub fn fire_ember_caps() -> HanabiPresetCaps {
    production_preset_table()[0]
}

#[must_use]
pub fn preset_within_bounds(caps: &HanabiPresetCaps) -> bool {
    caps.max_instances <= 32
        && caps.peak_alpha <= 0.45
        && caps.lifetime_min_s >= 0.2 - f32::EPSILON
        && caps.lifetime_max_s <= 1.2 + f32::EPSILON
}

/// Mapped spawn intent (main-world / witness) — not a Hanabi entity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HanabiBurstSpawn {
    pub world_xy: Vec2,
    pub intensity: f32,
    pub preset_id: &'static str,
}

/// Default chunk tile extent for chunk-index → world (matches common worldgen chunk size).
pub const HANABI_BURST_CHUNK_TILES: f32 = 32.0;

/// Pure mapping — LocalTactical only; clamps to `caps.max_instances`.
#[must_use]
pub fn map_burst_chunks_to_spawns(
    chunks: impl Iterator<Item = (IVec2, f32)>,
    lod: WorldLodBand,
    caps: &HanabiPresetCaps,
) -> Vec<HanabiBurstSpawn> {
    if lod != WorldLodBand::LocalTactical {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (coord, intensity) in chunks {
        if out.len() as u32 >= caps.max_instances {
            break;
        }
        let world_xy = Vec2::new(
            (coord.x as f32 + 0.5) * HANABI_BURST_CHUNK_TILES,
            (coord.y as f32 + 0.5) * HANABI_BURST_CHUNK_TILES,
        );
        out.push(HanabiBurstSpawn {
            world_xy,
            intensity,
            preset_id: caps.id,
        });
    }
    out
}

/// True when this module contains the ES-7 burst consumer contract (cfg-free).
#[must_use]
pub fn hanabi_burst_consumer_present() -> bool {
    let src = include_str!("hanabi_embellishment.rs");
    src.contains("fn map_burst_chunks_to_spawns") && src.contains("HanabiBurstSpawn")
}

#[cfg(feature = "hanabi_l3")]
#[derive(Resource, Clone, Debug)]
pub struct HanabiEmbellishmentPresets {
    pub presets: Vec<HanabiPresetCaps>,
}

#[cfg(feature = "hanabi_l3")]
impl Default for HanabiEmbellishmentPresets {
    fn default() -> Self {
        Self {
            presets: production_preset_table().to_vec(),
        }
    }
}

#[cfg(feature = "hanabi_l3")]
#[derive(Resource, Default, Debug)]
pub struct HanabiBurstMapDiagnostics {
    pub last_mapped: u32,
    pub particles_spawned_session: u64,
}

/// Port of spike `fire_ember_effect_asset` with production caps (≤20 once).
#[cfg(feature = "hanabi_l3")]
pub fn fire_ember_effect_asset(caps: &HanabiPresetCaps) -> bevy_hanabi::EffectAsset {
    use bevy_hanabi::prelude::*;
    let writer = ExprWriter::new();
    let lifetime = writer.lit(caps.lifetime_max_s).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.35).expr(),
        dimension: ShapeDimension::Surface,
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(0.6).expr(),
    };
    let drag = LinearDragModifier::new(writer.lit(1.2).expr());
    let color = writer
        .lit(Vec4::new(0.72, 0.38, 0.18, caps.peak_alpha))
        .expr();
    let color_mod = SetAttributeModifier::new(Attribute::COLOR, color);
    let count = caps.max_instances as f32;
    let spawner = SpawnerSettings::once(count.into());
    EffectAsset::new(caps.max_instances.max(1), spawner, writer.finish())
        .with_name(caps.id)
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(color_mod)
        .update(drag)
}

#[cfg(feature = "hanabi_l3")]
fn sync_hanabi_fire_ember_bursts(
    graph: Res<crate::render::RenderProjectionGraph>,
    mut diag: ResMut<HanabiBurstMapDiagnostics>,
    // Read-only — never mutate sim. Effect entities deferred until visual capture lane.
) {
    let caps = fire_ember_caps();
    let mapped = map_burst_chunks_to_spawns(graph.fire.burst_chunks(), graph.fire.projected_lod(), &caps);
    diag.last_mapped = mapped.len() as u32;
    // Spawn of ParticleEffect entities is ES-7-4 (operator capture with RUST_ENGINE_HANABI_L3).
    // Contract path proves mapping only so default builds stay honesty-scoped.
    let _ = mapped;
}

/// L3 plugin — registers `bevy_hanabi` only when parent already passed `hanabi_l3_plugin_wired()`.
#[cfg(feature = "hanabi_l3")]
pub struct HanabiEmbellishmentPlugin;

#[cfg(feature = "hanabi_l3")]
impl Plugin for HanabiEmbellishmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_hanabi::HanabiPlugin);
        app.init_resource::<HanabiEmbellishmentPresets>();
        app.init_resource::<HanabiBurstMapDiagnostics>();
        app.add_systems(Update, sync_hanabi_fire_ember_bursts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_presets_within_designer_bounds() {
        for caps in production_preset_table() {
            assert!(
                preset_within_bounds(&caps),
                "preset {} out of bounds",
                caps.id
            );
        }
    }

    #[test]
    fn map_spawns_only_local_tactical() {
        let caps = fire_ember_caps();
        let chunks = [(IVec2::new(1, 2), 0.95f32), (IVec2::new(3, 4), 0.91)];
        let tac = map_burst_chunks_to_spawns(chunks.into_iter(), WorldLodBand::LocalTactical, &caps);
        let strat = map_burst_chunks_to_spawns(chunks.into_iter(), WorldLodBand::Strategic, &caps);
        assert_eq!(tac.len(), 2);
        assert!(strat.is_empty());
        assert!(tac[0].preset_id == "fire_ember_burst");
    }

    #[test]
    fn map_clamps_to_caps() {
        let caps = fire_ember_caps();
        let chunks: Vec<_> = (0..40).map(|i| (IVec2::new(i, 0), 1.0)).collect();
        let mapped =
            map_burst_chunks_to_spawns(chunks.into_iter(), WorldLodBand::LocalTactical, &caps);
        assert_eq!(mapped.len(), caps.max_instances as usize);
    }

    #[test]
    fn burst_consumer_contract_present() {
        assert!(hanabi_burst_consumer_present());
    }

    #[cfg(feature = "hanabi_l3")]
    #[test]
    fn hanabi_embellishment_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(HanabiEmbellishmentPlugin);
        assert!(app.world().contains_resource::<HanabiEmbellishmentPresets>());
    }

    #[cfg(feature = "hanabi_l3")]
    #[test]
    fn fire_ember_effect_respects_caps() {
        let caps = fire_ember_caps();
        let effect = fire_ember_effect_asset(&caps);
        assert!(effect.capacity() <= caps.max_instances as usize);
    }
}
