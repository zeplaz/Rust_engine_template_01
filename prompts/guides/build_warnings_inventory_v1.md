# Build warnings inventory (v1)

`cargo build` was emitting **17 `dead_code` warnings** (lib). Each item is categorized and silenced with **`#![allow(dead_code)]` / `#[allow(dead_code)]` + module or item docs** so `cargo build` stays clean while intent stays visible.

| Category | Location | Notes |
|----------|----------|--------|
| **Superseded** | `src/engine/engine.rs` (`EnginePlugin`) | Canonical app: `engine_with_worldgen::EnginePlugin` (re-exported as `crate::engine::EnginePlugin`). Legacy file kept to revive old menu wiring. |
| **Superseded** | `src/gui/gui_assets.rs` | `Images` + `FromWorld` splash loads — current shell uses other bootstrap. |
| **Not wired / superseded by other state** | `src/engine/states.rs` (`EngineState`) | Flow uses `BaseState`, `WorldGenFlowState`, `SimControlState` instead of this struct. |
| **Stub — finish later** | `src/entities/components.rs` (`MaintenanceTimer`) | Intended to pair with `Operational`; no tick consumes it yet. |
| **Stub — finish later** | `src/entities/production/core/production_utils.rs` | `ResourceCategory` / `categorize_resources` — blocked on economy/UI pass wiring categories into production. |
| **Stub — finish later** | `src/traits/buildings.rs`, `src/traits/time.rs` | Placeholder traits; sim uses concrete components + `SimTick` today. |
| **Legacy** | `src/io/serialization/deserializers.rs` | Drez `.dat` loaders and unused serde helpers; **live** public API: `deserialize_road_vehicle_configs`. |
| **Legacy** | `src/render/light.rs` (`MAX_LIGHTS`, empty `LocalLightPlugin`) | File header documents Bevy 0.9-era removal; rewrite pending (`PointLight` / wgpu stack). |
| **Reserved enum** | `src/entities/types/e_flagz.rs` (`EmergencySeverity`) | For future facility/mission severity bands. |

**Policy:** Prefer **explicit `allow` + one-line “why”** over blanket crate-wide allows. Revisit when wiring the feature or delete truly obsolete code.
