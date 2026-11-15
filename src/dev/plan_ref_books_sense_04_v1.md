# PLAN-REF-BOOKS-SENSE-04-v1 — weather-degraded sensing over cells

| Field | Value |
|:---|:---|
| **Id** | **PLAN-REF-BOOKS-SENSE-04-v1** |
| **Matrix row** | **SENSE-04** · aig3:11.4.4–11.4.5 (Region + Finite Element Model sense managers) |
| **Parent** | [`plan_ref_books_upgrade_v2.md`](plan_ref_books_upgrade_v2.md) §8 · [`plan_weather_wind_v1.md`](plan_weather_wind_v1.md) |
| **Trigger** | **W1 (`SENSE-02`) ADOPTED** — `WeatherPollingStations` + migrated readers |
| **Book orient** | `python src/dev/ref_books_index/lookup.py --row SENSE-04` |
| **Owner lane** | `@coder` (weather + strategic consumers) |

## Summary

Propagate **per-chunk weather degradation** into a **coarse gameplay-cell sense field** so ISR, C2, and strategic visibility read one immutable snapshot per tick instead of recomputing `sensor_range_mul` / fog factors at arbitrary points. The book’s finite-element sense manager is approximated as a **deterministic relaxation** from station seeds (W1 cache) with smoke coupling — not a second weather solve.

## Authority map (sole writers)

| Surface | Sole writer | Consumers (read-only) |
|:---|:---|:---|
| `WeatherField` | `weather_field_step` | `sync_weather_polling_stations` only |
| `WeatherPollingStations` | `sync_weather_polling_stations` | sense propagation **input**; `air_ops`, combat, logistics |
| **`WeatherSenseCellField`** (new) | `build_weather_sense_cell_field` | `sensor_range_mul` helpers, `strategic/network_flow`, drone sortie gates, minimap recon overlays |
| `ChunkSmokeField` / L0 smoke | fire/atmosphere writers | sense propagation **input** (smoke01 per tile) |
| `AirOpsTuning` | startup load | pure functions only |

Relations:

```text
⊚WeatherField ═▶ ⊚WeatherPollingStations ◂⊳[chunk samples]
⊚WeatherSenseCellField ⊰ ⊚WeatherPollingStations (+ smoke RO)
⊚air_ops::sensor_range_mul ◂⊳[sample at cell] ⊚WeatherSenseCellField   (not direct field sample)
⊚strategic::effective_visibility ◂⊳ ⊚WeatherSenseCellField
```

**Reject:** per-entity weather caches, UI writes, second `WeatherField::sample` path for gameplay.

## ECS schedule placement

```text
ChunkEnvironmentSet::Weather (Update, after AdvanceSimTick)
  weather_field_step
  → sync_weather_polling_stations          (existing)
  → build_weather_sense_cell_field         (NEW — after stations, before air_ops status)
  → sync_air_ops_weather_status            (existing — may read cell field for centre tile)
```

Strategic overlay pass that compounds recon confidence runs **after** sense field build and **reads** `WeatherSenseCellField` instead of duplicating `chunk_sensor_weather_factor` on raw `ChunkWeather` where stations exist.

Register `build_weather_sense_cell_field` in `WeatherPlugin` / `plan_schedule_sync_v1.md` cross-ref when landed (SCH-D1 weather chain).

## Save impact

| Item | Verdict |
|:---|:---|
| `WeatherSenseCellField` | **DERIVED** — rebuild every tick from stations + smoke; **omit from save** |
| `WeatherPollingStations` | DERIVED (already) |
| Tuning (relaxation α, max iterations) | RON optional extension to `air_ops_v1.ron` — **version bump** if serialized constants change sim |

No save migration for v1. If a future slice persists “installed sensor emitters”, that is a **separate** component book — **DEFER** (see §DEFER).

## Implementation phases (DAG)

| Phase | Goal | Status |
|:---|:---|:---|
| **S4-P0** | Pure `relax_sense_degradation` kernel: seeds from station `WeatherSample` + smoke → cell `sense_mul ∈ [0.15,1]` | **SHIPPED** |
| **S4-P1** | `WeatherSenseCellField` resource + sole writer system; unit tests on 3×3 fixture | **SHIPPED** |
| **S4-P2** | Route `sensor_range_mul` call sites at tile positions through cell lookup; keep function signature for tests | **SHIPPED** |
| **S4-P3** | `strategic/network_flow.rs` visibility uses cell field for weather leg (retain bunker/insulation terms) | **SHIPPED** |
| **S4-P4** | Witness: reader audit — zero `WeatherField::sample` outside weather module (grep gate in lib test) | **SHIPPED** |

★ **First slice to cut:** **S4-P0** (kernel only, no ECS).

## Risks

| Risk | Mitigation |
|:---|:---|
| Dual authority with `ChunkWeather` projection | Prefer stations when `ready`; fall back to chunk weather only when stations absent |
| Cost at full map scale | Coarse grid (weather chunk or sim chunk), fixed iteration count, attention-tier skip for Dormant chunks |
| Drift vs `sensor_range_mul` point formula | Kernel output **defines** gameplay mul at cell centre; point function becomes thin wrapper |
| Schedule race (read before build) | Chain after `sync_weather_polling_stations`; consumers `.after(build_weather_sense_cell_field)` |

## SHIPPED / PLANNED / DEFER

| Label | Items |
|:---|:---|
| **SHIPPED** | W1 `WeatherPollingStations`; S4-P0–P4 cell field + air_ops/strategic wiring + sample audit |
| **PLANNED** | — |
| **DEFER** | Full **Region Sense Manager** (aig3:11.4.4) — irregular regions, emitter graphs, save-backed installations; EW mesh coupling beyond smoke scalar; GPU sense overlay |

## Diagnostics

- Counters on `WeatherSenseCellField`: `cells_written`, `relax_iters`, `max_delta` (test-only asserts, not production witness resources).
- Optional debug overlay slice **DEFER** until playcheck shows player-visible ISR change.

## Open questions

- ⌁? Grid resolution: align 1:1 with `tile_to_weather_chunk` vs finer sub-cell field?
- ⌁? Should passive RF ignore smoke in propagation (book suggests modality split) — default **yes**, match `SensorKind` branches.

---

## First slice card — **S4-P0** 🧠 Sense degradation kernel

| | |
|:---|:---|
| **Ref** | `python src/dev/ref_books_index/lookup.py --row SENSE-04` |
| **Files** | `src/systems/weather/sense_propagation.rs` (new), `src/systems/weather/mod.rs` (module export), `src/systems/weather/air_ops.rs` (re-export test helper only if needed) |
| **Recon** | `grep -n "sensor_range_mul\|chunk_sensor_weather" src/systems src/strategic src/combat` — list call sites; confirm no existing cell sense resource (`Done-if`: absent). |
| **Do** | Implement `relax_sense_degradation`: rectangular grid, 4-neighbour Jacobi/Gauss–Seidel **fixed N iterations**, seed cells from `WeatherSample` visibility/fog/rain via existing `air_ops` factors + per-cell `smoke01`. Deterministic order (row-major). Output `sense_mul` per cell. |
| **Don't** | ECS resources; `WeatherField::sample`; wall-clock; new crate deps. |
| **Accept** | `cargo test -p proc_A_dine01 --lib systems::weather::sense_propagation` — (1) clear weather → all 1.0; (2) single degraded station diffuses lower mul to neighbours; (3) same seed/grid → byte-identical second run. |

**Joint review question (@coder):** Is weather-chunk resolution sufficient for strategic overlay, or do we need sim-chunk alignment in S4-P1?