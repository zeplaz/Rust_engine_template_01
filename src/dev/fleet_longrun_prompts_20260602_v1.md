# Fleet long-run prompts — 2026-06-02 (v1.2)

Copy one block per agent session. **Drain in order** — do not skip because a later row looks easier.

**Snapshot:** [`fleet_snapshot_20260602_v3.md`](fleet_snapshot_20260602_v3.md) · **Workload:** [`fleet_coder_workload_queue_20260602_v1.md`](fleet_coder_workload_queue_20260602_v1.md)  
**Hub:** [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) · **Backlog:** [`coder_unified_backlog_v1.md`](coder_unified_backlog_v1.md) · **Proc/growth:** [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md)

---

## @coder A — long-run batch

```text
You are Coder A on Rust_engine_template_01 (branch master). Drain this queue in order; one slice per PR (≤3 files). Witness JSON wins. Update tools/orchestrator/queues/coder_active_queue.json when each row is done.

READ FIRST:
- src/dev/planner_program_alignment_v1.md (file territories)
- src/dev/coder_unified_backlog_v1.md § Coder A
- src/dev/plan_construction_stage_pipeline_exec_002_v1.md
- src/dev/construction_economy_growth_vision_v1.md (state builds + growth infill — your P2/PG work enables this)
- src/dev/agent_mcp_consumer_guide_v1.md § @coder (validation-first — you do NOT build tools/mcp/)

MCP / VALIDATION (required — consumer only):
- After cargo test/check: python -m rust_engine_mcp.cli validate-report cargo --compress 3
- Do NOT edit tools/mcp/python/ — route to @coder-mcp
- PG-2 meshes: request @designer-mcp batch; verify with validate-report asset_glb only

DRAIN ORDER (queue v5.4 — SET-P5/PG-1 CLOSED on disk):

1) CON-P3-S1 → S2 → S3 → CON-P3-WIT — scaling audit A-half (S4-S6 done on B)

2) INFRA-E0-003 → E1-001 → E1-002 → E2-001 → E2-002 → E3-003 → E4-002 → E5-002 → E6-001/002/004

3) PROC-PG-2-TAIL-001 — tier filter + procedural lib tests green

4) PROC-OG-4-001 — town rollup

5) PT-5-002 — fire frame tick in resolver

SKIP (done_2026_06_02): SET-P5-001/003, PROC-PG-1-001, CON-P2-*, fleet P2 tails

DO NOT: site_stage_tick.rs (B); Operational on commit; build MCP tools.

REGRESSION:
cargo test -p proc_A_dine01 --lib construction
python -m rust_engine_mcp.cli validate-report cargo --compress 3
```

---

## @coder B — long-run batch

```text
You are Coder B on Rust_engine_template_01 (branch master). Drain in order; ≤3 files per PR. Update coder_active_queue.json when done.

READ FIRST:
- src/dev/planner_program_alignment_v1.md
- src/dev/coder_unified_backlog_v1.md § Coder B
- src/dev/plan_construction_stage_pipeline_exec_002_v1.md
- src/dev/plan_organic_growth_exec_001_v1.md
- src/dev/construction_economy_growth_vision_v1.md (private infill + market saturation — your OG slices)
- src/dev/agent_mcp_consumer_guide_v1.md

MCP / VALIDATION (consumer only):
- validate-report cargo/bevy after tests — never paste raw compiler output
- Do NOT run Blender or edit tools/mcp/

DRAIN ORDER (queue v5.4 — OG/SET-P5 CLOSED on disk):

1) CON-PARAM-PARTIAL-ALPHA-001 — partial_alpha true on construction_parametric_placement_001

2) FIX-PROC-TEST-REGRESS-001 — construction::procedural 0 failures (module index / tile quarantine)

3) INFRA-E4-003 → E4-004 → E5-003 → INFRA-E3-WIT-001

4) PROC-OG-UX-WIRE-001 — growth approve HUD (design_organic_growth_ux_v1.md PASS)

5) PT-4-004/005 — power/night + damage variant inputs

6) FIX-BQ128-WIT-001, FIX-S7P-MV-PROOF-001 — witness/proof hygiene

SKIP (done_2026_06_02): SET-P5-002, ECON-OG-1-A/B/C, PROC-OG-2/3, CON-P3-S4-S6, CON-P2-002, infra E0-E3 partial

GROWTH RULES: proposals only; same funnel as player; no Operational on enqueue.

REGRESSION:
cargo test -p proc_A_dine01 --lib construction stage7
python -m rust_engine_mcp.cli validate-report cargo --compress 3
```

---

## @designer — long-run batch

**Status (2026-06-02): Six-phase proc/growth CLOSED — see fleet_planner_designer_prompts_20260602_v2.md for on-call mode.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Initial A/B long-run |
| v1.1.0 | 2026-06-02 | MCP consumer + economy growth vision hooks |
| v1.2.0 | 2026-06-02 | Post-return reconcile; link workload queue v1 |
