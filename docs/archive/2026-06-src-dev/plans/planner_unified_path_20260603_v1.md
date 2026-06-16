# Unified planner path — fleet + MCP productivity `v1`

| Field | Value |
|:---|:---|
| **ID** | **PLANNER-UNIFIED-PATH-001** |
| **Date** | 2026-06-03 |
| **Status** | **SIGNED recommendation** |
| **Merges** | [`planner_improvement_analysis_20260603_v1.md`](planner_improvement_analysis_20260603_v1.md) · [`plan_mcp_productivity_chain_v1.md`](plan_mcp_productivity_chain_v1.md) |
| **Rule** | One orchestration story — **planner unblocks coders**; **coder-mcp unblocks agents**; same week, disjoint territories |

---

## Executive verdict

The planner MCP productivity proposal is **correct and should ship first for agent loops**. It does **not** replace planner audit/queue/exec work — it **runs in parallel** on `@coder-mcp` while `@planner` reconciles truth and writes thin exec slices for Rust lanes.

**Biggest token leak today:** agents re-read `AGENTS.md`, full witnesses, and `assembly_snapshot` JSON every session. **P0 MCP briefs fix that immediately.**

**Biggest product leak today:** G-PLAY-01 operator sign-off + organic approve→execute + infra E5-002 without exec. **Planner P0 fixes that for coders.**

---

## Combined build order (authoritative)

### Lane A — @planner (docs + queue, no code)

| P | ID | Est | Unblocks |
|:---:|:---|:---:|:---|
| **A0** | **PLAN-AUDIT-019** | 4h | Truth for everyone; remove stale CON-P2/P3/infra rows from v18 |
| **A0** | **PLAN-QUEUE-SYNC-002** | 2h | `coder_*_next`, HANDOFF, planner `next_phase` |
| **A0** | **PLAN-G-PLAY-CLOSE-001** | 1h | Operator checklist → v19 G-PLAY-01 row |
| **A1** | **Sign APS-VALIDATOR-PLAIN-001** | 1h | Same day as `validate_p0_gate_plain` ship |
| **A1** | **PLAN-CON-P7-LOGISTICS-001** | 3h | **INFRA-E5-002** + CON-P7 (G-INFRA-07 unblocked) |
| **A1** | **PLAN-ORG-GROWTH-EXEC-002** | 3h | **PROC-OG-APPROVE/POLICY** after ECON-OG-SAVE |
| **A1** | **PLAN-INFRA-TAIL-001** | 2h | E4-002, E6-001/002/004 one-liner acceptance |
| **A2** | **PLAN-DSM-WRK-ATL-001** | 2h | Closure criteria — **references MCP tools**, not duplicate specs |
| **A2** | **PLAN-DEFER-REGISTRY-001** | 1h | Track B keyframe + APS Phase 9 E2E stay DEFER |

**Planner does not wait for MCP P0** — start A0 immediately.

---

### Lane B — @coder-mcp (micro tools + APS Tk, `tools/mcp/` only)

| P | ID | Est | DSM / token |
|:---:|:---|:---:|:---|
| **B0** | **pipeline_preflight** | ½d | Replaces scattered ping/path checks |
| **B0** | **snapshot_digest** | ½d | **Highest ROI** — no full snapshot in chat |
| **B0** | **validate_p0_gate_plain** | ½d | **SNAP★ quality** + artist sentences |
| **B1** | **grammar_iterate** | 1d | Track C iteration without regen |
| **B1** | **snapshot_diff_brief** | ½d | APS heatmap / iterate feedback |
| **B1** | **tile_spine_run** | 1d | Warehouse integration — **not** Track A blocker |
| **B1** | APS Tk **Phases 2–4** | parallel | Catalog thumb, atlas validate UX, tooltips |
| **B2** | **atlas_meta_brief**, **material_profile_brief** | 1d | **ATL○ / MAT★** briefs |
| **B2** | **agent_run_append** + **ops_intelligence_scan** | 1d | OPS JSONL — no Postgres |
| **B2** | **tile_promotion_honest_check** | ½d | B2 headless-as-ship rejection class |

**Ship B0 before B1.** Update [`MICRO_TOOLS_REGISTRY_v1.md`](../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) + `token_savings_guide` on each B0 tool.

---

### Lane C — @coder A / B (Rust, parallel — no MCP dependency)

