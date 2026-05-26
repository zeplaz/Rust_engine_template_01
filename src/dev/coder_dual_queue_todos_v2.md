# Coder dual queue — open todos `v2` (wave 3)

| Field | Value |
|:---|:---|
| **Version** | `2.1.0` |
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

## @coder A — all 14 (default start: **#1**)

| # | ☐ | ID | If blocked → start |
|:---:|:---:|:---|:---|
| 1 | ☐ | **FIRE7-F7-A-EXIT-001** | — |
| 2 | ☐ | **VFX-VISUAL-SIGNOFF-001** | — |
| 3 | ☐ | **TRIAGE-GPU-TILE-WGSL-001** | — |
| 4 | ☐ | **TRIAGE-VISUAL-TEARDOWN-001** | — |
| 5 | ☐ | **TRIAGE-PHASE-F-CULL-001** | — |
| 6 | ☐ | **UI-WP-VISUAL-001** | — |
| 7 | ☐ | **INFRA-GPU-TILE-GIZMO-001** | — |
| 8 | ☐ | **S7B-M4-SIM-001** | — |
| 9 | ☐ | **VFX-CAPTURE-HOOK-001** | — |
| 10 | ☐ | **TRIAGE-COMPILE-HYGIENE-001** | — |
| 11 | ☐ | **FIRE7-DESIGN-LOD-WIRE-001** | **#2** or **#3** |
| 12 | ☐ | **STAGE5-VT-DEEP-001** | — |
| 13 | ☐ | **FIRE7-F7-B-001** | **#1** |
| 14 | ☐ | **FIRE7-F7-C-001** | **#1** |

Detail: [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) § Coder A

---

## @coder B — all 17 (default start: **#3**)

| # | ☐ | ID | If blocked → start |
|:---:|:---:|:---|:---|
| 1 | ☐ | **IND-E02-DEFAULT-PLAY-001** | — |
| 2 | ☐ | **CONSTRUCTION-MV-SIM-001** | — |
| 3 | ☐ | **S7P-GRID-UX-UI-001** | **#1** (placeholder copy ok) |
| 4 | ☐ | **LOG-E01-VISUAL-CONFIRM-001** | **#1** or **#5** |
| 5 | ☐ | **UI-P3-M3-UNITS-001** | — |
| 6 | ☐ | **UI-P3-M3-REPLAY-001** | — |
| 7 | ☐ | **REPLAY-PARITY-001** | — |
| 8 | ☐ | **TRIAGE-PHASE-D-PARITY-001** | — |
| 9 | ☐ | **UX-E02-APPLY-POLISH-001** | — |
| 10 | ☐ | **WAVE-S-SHELL-POLISH-001** | — |
| 11 | ☐ | **IND-E03-SIM-UX-001** | **#3** |
| 12 | ☐ | **CONSTRUCTION-R4-PREP-001** | **#2** |
| 13 | ☐ | **INFRA-VM-DEEP-001** | — |
| 14 | ☐ | **STAGE6-OPS-WITNESS-001** | — |
| 15 | ☐ | **S7B-M3-SIM-001** | — |
| 16 | ☐ | **FIRE7-F7-B-001** | **#3** or **#1** |
| 17 | ☐ | **FIRE7-F7-C-001** | **#2** or **#5** |

Detail: [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) § Coder B

---

## v2 closure (do not re-queue)

28 IDs — [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md). **FIRE7-F7-A-001** = witness only → A **#1** is product exit.
