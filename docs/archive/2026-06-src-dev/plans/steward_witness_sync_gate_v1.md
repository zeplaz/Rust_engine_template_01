# STEWARD-WITNESS-SYNC-001 gate `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `STEWARD-WITNESS-SYNC-001` |
| **Date** | 2026-05-25 (re-run) |
| **Owner** | `@sim-steward` |
| **Index** | [`debug_runs/agent_debug_index.json`](../../debug_runs/agent_debug_index.json) |

## Verdict: **PASS (qualified)**

Lib-refreshed witness bundle is **aligned**. One **operator** tail: refresh `stage5_full_app_live.json` timestamp after major merges.

---

## Shift A — Observe (bundle matrix)

| Witness | Key gates | Sync method | Status |
|:---|:---|:---|:---:|
| `ui_shell_migration_live.json` | `phase2b_closed`, `ui_p2a_coder_b.green` | `ui_p2a_001_live_witness_refresh` | ✅ |
| `infrastructure_view_isolation_live.json` | `infrastructure_view_isolation_green`, `vm_09` | `steward_vm09_infrastructure_witness_refresh` | ✅ |
| `stage7_play_live.json` | `s7p_steward_green`, `production_green` | `s7p_steward_live_json_refresh` | ✅ |
| `industrial_activation_live.json` | `activation_green` | on-disk (sim / prior run) | ✅ |
| `logistics_throughput_live.json` | `throughput_green` | on-disk (LOG-E01 path landed) | ✅ |
| `construction_stage_live.json` | `operational_green` | on-disk | ✅ |
| `minimap_compositor_live.json` | `ui_p3_m3_green` | `minimap_compositor_live_witness_refresh` | ✅ |
| `stage5_full_app_live.json` | `stage5_closure.passes`, water + tactical VFX | **stale epoch** vs siblings | ⚠ qualified |

**Missing (non-blocking):** `replay_editor_parity_live.json`, `orchestrator_thread_health.json` (env-gated).

---

## Shift B — Decide

```yaml
shift: B
issue:
  id: STEWARD-WITNESS-SYNC-001
  severity: LOW
route:
  pass: close sync steward; docs/ledger may cite green bundle
  operator_tail: OPS-STAGE5-WITNESS-REFRESH — one --test visual after merge
  block: none for @coder
```

**Do not** re-run individual steward packages (WATER / S7P / VM-09 / UI-SHELL) unless a row above flips false.

**Note:** Partial shell writers can leave `ui_p2a_coder_b.green: false`; bundle test re-commits full shell witness first (see `steward_witness_sync_proof.rs`).

---

## Shift C — Act

```powershell
cargo test -p proc_A_dine01 --lib ui_p2a_001_live_witness_refresh
cargo test -p proc_A_dine01 --lib s7p_steward_live_json_refresh
cargo test -p proc_A_dine01 --lib steward_vm09_infrastructure_witness_refresh
cargo test -p proc_A_dine01 --lib simulation_egui_gate_witness_sync
cargo test -p proc_A_dine01 --lib minimap_compositor_live_witness_refresh
cargo test -p proc_A_dine01 --lib steward_witness_sync_001_lib_bundle
# Operator tail (stage5 timestamp):
cargo run -p proc_A_dine01 --release -- --test visual
```

| Action | Result |
|:---|:---|
| Lib refresh tests | ✅ **6/6** |
| Bundle test | ✅ `steward_witness_sync_001_lib_bundle` |
| `agent_debug_index.json` | ✅ refreshed on writes |

---

## Route to @coder

**None** from this steward lane.

Optional operator: **`--test visual`** once to align `stage5_full_app_live.json` `written_at_epoch_secs` with shell/minimap refreshes.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.1 | 2026-05-25 | Re-run — lib **6/6** + bundle green |
| v1.0.0 | 2026-05-25 | Initial fleet witness sync **PASS (qualified)** |
