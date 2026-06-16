# AGENT-LANG collective ritual `v1.1` — breakpoints, markers, joint review

| Field | Value |
|:---|:---|
| **ID** | **AGENT-COLLECTIVE-RITUAL-001** |
| **Status** | **ACTIVE** |
| **Normative for** | All `.cursor/agents/*.md` · slice close · idle/blocked recovery |
| **Lang parent** | [`agent_lang_v1.md`](agent_lang_v1.md) |
| **Tensor board** | [`master_chain_board_4d_v1.md`](master_chain_board_4d_v1.md) |

---

## Why

Agents stop when **their** queue row says blocked — but the **project** is shared. Another agent may have finished the slice, changed deps, or left a witness you must **extend**, not redo.

This ritual forces **collective look-back**, **honest markers** for agents who come after, and **resume** in the same turn.

---

## 1. Symbolic breakpoints `⟨BP:*⟩`

Breakpoints are **mandatory pause tokens** — not errors. Trigger them when:

- `BLANG:Q+` returns idle / drain / blocked
- You are about to rewrite a todo another agent owns
- Context feels stale (`⟨DRIFT⟩` → start with `⟨BP:COLLECT⟩`)
- Before slice close (always `⟨BP:SHARE⟩`)

| Breakpoint | Meaning | Tools / refs |
|:---|:---|:---|
| **⟨BP:COLLECT⟩** | Return to **collective** progress | `BLANG:HO` · `$ref:master_chain_tensor_v1.json` · `agent_queue_board` (all agents on ⟨ID⟩) |
| **⟨BP:MIRROR⟩** | Read **prior writers** | Tail `debug_runs/agent_ops/agent_markers.jsonl` · `BLANG:WIT` on slice witness |
| **⟨BP:SCAN⟩** | **Dimensional build scan** at locale | Role BLANG + `$sym:Authority@path` one line |
| **⟨BP:PRIOR⟩** | Prior artifact on **this path** | `$ref:` their deliverable · queue row `note` · git blame if implement |
| **⟨BP:SHARE⟩** | Leave **honest marker** for next agent | `agent_marker_append(...)` |
| **⟨BP:RESUME⟩** | Continue with updated model | `BLANG:Q+` · extend · or `ΔWF→@agent` — **same turn** |

**Chain:**

```text
… → ⟨BP:COLLECT⟩ → ⟨BP:MIRROR⟩ → ⟨BP:SCAN⟩ → work → ⟨BP:SHARE⟩ → ⟨BP:RESUME⟩ → …
```

---

## 2. Honest marker schema (prior-writer path)

Every agent **touches a slice** → append one marker. Encourages **review stop**, **joint criticism**, **why I paused**.

```yaml
schema: agent_marker_v1
ts: ISO8601
agent: "@coder-mcp"
slice_id: "⟨APS-MAT-003⟩"
breakpoint: "⟨BP:SHARE⟩"
dim: ["🟡", "🧩"]              # max 3 status emoji
mirror: "What @planner-mcp spec said vs what witness shows now"
scan: "BLANG:P0 assembly — footprint 4x3 🟢; ATL registry row still ○"
why: "Stopped so next agent sees ATL gap — not a silent handoff"
joint: "Suggest designer-mcp sign-off before promote — ship ≠ schema-only"
prior_writer: "@planner-mcp"
prior_ref: "$ref:docs/archive/2026-06-src-dev/plans/plan_mcp_agent_lang_program_v1.md§AGENT-LANG"
delta_wf: "ΔWF→@designer-mcp G4 row only"
```

**Ledger:** `debug_runs/agent_ops/agent_markers.jsonl`  
**Tool:** `agent_marker_append` / CLI `agent-marker-append`

---

## 3. Forced continuation (no wait-only turns)

| `BLANG:Q+` result | You must |
|:---|:---|
| **work** | Execute; on touch → `⟨BP:SHARE⟩` at end |
| **blocked** | `⟨BP:COLLECT⟩` → check if **other agent closed** dep → `⟨BP:MIRROR⟩` → fallback slice or smallest unblock |
| **idle / drain** | `⟨BP:COLLECT⟩` → read tensor + markers → **pick** ready row or **extend** open shared slice |
| **done elsewhere** | `BLANG:Q✓` note + marker `mirror: done by @X` — do not redo |

---

## 4. Todo already written by another agent

Their queue row is **authoritative**. You **collaborate**, not replace.

| Situation | Response |
|:---|:---|
| They finished ⟨ID⟩ | Marker 🟢 + verify witness + **next** slice only |
| They partial 🟡 | `extend ⟨ID⟩` in marker — append code/docs, don't new ID |
| They blocked 🔴 | `joint:` critique + your unblock step OR `ΔWF→@human` 💬 |
| You disagree | Marker `joint:` + implement **bounded** fix OR route `@sim-steward` |

**Paste to their row (queue `note`):**

```text
@coder ← mirror: extended WRK witness; see marker ts:… $ref:debug_runs/agent_ops/agent_markers.jsonl
```

---

## 5. Response networking (cross-agent lexicon)

| Sym | Meaning |
|:---|:---|
| **MIRROR** | Reflect prior agent state in marker |
| **SCAN** | Build/dimensional check at `$sym:` locale |
| **JOINT** | Constructive critique — required in `⟨BP:SHARE⟩` |
| **COLLECT** | Shared board + HANDOFF + tensor |
| **RESUME** | Same-turn continue after look-back |
| **SHARE** | Artifact for next agent on prior-writer path |

**3-layer projection** (don't merge in one blob):

```text
T[c,d,a,φ]  🔗  $sym:Writer@path  🔗  marker.mirror one line
```

---

## 6. Readonly agents (ops / debug / cleanup / planner)

No queue mutation. Ritual:

```text
⟨BP:COLLECT⟩ → ⟨BP:MIRROR⟩ → analysis → ⟨BP:SHARE⟩ → ΔWF table
```

Marker `joint:` is **required** — route fix to `@coder`, architecture to `@planner`.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-07 | Breakpoints, markers, forced continuation, joint review |
