# PLAN-WITNESS-QUEUE-INTEGRITY-001 — MCP prevention toolchain `v1`

```text
⟦SYMLANG⟧⟐v1  ◈EXEC
⟨ID⟩ PLAN-WITNESS-QUEUE-INTEGRITY-001
Date: 2026-06-07
Status: **READY** (@planner — handoff to @coder-mcp)
Owner: @coder-mcp (tools) · @coder (Rust refresh guards, Phase 6)
Parent audit: queue/witness reconcile session 2026-06-07
Authority: $ref:src/dev/coder_queue_hardening_rules_v1.md
           $ref:docs/archive/2026-06-src-dev/plans/witness_exec_shape_v1.md
           $ref:tools/orchestrator/scripts/ops_witness_index.py
Queue: $ref:tools/orchestrator/queues/mcp_active_queue.json#MCP-WITNESS-INTEGRITY-*
```

**Goal:** Ship **automated gates** so agents cannot mark queue rows `done` or refresh rollup witnesses `green` when sub-fields contradict product truth — **before** bulk reopen of stale rows.

**Non-goals (this program):**
- Reopening vegetation / G-PLAY / phase5 rows (separate `@coder` + `@orchestrator` slice **after** tools green)
- Fixing `refresh_lg4_preview_witness` logic ( `@coder` — blocked on MCP-WIT-004 rule ship)
- Auto-mutating queue JSON to `reopened` without operator ΔWF review

---

## Problem (recurring failure mode)

```text
cargo test --lib  →  refresh_*_witness()  →  "green": true
       ↓
agent_queue_update(id, "done")     queue board shows 78/82 done
       ↓
operator runs game                 no trees / no tint / G-PLAY still OPEN
```

Root causes the tools must catch:

| Class | Example on disk |
|:---|:---|
| **Inflated green** | `green: true` + `topology_tint_visible_chunks: 0` + `pixel_heterogeneity_wired: false` |
| **Rollup without sub-fields** | `veg_runtime_proof_live` checks child `green` only |
| **Lib fixture as operator** | `operator_visible: true` + `proof_grade: lib_fixture` |
| **Queue drift** | `VEG-F02` done in `mcp_active_queue`, blocked in `coder_vegetation_drain` |
| **Done without exit_predicate** | drain row `status: done` but witness `must` fails |
| **Art schema-only ship** | `validator_status: passed` + `art_quality: rejected_*` (partial — extend honest_gate) |
| **Gate label drift** | tile batch `gates.G4: planned` + sibling keyframe `proceed_ship: pass` |
| **APS tree green** | import broken but JSON untouched (mitigate via import gate in refresh path) |

---

## Architecture

```text
Rule catalog (JSON)          Shared by CLI + MCP + ops scan
        │
        ▼
validate-report witness_honesty [--scan]
        │
        ├──▶ witness_brief(..., profile=honesty)  — agent consumption
        │
        ├──▶ validate-report queue_integrity        — cross-queue + done rows
        │
        └──▶ ops_witness_index.py (honest_gate v2) — rollup scan

Hooks (fail closed when enabled):
  post_build.ps1          → witness_honesty scan (warn → fail on CI)
  agent_queue_update      → optional --enforce-exit-predicate
  refresh_*_witness (Rust) → @coder Phase 6 — calls rule ids via CLI subprocess or shared JSON
```

**Trust order (unchanged):** witness sub-fields + rule report > queue `status` > markdown status docs.

---

## Deliverables (@coder-mcp)

### Phase 1 — Rule catalog + single-witness validator

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **MCP-WIT-001** | `tools/mcp/schemas/witness_integrity_rules_v1.schema.json` | jsonschema validates catalog |
| **MCP-WIT-002** | `tools/mcp/schemas/witness_integrity_rules_v1.json` | ≥12 rules (see §Rules) |
| **MCP-WIT-003** | `rust_engine_mcp.validators.witness_honesty` | Returns `ValidationReport` compress 3–4 |
| **MCP-WIT-004** | CLI `validate-report witness_honesty <path>` + MCP `validate_report` kind | Parity with construction validator |
| **MCP-WIT-005** | CLI `validate-report witness_honesty --scan debug_runs` | Scans `*_live.json`; caps issues at compression level |
| **MCP-WIT-006** | Witness `debug_runs/mcp_witness_honesty_validator_live.json` | Self-test: known-bad fixtures FAIL, known-good PASS |

**Rule ids (minimum v1):**

