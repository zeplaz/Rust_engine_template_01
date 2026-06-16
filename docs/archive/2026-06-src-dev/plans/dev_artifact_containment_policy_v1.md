# DEV-ARTIFACT-CONTAINMENT-001 — Runtime witness and LLM artifact containment policy `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-28 |
| **Exec plan** | [`plan_dev_artifact_containment_exec_001_v1.md`](plan_dev_artifact_containment_exec_001_v1.md) (**PLAN-DEV-ARTIFACT-CONTAINMENT-EXEC-001**) |
| **Scope** | `src/**` runtime witness writers, live-proof modules, debug envelopes, planning artifacts |
| **Problem** | Dev/testing artifacts are spread across domain folders and look like production runtime code ownership |
| **Goal** | Keep release gameplay/runtime clean while preserving deterministic witness coverage for dev and CI |

---

## Policy statement

Dev/testing and LLM workflow artifacts must be **contained, discoverable, and disableable** without compromising release behavior.

This repo adopts a **single containment root** for operational dev artifacts:

- `src/dev/runtime_witness/` (new canonical home for witness writing, proof refresh, and debug JSON emit paths)

Allowed companion roots:

- `src/dev/` for plans, ledgers, and queue docs
- `debug_runs/` for generated outputs only

Disallowed end-state:

- New `*live_proof*.rs` modules under domain trees (`src/render/`, `src/construction/`, `src/economy/`, etc.)

---

## Non-negotiable release boundary

1. Release gameplay logic must not depend on witness file I/O success.
2. Witness writes are best-effort telemetry; sim correctness cannot require them.
3. If witness writing is disabled, app must still run and pass non-dev regression suites.
4. Domain modules may expose **read-only snapshot structs/functions**; file write orchestration lives in `src/dev/runtime_witness/`.

---

## Current spread (migration inventory)

Known `*live_proof*.rs` modules currently outside containment root:

- `src/render/stage6_live_proof.rs`
- `src/render/minimap_compositor/live_proof.rs`
- `src/render/view_runtime/live_proof.rs`
- `src/construction/live_proof.rs`
- `src/economy/activation/live_proof.rs`
- `src/economy/logistics/live_proof.rs`
- `src/io/streaming/wave_c_live_proof.rs`
- `src/io/save/wave_s_live_proof.rs`
- `src/systems/fire/live_proof.rs`
- `src/gui/editor/world_preview/wave_p_live_proof.rs`
- `src/dev/stage7_behavioral_live_proof.rs`
- `src/dev/stage7_play_live_proof.rs`

---

## Target architecture

### A) Domain-owned snapshots stay local

Domain folders keep pure data collectors only, for example:

- `gather_*_readiness(...)`
- `*_frame(...)`
- `*_witness_fields(...)` (no file writes)

### B) Witness writer ownership centralizes

All JSON/markdown emit + envelope wrapping centralizes in:

- `src/dev/runtime_witness/mod.rs`
- `src/dev/runtime_witness/<lane>.rs`

These modules call domain snapshot collectors and perform:

- `wrap_debug_run(...)`
- `write_debug_run_json(...)`
- write cadence/timers
- lane-level refresh helpers used by tests

### C) Explicit runtime gate (Slice B)

Implemented in [`runtime_witness/gate.rs`](runtime_witness/gate.rs) + [`runtime_witness/io.rs`](runtime_witness/io.rs).

Introduce one global enable switch for witness writing:

- `RUNTIME_WITNESS_WRITES=1` (dev/ci)
- default off for production packaging/profile

Behavior:

- when off: collectors can still run in memory; file writes are skipped
- when on: current live JSON behavior remains

---

## Migration plan (phased, safe)

### Phase 1 — Freeze and map (immediate)

1. No new `live_proof` files outside `src/dev/runtime_witness/`.
2. Every existing out-of-root writer gets a migration row in this doc.
3. Add PR checklist item: "Does this add witness I/O outside containment root?"

### Phase 2 — Extract writers

1. For each lane, split module into:
   - local collector (domain)
   - centralized writer (`src/dev/runtime_witness`)
2. Keep temporary re-export shims in old location with deprecation note.
3. Remove direct `std::fs` writes from domain modules.

### Phase 3 — Enforce

1. Linter/check script: [`tools/orchestrator/scripts/check_live_proof_containment.ps1`](../../tools/orchestrator/scripts/check_live_proof_containment.ps1)
   - deny new paths matching `src/**/live_proof.rs` outside `src/dev/runtime_witness/` (except [`exceptions_manifest.json`](runtime_witness/exceptions_manifest.json) shims)
2. CI: **warning mode** (Slice B–C) via `.github/workflows/ci.yml` + `tools/orchestrator/ci/run.ps1`; **hard fail** after Slice D (`-HardFail`).
3. Remove migration shims once callers move (open PR after stage6/view_runtime Slice C).

---

## Lane migration matrix

| Lane | Current writer path | Target writer path |
|:---|:---|:---|
| Stage 6 virtualization | `src/render/stage6_live_proof.rs` (shim) | `src/dev/runtime_witness/stage6.rs` **DONE Slice C** |
| Minimap compositor | `src/render/minimap_compositor/live_proof.rs` | `src/dev/runtime_witness/minimap.rs` |
| View runtime isolation | `src/render/view_runtime/live_proof.rs` (shim) + `witness_state.rs` | `src/dev/runtime_witness/view_runtime.rs` **DONE Slice C** |
| Construction | `src/construction/live_proof.rs` | `src/dev/runtime_witness/construction.rs` |
| Industrial activation | `src/economy/activation/live_proof.rs` | `src/dev/runtime_witness/industrial.rs` |
| Logistics throughput | `src/economy/logistics/live_proof.rs` | `src/dev/runtime_witness/logistics.rs` |
| Wave C streaming | `src/io/streaming/wave_c_live_proof.rs` (shim) | `src/dev/runtime_witness/wave_c.rs` **DONE Slice B** |
| Wave S save spine | `src/io/save/wave_s_live_proof.rs` (shim) | `src/dev/runtime_witness/wave_s.rs` **DONE Slice B** |
| Fire system | `src/systems/fire/live_proof.rs` | `src/dev/runtime_witness/fire.rs` |
| Wave P preview | `src/gui/editor/world_preview/wave_p_live_proof.rs` | `src/dev/runtime_witness/wave_p.rs` |
| Stage 7 behavioral | `src/dev/stage7_behavioral_live_proof.rs` | `src/dev/runtime_witness/stage7_behavioral.rs` |
| Stage 7 play | `src/dev/stage7_play_live_proof.rs` | `src/dev/runtime_witness/stage7_play.rs` |

---

## PR policy additions (effective now)

Every PR touching witness code must include:

1. Containment location check (inside `src/dev/runtime_witness` or justified shim).
2. Statement of release fallback when witness writes are disabled.
3. Witness JSON key compatibility note (if schema changed).

---

## Decision record

This policy does **not** delete witness capability; it relocates writer ownership and enforces clean boundaries so production structure remains coherent.
