---
name: cleanup-completion-intelligence
description: >-
  Classify code before deleting it — distinguish obsolete vs transitional vs dormant
  vs incomplete, map dependencies, and prefer completion/migration over destruction.
  Use before any large delete, "remove dead code", consolidation, or rename sweep, and
  when judging whether a half-built system is abandoned or just unfinished. Triggers:
  delete, remove dead code, cleanup, obsolete, refactor away, consolidate, rename,
  unused, prune, archive.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# cleanup-completion-intelligence — classify before you delete

## The pattern (form B + ⬡ gate)

Deletion is irreversible + feature-destroying; "unused" often means "not wired up *yet*."

```text
⬡ DELETE-GATE ⊨ ⦃ intent understood ║ deps mapped ║ value evaluated ║ migration status verified ║ replacement path confirmed ⦄
  ¬satisfied ▶ ⛔ delete
```

Classify every candidate (form N — pick the arm):

```text
◆ candidate ?
 ├─═[A obsolete 🧊]▶  superseded · no value · no refs        ▶ safe delete (w/ dep proof)
 ├─═[B transitional 🟡]▶ mid-migration scaffold              ▶ keep until migration lands → completion_plan
 ├─═[C dormant gameplay 🧩]▶ works · not currently wired      ▶ preserve · document how to re-enable
 └─═[D incomplete infra ○]▶ half-built · still intended       ▶ completion_plan ¬delete
```

Output = decision + dependency graph ¬raw `rm`. Prefer `completion_plan` for B/C/D. Route conflicts to the owning agent. Classes + gate are env-independent.

## In this repo (reference implementation)

Build evidence cheaply ⊰ digests ¬whole files:

```bash
node .claude/skills/agent-lang/driver.mjs file-digest src/gui/view_authority.rs --max-lines 40
node .claude/skills/agent-lang/driver.mjs file-digest assets/staging/assemblies/colonial_3x3_s44_cf75.json
node .claude/skills/agent-lang/driver.mjs agent-queue-board
node .claude/skills/agent-lang/driver.mjs witness-brief <latest-witness.json>
```

```text
file-digest takes path / --max-lines (see `cli --help`); snapshot-digest removed in refactor → file-digest
route conflicts ⤴ ⦃ @planner architecture ║ @orchestrator sequencing ║ @coder impl ║ @designer UX ║ @sim-steward ⦄
honor migration tags in comments (VM-* · TRIAGE-*) as B/D signals
```

## Gotchas

```text
⚠ no current callers ∧ migration tag   = class B/D ¬A → check comments + agent-queue-board before calling it dead
⚠ "gameplay value" ∧ "sim/emotional impact"   explicit evaluation axes here ¬afterthoughts → dormant (C) is preserved ¬pruned
```

## Source

```text
◎.cursor/skills/cleanup-completion-intelligence/   Cursor original — reference.md has full categories, value matrix, dependency-graph format
```

```text
⟦/cleanup-completion-intelligence⟧ NEXT ⚑ file-digest evidence → ⬡DELETE-GATE → ◆classify A/B/C/D → A:delete(w/proof) · B/C/D:completion_plan
```
