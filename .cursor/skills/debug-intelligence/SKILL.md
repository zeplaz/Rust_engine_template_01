---
name: debug-intelligence
description: >-
  Triage diagnostics into compressed, routed reports instead of fixing in place or
  dumping logs. Use when interpreting witness JSON, viewport/authority drift, render
  contract mismatches, multi-writer ECS resources, schedule hazards, or stale
  scaffolds. Produces a YAML routing packet (root cause, affected systems, owner,
  confidence) for @planner / @coder / @designer. Triggers: witness, drift, dual
  writer, render contract, viewport, debug, diagnostics, panic, regression triage.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# debug-intelligence — compress evidence, route the fix

## The pattern (form M — extract ▷ compress ⬡ route)

A triage agent does **not** fix and does **not** paste raw logs.

```text
◎evidence ◂⊳ smallest sufficient source (witness DIGEST ¬whole JSON · code REGION ¬file)
  ▷⊳ ▢compress ─⬡[tiered finding: severity · root_cause · affected · migration · conf ◔◑◕●]▶
  ▷⊳ ◆route ▶ ⦿owner-who-acts + single next step
  ⛔ log-walls — surface the DECISION ¬the dump · adapt sources + owner set to pipeline
```

Output = routing packet ¬patch:

```yaml
issue: <one line>
root_cause: <mechanism, not symptom>
affected: [<system/resource>, ...]
migration_status: <tag or n/a>
recommendation: <single next action>
owner: "@planner | @coder | @designer | @orchestrator"
confidence: 0.0–1.0
```

## In this repo (scope + watch surfaces)

```text
⊚own  ECS / viewport / render DRIFT only
¬own  pipeline DSM / Q-C-E / three-track ⤵ [operations-intelligence](../operations-intelligence/SKILL.md)
```

```text
watch ⦃ src/gui/view_authority.rs ║ src/gui/view_projection_authority.rs ║ src/render/viewport_pipeline.rs ║ src/render/extraction/fire_visual_extract.rs ║ src/gui/map_view/ ⦄
detects ⦃ multi-writer resources ║ hidden authority mutations ║ camera bleed ║ schedule hazards ║ stale scaffolds ⦄
authority map ◂⊳ [bevy-simulation-grade](../bevy-simulation-grade/SKILL.md)
```

Evidence commands via the agent-lang driver (compressed ¬raw):

```bash
node .claude/skills/agent-lang/driver.mjs witness-brief debug_runs/<witness>.json
node .claude/skills/agent-lang/driver.mjs agent-queue-board
node .claude/skills/agent-lang/driver.mjs doc src/gui/view_authority.rs    # file-digest (region peek)
```

```text
◎witness-brief    ▷⊳ {green, status, summary}
◎agent-queue-board ▷⊳ open ⟨ID⟩s + per-agent state (the ⟨BP:MIRROR⟩ orient before routing)
```

Collective ritual before routing (form M):

```text
⟨BP:MIRROR⟩ agent-queue-board + witness-brief <latest> ▷⊳ ⟨BP:SCAN⟩ witness-brief/doc ▷⊳ ▢emit YAML ▷⊳ ⟨BP:SHARE⟩ witness JSON + agent-queue-update <id> --note
```

## Gotchas

```text
⚠ lone ✅ ≠ verdict   close findings w/ evidence ⦃ 🟢✅🧪 measured ║ 🟢✅📜 witnessed ║ 🟢✅⊚ authority ⦄ → see [agent-lang](../agent-lang/SKILL.md)
⚠ ¬read full witness JSON   witness-brief returns green/status/summary you need · escalate to raw file only when conf < ◑(.7)
⚠ markers CLI removed in refactor   ⟨BP:MIRROR⟩ now reads agent-queue-board + witness-brief ; ⟨BP:SHARE⟩ records a witness JSON + queue --note (no agent-marker-append)
```

## Source

```text
◎.cursor/skills/debug-intelligence/reference.md   Cursor original — detection + routing rules (the SKILL.md there is a stub)
```

```text
⟦/debug-intelligence⟧ NEXT ⚑ ⟨BP:MIRROR⟩ agent-queue-board → ⟨BP:SCAN⟩ witness-brief → emit YAML packet → ΔWF→@owner (single next action)
```
