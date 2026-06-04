# PLAN-REPLAY-RING-EXEC-001 — live replay ring execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-REPLAY-RING-EXEC-001** |
| **Prior / parent** | `replay_live_ring_impl_plan_v1.md` — `PLAN-REPLAY-LIVE-RING-001` (1-page / stub) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Coder lane** | **REPLAY-RING-LIVE-001** (coder B) |
| **Status** | **READY (planner finalized)** — active next-phase coder B plan |

**Planner sign-off:** PASS (2026-05-27). Queue alignment: archived `PLAN-REPLAY-RING-EXEC-001`.

---

## Coder handoff (acceptance)

| Field | Value |
|:---|:---|
| **Witness (parity)** | `debug_runs/replay_editor_parity_live.json` → `parity_green=true`, `replay_ring_len>=2` |
| **Witness (minimap)** | `debug_runs/minimap_compositor_live.json` → `replay_scrub_enabled=true`, `ui_p3_m3_replay_001_green=true` |
| **Unblocks** | `REPLAY-RING-LIVE-001` |
| **Verify** | `cargo test -p proc_A_dine01 --lib replay_editor_parity minimap_compositor` |

---

## Scope

Drive **product depth** for live replay scrubbing by ensuring that during **Simulation**:

1. `CommittedSimReplayRing.stamps` grows (not just seeded / static).
2. The minimap scrub presentation becomes active once the ring has depth.
3. Replay determinism / editor parity stays **green** (`REPLAY-PARITY-001`).

This plan is an expansion of `replay_live_ring_impl_plan_v1.md` into a detailed coder slice with explicit authority and verification.

---

## Authority map (single writer per resource)

| Resource | Single writer | Green evidence | Must NOT be second-written by |
|:---|:---|:---|:---|
| `CommittedSimReplayRing.stamps` | `record_committed_sim_replay_stamp()` (inside `SimFrameDeltaPlugin`) | ring has `stamps.len() >= 2` at proof/write time | any UI system, minimap compositor, manual edits |
| `CommittedSimReplayRing.capacity` | initialized at resource creation time (coder fix if Default is incorrect) | scrub can become active and ring can hold depth | any later mutation that breaks determinism |
| `MinimapCompositorState.replay_scrub_enabled` | minimap compositor pass `paint_replay_scrub()` | `replay_scrub_enabled: true` when ring has depth | manual JSON hacks |
| `debug_runs/minimap_compositor_live.json` | minimap compositor live proof writer | `ui_p3_m3_replay_001_green: true` | manual edits |
| `debug_runs/replay_editor_parity_live.json` | replay parity live proof writer | `parity_green: true` | manual edits |

---

## Task list (≤3 files per PR)

### R1 — ensure Simulation writes ring stamps (and ring can actually grow)
**Problem the exec plan must address:** a ring that never grows will keep scrub presentation inactive forever.

1. Verify that `SimFrameDeltaPlugin` runs during **Simulation** and the record system is connected to the correct fence stimulus.
2. Verify ring initialization:
   - `CommittedSimReplayRing` must have `capacity >= 2` (or equivalent non-trivial buffering) so `stamps.len()` can reach the scrub activation threshold.
   - If current initialization is using `Default` with `capacity` too small/zero, adjust resource creation to use `CommittedSimReplayRing::with_capacity(N)` where `N >= 2`.

Files (≤3):
- `src/systems/sim_frame_delta.rs`

### R2 — keep `MinimapOverlayMask.replay_scrub` compatible with the ring threshold

`paint_replay_scrub()` is expected to:
- return `false` when the ring has fewer than 2 stamps
- return `true` when the ring has depth and the compositor toggle is enabled

Coder must ensure no regressions in:
- mapping of `MinimapOverlayMask.replay_scrub` to compositor pass inputs
- compositor uses the same live ring resource in Simulation (not an editor-only stub)

Files (≤3):
- `src/render/minimap_compositor/composite.rs`

### R3 — verify scrub activation without breaking editor parity

1. Confirm `replay_editor_parity_live.json` still reports `parity_green: true`.
2. Confirm minimap proof reports scrub enabled and UI presentation green:
   - `replay_scrub_enabled: true`
   - `ui_p3_m3_replay_001_green: true`

Files (≤3):
- `src/dev/replay_editor_parity.rs` (only if parity witness scheduling needs adjustment)
- otherwise regression only (no functional code changes)

---

## Witness JSON schema + green predicates

### A) Editor parity witness
**File:** `debug_runs/replay_editor_parity_live.json`

Required fields for lane exit:
- `/parity_green: bool` (must be `true`)
- `/replay_ring_len: number` (expected `>= 2` when ring depth exists)

Green predicate:
```text
REPLAY-PARITY-001 green :=
  parity_green == true
  AND replay_ring_len >= 2
```

### B) Minimap scrub presentation witness
**File:** `debug_runs/minimap_compositor_live.json`

Required fields for lane exit:
- `/replay_scrub_enabled: bool` (must be `true`)
- `/ui_p3_m3_replay_001_green: bool` (must be `true`)

Green predicate:
```text
ui_p3_m3_replay_001_green :=
  replay_scrub_enabled == true

replay_scrub_enabled :=
  MinimapOverlayMask.replay_scrub == true
  AND CommittedSimReplayRing.stamps.len() >= 2
```

---

## Verification (required test commands)

Run:
```powershell
cargo test -p proc_A_dine01 --lib replay_editor_parity
cargo test -p proc_A_dine01 --lib minimap_compositor
```

Then confirm on disk (witness JSON):
- `debug_runs/replay_editor_parity_live.json`
  - `parity_green: true`
  - `replay_ring_len >= 2`
- `debug_runs/minimap_compositor_live.json`
  - `replay_scrub_enabled: true`
  - `ui_p3_m3_replay_001_green: true`

---

## Anti-patterns / do-not-reopen list (replay ring exec)

Do NOT:
- hand-edit any `debug_runs/*replay*_live.json` green fields
- change replay parity determinism semantics beyond what is required to keep `parity_green` stable
- move scrub rendering into a new extract pass (presentation-only: minimap compositor reads the ring)
- re-open Wave 4/5 closure specs for minimap M3 units/replay (M3 stack should remain consistent)

Also do not reopen:
- F7-A / F7-B / F7-C exit gates
- dual-queue closure rows
- steward preflights

