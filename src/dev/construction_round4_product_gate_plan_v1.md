# PLAN-CONSTRUCTION-R4-001 — construction Round 4 product gate `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-R4-001** |
| **Coder prep** | **CONSTRUCTION-R4-PREP-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **PREP CLOSED** · **R4-PLAN-001/002 SIGNED** · impl **blocked on product board** |
| **Prep index** | [`construction_round4_prep_index_v1.md`](construction_round4_prep_index_v1.md) |
| **Recovery** | [`construction_recovery_todos.md`](construction_recovery_todos.md) |
| **MV sim** | [`construction_multiview_sim_spec_v1.md`](construction_multiview_sim_spec_v1.md) **CLOSED** |

**No Rust.**

---

## Gate

Round 4 implementation starts only when **product board** opens Round 4 — not Stage 5 / not wave 4.

---

## Prep done (coder)

| Item | Status |
|:---|:---:|
| MV sim writer + witness | ☑ |
| Catalog index reconcile | ☑ — [`construction_round4_prep_index_v1.md`](construction_round4_prep_index_v1.md) |
| `construction_r4_prep_001` witness | ☑ — `debug_runs/construction_stage_live.json` |
| Round 3 operational green | ☑ |

---

## R4 planner specs (signed)

| Phase | Deliverable | Status |
|:---|:---|:---:|
| **R4-PLAN-001** | [`construction_round4_corridor_phase_spec_v1.md`](construction_round4_corridor_phase_spec_v1.md) | ☑ **SIGNED** |
| **R4-PLAN-002** | [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md) | ☑ **SIGNED** |

**Until `product_board_open`:** prep only — **no new Round 4 Rust**.

## Product board open policy

| Field | While closed | When product opens Round 4 |
|:---|:---|:---|
| `construction_r4_prep_001.product_board_open` | **`false`** | **`true`** — product decision + witness refresh |
| Coder lanes | **BLOCKED** | **R4-CORRIDOR-001** · **R4-MV-GHOST-001** |
| Wave 4 F7/M3/MV specs | maintain | **do not re-plan** |

**Flip procedure:** product declares open → coder sets witness `product_board_open: true` → implement R4-CORRIDOR then R4-MV-GHOST per signed specs.

## When board opens (@coder)

| Lane | Exit witness |
|:---|:---|
| **R4-CORRIDOR-001** | `construction_r4_corridor_001.green` |
| **R4-MV-GHOST-001** | `construction_r4_mv_ghost_001.green` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-26 | Product board open policy § |
| v1.0.0 | 2026-05-26 | Product gate placeholder |
