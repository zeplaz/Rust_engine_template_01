# Snapshot review + drain prompts — 2026-06-07

| Field | Value |
|:---|:---|
| **Snapshot date** | 2026-06-07 |
| **Tensor** | $ref:tools/orchestrator/queues/master_chain_tensor_v1.json |
| **Prompt pack** | $ref:docs/archive/2026-06-fleet-drain/dev_dispatch/agent_prompt_pack_20260607_v1.md |

---

## Executive snapshot

**Grammar queue is drained.** 77 done · 0 ready · 3 deferred · 1 paused · 1 active orchestrator pause.

| Verdict | Detail |
|:---|:---|
| 🟢 **Closed lanes** | Chain C AGENT-LANG · Chain E Grammar iter · APS UX program · SIM-HUD slices · MCP-MAT-BRIEF-001 |
| ○ **Active drain** | Chain D Rowhouse prod · Chain H INFRA tail · Chain I Weather regional |
| 🧊 **Explicit defer** | MCP-SPINE-CHAIN · MCP-ATLAS-BRIEF · MCP-OPS-REPORT · WH-TRACK-B |
| 🟡 **HANDOFF drift** | Some φ cells stale vs queue — trust queue JSON + witnesses below |

```text
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL○ ⇢ RT🧊
Gates: G-CONTAIN 🟢 · G-STAB 🟢 · G-PLAY 💬 OPEN
```

---

## Queue board (authoritative)

### grammar_continuation_queue.json

| status | count | notes |
|:---|:---:|:---|
| done | 77 | includes AGENT-LANG 001–006, grammar iter, APS polish |
| deferred | 3 | MCP-SPINE-CHAIN-001 · MCP-ATLAS-BRIEF-001 · MCP-OPS-REPORT-001 |
| paused | 1 | MCP-PILOT-GRAMMAR-001 (designer-mcp warehouse) |
| active | 1 | WH-TRACK-B-PAUSE (orchestrator) |
| **ready** | **0** | **lane idle for grammar agents** |

### simulation_continuation_queue.json

| ⟨ID⟩ | agent | status |
|:---|:---|:---:|
| WEATHER-WITNESS-001 | coder_c | 🟢 done |
| WEATHER-CLIMATE-001 | coder_c | 🟢 done |
| **WEATHER-REGIONAL-001** | coder_c | **○ ready** |
| WEATHER-EFFECTS-001 | coder_c | 🔴 blocked |
| WEATHER-GPU-PRECIP-001 | coder_c | 🔴 blocked |

### coder_active_queue.json (parallel_infrastructure_lane)

| ⟨ID⟩ | owner | status |
|:---|:---|:---:|
| **INFRA-E5-002** | coder_a | ○ ready |
| **INFRA-E4-002** | coder_a | ○ ready |
| **INFRA-E6-001** | coder_a | ○ ready |
| **INFRA-E6-002** | coder_a | ○ ready |
| **INFRA-E6-004** | coder_a | ○ ready |

### mcp_active_queue.json (rowhouse sprint)

| ⟨ID⟩ | agent | status |
|:---|:---|:---:|
| **MCP-PROD-B2** | coder-mcp | ○ ready P0 |
| **MCP-PROD-C-PILOT** | coder-mcp | ○ ready P0 |
| **MCP-PROD-PBR-PILOT** | designer-mcp | ○ ready P0 |
| MCP-PROD-MOD-G0-G5 | designer-mcp | 🔴 blocked |

### continuation_queue.json

| ⟨ID⟩ | agent | status |
|:---|:---|:---:|
| SLICE-TRIAGE-VM-06 | coder | ○ ready |
| SLICE-MD-F2-01/02/03 | coder | ○ ready |
| SLICE-TRIAGE-FIRE-STREAM | sim-steward | ○ ready |

---

## Witness spot-check

| witness | green | note |
|:---|:---:|:---|
| mcp_mat_brief_001_live.json | 🟢 | MCP-MAT-BRIEF closed |
| aps_ux_async_001_live.json | 🟢 | APS-UX-ASYNC closed |
| grammar_iter_001_massing_live.json | 🟢 | grammar iter closed |
| build_worker_001_live.json | 🟢 | WRK★ |
| weather_sim_live.json | 🟡 | witness exists; regional slice open |
| logistics_throughput_live.json | exists | INFRA-E5 target — verify after implement |

---

## BLANG:Q+ matrix (live)

| Agent | queue | action | slice |
|:---|:---|:---|:---|
| @planner-mcp | grammar | **idle** | — |
| @planner | grammar | **idle** | — |
| @coder-mcp | grammar | **idle** | → **mcp_active_queue** |
| @coder | grammar | **idle** | → **continuation** or coder_a INFRA |
| @coder | continuation | **work** | ⟨SLICE-TRIAGE-VM-06⟩ |
| @coder_c | simulation | **work** | ⟨WEATHER-REGIONAL-001⟩ |
| @designer | grammar | **idle** | on-call |
| @designer-mcp | grammar | **idle** | → **MCP-PROD-PBR-PILOT** |

