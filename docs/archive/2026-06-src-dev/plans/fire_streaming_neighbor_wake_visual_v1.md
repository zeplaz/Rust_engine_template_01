# DESIGN-F7-STREAM-001 — neighbor wake/sleep visuals `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-F7-STREAM-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** (witness on disk 2026-05-26) |
| **Unblocks** | `F7-STREAM-DEEP-001` (coder A — **done**; `neighbor_wake_observed: true`) |
| **Witness** | `debug_runs/fire_streaming_live.json` → `/gate`, `/green`, `/neighbor_wake_observed`, `/sleep_transitions`, `/wake_transitions`, `/runtime_writer` |
| **Do not break** | `/gate == "FIRE7-F7-B-001"` and `/green == true` |

---
## Purpose
Player-readable neighborhood sleep/wake behavior for **chunk streaming** in Fire Phase 7 (F7-B).

This is a visual contract only:
- no runtime mutation expectations
- no changes to extract passes

---
## Visual contract (tactical zoom)
At tactical zoom, the operator should be able to read:
1. **Sleep**: chunks that go inactive (visual dim / disabled state).
2. **Wake**: chunks re-enabled due to a hot neighbor influence window.
3. Transitions must be temporally stable (no rapid flicker in one frame unless the sim itself changes).

---
## Debug overlay alignment (for engineering triage)
Debug overlay names and label wiring must align with `docs/archive/2026-06-src-dev/plans/fire_streaming_debug_overlay_names_v1.md`:
- sleep/wake/active transitions telemetry lines
- `F7B gate`, `focus_chunk`, `sleep=`, `wake=`, `runtime_writer`

---
## Acceptance checklist (designer)
1. `/sleep_transitions` increases (witness-backed) and correlates with visual sleep changes.
2. `/wake_transitions` is expected to be > 0 once neighbor-wake depth is exercised; if currently 0, mark as **DEFER** for the wake portion in coder QA.
3. Visual transitions are readable at tactical zoom without requiring editor-only panels.