| rule_id | When FAIL |
|:---|:---|
| `WIT-GREEN-TINT-ZERO` | `green==true` ∧ `topology_tint_visible_chunks==0` ∧ path matches `landscape_grammar_lg4*` |
| `WIT-OPERATOR-LIB-FIXTURE` | `operator_visible==true` ∧ (`proof_grade==lib_fixture` ∨ `_agent_meta.agent_commands` all `--lib`) |
| `WIT-ROLLUP-CHILD-ONLY` | path in rollup registry ∧ child fails any mandatory sub-rule |
| `WIT-PHASE-CLOSE-WITHOUT-SUB` | `phase_*_green` all true ∧ any listed child witness fails honesty |
| `WIT-GATE-DRIFT-G4` | `gates.G4==planned` ∧ sibling keyframe witness has `proceed_ship/pass` |
| `WIT-TINY-PNG-PILOT` | `png_count>=1` ∧ any `png_dimensions[].bytes < 512` ∧ `ship!=false` |
| `WIT-ART-DISHONEST` | `green==false` ∧ `art_quality` starts with `rejected` → **PASS** (report only); `green==true` ∧ `art_quality` rejected → **FAIL** |
| `WIT-ENV-BOOTSTRAP-ONLY` | witness green ∧ `_agent_meta.logging_env` has test-only flags ∧ `live_sim_required` registry hit |
| `WIT-MISSING-ENVELOPE` | `*_live.json` without `_agent_meta.schema` (warn v1, fail v2) |
| `WIT-EXIT-PREDICATE` | queue row `done` ∧ witness path fails `exit_predicate.must` |
| `WIT-QUEUE-CONTRADICTION` | same task `id` different `status` across queue registry |
| `WIT-SNAG-DONE` | row `status==done` ∧ `snag` non-empty |

**Rollup registry (initial):**

```json
[
  "debug_runs/veg_runtime_proof_live.json",
  "debug_runs/vegetation_program_close_live.json",
  "debug_runs/g_play_product_close_live.json",
  "debug_runs/mcp_landscape_sign_atlas_live.json"
]
```

---

### Phase 2 — Queue cross-sync + exit_predicate gate

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **MCP-WIT-010** | `tools/mcp/schemas/queue_registry_v1.json` | Lists authoritative queue paths + id field name |
| **MCP-WIT-011** | `rust_engine_mcp.validators.queue_integrity` | Emits `ValidationReport` |
| **MCP-WIT-012** | CLI `validate-report queue_integrity` + `--queue <path>` optional filter | Finds ≥6 known contradictions on current repo (report-only) |
| **MCP-WIT-013** | `agent_queue_update(..., enforce=True)` optional flag | Returns error if row lacks `exit_predicate` or witness fails WIT-EXIT-PREDICATE |
| **MCP-WIT-014** | Witness `debug_runs/queue_integrity_reconcile_live.json` | `green: false` until contradictions triaged; lists stale ids |

**Queue registry must include:**

- `coder_vegetation_drain_queue.json`
- `mcp_active_queue.json` (flatten `p2_tasks` + `coder_mcp_drain.done_*`)
- `post_drain_phase5_queue.json`
- `post_drain_phase4_queue.json`
- `post_drain_phase6_coder_queue.json`
- `coder_master_drain_queue.json`

**Status normalization map:**

```text
done | lib_done | signed  →  "closed"
blocked | paused | deferred  →  "open"
ready | active  →  "ready"
```

Contradiction = same `id`, one queue `closed`, another `open`.

---

### Phase 3 — OPS scan honest_gate v2 + hooks

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **MCP-WIT-020** | Extract shared rule engine to `tools/orchestrator/scripts/witness_honesty_lib.py` | Imported by MCP validator + ops_witness_index (no duplicate logic) |
| **MCP-WIT-021** | Upgrade `_honest_gate()` in `ops_witness_index.py` | New classes: `inflated_green`, `rollup_inflated`, `queue_stale` |
| **MCP-WIT-022** | `ops_intelligence_scan.ps1` exit code 1 when `$env:RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE=1` and FAIL count > 0 | CI opt-in |
| **MCP-WIT-023** | `post_build.ps1` calls `validate-report witness_honesty --scan` | Default warn; enforce via env |
| **MCP-WIT-024** | Witness `debug_runs/mcp_witness_integrity_ops_live.json` | Documents hook wiring |

**DSM rollup change:** `E_confusion_risk` must include `inflated_green_count` + `queue_contradiction_count`.

---

### Phase 4 — Agent BLANG + brief profiles

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **MCP-WIT-030** | `witness_brief(path, profile="honesty")` | Returns failed rule ids only (compress 4) |
| **MCP-WIT-031** | `slice_exec_brief` includes `exit_predicate` + last honesty scan result | Agents see blockers before Q✓ |
| **MCP-WIT-032** | `token_savings_guide` → key `witness_integrity` | Documents BLANG:WIT-HON |
| **MCP-WIT-033** | Register Tier **1i** in `MICRO_TOOLS_REGISTRY_v1.md` | Table shipped |

