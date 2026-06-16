`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# Operations Intelligence

Readonly pipeline + agent **ops analyst** — DSM surfaces, Q/C/E scores, complexity budget, ΔWF routing. **Does not implement fixes.**

## When to use

- After Track A/B/C lane close or warehouse spine attempt
- Before major architecture commits or new telemetry infra
- When HANDOFF and witnesses disagree (QUEST-loop lock)
- Proposal stress-test (value/complexity ≥ 1.0 gate)

## AGENT-LANG ritual (attach [agent-lang](../agent-lang/SKILL.md))

```text
BLANG:PRE → ops scan → BLANG:HO → emit T[c,d,a,φ] + ΔWF table → ⟨COMMIT:WIT⟩
```

**Tensor read:** `$ref:tools/orchestrator/queues/master_chain_tensor_v1.json` — collapse to ≤20 lines.

**Status paste:** max 3 emoji per program row · `⟨ID⟩` + `$ref:` only in routing package.

## Quick workflow

1. Run scan (all lanes):
   ```powershell
   powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
   ```
2. Read **only** structured fields:
   - `debug_runs/agent_ops/ops_report_latest.json` — `dsm_snapshot`, `qce`, `delta_wf`, `program_summary`
   - `debug_runs/unified_witness_index.json` — `programs.<id>`, `construction_sub_witnesses`
   - `tools/orchestrator/queues/OPS_LANE_REGISTRY.json` — program owners + HANDOFF priorities
3. Cross-check `tools/orchestrator/queues/HANDOFF.md` for human intent — not as sole track truth
4. Emit routing package (≤20 lines DSM + ΔWF table)
5. Route ECS/viewport drift to **debug-intelligence** — do not duplicate

## Witness spine (Track D) — all programs

| Artifact | Role |
|:---|:---|
| `OPS_LANE_REGISTRY.json` | stage5, fire_vfx, construction, infra, economy, wave, art A/B/C |
| `unified_witness_index.json` | All `*_live.json` by `program_id` |
| `agent_ops/ops_report_latest.json` | DSM + Q/C/E + ΔWF + `program_summary` |
| `construction_sub_witnesses` | Nested rows from `construction_stage_live.json` |
| `agent_debug_index.json` | Rust refresh on sim proof write |

**Honest gate classes:** `honest_green` · `dishonest_gate` · `schema_only` · `done_no_ship_flag`

Reject re-queue when `honest_gate: dishonest_gate` without operator ΔWF.

## DSM lexicon (compressed)

```text
AUTH: MAT★⇢APS★⇢SNAP★⇢WRK○⇢ATL○⇢RT○
FLOW: ART◇⇢APS⇢SNAP⇢WRK⇢PNG⇢ATL⇢RT
LOOP: RUN⇢TEL★⇢KPI★⇢OPS★⇢ΔWF↺
```

## Agent routing

| Finding | Owner |
|:---|:---|
| SNAP / MAT authority | `@sim-steward` + `@coder-mcp` |
| WRK dishonest bake | `@orchestrator-mcp` + `@designer-mcp` |
| APS preview gaps | `@coder-mcp` + `@designer` |
| Grammar/content | `@planner-mcp` Track C |
| Agent waste / wrong lane | `@orchestrator` + HANDOFF |
| ECS viewport drift | `@debug-intelligence` |

## Complexity budget (new proposals)

```text
Proposal Complexity: _ / 10
Expected Value: _ / 10
Recommendation: APPROVE | REVISE | DEFER | REJECT
```

Prefer Phase 1 JSON events over PostgreSQL until value/complexity ≥ 1.0.

## Related

- **agent-lang** — normative BLANG, stream delimiters, `$ref` grammar
- Agent: [`.cursor/agents/operations-intelligence.md`](../../agents/operations-intelligence.md)
- Plan: [`src/dev/plan_agent_operations_intelligence_v1.md`](../../src/dev/plan_agent_operations_intelligence_v1.md)
- Three-track: [`src/dev/plan_three_track_execution_v1.md`](../../src/dev/plan_three_track_execution_v1.md)
- Complements: **debug-intelligence**, **validation-first**, **mcp-production-rules**
