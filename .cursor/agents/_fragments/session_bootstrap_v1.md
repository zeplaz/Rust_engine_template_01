# Session bootstrap fragment (AGENT-LANG-004-RITUAL)

**Program:** `$ref:src/dev/plan_witness_queue_integrity_mcp_v1.md` · **Brief:** `$ref:prompts/llm_agent_brief.md`  
**SYMLANG:** `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` — chart-first laws L1–L8 · FIELD◈ ⊂ SYMLANG  
**Driver:** `.claude/skills/agent-lang/driver.mjs` · **Skill:** `.cursor/skills/agent-lang/SKILL.md`

Replace `<AGENT>` with this agent file's `name:` frontmatter (e.g. `coder`, `planner-mcp`).

---

## Skill parity (FIRST — every session, all agents)

**Empty or stale `.cursor/skills/*/SKILL.md` breaks BLANG** — agents lose queue/witness/validator protocol.

```text
1. Check .cursor/skills/agent-lang/SKILL.md is non-empty (>500 bytes)
2. If missing/empty/stale → run sync-claude-skills (see below)
3. Read the synced SKILL.md in this session (IDE Read or @agent-lang) — do NOT trust old MCP digest cache alone
4. After MCP tool changes in tools/mcp/python → reload Cursor MCP server (rust-engine-art)
```

| Action | Command |
|:---|:---|
| **Sync skills** | `powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1` |
| **Dry run** | `… sync.ps1 -WhatIf` |
| **Force overlay** | `… sync.ps1 -Force` |
| **Skill doc** | `$ref:.cursor/skills/sync-claude-skills/SKILL.md` |

**Authority:** `.claude/skills/` = authoring tree · `.cursor/skills/` = Cursor discovery base. Keep aligned.

**Domain skills** (read after agent-lang each session when in role matrix): validation-first, bevy-simulation-grade, debug-intelligence, mcp-production-rules, operations-intelligence, …

**VSS-001 / tribunal:** Only if the edit path is listed in `$ref:tools/orchestrator/queues/vss_trip_artifact_registry.json` — then read `$ref:.cursor/agents/_fragments/tribunal_trip_v1.md`. Use `symbolic_build_digest_tool` — never raw cargo walls. Not a global pre-edit gate.

**Lean defaults:** `$ref:prompts/guides/agent_orchestration_minimal_harness_v1.md` · rule `agent-orchestration-lean-path.mdc`.

---

## Harness tiers (default = L0)

| Tier | When | Ritual |
|:---|:---|:---|
| **L0 lean** | Bounded single-owner edit | Skill parity if empty; domain skill; `validate-report` on build/test |
| **L1 program** | HANDOFF lease, multi-domain, Task spawn | Full `driver.mjs boot <AGENT>` + `agent_flow_route` before Tasks |
| **L2 product close** | Queue row with `exit_predicate` | work → **WIT-HON** → WIT → **Q✓** |

Do **not** run the full SKILL-SYNC⊳BOOT⊳Q+ chain for every typo. Escalation personas (`@orchestrator`, `@main-thread-orchestrator`, `@coparent-orchestrator`, `@flow-controller`, `@operations-intelligence`) are **opt-in**.

---

## Mandatory skill attach (ALL agents)

```text
Base:  .cursor/skills/agent-lang/SKILL.md   (@agent-lang in chat)
Stack: domain skills ON TOP — never skip agent-lang for BLANG / $ref / validate-report
```

| Layer | Skill | When |
|:---|:---|:---|
| **Base** | [agent-lang/SKILL.md](../../skills/agent-lang/SKILL.md) | Every session · handoffs · queue · witnesses |
| Domain | per role matrix below | Before editing that lane |

---

## Session chain (L1/L2 — program slices and product closes)

```text
SKILL-SYNC ⊳ DRIVER-BOOT ⊳ PRE ⊳ Q+ ⊳ work ⊳ WIT-HON ⊳ WIT ⊳ Q✓
```

