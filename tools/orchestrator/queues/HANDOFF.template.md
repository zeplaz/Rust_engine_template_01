# Agent handoff (copy to `HANDOFF.md` for session continuity)

> Template: [`HANDOFF.template.md`](HANDOFF.template.md) · Playbook: [`prompts/guides/subagent_continuity_playbook_v1.md`](../../../prompts/guides/subagent_continuity_playbook_v1.md)

**Date:** YYYY-MM-DD  
**Lane:** Stage5 | Construction | LOG | VM | Industrial  
**Owner:** @coder | @designer | @sim-steward | @main-thread-orchestrator | parent Auto  
**Shift (sim-steward / main-thread-orchestrator):** A | B | C — see [`.cursor/agents/sim-steward.md`](../../../.cursor/agents/sim-steward.md) · [`.cursor/agents/main-thread-orchestrator.md`](../../../.cursor/agents/main-thread-orchestrator.md)

## Fail-cycle ledger (if Task failed)

```yaml
slice_id: …
attempts:
  - { cycle: 0, channel: Task/debug-intelligence, outcome: error | empty | partial, note: "…" }
  - { cycle: 2, channel: main-thread-shift-B, outcome: yaml_emitted }
next_cycle: 1 | 2 | 3 | defer
```

## Goal

One sentence: what “done” looks like for this slice.

## Context

- Prerequisite milestones (e.g. FULL_APP green, CONSTRUCTION_OPERATIONAL_GREEN):
- Playbook(s): `tools/orchestrator/agents/…`
- Architecture doc(s): `src/dev/…`

## State

| Item | Value |
|------|--------|
| Branch | |
| Last good commit | |
| Witness JSON | `debug_runs/…` — key fields: |
| Open board rows | e.g. LOG-A-*, VM-C-* |

## Files

- **Touched:**
- **Next:**

## Commands

```powershell
# Last run (pass/fail):
cargo test -p proc_A_dine01 …
```

## Blockers

- None | describe

## Next action (single step)

e.g. “Implement VM-C C1: route map camera input through ViewProjectionAuthority::commit_pose”