---

## Drain order (orchestrator)

```text
1. @coder_c       ⟨WEATHER-REGIONAL-001⟩     queue=simulation
2. @coder A       ⟨INFRA-E5-002⟩ first         $ref:infra_agent_orders_v1.md
3. @coder-mcp     ⟨MCP-PROD-B2⟩                mcp_active_queue
4. @designer-mcp  ⟨MCP-PROD-PBR-PILOT⟩         parallel with B2
5. @coder-mcp     ⟨MCP-PROD-C-PILOT⟩           after B2 or parallel bpy
6. @coder         ⟨SLICE-TRIAGE-VM-06⟩         continuation (infra B / VM)
7. @sim-steward   ⟨SLICE-TRIAGE-FIRE-STREAM⟩   continuation
8. @designer      on-call BLANG:WIT reviews
9. @orchestrator  tensor φ sync · HANDOFF drift fix
10. ⏸ deferred   spine · atlas brief · warehouse Track B
```

---

# DRAIN PROMPTS — copy one block per chat

---

## @orchestrator

```text
You are @orchestrator — sequencing only. Snapshot: docs/archive/2026-06-fleet-drain/dev_dispatch/snapshot_drain_review_20260607_v1.md

⟨BP:COLLECT⟩
  grammar queue: 77 done · 0 ready — DRAINED
  tensor: tools/orchestrator/queues/master_chain_tensor_v1.json
  AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL○ ⇢ RT🧊

ACTIVE DRAIN (issue these in parallel):
  @coder_c       BLANG:Q+ queue=simulation → WEATHER-REGIONAL-001
  @coder A       $ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md → INFRA-E5-002
  @coder-mcp     mcp_active_queue → MCP-PROD-B2
  @designer-mcp  mcp_active_queue → MCP-PROD-PBR-PILOT
  @coder         continuation → SLICE-TRIAGE-VM-06 OR route to coder A if same session

DO NOT UNBLOCK:
  ⏸ MCP-PILOT-GRAMMAR-001 · MCP-SPINE-CHAIN-001 · WH-TRACK-B

YOUR TURN:
  Fix HANDOFF φ drift (grammar iter / APS rows marked ○ but queue 🟢)
  Update master_chain_tensor drain_order to match above
  ⟨BP:SHARE⟩ marker joint: "@coder A — E5 before E4 or parallel disjoint files?"

BLANG:HO · agent_queue_board grammar · BLANG:Q✓ only if you close orchestrator rows
```

---

## @planner-mcp

$ref:docs/archive/2026-06-src-dev/plans/planner_mcp_maintenance_idle_v1.md — **canonical idle paste** (§Paste block).

---

## @planner

```text
You are @planner — readonly architecture. Grammar queue: IDLE.

BLANG:PRE → BLANG:Q+("planner") → expect idle

DRAIN OPTIONS (if you want to add value):
  1. COMMIT:SPEC delta for ⟨SLICE-TRIAGE-VM-06⟩ — $sym:ViewManager authority one-pager for @coder
  2. Review ⟨INFRA-E5-002⟩ vs ATL spine — marker joint: "@coder A — logistics graph touches render?"
  3. Weather regional plan check — joint: "@coder_c — clipmap sample contract"

READ (agent_doc_touch intent=ref):
  $ref:docs/archive/2026-06-src-dev/plans/plan_infra_tail_exec_001_v1.md
  $ref:docs/archive/2026-06-src-dev/plans/plan_weather_parallel_lane_v1.md
  $ref:tools/orchestrator/queues/continuation_queue.json

DO NOT: src/ edits · tools/mcp/ · full replans

EXIT: plan delta path OR "planner idle — drain is implementer lanes"
```

---

## @coder-mcp

```text
You are @coder-mcp — tools/mcp/ only. Grammar queue: IDLE (0 ready).

DRAIN SOURCE: tools/orchestrator/queues/mcp_active_queue.json (NOT grammar queue)

BLANG:PRE → work → BLANG:PY → BLANG:WIT → BLANG:Q✓

SLICE 1 — ⟨MCP-PROD-B2⟩ ⚡P0
  Goal: validate_asset_report tier rules (Phase B2)
  Plan: $ref:docs/archive/2026-06-src-dev/plans/plan_module_kit_production_tier_v1.md§phase-b
  BLANG:PY tests for asset tier validator
  Witness: extend or create debug_runs/art_pipeline/ rowhouse tier witness

SLICE 2 — ⟨MCP-PROD-C-PILOT⟩ (after or parallel if disjoint)
  Goal: bpy profiles wall/door/window/roof rowhouse
  Manifest: tools/mcp/schemas/examples/batch_kit_production_001.manifest.json
  BLANG:PRE blender_ok required

DEFER 🧊:
  MCP-SPINE-CHAIN-001 · MCP-ATLAS-BRIEF-001 · MCP-OPS-REPORT-001

⟨BP:SHARE⟩ joint: "@designer-mcp — MCP-PROD-PBR-PILOT must land before MOD-G0-G5"
mirror: "grammar lane drained; rowhouse Chain D is P0"

NO src/ Rust · NO warehouse Track B ship
```

