# User feedback — orchestration layer honesty `v1`

| Field | Value |
|:---|:---|
| **ID** | **USER-FEEDBACK-ORCH-LAYER-001** |
| **Date** | 2026-06-03 |
| **Audience** | Operator / you |
| **Response to** | “Mark done flowcharts are wildly limited — unacceptable” |

---

## Your feedback (recorded)

You asked for the **Master chain board** style **plus** what actually happens **behind and between agents** — including sub-levels — merged with production MCP, APS UX, and art-engine synergy. You correctly flagged that **“mark done” as a single box** misrepresents the system. **Agreed.**

---

## What we did (this session arc)

| Layer | Action | Artifact |
|:---|:---|:---|
| **Truth** | Audit v19 + queue sync — stopped wrong picks | `planner_status_audit_v19.md`, queue v5.6.0 |
| **Unblock coders** | CON-P7 exec, DSM closure, defer registry | thin exec plans + `defer_registry.json` |
| **Human overlay** | Chains A–J board | `master_chain_board_4d_v1.md` |
| **Machine overlay** | 4D tensor index | `master_chain_tensor_v1.json` |
| **Agent comms** | AGENT-LANG + BLANG + A2C commit grammar | `agent_lang_v1.md` |
| **UX path** | APS professional polish rules + async exec brief | not shipped yet — Chain F still ○ |

**What we did *not* do yet:** implement richer **runtime** orchestration UI, auto tensor refresh on every slice step, or replace Cursor’s limited todo “mark done” with a full state machine in the product.

---

## Why “mark done” feels broken

Today “done” collapses **six different commits** into one checkbox:

```text
❌ WRONG (what tools imply):
   work → [Mark done] → green

✓ ACTUAL (what the repo requires):
   pick slice → in_progress → substeps → witness file → regression →
   queue JSON update → run_events.jsonl → tensor/board refresh →
   audit matrix row → (maybe) operator sign-off
```

If any sub-step is skipped, the board lies: queue says done, witness missing, or witness green but operator gate still OPEN (G-PLAY).

**That is unacceptable for operations** — and it is a **documentation + tooling gap**, not your misunderstanding.

---

## Between agents — the real stack

Seven levels (L0–L6). Agents operate at L2–L5; you operate at L0–L1.

```text
L0  Operator / you          G-PLAY checklist, defer promotions, session pick
L1  Orchestrator paste      BOARD + DRAIN + RITUAL (one block)
L2  Planner commit          COMMIT:SPEC — exec md, queue row, unblocks
L3  Implementer commit      COMMIT:WIT — code + debug_runs/*.json
L4  Sub-slice steps         e.g. tile_spine: p0→build→pack→validate (per-step φ)
L5  BLANG / MCP tools       preflight, digest, validate, test, brief
L6  Witness keys            atomic booleans in JSON (not “slice done”)
```

**Between @planner and @coder-mcp:** planner does **not** “mark done” — it **COMMIT:SPEC** (`⟨ID⟩` + `$ref:exec.md`). Coder-mcp **cannot** start without that spec lock (or must risk inventing scope).

**Between @coder-mcp and @coder:** DSM boundary — Tk/MCP art vs Bevy `src/`. Handoff is **witness path + file prefix**, not chat summary.

**Between agents and subagents (Task):** Task quota failures mean the **same L3 work** must run in foreground (main chat) — the commit chain does not shorten; only the **worker** changes.

Full diagrams: [`a2c_commit_flow_v1.md`](a2c_commit_flow_v1.md)

---

## What the 4D tensor is (and is not)

| Is | Is not |
|:---|:---|
| Stable **index** for chains × DSM × agent × phase | Live dashboard |
| Updated on **queue sync** / planner session | Updated every chat turn |
| Overlay for MCP + UX synergy (`R_ux`) | Replacement for witnesses |

The tensor tells you **where pressure is** (φ=0 cells). It does not replace opening `debug_runs/*.json`.

---

## What should improve (committed follow-ups)

| Priority | ID | Fix |
|:---:|:---|:---|
| P0 | **AGENT-LANG-005-HANDOFF** | HANDOFF speaks `⟨⟩` + `$ref` only — no prose drift |
| P0 | **A2C-FLOW-001** | Normative commit state machine in `a2c_commit_flow_v1.md` (shipped below) |
| P1 | **QUEUE-STEP-001** | Queue rows gain `substeps[]` with per-step φ (not binary done) |
| P1 | **OPS-EVENT-002** | Every L4 step append → `run_events.jsonl` (not only final Q✓) |
| P2 | **BOARD-REFRESH-001** | Script: witness scan → patch `master_chain_tensor_v1.json` |

---

## Operator one-liner

Use the **board** to choose chain; use **A2C flow** to verify nothing was “marked done” without witness; use **defer registry** to veto agent enthusiasm.

```text
BOARD: $ref:src/dev/master_chain_board_4d_v1.md
FLOW:  $ref:src/dev/a2c_commit_flow_v1.md
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | User feedback recorded; mark-done gap acknowledged |
