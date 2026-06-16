# Fleet wave 7 — coder dispatch (pick-up board)

| Field | Value |
|:---|:---|
| **Version** | `1.2.0` |
| **Date** | 2026-05-28 |
| **Phase** | **PHASE-NEXT cycle 2** |
| **Active workboard** | [`fleet_coder_workboard_20260528_v3.md`](fleet_coder_workboard_20260528_v3.md) |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Prior** | [`fleet_wave6_coder_dispatch_v1.md`](fleet_wave6_coder_dispatch_v1.md) |

**Rule:** One primary per coder per day; parallel only when files do not overlap. **Never** `tile_world_fallback.rs` + witness migration in one PR.

---

## Status summary (2026-05-28)

See **§1** in [`fleet_coder_workboard_20260528_v3.md`](fleet_coder_workboard_20260528_v3.md) for full green/partial table.

**Both coders active.** Coder A wave-7 perf/VFX rows largely landed; **open:** perf witness on disk, P1-B GPU default, VT-5 visual confirm, logistics harness, deformation L2. Coder B **open:** containment 004→007 + UI P3/P4/P5 tails.

---

## Already landed (do not re-pick)

| ID | Owner | Evidence |
|:---|:---|:---|
| PERF-VIS-001-P1BC, P2A, P2B, P2D | A | lib tests + code on disk |
| PERF-VIS-003/004 | A | witness code (disk refresh pending) |
| DEV-CONTAIN-SLICE-1 / minimap / phase0 | A | `runtime_witness/minimap.rs` |
| DEV-CONTAIN-002-CONSTRUCTION | B | `runtime_witness/construction.rs` |
| DEV-CONTAIN-003-ECONOMY | B | `runtime_witness/economy.rs` |
| WSS witnesses + parity + deformation L1 | A | `wss_substrate_live.json` |
| VFX-VECTOR-SHAPES-001 | A | `bevy_vector_shapes` wire draw |
| FIRE-F2-EXTRACT-TAIL-001 | A | F2 rows on disk |
| UI-OH-2B-001 | B | `ui_oh_2b_001.green: true`, `product_egui_shell_in_simulation: false` |
| Wave 6 product (B) | B | S7B M3/M4, LOG-E01, BQ-128, parametric, R4 |

---

## Open — Lane A (@coder A)

| P | ID | Start |
|:---:|:---|:---|
| 1 | **PERF-WITNESS-DISK-REFRESH-001** | **Today** |
| 2 | **PERF-VIS-P1B-GPU-DEFAULT-001** | After P1 |
| 3 | **LOG-S7-HEADLESS-GUARDS-001** | Parallel with B-005 if files disjoint |
| 4 | **STAGE5-VT-FLICKER-VISUAL-001** | After OPS-VT5 or local visual run |
| 5 | **WSS-DEFORMATION-SLAB-L2-001** | Depth tail |
| 6 | **DEV-CONTAIN-HARDFAIL-CI-001** | After B-007 |

---

## Open — Lane B (@coder B)

| P | ID | Start |
|:---:|:---|:---|
| 1 | **DEV-CONTAIN-004-FIRE-WAVEP** | **Today** |
| 2 | **DEV-CONTAIN-005-STAGE7** | Next |
| 3 | **DEV-CONTAIN-006-WSS** | Next |
| 4 | **DEV-CONTAIN-007-SHIM-RETIRE** | After 004–006 parity |
| 5 | **UI-P3-SHELL-ROLLUP-001** | UI tail |
| 6 | **UI-OH-P4-001** / **UI-OH-P5-001** | After containment or parallel |

**UI gate:** `product_egui_shell_in_simulation` exit = **`false`** (not `true`). Real gate: `ui_oh_2b_001.green: true`.

---

## Operator

| ID | Task |
|:---|:---|
| **OPS-F01** | 60s release visual → `perf_attribution_60s.md` |
| **OPS-VT5-001** | VR-04 log for VT-5 flicker slice |
| **OPS-F03** | Optional stage6 refresh |

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib stage5 wss_substrate construction stage7 logistics
.\tools\orchestrator\scripts\check_live_proof_containment.ps1
```

---

## Changelog

| Version | Date |
|:---|:---|
| v1.2.0 | 2026-05-28 — cycle 2 workboard; B 002/003 done; dual-coder active |
| v1.1.0 | 2026-05-28 — VR-04 / P2D / vector shapes policy |
| v1.0.0 | 2026-05-28 — wave 7 open board |
