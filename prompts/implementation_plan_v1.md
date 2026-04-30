# Implementation plan `v1` — iteration-loop scaffold

**Goal:** unblock **iteration sims** with the smallest tools/UI surface, then layer real systems against existing prompts/matrices.

**Read first:** `prompts/llm_agent_brief.md` § **Workflow — design Q&A vs implementation**.

---

## Just-landed scaffold (compile-safe stubs)

| File | Owner | Designer source |
|:---|:---|:---|
| `src/systems/sim_control.rs` | `SimControlPlugin` (`SimControlState`, `SimTick`) | `tools_ui/spec/01_plugin_schedule_patterns.md` |
| `src/gui/diagnostics_ui.rs` | `DiagnosticsUiPlugin` (F3 egui — FPS, pause/step/speed, entity count) | `tools_ui/spec/04_metrics_diagnostics.md`, `implementation_questions_v1.md` §5–10 |
| `src/gui/faction_tools_ui.rs` | `FactionToolsUiPlugin` (F4 egui — Roster / Blueprint / Diplomacy / Import-Export tabs) | `factions/faction_editor/02_ui_egui_panels.md`, `01_data_model.md` |
| `src/entities/production/core/manufacturing_plugin.rs` | `ManufacturingCorePlugin` (per-tick scaffold for `ManufacturingNode`, gated by `SimControlState`) | `production_economy/spec/01_data_model_manifest.md`, `implementation_questions_v1.md` §12–13 |
| `src/engine/engine_with_worldgen.rs` | Wires all of the above into `EnginePlugin` | — |

**Hotkeys (current):** `F3` Diagnostics · `F4` Faction Tools · `F7` Agents · `F8` World Generator. Conflicts: none today; document new keys in `tools_ui/spec/01_plugin_schedule_patterns.md`.

---

## Population queue (next, ordered)

1. **Production serialization plugins** — fill `ConcreteSerializationPlugin` / `AluminumSerializationPlugin` / `PowerSerializationPlugin` in `src/systems/production/serialization.rs`. Pair: `matrix/serialization/`, `production_economy/spec/03_persistence_snapshots.md`.
2. **Damage DTOs serde + building damage system** — derive `Serialize/Deserialize` on `RoadVehicleDamageInfo` / `BuildingDamageInfo`; add `apply_building_damage` system. Pair: `production_economy/implementation_questions_v1.md` §17–19.
3. **Faction data layer** — `FactionBlueprint`, archetype hook, diplomacy graph DTO; bind `FactionToolsUiPlugin` panels. Pair: `factions/faction_editor/01_data_model.md`.
4. **Diagnostics tabs** — chunk streamer stats, production manifest summary, faction roster. Extend `diagnostics_ui_system` or chain new systems in `EguiPrimaryContextPass`.
5. **Sim systems honour `SimControlState`** — gameplay systems multiply `time.delta_secs()` by `ctrl.dt_scale()` so pause/step/speed work. Pair: `tools_ui/spec/02_egui_bevy_split.md`.
6. **Navigation cleanup** — rename `potental_feild_nav.rs` → `potential_field_nav.rs`, register modules. Pair: `navigation/implementation_questions_v1.md` §11–14.

---

## Iteration loop (how to use it)

1. `cargo run` → window opens; **F3** for Diagnostics.
2. **Pause** → click **Step** to advance N ticks; tweak **speed** to see effects accumulate.
3. **F8** to author a world; **F4** to scaffold faction data; **F7** for agent permissions.
4. As real systems land, gate them on `SimControlState::should_tick()` so sim stays single-source-of-truth and pause works everywhere.

---

## Discipline (no scope creep)

- **No new gameplay numbers** in code without designer sign-off (`📎` / `ASK:` in prompts).
- **Stubs stay stubs** until paired with a `spec/` decision.
- **Update matrices** when a row goes ⏳ → ✅; do **not** reword existing entries unless code moved.
