---
name: agent-lang
description: Bootstrap, orient, and drive the Rust_engine_template_01 multi-agent system (BLANG / agent-lang, authored in SYMLANG). Use at session start, on handoffs, on context drift, or when asked to run/start/bootstrap an agent, load the llm_agent_brief, pick a queue slice, read a witness, run validate-report, or run the MCP CLI. The driver (.claude/skills/agent-lang/driver.mjs) wraps `python -m rust_engine_mcp.cli` and runs the session ritual. Triggers: agent-lang, BLANG, SYMLANG, $ref, $sym, witness, handoff, session bootstrap, MCP CLI, validate-report, token savings.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md` — authored in SYMLANG (concrete/live, not a template).

# agent-lang — drive the BLANG / SYMLANG multi-agent system

This repo runs on a **symbolic multi-agent protocol**: agents share **SYMLANG** (the chart/glyph
language — `$ref` above) + a **Python MCP CLI** (`python -m rust_engine_mcp.cli`). A thin driver
wraps the CLI — forces UTF-8, sets the package cwd, picks the interpreter with deps.

## Session start (form D — boot ritual across time `⊳`)

```text
node .claude/skills/agent-lang/driver.mjs boot <agent>
```

```text
⦿driver │ boot <agent> ⤳·······································⤳ Q✓★
⦿CLI    │  ╰⤳ ▢PRE  pipeline-preflight ▷⊳ ◎env + queue-staleness
⦿read   │       ▢BOOT read prompts/llm_agent_brief.md §FIELD◈ + prompts/SYMBOLIC_LANGUAGE.meta.md  (direct — orient)
⦿CLI    │       ▢HO   handoff-brief ▷⊳ ◎AUTH-spine ⇢ per-agent queue picks
⦿agent  │            ◂⊳ ▢work ─⬡[validate-report 🟢]▶ ⟨COMMIT:WIT⟩ ▷⊳ debug_runs/…
        └────────────────────────────────────────────────────▶ t ⊳
```
Re-run every session. Orient via `… doc <path>` (file-digest ⊚digest) ¬raw-Read where you can.

## BLANG session loop ⟶ command

```text
SKILL-SYNC ⊳ PRE ⊳ BOOT ⊳ Q+ ⊳ work ⊳ WIT-HON ⊳ WIT ⊳ Q✓
```

## Skill parity (mandatory — every session)

**If this file or any domain `SKILL.md` was empty/stale, agents lose BLANG protocol.**

```powershell
# From repo root — repair .cursor/skills from .claude/skills
powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1
```

1. Run sync when `.cursor/skills/agent-lang/SKILL.md` is missing or <500 bytes  
2. **Read** the synced skill in-session (do not rely on old digest cache alone)  
3. After `tools/mcp/python` tool changes → **reload Cursor MCP** (`rust-engine-art`)  
4. Skill sync doc: `$ref:.cursor/skills/sync-claude-skills/SKILL.md`

**Rule:** `BLANG:Q✓` forbidden when `BLANG:WIT-HON` FAIL on row witness + rollup parents (see `plan_witness_queue_integrity_mcp_v1.md`).

| BLANG | `node .claude/skills/agent-lang/driver.mjs …` |
|:--|:--|
| `PRE` | `pipeline-preflight` (env + queue staleness) |
| `BOOT` | read `prompts/llm_agent_brief.md` §FIELD◈ + `prompts/SYMBOLIC_LANGUAGE.meta.md` (direct) |
| `Q+` | `get-que <agent> [--demand --minutes 60]` (multi-parallel) · legacy `agent-queue-next` |
| `Q✓` | `agent-queue-update <id> done --note <witness>` (auto queue; dual Q✓ dispatch) |
| `HO` | `handoff-brief` |
| `WIT` | `witness-brief <path>` |
| `WIT-HON` | `validate-report witness_honesty` · `intel_officer_sweep` · `validate-report queue_integrity` |
| `CARGO` | `validate-report cargo --cached --compress 4` (structured report ¬raw stderr) |
| `REF` | `doc <path>` → `file-digest` (⊚digest, compressed) |
| `GUIDE` | `guide` (token-savings-guide: BLANG token → CLI map) |

## Driver verbs (verified against the current CLI)

```text
boot <agent>   PRE⨟BOOT⨟HO ritual (above) — PRE+HO are CLI; BOOT is a direct read
guide          token-savings-guide — BLANG token ⟶ CLI command map
doc <path>     file-digest — compressed ⊚digest read
where          resolved repo + interpreter (portability check)
demo           health smoke: pipeline-preflight + handoff-brief
<args…>        passthrough → python -m rust_engine_mcp.cli <args>
```

## AUTH spine + tensor (orchestration overlay — SYMLANG, not CLI)

```text
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK○ ⇢ ATL○ ⇢ RT○        (★ closed/witness-green · ○ open · ○→★ this-session)
T[c,d,a,φ]   c=chain A…J · d=DSM-node · a=writer-role · φ∈{−1 🧊, 0 ○, 1 🟡, 2 🟢}
```

## Stream / handoff delimiters

```text
⟨BRK⟩ hand off → HO + Q+   ⟨CONT⟩ continue slice ($ref + last ⟨ID⟩)   ⟨DRIFT⟩ re-anchor ($ref + witness + T-cell)
⟨COMMIT:WIT⟩ witness landed (path only)   idle/blocked ⟹ ⟨BP:COLLECT⟩→⟨BP:MIRROR⟩→⟨BP:SCAN⟩→⟨BP:SHARE⟩→⟨BP:RESUME⟩ (¬wait-only)
```

## The language — read the spec first

`$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` (SYMLANG) covers: the 13-dim lattice (**1 glyph each** —
Ct💰 Cx🌀 Au🏛 Rk⚠ Q🎯 E🔬 …), status + evidence clusters (`🟢✅🧪`, no lone ✅), edge grammar +
annotations (`▷⊳ ◂⊳ ═[guard]▶ ⟨↻k⟩`), the **chart forms A–P + §3.11 graph algebra**, **§2.13 domain
glyph extensions**, graded confidence ◔◑◕●, the formal grammar, and the enforcement card. **Author
replies chart-first** (§1) — emoji=STATUS, geometric=STRUCTURE; every glyph earns ⏩∨💰↓∨🎯.

## Anti-patterns (use the tool, not the wall)

```text
read full witness JSON → witness-brief   ·   raw cargo stderr → validate-report   ·   5-line md links → $ref:path
"waiting on planner" → Q+ + fallback slice   ·   chat-only memory → witness JSON + agent-queue-update --note   ·   lone ✅ → close 🧪/📜/⊚
```

## ⚠ CLI refactor (current — re-map if these are restored)

The CLI was refactored; the doc-ledger / session / demo command family was **removed**:
`agent-session-bootstrap`, `agent-doc-touch`, `agent-lang-demo`, `agent-markers-brief`,
`agent-run-append`, `snapshot-digest`. The driver adapts: **BOOT** = direct brief+SYMLANG read,
**doc** = `file-digest`, **demo** = preflight+handoff health smoke. Still live and used above:
`pipeline-preflight · handoff-brief · agent-queue-next/update · witness-brief · validate-report ·
token-savings-guide · file-digest`.

```text
⟦/agent-lang⟧ NEXT ⚑ SKILL-SYNC → boot <agent> → Q+ → work → WIT-HON → WIT → Q✓
```
