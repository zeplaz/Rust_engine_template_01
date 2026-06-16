# Session bootstrap fragment (AGENT-LANG-004-RITUAL)

**Program:** `$ref:docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md` · **Brief:** `$ref:prompts/llm_agent_brief.md`  
**SYMLANG:** `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` — chart-first laws L1–L8 · FIELD◈ ⊂ SYMLANG  
**Telemetry:** `$ref:debug_runs/agent_ops/doc_reads.jsonl` · **Rollup:** `agent_doc_reads_brief()`

Replace `<AGENT>` with this agent file's `name:` frontmatter (e.g. `coder`, `planner-mcp`).

---

## Mandatory skill attach (ALL agents — every session)

```text
Attach: .cursor/skills/agent-lang/SKILL.md   (or @agent-lang in chat)
Stack domain skills ON TOP — never skip agent-lang for BLANG / $ref / validate-report
```

| Layer | Skill | When |
|:---|:---|:---|
| **Base** | [agent-lang/SKILL.md](../../skills/agent-lang/SKILL.md) | Session start · handoffs · queue · witnesses |
| Domain | bevy-simulation-grade · validation-first · debug-intelligence · … | Per role matrix below |

**Orchestrator rule:** dispatch orders must cite `$ref:` paths + `⟨ID⟩` — not prose-only task walls.

---

## Mandatory session chain (every turn / new chat)

```text
BLANG:STATS → BLANG:BOOT → BLANG:ROLE → BLANG:PRE → BLANG:Q+
```

| Step | MCP / CLI | Purpose |
|:---|:---|:---|
| **BLANG:STATS** | `agent_doc_reads_brief()` | Hot re-read paths · repeat-in-session · promotion hints |
| **BLANG:BOOT** | `agent_session_bootstrap(agent='<AGENT>')` | Ledger + digest for canonical brief stack (not full Read) |
| **BLANG:ROLE** | `agent_doc_touch(path, agent='<AGENT>', intent='ref')` | Role matrix paths below — one call per path |
| **BLANG:PRE** | `pipeline_preflight()` | Environment + queue stale check |
| **BLANG:Q+** | `agent_queue_next('<AGENT>')` or `handoff_brief()` | Pick slice / orient HANDOFF |

**End slice:** `BLANG:RUN` → `agent_run_append({slice_id, tools_called, witness})`

---

## Canonical brief stack (BLANG:BOOT touches these)

| Path | Section | Intent |
|:---|:---|:---|
| `prompts/llm_agent_brief.md` | **FIELD◈ · SYMLANG◈** | `orient` — fast legend + merged SYMLANG section |
| `prompts/SYMBOLIC_LANGUAGE.meta.md` | **SYMLANG** | `ref` — laws, chart forms A–P, bindings, EBNF (canonical in-repo) |
| `docs/archive/2026-06-src-dev/plans/agent_meta_grammar_v3_lattice.md` | ΩMETA-LATTICE | `ref` — STATE · FLOW · REVIEW clusters |
| `.cursor/skills/agent-lang/SKILL.md` | Session loop | `ref` — BLANG token map |
| `src/dev/agent_lang_v1.md` | BLANG spec | `ref` — $ref / stream delimiters |

**Rule:** Use returned **digest** from `agent_doc_touch` / bootstrap — **not** IDE `Read` unless `intent=implement`.

---

## Role reads (BLANG:ROLE — after bootstrap)

Touch via `agent_doc_touch(path, agent='<AGENT>', intent='ref|orient')`. Full matrix: `AGENTS.md` § Agent routing.

| Agent | Also touch (ref/orient) |
|:---|:---|
| `orchestrator` | `tools/orchestrator/NEXT.md`, `tools/orchestrator/queues/agent_queue.md` |
| `orchestrator-mcp` | `tools/mcp/README.md`, `tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`, `tools/orchestrator/queues/mcp_lane_order_v1.md`, `tools/orchestrator/queues/mcp_active_queue.json` |
| `planner` | `prompts/llm_agent_brief.md`, migration matrices in `tools/orchestrator/` |
| `planner-mcp` | `docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md`, MCP exec drafts |
| `coder` | `.cursor/skills/bevy-simulation-grade/SKILL.md` → `07-repo-authority-map.md`, `.cursor/skills/validation-first/SKILL.md` |
| `coder-mcp` | `tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`, `docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md` |
| `designer` | `prompts/guides/ui_boundary_guide_v1.md` |
| `designer-mcp` | MCP art exec plan, four MCP skills |
| `sim-steward` | bevy-simulation-grade + debug-intelligence + cleanup-completion-intelligence skills |
| `main-thread-orchestrator` | Same as sim-steward + `tools/orchestrator/queues/HANDOFF.md` |
| `debug-intelligence` | `.cursor/skills/debug-intelligence/SKILL.md` |
| `cleanup-intelligence` | `.cursor/skills/cleanup-completion-intelligence/SKILL.md` |
| `operations-intelligence` | `src/dev/plan_agent_operations_intelligence_v1.md`, `debug_runs/agent_ops/ops_report_latest.json` |
| `coparent-orchestrator` | `tools/orchestrator/queues/HANDOFF.md`, bevy-simulation-grade conflict matrix |

---

## Read telemetry + hot-path promotion (MCP project)

| Tool | When |
|:---|:---|
| `agent_doc_reads_brief(min_reads=2)` | Session start · ops review · before blaming "agents didn't read X" |
| `agent_doc_promote_hot_reads(min_reads=3)` | Paths read ≥3× without cache — writes MCP digest cache |
| `agent_doc_digest_cached(path)` | Prefer cached digest over re-touch when source mtime unchanged |

**Ledger:** `debug_runs/agent_ops/doc_reads.jsonl` — one row per `agent_doc_touch`.  
**Rollup witness:** `debug_runs/agent_ops/doc_reads_brief_latest.json`  
**MCP cache:** `tools/mcp/cache/agent_doc_digests/<path_slug>.json`

When `hot_paths` or `repeat_in_session` is non-empty → run `agent_doc_promote_hot_reads()` so repeated full orient reads shrink to cache hits.

---

## Anti-patterns (forbidden)

- Raw IDE `Read` on brief/plan/AGENTS without `agent_doc_touch` ledger row
- Re-read same path every turn without checking `agent_doc_reads_brief()` / cache
- Paste full file contents in chat when digest exists
- Skip `prompts/llm_agent_brief.md` FIELD◈ / SYMLANG◈ at session start
- NL status walls in HANDOFF or replies when a SYMLANG packet suffices (L1/L8)
