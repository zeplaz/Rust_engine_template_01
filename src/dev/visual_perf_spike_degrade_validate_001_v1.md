# DESIGN-PERF-DEGRADE-VALIDATE-001 — Spike degrade validation vs OPS p95 `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-PERF-DEGRADE-VALIDATE-001** |
| **Baseline contract** | [`visual_perf_spike_degrade_ux_v1.md`](visual_perf_spike_degrade_ux_v1.md) (**DESIGN-VISUAL-PERF-DEGRADE-001**) |
| **Operator lane** | **OPS-PLAY-001** → [`debug_runs/perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** — pending operator p95 table fill |
| **No Rust** | Validation record only |

---

## Purpose

Confirm observed degradation under load matches the signed **player vs dev** matrix after a **60s release Simulation** run. Closes designer loop before **PERF-P2** ships production budgets.

---

## Validation method

1. Operator runs **OPS-PLAY-001** (release, ~60s sim, `STALL=1` + `perf=info` per [`perf_attribution_60s.md`](../../debug_runs/perf_attribution_60s.md)).
2. Operator fills **p95 table** below from console `PERF` / `STALL` lines.
3. Designer checks each observed suppressible bucket against [`visual_perf_spike_degrade_ux_v1.md`](visual_perf_spike_degrade_ux_v1.md) § Suppression matrix.

---

## OPS-F01 p95 table (operator fill)

| Bucket | p95 ms | Spike frames observed? | Matches degrade contract? | Notes |
|:---|---:|:---:|:---|:---|
| `wall` (frame) | _TBD_ | ☐ | Target &lt; 33 ms or documented HW baseline | |
| `upd_streaming_reconstruct` | _TBD_ | ☐ | Dev-only impact; player sees slow tile catch-up only | |
| `view_fire` / fire extract | _TBD_ | ☐ | Subtle lag OK; **no** zero fire buffer | |
| `raster_b` / tile fallback | _TBD_ | ☐ | Chunk cap min(2) OK | |
| `egui_world_gen_ui` | _TBD_ | ☐ | Must be **0** in Simulation (PLAY-02 gate) | |
| `hud_egui_ms` | _TBD_ | ☐ | Shell budget; no player perf toast | |

**Capture date:** _TBD_ · **Machine:** _TBD_

---

## Designer crosswalk (pre-OPS)

| Contract row | Expected under spike | Pre-validate |
|:---|:---|:---:|
| Preview suppressed | WorldGen only | ☑ code path |
| Diagnostics entity scan suppressed | Dev-only | ☑ |
| Fire extract defer | Hold last frame | ☑ policy |
| Tile chunks → 2 | Accept pop-in | ☑ `TileRasterBudget` |
| No player perf toast | Sim v1 | ☑ |
| Minimap GPU not blank | Hard must-not | ☑ compositor policy |

---

## Player-visible observation checklist (operator)

During OPS-PLAY-001, note **Y/N**:

| Observation | Acceptable per contract? | Observed |
|:---|:---:|:---:|
| Fire/sparks briefly lag after pan | Y (subtle) | _ |
| Terrain edges catch up slowly | Y | _ |
| Minimap never empty | Required | _ |
| No “performance mode” toast | Required | _ |
| Construction ghosts remain visible | Required | _ |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-05-28 |
| Operator | ☐ p95 table filled | — |

**Qualified until:** OPS-PLAY-001 p95 row filled → re-sign **PASS** (full).

**Unblocks:** **PERF-P2-TILE-RASTER-BUDGET-001**, **PERF-P2-FIRE-EXTRACT-CADENCE-001** (policy validated; measurement optional for coder).
