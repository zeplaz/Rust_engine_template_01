# PLAN-CONSTRUCTION-R4-PRODUCT-OPEN-001 — Round 4 product board policy `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-R4-PRODUCT-OPEN-001** |
| **Parent** | [`construction_round4_product_gate_plan_v1.md`](construction_round4_product_gate_plan_v1.md) (**PLAN-CONSTRUCTION-R4-001** SIGNED) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **SIGNED (policy)** — optional planner doc |

**Planner sign-off:** PASS (2026-05-27). **Witness today:** board is **OPEN** on disk — R4 corridor/MV **CLOSED**. This doc governs **future** flips and new R4 scope only.

---

## Purpose

Document when `product_board_open` may be true, who may flip it, and what coder lanes unlock — without reopening archived R4 exec plans (`PLAN-CONSTRUCTION-R4-EXEC-001`, `PLAN-CONSTRUCTION-R4-MV-EXEC-001`).

---

## Witness fields (authoritative)

**File:** `debug_runs/construction_stage_live.json`

| Path | Meaning |
|:---|:---|
| `/construction_r4_prep_001/product_board_open` | Product gate for Round 4 prep |
| `/construction_r4_corridor_001/product_board_open` | Corridor lane gate (rollup) |

**Current disk (2026-05-27):** both **`true`** — R4 implementation witnesses green.

---

## Policy states

| State | `product_board_open` | Coder lanes |
|:---|:---:|:---|
| **Closed** | `false` | R4-CORRIDOR / R4-MV **blocked** (prep + specs only) |
| **Open** | `true` | R4 implementation allowed per signed R4-PLAN-001/002 |
| **Open + closed** | `true` + witnesses green | **Regression only** — do not re-plan exec docs |

---

## Flip procedure (product → coder)

1. Product declares Round 4 open (or new R4 slice) in writing.
2. Coder sets witness `product_board_open: true` via live proof writer (not hand-edit).
3. Implement only lanes named in signed specs — corridor before MV overlay.
4. Refresh `construction_stage_live.json` after each ≤3-file PR.

**Flip closed (rare):** product freeze → coder sets `false` only with planner + steward ack; existing committed topology **not** rolled back.

---

## Acceptance when open (already met on disk)

| Lane | Witness block | Green when |
|:---|:---|:---|
| **R4-CORRIDOR-001** | `construction_r4_corridor_001` | `green` + `r8_roundtrip_ok` + `corridor_phase_visual_wired` + `sim_tick_writer_wired` |
| **R4-MV-GHOST-001** | `construction_r4_mv_ghost_001` | `green` + `corridor_overlay_tokens_wired` + `legend_wired` + `mv_001_still_green` |

---

## New Round 4 work (future)

When product requests **additional** R4 features beyond corridor/MV:

1. New **planner** spec row (not reopening archived exec markdown).
2. New witness block in `construction_stage_live.json`.
3. Must preserve `construction_mv_001.green` and parametric placement green.

---

## Anti-patterns

- Reopening `plan_construction_r4_exec_001_v1.md` / `plan_construction_r4_mv_exec_001_v1.md` for closure work
- Hand-editing `product_board_open` without proof writer
- Treating Stage 5 FULL_APP as product board flip authority

---

## Operator / planner

| Role | Action |
|:---|:---|
| **@planner** | Amend this policy when product defines new R4 scope |
| **@coder B** | No action while board open + witnesses green |
| **@operator** | No witness refresh required for policy doc alone |

---

## Verification (read-only)

```powershell
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
```

Confirm: `construction_r4_prep_001.product_board_open` and corridor/MV greens match product intent.
