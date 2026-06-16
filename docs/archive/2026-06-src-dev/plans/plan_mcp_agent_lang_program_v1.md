# PLAN-MCP-AGENT-LANG-001 — Toolchain upgrade + agent symbolic language `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-MCP-AGENT-LANG-001** |
| **Owner** | @planner-mcp (spec) → @coder-mcp (MCP) → @orchestrator (rollout) |
| **Status** | **ACTIVE** — planner orders below |
| **Date** | 2026-06-03 |
| **Parents** | [`plan_mcp_productivity_chain_v1.md`](plan_mcp_productivity_chain_v1.md) · [`plan_agent_queue_mcp_v1.md`](plan_agent_queue_mcp_v1.md) · [`AGENTS.md`](../../AGENTS.md) |

---

## Executive verdict (critical assessment)

**The MCP spine is real and shippable** — CLI ≡ MCP, validators return `ValidationReport`, P0 productivity tools (`pipeline_preflight`, `snapshot_digest`, `validate_p0_gate_plain`) exist in code. **The failure mode is not missing tools — it is ritual drift:** agents still `Read` full files, paste long path lists, use three different build-loop vocabularies, and carry no read telemetry.

| Dimension | Grade | Evidence |
|:---|:---:|:---|
| **Determinism / CLI parity** | A | [`MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) — Tier 1/1a/1b/1c shipped |
| **Token compression** | B+ | `witness_brief`, `handoff_brief`, `file_digest`, productivity P0 — **underused** |
| **Agent coordination language** | C+ | ★/○/ΔWF in HANDOFF; **no single grammar**; mixed `§`, paths, markdown links |
| **Read tracking** | B | `agent_doc_touch` ledger · `agent_doc_reads_brief` rollup · MCP digest cache promotion |
| **Build-loop shorthand (BLANG)** | C | Split: validation-first vs `cargo orchestrate` vs raw `cargo test` |
| **Emotional / dimensional cues** | D | Ops docs avoid emoji; agents lose urgency/blocker tone in long prose |
| **Queue ↔ doc sync** | C | Stale `blocked` rows; duplicate IDs (`APS-MAT-003`); dispatch docs lag queue |

**Upgrade thesis:** Do **not** add 20 new MCP tools first. **Enforce a language + read ritual** on top of existing Tier 1a/1c, then fill **5 high-ROI gaps** (below).

---

## Toolchain upgrade roadmap (ordered)

### Tier 0 — enforce what exists (0–1 day, @orchestrator + doc edits)

| Action | Replaces | Owner |
|:---|:---|:---|
| Session start: `token_savings_guide()` → `pipeline_preflight()` → `agent_queue_next` | Ad-hoc orientation | All agents |
| Touch snapshot: `snapshot_digest(path)` | `Read` 2–4K JSON | @coder-mcp |
| P0 check: `validate_p0_gate_plain(path)` | Raw validate-report parse | @coder-mcp |
| Witness: `witness_brief(path)` | Full witness `Read` | All |
| Plan slice: `handoff_brief()` | Full HANDOFF `Read` | All |
| Orientation: `file_digest(path)` | Full source `Read` | All |

Add to **every** `.cursor/agents/*.md` frontmatter ritual block (planner task **AGENT-LANG-004-RITUAL**).

### Tier 1 — five MCP gaps (P1, @coder-mcp)

| ID | Tool | Why |
|:---|:---|:---|
| **MCP-DOC-READ-001** | `agent_doc_touch(path, agent, intent)` | Read tracker + auto `file_digest` return — **shipped** |
| **MCP-DOC-READ-002** | `agent_run_append(event)` | Session telemetry → `run_events.jsonl` — **shipped** |
| **MCP-DOC-READ-003** | `agent_doc_reads_brief(min_reads)` | Hot-path rollup → `doc_reads_brief_latest.json` — **shipped** |
| **MCP-DOC-READ-004** | `agent_doc_promote_hot_reads()` | Repeated orient reads → `tools/mcp/cache/agent_doc_digests/` — **shipped** |
| **MCP-DOC-READ-005** | `agent_session_bootstrap(agent)` | Canonical brief stack + FIELD◈ every session — **shipped** |
| **MCP-GRAMMAR-ITER-TOOL** | `grammar_iterate(request_path)` | MCP wrapper — CLI exists, server tool **missing** |
| **MCP-SNAPSHOT-DIFF-001** | `snapshot_diff_brief(before, after)` | Iterate loop without diff JSON in chat |
| **MCP-SPINE-CHAIN-001** | `tile_spine_run(steps[])` | WRK→ATL chain — one call, per-step witness |

### Tier 2 — APS UI parity (already partially done)

Wire [`aps_validator_plain.py`](../../tools/mcp/python/rust_engine_mcp/aps_validator_plain.py) everywhere Tk shows P0 (APS-VALIDATOR-PLAIN-002). Load [`material_category_tree_v1.json`](../../assets/materials/profiles/material_category_tree_v1.json) in Materials tab (APS-MAT-003).

### Tier 3 — defer

Postgres telemetry, LLM-in-validator, duplicate APS panels in MCP without headless path, macro “do everything” prompts.

---

## AGENT-LANG — symbolic inter-agent vocabulary

**Goal:** Same **dimensional semantics** in HANDOFF, agent orders, queue `note`, and paste blocks — **fewer tokens**, clearer handoffs, humane urgency without prose.

### 1. Pipeline nodes (keep — already good)

**Live authority:** $ref:tools/orchestrator/queues/master_chain_tensor_v1.json

```text
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL★ ⇢ RT★
```

| Glyph | Meaning |
|:---|:---|
| `★` | Node closed / witness green |
| `○` | Node open / in progress |
| `○→★` | Just closed this session |
| `⇢` | Flow direction (preferred over `→` in AUTH lines) |
| `↺` | Retry / loop back |

### 2. Work slice refs (new — use in all agent docs)

| Form | Meaning | Example |
|:---|:---|:---|
| `⟨ID⟩` | Queue slice / program row | `⟨GRAMMAR-ITER-001-API⟩` |
| `@agent` | Cursor agent role | `@coder-mcp` |
| `ΔWF→@agent` | Route next work | `ΔWF→@coder-mcp P0 polish` |
| `⏸` | Orchestrator pause (non-blocking) | `⏸ WH-TRACK-B` |
| `⚡P0` | Hot path — do before other reads | `⚡P0 validate_p0_gate_plain` |

### 3. Dimensional status (emotional + operational — use sparingly, one per line max)

| Glyph | Dimension | When |
|:---|:---|:---|
| 🟢 | Witness / gate **green** | `🟢 grammar_iter_001_massing_live.json` |
| 🟡 | **Qualified** — ship with notes | `🟡 SIM-HUD-PRODUCT-001` |
| 🔴 | **Blocked** — hard dependency | `🔴 G2S-2 ← massing` |
| 🧊 | **Deferred** — explicit non-goal | `🧊 headless bake ≠ ship art` |
| 🧩 | **Dependency** edge | `🧩 ⟨API⟩ before ⟨G2S-2⟩` |
| 🔗 | **Cross-lane** link only | `🔗 see ⟨APS-MAT-003⟩` |
| 💬 | **Human** gate (operator/manual) | `💬 manual keyframe 24 PNGs` |

**Rule:** Emoji = **status dimension**, not decoration. Max **3 emoji per section** in HANDOFF/orchestrator docs.

### 4. Internal file refs — `$ref` syntax (new)

Replace long markdown links in agent-facing docs with **one-line refs**:

```text
$ref:<repo-path>[§<heading-id>]
```

| Example | Replaces |
|:---|:---|
| `$ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md§Response` | 3-line markdown link + "see section Response" |
| `$ref:tools/mcp/schemas/grammar_iterate_request_v1.schema.json` | Relative `../../tools/...` links |
| `$ref:debug_runs/grammar_iter_001_massing_live.json` | Witness path prose |

**Agent rule:** On seeing `$ref:…`, call **`agent_doc_touch(path, intent="ref")`** (MCP) — **not** `Read` — unless `intent=implement`.

**Planner rule:** Convert top 10 agent order docs to `$ref` first ([`orchestrator_coder_dispatch_20260603_v1.md`](orchestrator_coder_dispatch_20260603_v1.md), [`grammar_iter_agent_orders_v1.md`](grammar_iter_agent_orders_v1.md), [`bevy_hud_lanes_agent_orders_v1.md`](bevy_hud_lanes_agent_orders_v1.md), HANDOFF Active programs table).

### 5. BLANG — build loop language (one vocabulary)

Agents must use **BLANG** tokens instead of describing commands in prose.

| BLANG | MCP / CLI | Never say |
|:---|:---|:---|
| `BLANG:CARGO` | `validate_cargo_report(compress=4, use_cached=true)` | "run cargo check and read errors" |
| `BLANG:BEVY` | `validate_bevy_report(compress=4)` | raw bevy compile log |
| `BLANG:P0` | `validate_p0_gate_plain(snapshot)` | parse assembly_p0 hints manually |
| `BLANG:DIGEST` | `snapshot_digest(snapshot)` | Read full assembly JSON |
| `BLANG:WIT` | `witness_brief(path)` | Read witness JSON |
| `BLANG:HO` | `handoff_brief()` | Read full HANDOFF |
| `BLANG:PRE` | `pipeline_preflight()` | ping + locate_blender ad hoc |
| `BLANG:Q+` | `agent_queue_next(agent)` | "what should I do?" |
| `BLANG:Q✓` | `agent_queue_update(id, status, note)` | manual queue JSON edit |
| `BLANG:ORCH` | `cargo orchestrate` (after local edit) | only when hook needed; still prefer `BLANG:CARGO` for agents |
| `BLANG:PY` | `pytest tools/mcp/python/tests/ -k <filter>` | full pytest output in chat |
| `BLANG:S5` | `cargo test -p proc_A_dine01 --lib stage5` | stage5 regression (explicit) |

**Session loop (canonical — paste in all agent files):**

```text
BLANG:PRE → BLANG:Q+ → work → BLANG:WIT → BLANG:Q✓
```

---

## MCP-DOC-READ-001 — read tracker spec (for @coder-mcp)

**Problem:** Cursor `Read` tool is invisible to repo telemetry. Agents re-read AGENTS.md / HANDOFF every session.

**Behavior:**

```json
// agent_doc_touch(path, agent, intent="orient|ref|implement", max_lines=40)
{
  "schema": "agent_doc_touch_v1",
  "path": "docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md",
  "intent": "ref",
  "digest": { "total_lines": 264, "head": ["..."], "truncated": true },
  "ledger_appended": true,
  "ledger_path": "debug_runs/agent_ops/doc_reads.jsonl",
  "hint": "Use digest; full Read only when intent=implement"
}
```

**Ledger line (`doc_reads.jsonl`):**

```json
{"ts":"ISO8601","agent":"coder-mcp","path":"...","intent":"ref","session_hint":"GRAMMAR-ITER"}
```

**Policy (planner → `.cursor/rules/` + `token_savings_guide`):**

| intent | Allowed |
|:---|:---|
| `ref` | MCP digest only |
| `orient` | digest + `witness_brief` if witness |
| `implement` | Full `Read` / edit allowed |

**Also append tool list to `agent_run_append` on slice close** (MCP-DOC-READ-002).

---

## Documents agents read regularly — rollout targets

| Priority | Doc | AGENT-LANG pass |
|:---:|:---|:---:|
| 1 | [`tools/orchestrator/queues/HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) | `$ref`, BLANG, emoji status |
| 2 | [`AGENTS.md`](../../AGENTS.md) | BLANG session loop + `$ref` index |
| 3 | [`.cursor/agents/*.md`](../../.cursor/agents/) | Ritual block + BLANG |
| 4 | `src/dev/*_agent_orders_v1.md` | `$ref` tables; remove duplicate READ lists |
| 5 | [`development_plan_index.md`](development_plan_index.md) | `$ref` hub only — trim prose |
| 6 | [`MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) | BLANG column |

---

## @planner-mcp — ordered program (execute in sequence)

| P | ID | Deliverable | Unblocks | Est |
|:---:|:---|:---|:---|:---:|
| **1** | **AGENT-LANG-001-SPEC** | [`agent_lang_v1.md`](agent_lang_v1.md) — normative grammar (this doc § AGENT-LANG + BLANG + $ref) | All doc passes | 2h |
| **2** | **AGENT-LANG-002-REF** | Convert 4 agent order docs to `$ref` syntax | Token savings | 2h |
| **3** | **AGENT-LANG-003-BLANG** | One-page BLANG card in `token_savings_guide` output + HANDOFF | Agent ritual | 1h |
| **4** | **MCP-DOC-READ-SPEC-001** | JSON schema `agent_doc_touch_v1` + ledger format in `tools/mcp/schemas/` | @coder-mcp | 1h |
| **5** | **AGENT-LANG-004-RITUAL** | Patch `.cursor/agents/orchestrator.md`, `coder-mcp.md`, `coder.md`, `planner-mcp.md` with BLANG loop | Enforcement | 1h |
| **6** | **AGENT-LANG-005-HANDOFF** | HANDOFF rewrite: Active programs use ⟨ID⟩ + emoji status + `$ref` only | Orchestrator | 2h |
| **7** | **MCP-PRODUCTIVITY-P1-PLAN** | Thin plan: grammar_iterate MCP tool + snapshot_diff_brief + tile_spine_run | @coder-mcp P1 | 2h |

**Stop point:** Do **not** implement Python — hand **MCP-DOC-READ-001** + **MCP-GRAMMAR-ITER-TOOL** to @coder-mcp after specs land.

---

## @coder-mcp — orders (after planner P1–P4)

| ID | Task | Witness |
|:---|:---|:---|
| **MCP-DOC-READ-001-IMPL** | `agent_doc_touch` + CLI + server tool + pytest | `debug_runs/agent_doc_read_001_live.json` |
| **MCP-DOC-READ-002-IMPL** | `agent_run_append` → `run_events.jsonl` | `debug_runs/agent_run_append_001_live.json` |
| **MCP-GRAMMAR-ITER-TOOL** | Server wrapper over existing CLI | `grammar_iter_001_massing_live.json` |
| **MCP-SNAPSHOT-DIFF-001-IMPL** | `snapshot_diff_brief` | `grammar_iter_001_aps1_live.json` |
| Update `token_savings_guide()` | Add BLANG + doc_touch policy | — |

---

## @orchestrator — paste

```text
Program: PLAN-MCP-AGENT-LANG-001

NOW @planner-mcp:
  AGENT-LANG-001-SPEC → agent_lang_v1.md
  AGENT-LANG-002-REF → $ref pass on agent order docs
  MCP-DOC-READ-SPEC-001 → schema
  AGENT-LANG-005-HANDOFF → symbolic HANDOFF

THEN @coder-mcp:
  MCP-DOC-READ-001-IMPL (read tracker + digest)
  MCP-DOC-READ-002-IMPL (run_events)
  MCP-GRAMMAR-ITER-TOOL

Enforce BLANG:PRE → BLANG:Q+ in all agent chats.
NO new macro tools beyond P1 list without complexity budget ≥ 2.0.
```

---

## Complexity budget (new MCP proposals)

| Proposal | Value | Complexity | Ratio | Verdict |
|:---|:---:|:---:|:---:|:---|
| doc_touch + run_append | 8 | 3 | 2.7 | **Approve** |
| AGENT-LANG doc pass | 7 | 2 | 3.5 | **Approve** |
| grammar_iterate MCP wrap | 6 | 1 | 6.0 | **Approve** |
| tile_spine_run | 7 | 6 | 1.2 | After P0 ritual green |
| Chat-only agent memory | 3 | 8 | 0.4 | **Reject** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Initial MCP review + planner program |
