# DESIGN-VISUAL-PERF-DEGRADE-001 — Spike degradation UX (player vs dev) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-VISUAL-PERF-DEGRADE-001** |
| **Parent** | [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md) · [`plan_visual_perf_production_v1.md`](plan_visual_perf_production_v1.md) |
| **Guard resource** | `UxFrameSpikeGuard` (`src/engine/ux_states.rs`) |
| **Coder lanes** | **PERF-P2-TILE-RASTER-BUDGET-001** · **PERF-P2-FIRE-EXTRACT-CADENCE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | PERF-P2 ship policy — what may throttle under spike |
| **No Rust** | Degradation contract only |

---

## Purpose

Before **PERF-P2** replaces env throttles with `TileRasterBudget` + `FireExtractCadence`, coders need a signed list of **what `UxFrameSpikeGuard` may suppress**, what players may notice, and what must **never** drop silently.

**Rule:** Degrade **optional** lanes first; hold **last good frame** on tactical surfaces; never break Stage 5 spine contracts.

---

## Spike trigger (current)

| Field | Default | Meaning |
|:---|:---|:---|
| `max_ms` | `33.0` | Budget vs ~30 FPS |
| `spike_enter_frames` | `2` | Consecutive over-budget frames before throttle |
| `spike_active` | derived | Sets `suppress_preview_this_frame` + `suppress_optional_diagnostics` |

Log target: `ux::perf` — **dev-only** (`warn!` on spike enter).

---

## Suppression matrix

| Lane | Guard signal | Behavior under spike | Player-visible? | Surface |
|:---|:---|:---|:---:|:---|
| **World-gen preview raster** | `suppress_preview_this_frame` | Skip preview GPU/CPU pipeline for frame | **No** in Simulation (PLAY-01 dismisses chrome) | WorldGen only |
| **Map fit validator** | `suppress_optional_diagnostics` | Skip fit validation pass | **No** | Dev / diagnostics |
| **Map presentation diagnostics** | `suppress_optional_diagnostics` | Skip heavy presentation debug rows | **No** | Dev |
| **Diagnostics entity count** | `suppress_optional_diagnostics` | Skip full-world `Entity` scan (0.5s cadence) | **No** — only when diagnostics panel open | Dev |
| **Fire ECS full extract** | `spike_active` | Defer full-world scan (band-aid until P2-C) | **Subtle** — fire/sparks may lag 1–2 frames | Tactical |
| **Tile raster chunk budget** | `spike_active` | Cap dirty chunks to **min(2)** per frame | **Subtle** — slower terrain catch-up / pop-in | WorldMain |
| **CPU minimap sub-pass** | `spike_active` + policy | Skip duplicate CPU minimap raster | **No** when GPU compositor is authoritative | Minimap |
| **Zoom-band dirty storm** | `spike_active` | Defer zoom-band full dirty mark | **Subtle** on rapid zoom | WorldMain |

---

## PERF-P2 policy (ship path)

| Resource | Spike interaction | Player-visible cap |
|:---|:---|:---|
| **`TileRasterBudget`** | `effective_chunks_per_frame(spike_active)` → floor **2** | Accept slower dirty flush; **never** zero main-map raster for entire frame |
| **`FireExtractCadence`** | Interval + sim-tick policy **replaces** spike-only skip as sole throttle | Max **1 overlay Hz** lag; **hold** last `FireVisualFrame` — do not clear instances |
| **`VisualCadence`** | May skip preview/minimap **optional** passes | Minimap **GPU** path stays on when compositor green |

---

## Must NOT suppress (hard)

| Contract | Why |
|:---|:---|
| **SimulationMap** authoritative viewport commit | Viewport blink / ortho mismatch |
| **Minimap GPU compositor** when `presentation_source` = shared RT | Blank minimap is unacceptable |
| **Construction ghosts / parametric preview** | Gameplay read; construction invariants |
| **Replay scrub needle** (M3) | Operator timeline read |
| **Sim step / enqueue** | Gameplay correctness |
| **Projection graph merge** | Stage 5 spine |
| **Clearing fire/VFX buffers** on spike | Reads as “VFX broke”; hold stale |
| **Collapsing sim HUD to hide perf** | PLAY-01 chrome rules unchanged |

---

## Player-facing UX policy

| Policy | Spec |
|:---|:---|
| **No perf toast in Simulation v1** | Players do not see “performance mode” or frame ms |
| **No alarm chrome** | Spike is not an error state for operators in sim |
| **Acceptable visible degrade** | Slightly stale fire spark motion; delayed tile edge refresh on pan/zoom |
| **Unacceptable visible degrade** | Empty minimap; missing ghosts; fire/smoke pop to zero; frozen sim input |

### Optional dev-only F3 row (coder may wire)

```text
Perf spike: optional lanes throttled (preview/diagnostics/raster cap)
```

Show only when diagnostics section expanded **and** `spike_active==true`. **Muted** label — not a player surface.

---

## Dev / operator surfaces

| Surface | Under spike | Notes |
|:---|:---|:---|
| **`ux::perf` log** | Warn on spike enter | grep-friendly |
| **Diagnostics panel** | Entity count may be stale | Acceptable |
| **STALL_SPAN_DEBUG / PERF line** | Unchanged | Profiling recipe |
| **World-gen preview** | Paused while over budget | Expected during heavy gen |

**Clean visual run** ([`visual_test_runbook_v1.md`](visual_test_runbook_v1.md)): no `RASTER_*` env; spike guard is production policy.

---

## Relation to PLAY-01

Simulation session defaults already hide world-gen preview and verbose diagnostics. Spike suppression of preview/diagnostics **aligns** with PLAY-01 — no new player chrome required.

---

## Coder wiring checklist

1. ☑ Under spike, **hold** last fire/minimap/world textures — no empty clears.
2. ☑ PERF-P2: spike clamp goes through `TileRasterBudget` / `FireExtractCadence`, not ad-hoc env.
3. ☑ `suppress_preview_this_frame` remains WorldGen-only effective path in sim.
4. ☑ Witness `tactical_vfx_witness.all_green` must remain achievable on 60s visual run **without** `RASTER_*`.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-28 |

**Unblocks:** **PERF-P2-TILE-RASTER-BUDGET-001**, **PERF-P2-FIRE-EXTRACT-CADENCE-001**, **PLAN-VISUAL-PERF-EXEC-001** degrade rows.

---

## On-call holds (same session — not this deliverable)

| P | ID | Status |
|:---:|:---|:---|
| 2 | **DESIGN-S7B-M4-PLAY-READ-001** | **DEFER** — coder B has [`s7b_m4_sim_playtest_spec_v1.md`](s7b_m4_sim_playtest_spec_v1.md); open only if **S7B-M4-PLAY-REMEDY-001** requests enqueue UX |
| 3 | **DESIGN-HANABI-H-A2-PROD-001** | **HOLD** — default binary wiring not chartered |
| 4 | **DESIGN-CONSTRUCTION-R4-PRODUCT-001** | **Planner horizon** — not designer on-call |
