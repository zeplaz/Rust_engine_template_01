# S2 — Weather simulation — step pack `v1`

> **Parent:** [`../../guides/weather_simulation_runbook_v1.md`](../../guides/weather_simulation_runbook_v1.md)  
> **Parallel with:** **S8** ([`python_asset_tools_alignment_runbook_v1.md`](../../guides/python_asset_tools_alignment_runbook_v1.md)) — different workstreams; see [`new_propsal_guide_may202608.md`](../../guides/new_propsal_guide_may202608.md) §5.

Version: `v1.0.1`  
**STATUS:** **In progress** — S2-S00..S2-S02 code landed 2026-05-06.

---

## S2-S00 — Plugin shell + registration

| Field | Content |
|:---|:---|
| **Goal** | `WeatherPlugin` exists and is registered in `EnginePlugin` so weather systems have a single owner. |
| **Anchor reads** | `src/systems/weather/mod.rs`; `src/engine/engine_with_worldgen.rs` |
| **Touch** | Rust: module + `add_plugins(WeatherPlugin)` |
| **Verify** | `cargo check -p proc_A_dine01` |
| **DoD** | Build green; plugin documented with runbook link. |

**Result (2026-05-06):** Done.

---

## S2-S01 — Chunk weather resource (design)

| Field | Content |
|:---|:---|
| **Goal** | Specify `ChunkWeather` (or resource keyed by chunk) fields that align with [`weather_simulation_runbook_v1.md`](../../guides/weather_simulation_runbook_v1.md) §4.3 and [`chunk_scheduler_gap_table_v1.md`](../chunk_scheduler_gap_table_v1.md). |
| **Anchor reads** | Weather runbook §7 rule (no ontology mutation); `DynamicTerrainOverlay` in `src/terrain/dynamic_overlay.rs` |
| **Touch** | `chunk_scheduler_gap_table_v1.md` §2.1; `src/systems/weather/chunk_weather.rs` |
| **Verify** | `cargo check -p proc_A_dine01` |
| **DoD** | One committed markdown lists proposed ECS types + consumers (mobility, power stubs). |

**Result (2026-05-06):** **ChunkWeather** component + gap table §2.1; mobility/power still future consumers. **`soil_moisture`** (v1 follow-up 2026-05-09): derived per tick from cell moisture + rain; feeds [`ChunkEcology`](../../../../src/systems/ecology/chunk_ecology.rs) coupling.

---

## S2-S02 — Tick gate + `SimControlState`

| Field | Content |
|:---|:---|
| **Goal** | First weather tick system runs only when `SimControlState::should_tick()` (match manufacturing / other sims). |
| **Anchor reads** | `src/systems/sim_control/`; `src/entities/production/core/manufacturing_plugin.rs` |
| **Touch** | `src/systems/weather/` (`weather_chunk_tick` uses `ctrl.should_tick()` + `dt_scale`) |
| **Verify** | `cargo check -p proc_A_dine01` |
| **DoD** | Pause/step does not advance placeholder weather clock. |

**Result (2026-05-06):** **`weather_chunk_tick`** returns early when `!should_tick()`; uses `dt_scale` for decay.

---

## S2-S03 — Matrix ownership row

| Field | Content |
|:---|:---|
| **Goal** | [`asset_sim_ownership_matrix_v1.md`](../asset_sim_ownership_matrix_v1.md) lists **Weather** primary owner `WeatherPlugin` / future `systems/weather/*`. |
| **Anchor reads** | Matrix template; orchestrator §3 **Weather / climate** layer |
| **Touch** | `asset_sim_ownership_matrix_v1.md` |
| **Verify** | Link from matrix to this step pack. |
| **DoD** | Row status **Partial** until chunkweather + effects exist. |

**Result (2026-05-06):** Weather row in `asset_sim_ownership_matrix_v1.md` **v1.2.2** (`chunk_weather.rs` anchor); cross-links updated.

---

## Document history

- **2026-05-06:** Initial pack; S2-S00 executed; **S2 ∥ S8** policy in proposal index.
- **2026-05-06:** `v1.0.1` — `ChunkWeather` + sim-gated tick (S2-S01/S2-S02).
