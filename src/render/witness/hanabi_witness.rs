//! H-A2 / ES-7 witness helpers — spike report, double gate, no-writeback, no minimap bleed.

use std::path::PathBuf;

use crate::render::fx_spine::hanabi_embellishment::{
    fire_ember_caps, hanabi_burst_consumer_present, map_burst_chunks_to_spawns, preset_within_bounds,
};
use crate::gui::WorldLodBand;
use bevy::math::IVec2;

/// `experiments/hanabi_validation/report_v1.md` on disk (PLAN-HANABI-H-A2-EXEC-001 / ES-7-1).
#[must_use]
pub fn hanabi_spike_report_present() -> bool {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    root.join("experiments/hanabi_validation/report_v1.md")
        .is_file()
}

/// Default binary must not wire Hanabi unless feature + env (double gate).
#[must_use]
pub fn hanabi_l3_plugin_wired() -> bool {
    cfg!(feature = "hanabi_l3")
        && std::env::var_os("RUST_ENGINE_HANABI_L3")
            .is_some_and(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

/// Static audit: Hanabi embellishment surface does not mutate sim / weather authority.
#[must_use]
pub fn hanabi_no_sim_writeback() -> bool {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    // Only scan the consumer module — this witness file intentionally lists deny tokens.
    let path = root.join("src/render/fx_spine/hanabi_embellishment.rs");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    let deny = [
        ("ResMut<", "ChunkSurfaceFire"),
        ("ResMut<", "ChunkWeather"),
        ("ResMut<", "AtmosphereField"),
        ("Mut<", "ChunkSurfaceFire"),
        ("Query<&mut ", "ChunkWeather"),
    ];
    for (a, b) in deny {
        if text.contains(&format!("{a}{b}")) {
            return false;
        }
    }
    true
}

/// LocalTactical gate ⇒ strategic/minimap bands never get mapped spawns.
#[must_use]
pub fn hanabi_minimap_bleed_free() -> bool {
    let caps = fire_ember_caps();
    let chunks = [(IVec2::new(0, 0), 1.0f32)];
    map_burst_chunks_to_spawns(chunks.into_iter(), WorldLodBand::Strategic, &caps).is_empty()
        && map_burst_chunks_to_spawns(chunks.into_iter(), WorldLodBand::Operational, &caps)
            .is_empty()
        && map_burst_chunks_to_spawns(chunks.into_iter(), WorldLodBand::Macro, &caps).is_empty()
        && !map_burst_chunks_to_spawns(
            chunks.into_iter(),
            WorldLodBand::LocalTactical,
            &caps,
        )
        .is_empty()
}

#[must_use]
pub fn hanabi_es7_contract_json() -> serde_json::Value {
    let caps = fire_ember_caps();
    let lt = map_burst_chunks_to_spawns(
        [(IVec2::new(2, 3), 0.95)].into_iter(),
        WorldLodBand::LocalTactical,
        &caps,
    );
    let op = map_burst_chunks_to_spawns(
        [(IVec2::new(2, 3), 0.95)].into_iter(),
        WorldLodBand::Operational,
        &caps,
    );
    serde_json::json!({
        "hanabi_spike_report_present": hanabi_spike_report_present(),
        "hanabi_l3_wired": hanabi_l3_plugin_wired(),
        "hanabi_l3_feature_compiled": cfg!(feature = "hanabi_l3"),
        "burst_hint_consumer_present": hanabi_burst_consumer_present(),
        "hanabi_no_sim_writeback": hanabi_no_sim_writeback(),
        "hanabi_minimap_bleed_free": hanabi_minimap_bleed_free(),
        "fire_ember_caps_ok": preset_within_bounds(&caps),
        "peak_instances_cap": caps.max_instances,
        "mapped_spawns_localtactical": lt.len(),
        "mapped_spawns_operational": op.len(),
        "particles_rendered": false,
        "contract_proven_without_backend": !hanabi_l3_plugin_wired(),
        "plan_note": "ES-7 contract-wired; emitter visual capture = ES-7-4 with RUST_ENGINE_HANABI_L3. Reject weather rain / sim writeback / minimap.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanabi_spike_report_on_disk() {
        assert!(hanabi_spike_report_present());
    }

    #[test]
    fn hanabi_l3_off_by_default() {
        assert!(!hanabi_l3_plugin_wired());
    }

    #[test]
    fn es7_no_writeback_and_no_minimap_bleed() {
        assert!(hanabi_no_sim_writeback());
        assert!(hanabi_minimap_bleed_free());
        assert!(hanabi_burst_consumer_present());
    }
}
