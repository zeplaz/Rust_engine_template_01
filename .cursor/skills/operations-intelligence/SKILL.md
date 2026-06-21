---
name: operations-intelligence
description: >-
  Read-only pipeline/ops analyst — compress witness, queue, and run telemetry into a
  DSM authority/risk/cost surface with Q/C/E scores and ΔWF routing, and gate new
  proposals on a complexity budget. Use after a lane closes, before a big architecture
  commit, when HANDOFF and witnesses disagree, or to stress-test a proposal. Triggers:
  ops, DSM, complexity budget, value vs complexity, lane close, ΔWF, witness disagree,
  proposal review, Q/C/E, project brief.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# operations-intelligence — DSM · Q/C/E · the complexity gate

`◉Q🎯↑ · 💰↓ · read-only⊚` — make the operating surface legible, gate spend on value.

## Pattern (transferable)

```text
◎telemetry ▷⊳ ▢DSM ─⬡[Q/C/E scored]▶ ◆EV/Cx? ▷⊳ ◎ΔWF-table ⤳ @owner
  Q🎯 coherence/stability · 💰 compute/tokens · E🔬 clarity/confusion-risk
```
Analyst ⊚: emit the routing package, ¬implement. Cheapest mechanism that clears the gate (JSON events ≺ database) until value justifies more. Swap telemetry sources + node set → transfers to any pipeline.

## Complexity gate (form B — ◆ EV/Cx, headline output ¬footnote)

```text
        ◆ EV/Cx ?
   ┌──═[≥1.0]▶ 🟢 ✅APPROVE ▷⊳ @owner
   ├──═[.5–1)▶ 🟡 ⚠REVISE ↻[≤2] ◆
   └──═[<0.5]▶ 🧊 DEFER ▷⊳ ◎backlog        clever∧low-EV/Cx ⟶ 🧊, saying so = the job
```

## In this repo — DSM surface (verified)

```bash
node .claude/skills/agent-lang/driver.mjs ops-get-project-brief
node .claude/skills/agent-lang/driver.mjs orchestrator-brief
```
`ops-get-project-brief` ▷⊳ `{quality_score, utility_score, auth_spine, known_failures, top_failures_ranked[severity]}` — ready-made DSM + risk surface.

Deeper scan (PowerShell) ▷⊳ `debug_runs/agent_ops/ops_report_latest.json`:

```powershell
powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
```

**Intel officer sweep** (false-green / stub-done cull):

```bash
node .claude/skills/agent-lang/driver.mjs intel-officer-sweep
node .claude/skills/agent-lang/driver.mjs intel-officer-apply --ids TASK-ID --apply
```

Witness: `debug_runs/agent_ops/intel_officer_sweep_live.json`

## DSM AUTH spine (form I — from `handoff-brief`)

```text
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL★ ⇢ RT★        (★ closed/witness-green · ○ open)

        │ MAT APS SNAP WRK ATL RT
   MAT  │  ★   ·    ·    ·   ·  ·     cell = dep weight · ⇢ = forward spine
   APS  │  3   ★    ·    ·   ·  ·     below-diag = forward dep
   SNAP │  ·   2    ★    ·   ·  ·     above-diag = feedback ⟶ re-sequence (⥁)
   WRK  │  ·   ·    3    ★   ·  ·     col-sum = fan-in load · row-sum = blast radius
   ATL  │  ·   ·    ·    3   ★  ·
   RT   │  ·   ·    ·    ·   2  ★
```
Witness indices: `OPS_LANE_REGISTRY.json` · `unified_witness_index.json`. ECS/viewport/render drift ⤴ [debug-intelligence](../debug-intelligence/SKILL.md); this skill ⊚ pipeline/agent DSM.

## Gotchas

```text
🏛 read-only        findings ▷⊳ ΔWF routing table for owners — never a direct edit
✅ intel-officer    ONLY @operations-intelligence may `intel-officer-apply --apply` after sweep review
⚖ gate=headline    a clever-but-low-EV/Cx proposal is a 🧊 DEFER · state it
```

## Source

Cursor original: [.cursor/skills/operations-intelligence/SKILL.md](../../../.cursor/skills/operations-intelligence/SKILL.md) · agent: [.cursor/agents/operations-intelligence.md](../../../.cursor/agents/operations-intelligence.md). Decision gate also in [`prompts/llm_agent_brief.md`](../../../prompts/llm_agent_brief.md) §DECISION.

```text
⟦/operations-intelligence⟧ NEXT ⚑ ops-get-project-brief → DSM/Q-C-E → ◆EV/Cx → ◎ΔWF ⤳ @owner
```
