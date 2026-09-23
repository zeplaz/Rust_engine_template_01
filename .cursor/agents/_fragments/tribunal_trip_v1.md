# Tribunal trip fragment (VSS-001) — all agents

**Program:** `$ref:src/dev/plan_vfx_spectator_scale_program_v1.md`  
**Registry:** `$ref:tools/orchestrator/queues/vss_trip_artifact_registry.json`  
**Touches:** `$ref:tools/orchestrator/queues/vss_tribunal_touch_registry.json`

---

## On session start (after agent-lang boot)

```text
1. agent_doc_reads_brief OR driver.mjs boot → check program_id VSS-001 active
2. If editing paths in tribunal registry → read linked TRIP_*.md stub
3. Never paste raw cargo walls — symbolic_build_digest_tool first
```

---

## On artifact touch

```text
EDIT path ∈ vss_tribunal_touch_registry
  → READ trip stub
  → RUN symbolic_build_digest_tool (if Rust touched)
  → WRITE debug_runs/tribunals/{trip_id}_{date}_{agent}.yaml
       verdict: ship | revise | dissent
       majority: { agent, rationale }
       dissent[]: OPTIONAL — always allowed, never deleted by majority
       routing: @orchestrator | @planner | @sim-steward
  → WIT-HON before Q✓
```

---

## Dissent policy

- Minority agents **may** file `dissent` even when majority says ship.
- Orchestrator resolves in HANDOFF — does not erase dissent rows.
- Long-term reflection > short-term compromise; complexity budget (Q/C/E) from `@operations-intelligence`.

---

## Token efficiency

| Do | Don't |
|:---|:---|
| symcodes + known_fixes | 200-line compiler output in chat |
| witness_brief | full tactical_map_debug JSON |
| L0 ops brief before L2 coder | parallel Tasks on same authority files |

---

## Fleet slot (~1/7)

One agent per session should be **reflection-only**: `@operations-intelligence` or `@coder-mcp` on VSS-T5 slices — builds encoding, tribunal hygiene, Postgres witness index (dev).
