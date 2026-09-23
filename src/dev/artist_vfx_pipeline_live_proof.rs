//! **VSS-T4-004** — refresh `debug_runs/artist_vfx_pipeline_live.json` consumable ECS slice.

use bevy::prelude::*;

use crate::render::{
    effect_spawn_hook_resolver_wired, resolve_effect_spawn, EffectConsumableRegistry,
    EffectConsumableRegistryPlugin, EffectResolvedSpawn, EffectSpawnResolveContext,
    SpawnHookKind,
};
use crate::render::extraction::sim_visual_extract::ClimateVisualAggregate;

pub const ARTIST_VFX_PIPELINE_LIVE_JSON: &str = "debug_runs/artist_vfx_pipeline_live.json";

#[must_use]
pub fn artist_vfx_t4_004_witness_body() -> serde_json::Value {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(EffectConsumableRegistryPlugin);
    app.update();

    let registry = app.world().resource::<EffectConsumableRegistry>();
    let wired = effect_spawn_hook_resolver_wired();
    let loaded = registry.len() == 3 && registry.load_errors.is_empty();

    let spark = registry.get("spark_shower");
    let smoke = registry.get("smoke_column");
    let rain = registry.get("rain_streaks");

    let world_xy_ok = spark.is_some_and(|d| {
        matches!(
            resolve_effect_spawn(d, &EffectSpawnResolveContext::default()),
            EffectResolvedSpawn::WorldPosition(_)
        )
    });
    let attach_ok = smoke.is_some_and(|d| {
        d.spawn_hook == SpawnHookKind::AttachEntity
    });
    let field_ok = rain.is_some_and(|d| {
        let climate = ClimateVisualAggregate {
            mean_rain: 0.5,
            ..Default::default()
        };
        let ctx = EffectSpawnResolveContext {
            climate: Some(&climate),
            ..Default::default()
        };
        matches!(
            resolve_effect_spawn(d, &ctx),
            EffectResolvedSpawn::FieldSample { emits: true, .. }
        )
    });

    let green = wired && loaded && world_xy_ok && attach_ok && field_ok;

    serde_json::json!({
        "program_id": "VSS-001",
        "track": "VSS-T4",
        "slice_id": "VSS-T4-004",
        "green": green,
        "effect_consumable_ecs": {
            "module": "render::fire_vfx::consumable",
            "registry_root": "assets/effects/registry/",
            "loaded_count": registry.len(),
            "spawn_hook_resolver_wired": wired,
            "spawn_hooks": {
                "world_xy": world_xy_ok,
                "attach_entity": attach_ok,
                "field_sample": field_ok,
            },
            "no_parallel_draw_path": true,
            "registry": registry.witness_json(),
        },
        "pending": {
            "preview_witness_capture": "G4 preview worker — capture_hash + honest_gate=honest",
            "aps_panel": "VSS-T4-005",
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artist_vfx_t4_004_witness_green() {
        let body = artist_vfx_t4_004_witness_body();
        assert_eq!(body["slice_id"], "VSS-T4-004");
        assert_eq!(body["green"], true);
        assert_eq!(body["effect_consumable_ecs"]["loaded_count"], 3);
    }
}