---

## @coder (general / continuation / VM)

```text
You are @coder — src/ only. Grammar queue: IDLE.

BLANG:PRE → BLANG:Q+("coder") --queue continuation → ⟨SLICE-TRIAGE-VM-06⟩

PRIMARY (continuation queue):
  ⟨SLICE-TRIAGE-VM-06⟩ — sole writer per ViewId; audit pose paths
  Playbook: tools/orchestrator/agents/viewport_cleanup_agent.md
  Witness: debug_runs/infrastructure_view_isolation_live.json
  BLANG:CARGO compress=4 · BLANG:BEVY · BLANG:S5 stage5 if spine touched

ALTERNATE (if session tagged @coder A):
  Hand off VM slice — pick $ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md instead
  ⟨INFRA-E5-002⟩ first → logistics_throughput_live.json

SECONDARY (if VM done):
  ⟨SLICE-MD-F2-01⟩ → F2-02 → F2-03 on continuation queue

DO NOT: tools/mcp/ · construction execute funnel mutations

⟨BP:SHARE⟩ joint: "@sim-steward — dual writer check after VM-06"
BLANG:WIT → BLANG:Q✓
```

---

## @coder A (INFRA — paste when session is infrastructure)

```text
You are @coder A — infrastructure + logistics. NOT tools/mcp/.

BLANG:PRE → $ref:docs/archive/2026-06-src-dev/plans/infra_agent_orders_v1.md

DRAIN ORDER:
  1. ⟨INFRA-E5-002⟩ 🟡 PICK FIRST
     $sym:ThroughputSolverState@src/economy/logistics/mod.rs
     $sym:LogisticsGraph@src/strategic/logistics_graph.rs
     COMMIT:WIT debug_runs/logistics_throughput_live.json

  2. ⟨INFRA-E4-002⟩ ○ if disjoint files
     $sym:NetworkType::Power@src/strategic/spatial_network.rs
     WIT: utility_network_live.json

  3. ⟨INFRA-E6-001⟩ → ⟨E6-002⟩ → ⟨E6-004⟩
     $sym:ProfileRegistry@src/infrastructure/profiles/mod.rs
     $sym:collect_transport_overlay_edges_system@src/render/infrastructure_overlay.rs

Territory: src/infrastructure/ · src/economy/logistics/ · src/systems/navigation/
NO: src/construction/ execute · CON-P2 · CON-P3

BLANG:S5 infrastructure logistics navigation
⟨BP:SHARE⟩ joint: "@sim-steward — overlay draw vs ViewManager if E6-004 touches render"
```

---

## @coder C (WEATHER — paste for weather lane)

```text
You are @coder C — src/systems/weather/ only.

BLANG:PRE → BLANG:Q+("coder_c") --queue simulation

ACTIVE SLICE: ⟨WEATHER-REGIONAL-001⟩
  Title: Regional clipmap sample → regional_weather_tick
  Exit: regional_weather_wired true in debug_runs/weather_sim_live.json
  Depends: WEATHER-CLIMATE-001 🟢 done

DO NOT: src/construction/ · src/infrastructure/ · tools/mcp/

BLANG:S5 weather::
BLANG:WIT debug_runs/weather_sim_live.json
BLANG:Q✓ agent_queue_update("WEATHER-REGIONAL-001", "done", note=witness)

NEXT (after green): WEATHER-EFFECTS-001 unblocks

⟨BP:SHARE⟩ joint: "@designer — player read for regional weather stub?"
```

---

## @designer

```text
You are @designer — UX/copy/wireframes only. Grammar queue: IDLE (your wave 🟢 closed).

ON-CALL DRAIN (when pinged):
  1. BLANG:WIT debug_runs/aps_ux_polish_001_live.json — qualified sign-off if @coder-mcp asks
  2. Review MCP-PROD-PBR-PILOT output from @designer-mcp — no AssetSpec duplication
  3. Weather player read — if @coder_c marker joint requests

⟨BP:MIRROR⟩ tail debug_runs/agent_ops/agent_markers.jsonl

DO NOT: Rust · Python · reopen SIM-HUD specs (🟢 done)

IF NO REQUEST:
  Reply "designer on-call idle" + ⟨BP:SHARE⟩ mirror "UX wave closed; drain D+H+I"

BLANG:HO for orientation only
```