**New BLANG token:**

```text
BLANG:WIT-HON  →  validate-report witness_honesty <path>|--scan
                   validate-report queue_integrity
```

**Agent rule (all roles):** `BLANG:Q✓` **forbidden** if `BLANG:WIT-HON` FAIL on row witness path + rollup parents.

---

### Phase 5 — CI + regression fixtures

| ID | Deliverable | Acceptance |
|:---|:---|:---|
| **MCP-WIT-040** | `tools/mcp/python/tests/test_witness_honesty.py` | Fixtures for each rule id (pass + fail) |
| **MCP-WIT-041** | `tools/mcp/python/tests/test_queue_integrity.py` | Synthetic queue pair contradiction |
| **MCP-WIT-042** | `tools/mcp/python/tests/test_aps_imports.py` chained in witness refresh gate | APS refresh refuses green if import fails |
| **MCP-WIT-043** | `tools/orchestrator/ci/run.ps1` optional stage `-WitnessIntegrity` | Runs when env set |

**Fixtures directory:** `tools/mcp/schemas/examples/witness_honesty_fixtures/` (bad/good pairs — teachable `_meta.teaches`).

---

## Phase 6 — @coder handoff (not @coder-mcp)

After MCP-WIT-001…043 **green**, `@coder` implements Rust-side **write guards** (separate queue):

| ID | Deliverable |
|:---|:---|
| **WIT-RUST-001** | `refresh_lg4_preview_witness*` fails WIT-GREEN-TINT-ZERO (no green when tint=0) |
| **WIT-RUST-002** | `refresh_veg_runtime_proof_live_witness` evaluates child sub-rules, not top-level green only |
| **WIT-RUST-003** | `refresh_product_verify_live_witnesses` requires `proof_grade != lib_fixture` for G-PLAY rollup green |
| **WIT-RUST-004** | `debug_run_envelope` helper: `assert_witness_honesty_before_write(path, body)` → subprocess CLI |

**Do not start WIT-RUST-* until** `debug_runs/mcp_witness_honesty_validator_live.json` green.

---

## Bulk reopen program (after tools — not now)

Separate orchestrator slice **`PLAN-WITNESS-REOPEN-001`**:

1. Run `validate-report queue_integrity` → emit `debug_runs/witness_reopen_candidates_v1.json`
2. Operator reviews table → approves reopen list
3. Script sets `status: reopened` + `reopen_reason` per `coder_queue_hardening_rules_v1.md`
4. `@coder` / `@coder-mcp` fix witnesses row-by-row with tools enforcing gates

---

## Verification (exit gate for this program)

```powershell
cd tools/mcp/python
python -m pytest tests/test_witness_honesty.py tests/test_queue_integrity.py -q
python -m rust_engine_mcp.cli validate-report witness_honesty --scan debug_runs --compress 3
python -m rust_engine_mcp.cli validate-report queue_integrity --compress 3
python -m rust_engine_mcp.cli mcp-witness-integrity-witness   # refreshes MCP-WIT-006 + MCP-WIT-014 + MCP-WIT-024
powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
```

**Program green when:**

- All MCP-WIT-001…043 witnesses green
- `validate-report witness_honesty --scan` returns structured FAIL list (expected on current repo — **report-only mode OK**)
- `agent_queue_update(..., enforce=True)` blocks a synthetic bad done in test
- `MICRO_TOOLS_REGISTRY_v1.md` Tier 1i registered

---

## Dependency order

```text
MCP-WIT-001..006 (rules + single witness validator)
      ▼
MCP-WIT-010..014 (queue cross-sync)
      ▼
MCP-WIT-020..024 (ops + hooks)
      ▼
MCP-WIT-030..033 (agent BLANG)
      ▼
MCP-WIT-040..043 (CI)
      ▼
ΔWF → @coder WIT-RUST-*  →  PLAN-WITNESS-REOPEN-001 (bulk triage)
```

**Parallel OK:** MCP-WIT-040 tests written alongside 003 (TDD).

---

## Sign-off

| Role | Action |
|:---|:---|
| **@orchestrator-mcp** | Pick MCP-WIT-001; block art/veg reopen until Phase 1 witness green |
| **@coder-mcp** | Implement Phases 1–5 only |
| **@coder** | Phase 6 Rust guards after MCP green |
| **Operator** | Set `RUST_ENGINE_WITNESS_INTEGRITY_ENFORCE=1` on CI when ready |

```text
⟦/PLAN-WITNESS-QUEUE-INTEGRITY-001⟧  ΔWF→@coder-mcp MCP-WIT-001
```
