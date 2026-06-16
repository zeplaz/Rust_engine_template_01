# Orchestrator collective dispatch — tensor + breakpoint chain `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-06-07 |
| **Hub** | $ref:master_chain_board_4d_v1.md · $ref:tools/orchestrator/queues/master_chain_tensor_v1.json |
| **Lang** | $ref:agent_lang_v1.md · $ref:agent_collective_ritual_v1.md |
| **Rule** | **Every agent** runs ⟨BP:COLLECT⟩ before work; **every slice close** runs ⟨BP:SHARE⟩ with `joint:` reviewer |

---

## ⟨BP:COLLECT⟩ — queue board + tensor

### Grammar queue counts

| status | count |
|:---|:---:|
| done | 72 |
| ready | 4 |
| deferred | 3 |
| paused | 1 |
| active | 1 |

### Tensor AUTH spine (φ)

```text
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL○ ⇢ RT○
```

| Chain | φ | Next ⟨ID⟩ | Owner | BLANG:Q+ |
|:---|:---:|:---|:---|:---|
| **C AGENT-LANG** | 🟡 | ⟨AGENT-LANG-002-REF⟩ → ⟨003⟩ → ⟨005⟩ | @planner-mcp | **work** |
| **B MCP briefs** | ○ | ⟨MCP-MAT-BRIEF-001⟩ | @coder-mcp | **work** |
| **A AUTH ATL** | ○ | ⟨MCP-ATLAS-BRIEF-001⟩ 🧊 · atlas QC | @coder-mcp | deferred |
| **D Rowhouse** | ○ | ⟨MCP-PROD-B2⟩ | @coder-mcp | continuation queue |
| **E Grammar** | 🟢 | maintain only | — | idle |
| **G Bevy HUD** | 🟢 | maintain witnesses | @coder | **idle** |
| **H Con/Infra** | ○ | ⟨INFRA-E4-002⟩… | @coder A | continuation |
| **I Weather** | ○ | ⟨WEATHER-WITNESS-001⟩ | @coder C | continuation |
| **J Defer** | 🧊 | ⏸ WH-TRACK-B · MCP-PILOT-GRAMMAR-001 | — | paused |

### BLANG:Q+ snapshot (grammar queue)

| Agent | action | slice |
|:---|:---|:---|
| @planner-mcp | **work** | ⟨AGENT-LANG-002-REF⟩ |
| @planner | **work** | ⟨AGENT-LANG-002-REF⟩ (alias) |
| @coder-mcp | **work** | ⟨MCP-MAT-BRIEF-001⟩ |
| @coder | **idle** | ⟨BP:COLLECT⟩ → continuation INFRA or extend witness |
| @designer | **idle** | ⟨BP:MIRROR⟩ UX witnesses · review-on-call |
| @designer-mcp | **idle** | ⏸ ⟨MCP-PILOT-GRAMMAR-001⟩ only |
| @orchestrator | **active** | ⏸ WH-TRACK-B-PAUSE |

**Ready rows (grammar):** ⟨AGENT-LANG-002-REF⟩ · ⟨AGENT-LANG-003-BLANG⟩ · ⟨AGENT-LANG-005-HANDOFF⟩ · ⟨MCP-MAT-BRIEF-001⟩

---

## Forced breakpoint chain (all agents)

```text
BLANG:PRE → BLANG:Q+ → ⟨BP:COLLECT⟩ → ⟨BP:MIRROR⟩ → ⟨BP:SCAN⟩ → work → ⟨BP:SHARE⟩ → ⟨BP:RESUME⟩ → BLANG:WIT → BLANG:Q✓
```

| Step | Required output |
|:---|:---|
| **⟨BP:COLLECT⟩** | Tensor row + board counts (this doc or `master_chain_tensor_v1.json`) |
| **⟨BP:MIRROR⟩** | Tail `debug_runs/agent_ops/agent_markers.jsonl` — do not redo prior writer |
| **⟨BP:SCAN⟩** | One `$sym:Authority@path` + role BLANG line |
| **⟨BP:SHARE⟩** | `BLANG:MARK` with **`joint:`** naming reviewer |
| **⟨BP:RESUME⟩** | Same turn — no "waiting on X" without fallback slice |

**Marker template:**

```yaml
breakpoint: "⟨BP:SHARE⟩"
mirror: "prior writer state vs witness now"
scan: "BLANG:* at locale — dim emoji"
why: "honest pause reason"
joint: "Reviewer @agent — critique question for next handoff"
delta_wf: "ΔWF→@agent"
```

---

## Per-agent orders (paste)

### @orchestrator

```text
⟨BP:COLLECT⟩ agent_queue_board + $ref:tools/orchestrator/queues/master_chain_tensor_v1.json
Force breakpoint chain on every dispatch (this doc § Forced breakpoint chain).
Maintain tensor φ sync when queue rows close.
⏸ WH-TRACK-B — do not unblock without operator 💬
ΔWF: Chain C → Chain B → Chain A ATL → continuation H/I
```

---

### @planner

```text
BLANG:REF + $sym: writers — plan delta ONLY
READ: $ref:agent_lang_v1.md§4 · $ref:agent_collective_ritual_v1.md§4

NOW:
- COMMIT:SPEC for open continuation slices (INFRA, weather) — no rewrites
- Extend ⟨AGENT-LANG-002-REF⟩ plan with $sym: markers for coder review locales
- ⟨BP:SHARE⟩ marker joint: "Review question — does INFRA-E5 block ATL spine?"

DO NOT: rewrite @coder-mcp todo rows · full exec replans
ROUTES: ΔWF→@coder with review question in marker
```

---

### @planner-mcp