| Step | Command | Purpose | Tier |
|:---|:---|:---|:---|
| **SKILL-SYNC** | `sync.ps1` if agent-lang empty/stale | Repair broken skill attach | L0+ |
| **DRIVER-BOOT** | `node .claude/skills/agent-lang/driver.mjs boot <AGENT>` | PRE + read brief/SYMLANG + HO | L1+ |
| **BLANG:PRE** | `pipeline-preflight` (via driver or CLI) | Env + queue staleness | L1+ |
| **BLANG:Q+** | `agent-queue-next '<AGENT>'` / `get-que` | Pick slice when on a queue lease | L1+ |
| **BLANG:HO** | `handoff-brief` | HANDOFF spine only | L1+ |
| **work** | slice implementation | validate-report not raw logs | L0+ |
| **BLANG:WIT-HON** | `validate-report witness_honesty <witness>` or `--scan debug_runs` | **Required before Q✓** on product rows | L2 |
| **BLANG:WIT** | `witness-brief <path>` | Compressed witness read | L2 / any witness read |
| **BLANG:Q✓** | `agent-queue-update <id> done --note <witness>` | Only if WIT-HON pass (or report-only mode) | L2 |

**Passthrough:** `node .claude/skills/agent-lang/driver.mjs <any-cli-args>` → `python -m rust_engine_mcp.cli`

**End slice:** `⟨COMMIT:WIT⟩` path only — no prose-only done.

---

## Canonical orient stack (BOOT reads — direct Read OK)

| Path | Section | Purpose |
|:---|:---|:---|
| `prompts/llm_agent_brief.md` | **FIELD◈ · SYMLANG◈** | Fast legend |
| `prompts/SYMBOLIC_LANGUAGE.meta.md` | **SYMLANG** | Laws · chart forms · bindings |
| `.cursor/skills/agent-lang/SKILL.md` | BLANG loop | Token → CLI map |
| `tools/orchestrator/queues/HANDOFF.md` | Active programs | Human intent overlay |

**Compress large docs:** `node … driver.mjs doc <path>` → file-digest (⊚digest)

---

## Role reads (after BOOT — domain skills)

| Agent | Also read / attach |
|:---|:---|
| `orchestrator` | `tools/orchestrator/NEXT.md`, `tools/orchestrator/queues/agent_queue.md` |
| `orchestrator-mcp` | `tools/mcp/README.md`, `MICRO_TOOLS_REGISTRY_v1.md`, `mcp_active_queue.json` |
| `planner` | migration matrices · debug-intelligence when drift |
| `planner-mcp` | MCP exec plans · schema dirs |
| `coder` | bevy-simulation-grade (`07-repo-authority-map` first), validation-first |
| `coder-mcp` | `MICRO_TOOLS_REGISTRY_v1.md`, witness_integrity plan |
| `designer-mcp` | mcp-asset-pipeline, mcp-production-rules, tile-generation |
| `sim-steward` | bevy-simulation-grade + debug-intelligence + cleanup-completion-intelligence |
| `main-thread-orchestrator` | same as sim-steward + HANDOFF |
| `debug-intelligence` | debug-intelligence skill |
| `operations-intelligence` | operations-intelligence skill, OPS_WITNESS_SPINE |
| `coparent-orchestrator` | HANDOFF + conflict matrix |

---

## Removed CLI (do NOT call — refactored out)

```text
agent_session_bootstrap · agent_doc_touch · agent_doc_reads_brief · agent_doc_promote_hot_reads
agent-lang-demo · agent-run-append · agent-markers-brief · snapshot-digest
```

Use **driver boot** + **file-digest** + **handoff-brief** + **witness-brief** instead.

---

## Anti-patterns (forbidden)

- Skip skill sync when `agent-lang/SKILL.md` was empty last session
- Trust stale MCP digest cache instead of reading current SKILL.md
- `BLANG:Q✓` without `BLANG:WIT-HON` on rows with `exit_predicate` / product witnesses
- Raw IDE Read on full witness JSON → use `witness-brief`
- Raw cargo stderr → use `validate-report cargo`
- Mark queue `done` from `cargo test --lib` alone
