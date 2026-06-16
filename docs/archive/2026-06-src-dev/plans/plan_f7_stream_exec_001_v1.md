# PLAN-F7-STREAM-EXEC-001 — F7-B streaming depth execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-F7-STREAM-EXEC-001** |
| **Prior** | `fire7_streaming_depth_plan_v1.md` — [`fire7_streaming_depth_plan_v1.md`](fire7_streaming_depth_plan_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Coder lane** | **F7-STREAM-DEEP-001** (coder A primary; product depth) |
| **Status** | implemented (2026-05-26) — keep for regression + witness refresh |

---

## Scope

Increase product depth for **F7-B** fire chunk streaming by:
- making neighbor wake behavior deterministic in lib fixtures
- tying the sleep/wake transition threshold to Stage 6 residency window constraints
- extending `debug_runs/fire_streaming_live.json` with a witness field for neighbor wake observation

This plan is a depth-only extension: it must preserve the existing F7-A exit gate.

---

## Authority map (single writer per resource)

| Resource | Single writer | Allowed mutation | Must NOT be second-written by |
|:---|:---|:---|:---|
| `FireChunkRuntime` visual activity | `apply_fire_streaming_sleep_wake_system` | toggles `chunk.visual_active` based on sleep/wake rules | other fire extraction systems |
| `FireStreamingWitness` | `apply_fire_streaming_sleep_wake_system` | updates `sleep_transitions`, `wake_transitions`, `focus_chunk` | live proof writer; tests should only refresh |
| `debug_runs/fire_streaming_live.json` | `write_fire_streaming_live_proof_system` | write `FIRE7-F7-B-001` payload (and neighbor_wake_observed when added) | manual JSON edits |
| Sleep radius parameterization | fire streaming system logic (depth extension) | derive effective threshold from Stage 6 residency window | any other constant override in unrelated modules |

---

## Task list (B-DEEP-1..B-DEEP-3)

### B-DEEP-1 — Fixed-seed neighbor wake lib fixtures (deterministic)
1. Extend / refactor the existing deterministic test harness in `src/render/fire_streaming.rs` so the fixture always triggers:
   - `wake_transitions > 0` when the neighbor heat becomes hot
2. Keep test logic aligned with the runtime wake rule used in the main system.

Files (≤3):
- `src/render/fire_streaming.rs`

### B-DEEP-2 — Tie `FIRE_STREAMING_SLEEP_RADIUS` to Stage 6 residency window
1. Replace the fixed Chebyshev distance threshold with a value derived from Stage 6 residency constraints.
2. Contract for the derived value:
   - must be computable from Stage 6 residency data (`PerViewResidencyConsumerWindow` and/or `ChunkResidencyTable`) plus focus chunk (`CameraFocusDebug.focus_chunk`)
3. Determinism requirement:
   - derived radius must be pure and frame-stable given the same Stage 6 residency window and focus.

Files (≤3):
- `src/render/fire_streaming.rs`
- `src/render/per_view_residency.rs`
- `src/render/stage6_virtualization.rs`

### B-DEEP-3 — Witness extension: `neighbor_wake_observed: true`
1. Add a new field to the payload written by the fire streaming live proof:
   - `neighbor_wake_observed: bool`
2. Witness definition (must match runtime behavior):
   - `neighbor_wake_observed` is true when at least one neighbor-based wake occurs
   - practical rollup target: `neighbor_wake_observed == (wake_transitions > 0)`
3. Ensure the field appears in `debug_runs/fire_streaming_live.json`.

Files (≤3):
- `src/render/fire_streaming.rs`

---

## Witness JSON schema + green predicates

**File:** `debug_runs/fire_streaming_live.json`

Existing required fields (from current `FIRE7-F7-B-001` writer):
- `/gate: "FIRE7-F7-B-001"`
- `/green: bool`
- `/sleep_transitions: number`
- `/wake_transitions: number`
- `/focus_chunk: [x, y]`
- `/active_chunk_count: number`
- `/runtime_writer: true`

Extended depth fields for this exec plan:
- `/neighbor_wake_observed: bool`

Green predicate (baseline F7-B rollup):
```text
fire_streaming_b_green :=
  (sleep_transitions > 0 OR wake_transitions > 0)
  AND active_chunk_count > 0
```

Depth extension witness predicate:
```text
neighbor_wake_observed :=
  wake_transitions > 0
```

Regression guard (do not break):
- `fire7_f7_a_exit_001.green` remains true (F7-A product gate stays closed)

---

## Verification (required test commands)

```powershell
cargo test -p proc_A_dine01 --lib fire_streaming
cargo test -p proc_A_dine01 --lib stage6_virtualization
```

Then validate the written witness fields:
- `green: true`
- `neighbor_wake_observed: true` when neighbor wake should occur
- `sleep_transitions` / `wake_transitions` are non-zero in the extended deterministic fixture path

---

## Anti-patterns / do-not-reopen list (F7 streaming depth)

Do NOT:
- hand-edit `debug_runs/fire_streaming_live.json`
- loosen / bypass F7-A exit gating
- introduce a second global fire streaming writer or duplicate extraction
- change the meaning of `FIRE7-F7-B-001.green` to something not backed by transitions

