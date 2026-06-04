//! H-A2-001 — optional Hanabi L3 embellishment (`hanabi_l3` feature + env gate only).

#[cfg(feature = "hanabi_l3")]
use bevy::prelude::*;

/// Production caps (TUNE headroom vs spike report) — [`hanabi_event_vfx_style_bounds_v1.md`](../dev/hanabi_event_vfx_style_bounds_v1.md).
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
pub fn preset_within_bounds(caps: &HanabiPresetCaps) -> bool {
    caps.max_instances <= 32
        && caps.peak_alpha <= 0.45
        && caps.lifetime_min_s >= 0.2 - f32::EPSILON
        && caps.lifetime_max_s <= 1.2 + f32::EPSILON
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

/// L3 plugin — registers `bevy_hanabi` only when parent app already passed `hanabi_l3_plugin_wired()`.
#[cfg(feature = "hanabi_l3")]
pub struct HanabiEmbellishmentPlugin;

#[cfg(feature = "hanabi_l3")]
impl Plugin for HanabiEmbellishmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_hanabi::HanabiPlugin);
        app.init_resource::<HanabiEmbellishmentPresets>();
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

    #[cfg(feature = "hanabi_l3")]
    #[test]
    fn hanabi_embellishment_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(HanabiEmbellishmentPlugin);
        assert!(app.world().contains_resource::<HanabiEmbellishmentPresets>());
    }
}