```text
BLANG:Q+("planner-mcp") → ⟨AGENT-LANG-002-REF⟩

READ: $ref:agent_lang_v1.md — extend existing $ref; do NOT rewrite coder-mcp todo

NOW (sequential):
1. ⟨AGENT-LANG-002-REF⟩ — $ref pass on agent order docs (delta only)
2. ⟨AGENT-LANG-003-BLANG⟩ — BLANG column in token_savings_guide output
3. ⟨AGENT-LANG-005-HANDOFF⟩ — HANDOFF Active programs ⟨ID⟩ + 🟢/🔴 + $ref

⟨BP:SHARE⟩ joint: "@coder-mcp confirm MCP-MAT-BRIEF schema matches category tree"
BLANG:Q✓ on each close
```

---

### @coder

```text
BLANG:CARGO / BLANG:BEVY / BLANG:S5
Grammar queue: idle → ⟨BP:COLLECT⟩ → continuation queue

NOW:
- BLANG:CARGO (compress=4, use_cached=true) — baseline after any edit
- BLANG:S5 if touching stage5 spine
- Extend prior witnesses — do NOT new IDs for same slice
- If @coder-mcp landed ⟨BUILD-WORKER-001⟩ 🟢: marker mirror "WRK★ — extend assembly_build witness only"

FALLBACK slices (continuation): ⟨INFRA-E4-002⟩ · weather only if assigned C
⟨BP:SHARE⟩ joint: "@sim-steward — any dual-writer on viewport if infra touches render"
```

---

### @coder-mcp

```text
BLANG:P0 / BLANG:DIGEST / BLANG:PY
BLANG:Q+ → ⟨MCP-MAT-BRIEF-001⟩

NOW:
1. Implement material_profile_brief — extend staging + category tree path
2. BLANG:PY -k mat_brief (or filter) before witness
3. Mirror @planner-mcp / @designer-mcp specs — extend, don't fork AssetSpec

WRK context: BUILD-WORKER-001 🟢 — extend build_worker witness if touching assembly_build
⟨BP:SHARE⟩ joint: "@designer-mcp G3/G4 sign-off before promote to modules/"
DEFER: ⟨MCP-SPINE-CHAIN-001⟩ until Chain C AGENT-LANG φ→🟢 x2 sessions
```

---

### @designer

```text
BLANG:WIT on UX witnesses
Grammar queue: idle — review-on-call

NOW:
- BLANG:WIT debug_runs/aps_ux_*_live.json — qualified sign-offs only
- Review @designer-mcp output — do NOT duplicate AssetSpec prose
- ⟨BP:MIRROR⟩ markers from designer-mcp G3/G4 rows

WHEN notified: UX review for ⟨MCP-MAT-BRIEF-001⟩ APS copy
⟨BP:SHARE⟩ joint: "@coder-mcp — tooltip key for material brief status line?"
```

---

### @designer-mcp

```text
⏸ primary: ⟨MCP-PILOT-GRAMMAR-001⟩ — do not run headless ship

WHEN resumed or rowhouse:
- validate + staging witness per G0–G5
- BLANG:WIT debug_runs/art_pipeline/*_live.json

⟨BP:SHARE⟩ sign-off marker:
  joint: "G3/G4 joint: @coder-mcp — promote only after validate_asset_report + list_staging"
  dim: [🟡, 💬]
ΔWF→@coder-mcp on promote path only
```

---

### @orchestrator-mcp

```text
DSM AUTH + G0–G5 gates
$ref:designer_mcp_onboarding_v1.md

NOW:
- Route spec → validate → tool → staging → promote → registry
- AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL○ ⇢ RT○
- No "waiting on Blender" — BLANG:PRE + locate_blender or honest 🔴 marker

⏸ warehouse Track B — do not sequence G4 before operator 💬
⟨BP:SHARE⟩ joint: "@designer-mcp manual keyframe checklist before G4"
```

---

### @sim-steward

```text
Shifts A→B→C + BLANG
READ: $ref:agent_collective_ritual_v1.md

Per shift:
  A — witness triage · BLANG:WIT + BLANG:CARGO
  B — authority drift · $sym:ViewManager@src/gui/view_manager.rs (example)
  C — cleanup classification · extend prior writer shims only

Each shift: ⟨BP:SHARE⟩ marker with shift label (A|B|C)
joint: name reviewer (@coder | @planner) + one review question
Never stop on Task quota — foreground same turn
```

---

### @main-thread-orchestrator

```text
Fail-cycle queue — never stop on Task quota alone

ON Task error (usage):
  1. ⟨BP:COLLECT⟩ tensor + markers
  2. Implement slice inline OR @sim-steward shift
  3. BLANG:Q✓ + marker "fail-cycle N closed"

Queue: tools/orchestrator/queues/HANDOFF.md fail-cycle ledger
ΔWF→foreground Auto when subagent pool dry
```

---

### @coparent-orchestrator

```text
Secondary pathways — parallel lanes vs primary P1

NOW:
- Operator · VFX capture · designer tails · parametric placement
- Marker on promotion to primary: joint: "conflict matrix vs AUTH spine"
- Join via ⟨BP:SHARE⟩ — do not preempt Chain C or MCP-MAT-BRIEF

Promotion rule: secondary slice → orchestrator board row only after 🟢 witness
```

---

## Drain order (orchestrator)

```text
1. @planner-mcp  ⟨AGENT-LANG-002-REF⟩ → ⟨003⟩ → ⟨005⟩
2. @coder-mcp    ⟨MCP-MAT-BRIEF-001⟩
3. @coder A      continuation INFRA tail (if green cargo)
4. @designer     on-call BLANG:WIT when coder-mcp requests joint review
5. ⏸ Track B     operator only
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-07 | Collective dispatch — tensor + breakpoint + joint markers |
