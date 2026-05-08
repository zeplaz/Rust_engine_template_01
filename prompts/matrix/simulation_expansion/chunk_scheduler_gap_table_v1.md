# Chunk scheduler — coupling & gap table `v1`

> **Parent:** [`../../guides/chunk_scheduler_runbook_v1.md`](../../guides/chunk_scheduler_runbook_v1.md) · **Step pack:** [`runbook/s1_steps_v1.md`](runbook/s1_steps_v1.md) (S1-S02 / S1-S03)

Version: `v1.1.0`  
**STATUS:** **Partial** — updated for `ChunkWeather` (2026-05-06).

---

## 1. Purpose

Inventory **which simulation concerns** are already **chunk-scoped** vs **global / entity-only**, and default **persistence** hints per [`chunk_scheduler_runbook_v1.md`](../../guides/chunk_scheduler_runbook_v1.md) §5.

---

## 2. Gap table

| Concern | Chunk coupling today | Systems / paths | Persist (default) | Recompute (default) |
|:---|:---|:---|:---|:---|
| Terrain **material pipeline** | **Strong** — `Chunk`, `ChunkCellMatrix`, `ChunkDirty`, `ChunkDependency`, `MaterializedChunk` | `src/systems/terrain/material_plugin.rs`; `src/terrain/material/`; `src/terrain/generation/` | Registry handles + authored tuning (assets); chunk **entity** state if saves include world tiles | Derived `ChunkDerivedMetrics`, materialize passes, dirty rebuilds |
| **Dynamic terrain overlay** (mud, snow, …) | **Reads** `Chunk` + `ChunkCellMatrix`; stub writers | `src/terrain/dynamic_overlay.rs`; `material_plugin.rs` overlay systems | **Situational** — major overlays if design needs continuity | Decay / short-lived fields |
| **Navigation / pathing** | **Weak** — comment references future spatial index + streaming | `src/systems/navigation/nav.rs` | Topology / built roads if persisted | Path caches, potential fields |
| **Transport** | **None** in `systems/transport` (no `Chunk` symbol match) | `src/systems/transport/` | Convoy / vehicle state per save design | Cost caches when chunk inputs change |
| **Damage** | **None** in `systems/damage` | `src/systems/damage/` | Building / vehicle damage if in save | Transient combat effects |
| **Weather** | **Component on chunk entities** — `ChunkWeather` (`src/systems/weather/chunk_weather.rs`) for `Added<Chunk>`; tick honors `SimControlState` | `src/systems/weather/` (`WeatherPlugin`) | Chunk-entity components if saves include world | Regional / climate drivers recompute fast fields when designed |

**Rule:** extend **ChunkDirty-style** invalidation only when a system’s inputs are **chunk-addressable** (registry hash, neighbor chunk, local policy). Do not duplicate global economies per chunk without an explicit LOD design.

---

## 2.1 Chunk weather fields (S2-S01)

| Field | Role |
|:---|:---|
| `rain_intensity` | Local rainfall rate stub |
| `fog_density` | Visibility / sensor interference |
| `snow_depth` | Pack depth (distinct from overlay `snow` scalar if both used) |
| `wind_speed` | Power / aviation / dispersion stubs |
| `lightning_risk` | Fire / damage hooks |
| `visibility_factor` | Derived convenience (0–1) |

**Consumers (future):** mobility, power efficiency, combat sensors — read only; do not write material registry.

---

## 3. Cross-links

- [`asset_sim_ownership_matrix_v1.md`](asset_sim_ownership_matrix_v1.md) — `ChunkDependency` / `ChunkDirty` row.  
- [`simulation_expansion_orchestrator_v1.md`](../../guides/simulation_expansion_orchestrator_v1.md) §3 layers.

---

## Document history

- **2026-05-06:** `v1.1.0` — **ChunkWeather** ECS + §2.1 field table; weather row **Applied** (chunk-component level).
- **2026-05-06:** `v1.0.1` — Weather row cites `WeatherPlugin` scaffold (S2 ∥ S8).
