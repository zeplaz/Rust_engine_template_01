# Industrial activation board reconcile `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Cycle** | **PLAN-LEDGER-REFRESH-003** |
| **Witness** | `debug_runs/industrial_activation_live.json` |
| **Alias** | [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) |

---

## Truth table (machine state)

| Lane | Predicate | Board row | Default JSON writer | Commit-path test |
|:---|:---|:---:|:---:|:---:|
| **IND-E01** | `production_green()` | **[x]** | ☑ | `simulation_writes_industrial_activation_live_json` |
| **IND-E02** | `in_play_green()` | **[x] commit only** | ☐ `ind_e02_green` on seed | `simulation_writes_industrial_activation_live_json_ind_e02_in_play` |
| **IND-E03** | `ind_e03_green` | **[x]** | ☑ | `ind_e03` block + overload cluster |

**IND-E02 requires:** `production_green && placed_via_construction && sites_committed >= 3`

**Policy:** Do not fail IND-E02 when default proof JSON lacks `ind_e02_green` — seed path only.

### IND-E02-DEFAULT-WITNESS (2026-05-25)

| Check | Default JSON writer | Commit-path test |
|:---|:---:|:---:|
| `production_green` | ☑ | ☑ |
| `ind_e02_green` | ☐ **by design** | ☑ |
| `placed_via_construction` | `false` on seed | `true` in-play |
| `sites_committed` | `0` on seed | `>= 3` in-play |

**Planner spec:** [`ind_board_reconcile_plan_v1.md`](ind_board_reconcile_plan_v1.md) § IND-E02-DEFAULT-WITNESS.

---

## Ledger / board targets

| Artifact | Action |
|:---|:---|
| [`post_stage6_active_todos.md`](post_stage6_active_todos.md) | IND-E02 row: commit path vs default JSON |
| [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) | IND-E01/E02/E03 **CURRENT** |
| [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) | **IND-E03-CODER-A** **DONE** + `plan_doc` |
| [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) | Mirror done rows |

---

## plan_doc

| Coder ID | Plan |
|:---|:---|
| **IND-E03-CODER-A** | [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) |
