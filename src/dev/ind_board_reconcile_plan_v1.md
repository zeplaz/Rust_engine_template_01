# Industrial activation board — IND-E01/E02/E03 truth `v1` (PLAN-IND-BOARD-RECONCILE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-IND-BOARD-RECONCILE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Status** | **SIGNED** |
| **Canonical** | [`industrial_activation_board_reconcile_v1.md`](industrial_activation_board_reconcile_v1.md) |
| **Witness** | [`debug_runs/industrial_activation_live.json`](../../debug_runs/industrial_activation_live.json) |
| **E03 plan** | [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) |

---

## Problem (board conflict)

Docs marked **IND-E02** **[x]** while default live JSON shows:

```json
"concrete_chain_e2e": {
  "production_green": true,
  "ind_e02_green": false,
  "placed_via_construction": false,
  "sites_committed": 0
}
```

**This is not a contradiction** — two proof paths.

---

## Truth table

| Lane ID | Predicate | Default live JSON writer | Commit / play path |
|:---|:---|:---|:---|
| **IND-E01** | `production_green()` | ☑ `production_green: true` | Lib test `simulation_writes_industrial_activation_live_json` |
| **IND-E02** | `in_play_green()` | ☐ `ind_e02_green: false` (seed) | Test `simulation_writes_industrial_activation_live_json_ind_e02_in_play` |
| **IND-E03** | `ind_e03_green` | ☑ `ind_e03_green: true` | Overload cluster + `production_green` |

### `in_play_green()` requires

```rust
production_green()
  && placed_via_construction
  && sites_committed >= 3
```

Default proof uses `spawn_concrete_portland_chain_operational` — **not** construction commits.

---

## Board / doc policy (003)

| Doc | IND-E01 | IND-E02 | IND-E03 |
|:---|:---:|:---:|:---:|
| [`post_stage6_active_todos.md`](post_stage6_active_todos.md) | [x] | [x] **commit path** / note JSON | [x] |
| [`stage_tracks_audit_signoff_20260525.md`](stage_tracks_audit_signoff_20260525.md) | production_green | in_play via dedicated test | ind_e03 block |
| **INDUSTRIAL-I3-02** board row | witness hook | N/A | overload |

**Do not** mark IND-E02 failed when only default JSON lacks `ind_e02_green`.

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib industrial_activation
```

| Test | Proves |
|:---|:---|
| `simulation_writes_industrial_activation_live_json` | E01 + E03 + board |
| `simulation_writes_industrial_activation_live_json_ind_e02_in_play` | E02 commit path |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | PLAN-IND-BOARD-RECONCILE-001 — E01/E02/E03 truth |