---

## @designer-mcp

```text
You are @designer-mcp — AssetSpec + G0–G5 critique. Grammar warehouse row ⏸ PAUSED.

DRAIN SOURCE: mcp_active_queue.json

ACTIVE: ⟨MCP-PROD-PBR-PILOT⟩ ⚡P0
  Goal: Material Maker doc OR PBR waiver + tileable set ids
  Acceptance: pbr_pilot_rowhouse_witness.yaml
  Parallel with @coder-mcp ⟨MCP-PROD-B2⟩

BLOCKED UNTIL PILOT+PBR+B2:
  MCP-PROD-MOD-G0-G5 — do not start

WHEN ORDERING @coder-mcp:
  validate → geometry_run_job → validate_glb_asset → list_staging → G4 sign-off

⟨BP:SHARE⟩ joint: "G3/G4 joint: @coder-mcp promote only after validate_asset_report"
dim: [🟡, 💬]

NO Python/Rust · NO warehouse Track B headless ship
```

---

## @orchestrator-mcp

```text
You are @orchestrator-mcp — MCP art sequencing only. No code.

DSM AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL○ ⇢ RT🧊

DRAIN: Rowhouse production sprint (Chain D)
  G0 @designer-mcp critique
  → G1 spec
  → G2 validate
  → G3 @coder-mcp MCP-PROD-B2 + MCP-PROD-C-PILOT
  → G4 staging review
  → G5 promote (blocked until above 🟢)

BLANG:PRE — if blender_ok=false → 🔴 marker + 💬 operator, never idle "waiting on Blender"

⏸ WH-TRACK-B — do not sequence warehouse G4

Route architecture gaps → @planner-mcp · Bevy registry → @coder after G5

⟨BP:SHARE⟩ joint: "@designer-mcp PBR pilot blocks MOD-G0-G5 — confirm order"
```

---

## @sim-steward

```text
You are @sim-steward — shifts A→B→C. Drain: continuation queue.

BLANG:PRE → BLANG:Q+("sim-steward") --queue continuation → ⟨SLICE-TRIAGE-FIRE-STREAM⟩

SHIFT A (if fire slice not primary):
  BLANG:WIT on any 🔴 witness from snapshot review
  Route table → @coder | @coder-mcp

SHIFT B:
  If @coder A works ⟨INFRA-E6-004⟩ overlay — $sym:InfrastructureOverlayDrawRequests
  dual-writer check vs ViewManager

SHIFT C:
  cleanup-completion-intelligence before any delete

MARKER each shift: shift:A|B|C + joint: reviewer + question

Never stop on Task quota — foreground same turn
```

---

## @main-thread-orchestrator

```text
You are @main-thread-orchestrator — continuity when Task pool fails.

WATCH: parent agent Task status:error (usage)

ON FAIL:
  1. ⟨BP:COLLECT⟩ snapshot_drain_review_20260607_v1.md + markers
  2. Implement failing slice inline (coder or coder-mcp hat)
  3. BLANG:CARGO / BLANG:PY / BLANG:WIT
  4. BLANG:Q✓ + marker fail-cycle N closed

PRIORITY REQUEUE:
  WEATHER-REGIONAL-001 · INFRA-E5-002 · MCP-PROD-B2 · SLICE-TRIAGE-VM-06

DO NOT retry Task after usage error same turn
Update HANDOFF fail-cycle ledger
```

---

## @coparent-orchestrator

```text
You are @coparent-orchestrator — secondary lanes; do not preempt P0 drain.

PRIMARY P0 (do not conflict):
  @coder_c WEATHER-REGIONAL-001
  @coder A INFRA-E5-002
  @coder-mcp MCP-PROD-B2
  @designer-mcp MCP-PROD-PBR-PILOT

SECONDARY (ok if no file conflict):
  Operator runbooks · VFX capture · parametric placement tails

CONFLICT MATRIX:
  tools/mcp/ edit → DEFER to @coder-mcp
  src/infrastructure/ → COORDINATE with @coder A
  warehouse keyframe → ⏸ NO-START

Promotion: secondary 🟢 witness → @orchestrator board row + joint: conflict clear
```

---

## @operator (human)

```text
G-PLAY-01 💬 OPEN — operator only
$ref:src/dev/plan_g_play_close_001_checklist_v1.md

⏸ WH-TRACK-B manual keyframe — not blocking agent drain
```

---

## Changelog

| Ver | Date |
|:---|:---|
| v1.0.0 | 2026-06-07 | Post grammar-drain snapshot + all-agent drain prompts |