| Owner | Now | Blocked by planner? |
|:---|:---|:---|
| **B** | **ECON-OG-SAVE-001** → **PROC-OG-APPROVE-001** | APPROVE needs **PLAN-ORG-GROWTH-EXEC-002** |
| **A** | **INFRA-E4-002** → **E5-002** → **E6-*** | E5-002 needs **PLAN-CON-P7-LOGISTICS-001** |
| **C** | **WEATHER-WITNESS-001** | Downtime only |

**Construction lib:** 144/144 — do not re-open P1–P6.

---

### Lane D — @designer (on-call)

- **APS-UX-TOOLTIPS-002-REVIEW** when coder-mcp opens Phase 4 PR
- **APS-UX-POLISH-001-SIGNOFF** after Phase 5 lands
- **No** Track B keyframe (DEFER)

---

## DSM closure — who closes what

```
MAT★ → APS★ → SNAP★ → WRK○ → ATL○ → RT○
```

| Node | Closes via | Owner |
|:---|:---|:---|
| **APS★** | Phase 0 audit done; Phases 2–4 Tk | @coder-mcp |
| **SNAP★** | `validate_p0_gate_plain` + P0 gate green | @coder-mcp + planner sign plain doc |
| **WRK○** | BUILD-WORKER-001 + worker status plain copy | @coder-mcp Phase 8 + @coder Bevy |
| **ATL○** | APS-ATLAS-PREVIEW-002 + `atlas_meta_brief` | @coder-mcp |
| **RT○** | Deferred until WRK+ATL ★ | — |

**Planner PLAN-DSM-WRK-ATL-001** = witness key names + PASS table — **not** a second MCP spec.

---

## Agent ritual (enforce in HANDOFF — non-optional)

```text
Start:  token_savings_guide → pipeline_preflight → agent_queue_next
Touch:  snapshot_digest | validate_p0_gate_plain | witness_brief
End:    agent_queue_update(note=witness_path) → agent_run_append (after B2)
```

**Throughput:** one queue slice per turn; no full `AGENTS.md` + plan re-read every session.

---

## What not to add (both plans agree)

| Don't | Why |
|:---|:---|
| Postgres / dashboards | Before JSONL `agent_run_append` proves value |
| LLM-in-the-loop validators | Non-deterministic; breaks validation-first |
| MCP duplicates of APS Tk without headless path | Token waste + dual maintenance |
| Macro tools that don't beat `witness_brief` + CLI | Complexity budget |
| Re-open CON-P2/P3 or infra E0–E3 | Closed on disk |
| Un-defer Track B / APS E2E | Until WRK+ATL ★ |

---

## Week map (suggested)

| Day | Planner | Coder-mcp | Coders |
|:---|:---|:---|:---|
| **D1** | A0 audit + queue sync | **B0** preflight + digest + p0 plain | B: ECON-OG-SAVE; A: E4-002 |
| **D2** | Sign plain validator; start CON-P7 exec | B0 registry update; start APS Phase 2 | A: continue infra |
| **D3–D4** | CON-P7 + ORG-GROWTH exec | B1 grammar_iterate + APS 3–4 | B: APPROVE (after exec) |
| **D5** | DSM-WRK-ATL + defer registry | B1 tile_spine_run (warehouse test) | A: E5-002 (after CON-P7) |
| **W2** | PLAN-AUDIT-020 prep | B2 briefs + honest bake check | Operator G-PLAY runbook |

---

## Orchestrator paste

```text
Unified path: docs/archive/2026-06-src-dev/plans/planner_unified_path_20260603_v1.md

@planner (Lane A, start now):
  PLAN-AUDIT-019 → PLAN-QUEUE-SYNC-002 → PLAN-G-PLAY-CLOSE-001
  then PLAN-CON-P7-LOGISTICS-001 + PLAN-ORG-GROWTH-EXEC-002 + sign aps_validator_plain_language_v1.md

@coder-mcp (Lane B, parallel):
  P0: pipeline_preflight, snapshot_digest, validate_p0_gate_plain
  Then P1: grammar_iterate, snapshot_diff_brief, tile_spine_run + APS Phases 2–4
  Update MICRO_TOOLS_REGISTRY + token_savings_guide when shipped.

@coder A/B: construction drain + infra tail — no wait on MCP P0.

Agent ritual: token_savings_guide → pipeline_preflight → agent_queue_next
DEFER: Track B MCP-PILOT-GRAMMAR-001 manual keyframe · APS Phase 9 E2E
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Merged fleet planner analysis + MCP productivity chain |
