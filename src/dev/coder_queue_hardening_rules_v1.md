# Coder queue hardening rules `v1` (2026-06-13)

**Problem:** @coder closes rows on lib unit tests + `lib_green: true` while product witnesses show `green: false` and counters at zero.

**Authority:** Applies to all drains in `tools/orchestrator/queues/coder_*_drain_queue.json`.

---

## Forbidden done signals

A row **cannot** be `status: done` if any of these are true:

| Signal | Example |
|:---|:---|
| `lib_test_only` | Only `cargo test --lib` passed; no witness refresh |
| `witness_lib_green_without_green` | `lib_green: true` but `green: false` on disk |
| `witness_counter_zero` | Required counter (`fire_disturbances`, `chunks_with_program`, etc.) below `exit_predicate` |
| `single_chunk_pilot` | Map work still scoped to `LG1_PILOT_CHUNK` only |
| `eval_math_without_render` | Topology counts in eval JSON; preview raster unchanged |
| `snag_present` | Row has `snag` field non-empty |
| `operator_visible_missing` | Phase C+ row without `operator_visible: true` in witness |
| `forbidden_env_bootstrap` | Witness only green under `RUST_ENGINE_*` test env |

---

## Required row fields (v3+)

Every drain row must include:

```json
{
  "exit_predicate": {
    "witness": "debug_runs/<file>.json",
    "must": [{ "path": "field", "gte": 1 }, { "path": "green", "eq": true }]
  },
  "forbidden_exit": ["lib_test_only", "witness_counter_zero"],
  "verify_commands": ["cargo test -p proc_A_dine01 --lib <filter>"],
  "live_sim_required": false,
  "operator_visible": false
}
```

`live_sim_required: true` → witness must be refreshed from FULL_APP / play scenario / headless sim harness, not test-only fixture.

`operator_visible: true` → witness must include `operator_visible: true` or named play key (e.g. `veg_topology_visible_at_operational_zoom`).

---

## Phase gates (vegetation)

| Phase | Minimum before next phase |
|:---|:---|
| **A** | `fire_disturbances >= 1` AND `construction_disturbances >= 1` AND lg2 `green: true` |
| **B** | `chunks_with_program >= 16` AND `presets_used >= 3` AND map rollout `green: true` |
| **C** | Preview `operator_visible: true` AND play_scenario veg key AND stage5 `ecology_active_rows > 0` |
| **D** | lg3 witness `green: true` without district hack |
| **E** | Snapshot round-trip + `instance_count > 0` in witness |
| **F** | Program close rollup all phase A–E `green: true` |

---

## Reopen policy

When audit finds `status: done` but witness fails `exit_predicate`:

1. Set row `status: reopened`
2. Add `reopen_reason` + `reopen_date`
3. Add `superseded_by` only if replaced by finer slices — do not delete history
4. Increment queue `_meta.version`

**Trust order:** on-disk witness JSON > `vegetation_system_honest_status_v1.md` > queue `status` field.

---

## Coder discipline

```text
Q✓ row → read witness JSON fields in exit_predicate → ALL must pass → then mark done
Never mark done from cargo test stdout alone.
One row = one PR-sized slice (≤3 files unless witness-only refresh).
```
