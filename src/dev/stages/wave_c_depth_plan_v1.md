# Wave C depth — streaming under residency `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `WAVE-C` |
| **Version** | `1.1.0` |
| **Status** | **ACTIVE** — **WC-D04** via [`../infra_slice3_wc_d04_ops_f01_plan_v1.md`](../infra_slice3_wc_d04_ops_f01_plan_v1.md) |
| **Prereq** | **OPS-F01** operator sample |
| **Exit milestone** | **Wave C operational** — churn tuned + backlog rows closed |
| **Runbook** | [`../../prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md`](../../prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md) §6 |
| **Prerequisite** | Stage 6 closed · Wave P operator refresh green |

---

## North star

Streaming apply behavior matches **Stage 6** `ChunkResidencyTable` + ghost-band contracts — not a new boolean gate flip.

---

## Witness bundle

| File | Fields |
|:---|:---|
| `debug_runs/stage6_virtualization_live.json` | `stage6_readiness.passes`, upload bytes, per-view windows |
| `debug_runs/wave_c_live.json` | `tile_storage_apply`, backlog closure |
| `debug_runs/wave_p_live.json` | Wave P green before deep C work |

---

## @designer instructions

**None** for Wave C depth. Optional: document expected player-visible pop-in vs ghost band in one paragraph for Stage 6 HUD (BQ-134 context).

---

## @coder instructions

### Slice map

| ID | Goal | Files / area |
|:---|:---|:---|
| **WC-DEPTH-001** | Close one `WAVE_C_OPEN_BACKLOG_ITEMS` row | **DONE** — **BQ-101** · `wc_depth_001_green` |
| **WC-DEPTH-002** | BQ-101 TileStorage diff witness | `wave_c_live_proof.rs`, apply report |
| **WC-DEPTH-003** | Residency churn tune (WC-D04) | `stage6_virtualization.rs`, perf notes |
| **WC-OPS-001** | Refresh stage6 JSON in sim | operator + `stage6_live_proof.rs` |

**Invariant:** TaskPool → main-thread apply only via `PendingStreamApplyQueue` (S6-22). No ECS apply from background threads.

### Copy-paste — WC-DEPTH-001

```
Track: WAVE-C — WC-DEPTH-001
Read: src/dev/stages/wave_c_depth_plan_v1.md
      prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md §6
First: pick one open WAVE_C backlog row; implement + test
Do NOT: bypass ChunkResidencyTable authority
Verify: cargo test -p proc_A_dine01 --lib stage6 wave_c
Witness: debug_runs/wave_c_live.json updated
```

### Copy-paste — WC-DEPTH-003

```
Track: WAVE-C — WC-DEPTH-003
Read: post_stage6_active_todos.md WC-D04
First: capture residency churn metric in stage6 witness over 60s sim
Do NOT: change Stage 6 readiness formula without planner
Verify: cargo test -p proc_A_dine01 --lib stage6
```

### Acceptance — Wave C operational

| # | Criterion |
|:---:|:---|
| W1 | `wave_c_live.json` shows closed backlog or explicit empty list |
| W2 | `stage6_virtualization_live.json` refreshed with `gpu_upload_bytes_frame` |
| W3 | `wave_p_live.json` green in same release train |
| W4 | `cargo test -p proc_A_dine01 --lib stage6 wave_c` green |
| W5 | No new viewport authority violations in infra witness |

---

## @operator instructions

1. Enter Simulation (not only `--test visual`)
2. Run until residency stabilizes (~30–60s)
3. Confirm `stage6_virtualization_live.json` and `wave_c_live.json` timestamps updated
4. Record churn notes in handoff if spikes > 2× baseline

---

## Sequencing

| Before | After |
|:---|:---|
| Wave P witness refresh | WC-DEPTH-001 |
| WC-DEPTH-001 | WC-DEPTH-003 churn |
| Large infra VM-09 refactor | defer WC-DEPTH-003 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | WC-D04 → infra_slice3 plan (Coder B) |
| v1.0.0 | 2026-05-24 | Wave C depth plan |
