# Coder dual queue — closure snapshot `v2.2` (wave 3)

| Field | Value |
|:---|:---|
| **Version** | `2.2.0` |
| **Date** | 2026-05-26 |
| **Full lists** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) ← **start here** |
| **Authority** | [`coder_dual_queue_v3.md`](coder_dual_queue_v3.md) |
| **Prior closure** | [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md) — 28 IDs ☑ |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json) |

**Regression:**

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib coder_a_dual_queue coder_b_queue_bundle
```

---

## @coder A — closed in queue bundle

| # | ☑ | ID | Status |
|:---:|:---:|:---|:---|
| 1 | ☑ | **FIRE7-F7-A-EXIT-001** | done |
| 2 | ☑ | **VFX-VISUAL-SIGNOFF-001** | done_qualified |
| 3 | ☑ | **TRIAGE-GPU-TILE-WGSL-001** | done |
| 4 | ☑ | **TRIAGE-VISUAL-TEARDOWN-001** | done |
| 5 | ☑ | **TRIAGE-PHASE-F-CULL-001** | done |
| 6 | ☑ | **UI-WP-VISUAL-001** | done_qualified |
| 7 | ☑ | **INFRA-GPU-TILE-GIZMO-001** | done |
| 8 | ☑ | **S7B-M4-SIM-001** | done |
| 9 | ☑ | **VFX-CAPTURE-HOOK-001** | done |
| 10 | ☑ | **TRIAGE-COMPILE-HYGIENE-001** | done |
| 11 | ☑ | **FIRE7-DESIGN-LOD-WIRE-001** | done |
| 12 | ☑ | **STAGE5-VT-DEEP-001** | done |
| 13 | ☑ | **FIRE7-F7-B-001** | done |
| 14 | ☑ | **FIRE7-F7-C-001** | done |

Detail: [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) § Coder A

---

## @coder B — closed in queue bundle

| # | ☑ | ID | Status |
|:---:|:---:|:---|:---|
| 1 | ☑ | **IND-E02-DEFAULT-PLAY-001** | done |
| 2 | ☑ | **CONSTRUCTION-MV-SIM-001** | done |
| 3 | ☑ | **S7P-GRID-UX-UI-001** | done |
| 4 | ☑ | **LOG-E01-VISUAL-CONFIRM-001** | done |
| 5 | ☑ | **UI-P3-M3-UNITS-001** | done |
| 6 | ☑ | **UI-P3-M3-REPLAY-001** | done |
| 7 | ☑ | **REPLAY-PARITY-001** | done |
| 8 | ☑ | **TRIAGE-PHASE-D-PARITY-001** | done |
| 9 | ☑ | **UX-E02-APPLY-POLISH-001** | done |
| 10 | ☑ | **WAVE-S-SHELL-POLISH-001** | done |
| 11 | ☑ | **IND-E03-SIM-UX-001** | done |
| 12 | ☑ | **CONSTRUCTION-R4-PREP-001** | done |
| 13 | ☑ | **INFRA-VM-DEEP-001** | done |
| 14 | ☑ | **STAGE6-OPS-WITNESS-001** | done |
| 15 | ☑ | **S7B-M3-SIM-001** | done |
| 16 | ☑ | **FIRE7-F7-B-001** | done |
| 17 | ☑ | **FIRE7-F7-C-001** | done |

Detail: [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) § Coder B

---

## Closure and source of truth

Wave-3 dual-queue rows are witness-closed and reflected in `done_2026_05_26` in [`coder_active_queue.json`](../../tools/orchestrator/queues/coder_active_queue.json).  
Do not re-queue Coder A/B wave-3 checklist rows from this file.
