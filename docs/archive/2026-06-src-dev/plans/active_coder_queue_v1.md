# Active coder queue `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.6.0` |
| **Date** | 2026-05-26 |
| **Wave 3 full lists** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) |
| **Checklist compact** | [`coder_dual_queue_todos_v2.md`](coder_dual_queue_todos_v2.md) |
| **Machine queue** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) v3.0 |
| **Prior closure** | [`coder_dual_queue_todos_v1.md`](coder_dual_queue_todos_v1.md) — 28 IDs |

---

## Status snapshot

**Wave 3 active** — pick **one P1** per coder per session.

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib coder_a_dual_queue coder_b_queue_bundle
```

| Track | Coder A active | Coder B active | Blocked |
|:---|:---:|:---:|:---|
| Count | 12 queued | 15 queued | F7-B/C until **FIRE7-F7-A-EXIT-001** |

---

## @coder A — P1 primary (pick one)

| ID | Task |
|:---|:---|
| **FIRE7-F7-A-EXIT-001** | F7-A **product** gate — unblocks F7-B/C |
| **VFX-VISUAL-SIGNOFF-001** | `--test visual` P2 VFX |
| **TRIAGE-GPU-TILE-WGSL-001** | WGSL instanced tiles |
| **TRIAGE-VISUAL-TEARDOWN-001** | VR-02 teardown |
| **UI-WP-VISUAL-001** | World preview visual sign-off |
| **S7B-M4-SIM-001** | M4 play in sim |

Full list: [`coder_dual_queue_todos_v2.md`](coder_dual_queue_todos_v2.md)

---

## @coder B — P1 primary (pick one)

| ID | Task |
|:---|:---|
| **S7P-GRID-UX-UI-001** | Grid overload toast in sim |
| **CONSTRUCTION-MV-SIM-001** | MV construction ghosts in sim |
| **IND-E02-DEFAULT-PLAY-001** | Industrial default in play |
| **UI-P3-M3-UNITS-001** | Minimap unit markers |
| **REPLAY-PARITY-001** | Replay editor parity |

Full list: [`coder_dual_queue_todos_v2.md`](coder_dual_queue_todos_v2.md)

---

## Suggested parallel pair (cycle 1)

| Coder A | Coder B |
|:---|:---|
| **FIRE7-F7-A-EXIT-001** | **S7P-GRID-UX-UI-001** |

Disjoint domains: fire extract vs industrial HUD.

---

## Operator / design

| Owner | ID |
|:---|:---|
| @operator | OPS-F01 · OPS-F03 · VFX-CAPTURE-INSIM-001 |
| @designer | FIRE7-DESIGN-001 · S7P-DESIGN-002 |

---

## Closed — do not re-queue

Dual-queue v2 (28 IDs) · UI shell Wave 3 · S7B M1–M4 witness · minimap M1–M4 · VM-09 v2 · WC-D04

**Note:** v2 **FIRE7-F7-A-001** = witness bundle only → product work is **FIRE7-F7-A-EXIT-001**.
