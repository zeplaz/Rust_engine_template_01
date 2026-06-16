# LOG-E01 — FULL_APP logistics witness spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **LOG-E01-WITNESS-SPEC** |
| **Lane** | **LOG-E01** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Impl plan** | [`logistics_projection_impl_plan_v1.md`](logistics_projection_impl_plan_v1.md) |
| **Lane spec** | [`logistics_visual_lane_spec_v1.md`](logistics_visual_lane_spec_v1.md) |
| **Witness** | `debug_runs/stage5_full_app_live.json` |

**No Rust in this deliverable.** Defines when `projection_graph.logistics_active_rows` must be **> 0** vs **STALE**.

---

## Rule (one line)

**`logistics_active_rows > 0` is required only when the FULL_APP / visual harness has run a sim frame with transport seed + logistics snapshot publish — not on every arbitrary disk snapshot.**

---

## When `> 0` is required

| Context | Required | Evidence |
|:---|:---:|:---|
| After `cargo run … --test visual` (Stage 5 regression) | **Yes** | `stage5_full_app_live.json` refreshed same session |
| After lib test `stage5_full_app_harness` / readiness writer with seed | **Yes** | test asserts `log_rows` in build signature |
| **S7P-LOG-001** play throughput path | **Yes** | `logistics_throughput_live.json` — separate file |
| **UI-P3-M2** minimap closure | **No** (cross-check only) | `minimap_compositor_live.json` `logistics_rows` is authoritative for M2 |
| Cold disk JSON days later | **No** — label **STALE** | refresh, do not reopen LOG-E01 code |

---

## Witness paths

| File | Field | Green when |
|:---|:---|:---|
| `stage5_full_app_live.json` | `projection_graph.logistics_active_rows` | **> 0** after visual/harness refresh |
| `stage5_full_app_live.json` | `log_e01_visual_confirm_001.log_e01_fixture_green` | `true` on fixture (`capture_lane: "lib_fixture"`) |
| `stage5_full_app_live.json` | `log_e01_visual_confirm_001.full_visual_confirm` | `true` only on visual run (`capture_lane: "visual_run"`) |
| `stage5_full_app_live.json` | `readiness.passes` | `true` (independent — spine may pass with STALE log_rows) |
| `stage5_full_app_live.json` | `build_signature` | contains `log_rows=N` with **N > 0** when logistics lane exercised |
| `logistics_throughput_live.json` | `s7p_log_001_green` / `throughput_green` | play path — **not** merged into stage5 JSON |
| `minimap_compositor_live.json` | `logistics_rows` | UI minimap — **authoritative for M2** over stale stage5 field |

---

## STALE policy

| Observation | Verdict | Action |
|:---|:---|:---|
| `logistics_active_rows: 0` but `readiness.passes: true` | **STALE** | Operator: `--test visual` or harness test |
| `log_e01_fixture_green: true` and `full_visual_confirm: false` | **Fixture-only** | Do not treat as operator visual closure |
| `logistics_active_rows: 0` but `minimap_compositor_live.json` `logistics_rows: 2` | **STALE** stage5 only | Do **not** fail UI-OH / M2 closure |
| Lib seed tests pass, disk old timestamp | **STALE** | Re-run witness writer |
| `logistics_active_rows > 0` after refresh | **CURRENT** | Close LOG-E01 operator row |

**Do not** mark LOG-E01 coder slice **OPEN** when only timestamp is stale.

---

## Verification commands

```powershell
cargo test -p proc_A_dine01 --lib stage5 logistics_visual
cargo run -p proc_A_dine01 --release -- --test visual
```

**Lib-only refresh (no full visual):** tests that write `stage5_full_app_live.json` with transport seed — see `stage5_full_app_harness.rs`.

---

## Ledger / triage

| Artifact | LOG-E01 row |
|:---|:---|
| [`witness_status_live_v1.md`](witness_status_live_v1.md) | STALE optional |
| [`coder_triage_list_v1.md`](coder_triage_list_v1.md) | **done** unless regression |
| [`stage5_triage_backlog.md`](stage5_triage_backlog.md) | TRIAGE-LOGISTICS-VIS → **Done** when spec satisfied |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **LOG-E01-WITNESS-SPEC** signed |
