# AGENT-HUB-QUEUE-001 — unified pick board for all agents

```text
Generated: 2026-06-17 (refresh with scan_queues_hub.py)
Machine truth: tools/orchestrator/queues/agent_hub_queue_v1.json
Plans index:   src/dev/plan_designer_work_202606_v1.md · src/dev/development_plan_index.md
Handoff:       tools/orchestrator/queues/HANDOFF.md
```

## How to use this hub

1. **Session start:** read this file (or `agent_hub_queue_v1.json`) + `HANDOFF.md` — do not guess from memory.
2. **Pick PRIMARY** for your agent role below (highest priority `ready` / `open` row).
3. **If blocked:** do **not idle**. Pick any **FALLBACK** row for the same agent (or `parallel_ok` lane). Mark blocked row `blocked_by` in your witness; re-check primary next session after dependency Q✓.
4. **If your lane is closed:** pull from **Plan backlog** section — many rows are signed in plans but not yet machine-queued.
5. **Refresh:** `python tools/orchestrator/scripts/scan_queues_hub.py` after any queue edit.

```text
BLOCKED on X  →  pick FALLBACK same agent  →  fellow agent lands Q✓  →  re-check X
```

---

## Core lanes — APS · grammar · veg · landscape · fire

**Why this section exists:** HANDOFF and the per-agent picks below overweight **power grid**. These five programs are where most of the “we were in the middle of…” work actually lives. Machine queues often say **done** while **ship:false**, **operator pending**, or **plan tails** remain — read the **Honest gap** column.

### Lane map (one glance)

| Lane | Machine queue says | Honest gap (still open) | Where to look |
|:---|:---|:---|:---|
| **APS UI refactor** | CLOSED 24/24 | 8/10 artist score; preview/onboarding/interaction tails | [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md) Track A · [`designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) |
| **APS veg evolution (Option D / E0–E5)** | E0–E5 **done** in [`mcp_aps_evolution_queue.json`](../tools/orchestrator/queues/mcp_aps_evolution_queue.json) | Landscape atlas **`ship:false`** · burn/scar states not in catalog · G4 keyframe QC | [`plan_aps_evolution_veg_capability_20260616_v1.md`](plan_aps_evolution_veg_capability_20260616_v1.md) · [`parallel_wave_aps_veg_dispatch_v1.json`](../tools/orchestrator/queues/parallel_wave_aps_veg_dispatch_v1.json) |
| **Building grammar** | CLOSED 15/15 | Operator eyeball pending; **G1 content** thin (need civic archetype); WH-Track-B **paused** | [`aps_grammar_evolution_queue.json`](../tools/orchestrator/queues/aps_grammar_evolution_queue.json) · [`grammar_continuation_queue.json`](../tools/orchestrator/queues/grammar_continuation_queue.json) |
| **Veg sim + art tooling** | Drain **~78/82 done** | Visual smoke not operator-signed; F01/F02 atlas **blocked**; minimap legend unwired | [`coder_vegetation_drain_queue.json`](../tools/orchestrator/queues/coder_vegetation_drain_queue.json) · [`vegetation_system_honest_status_v1.md`](vegetation_system_honest_status_v1.md) |
| **Landscape LG-5/6** | Pilot + 16-tile teach batch green | **Not production-ship** · keyframe QC · burn rows · LG-6 flowers deferred | [`design_landscape_lg5_expansion_matrix_v1.md`](design_landscape_lg5_expansion_matrix_v1.md) · witness `tile_tile_landscape_expanded_v1_live.json` (`ship: false`) |
| **Fire** | F2 extract **done**; G-PLAY-FIRE **done** | F2 fuel spread · smoke bridge · steward regress **deferred** · ecology play read | [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md) · [`stage5_triage_backlog.md`](stage5_triage_backlog.md) T3 |

---

### APS refactor (UI/UX overhaul + evolution)

**Two different programs — don’t merge them:**

| Program | Queue | Status | Your peeps pick |
|:---|:---|:---|:---|
| **UI/UX overhaul** (Tk chrome, tabs, design system) | `aps_uiux_overhaul_queue.json` | Machine **CLOSED** | **@designer:** DES-APS-PREVIEW-V2, INTERACTION, ONBOARD, OPERATOR-RUBRIC · **@coder-mcp:** status_atom tail (OVR-P5) |
| **Veg capability evolution** (domain router, landscape tab, LG-5) | `mcp_aps_evolution_queue.json` + `parallel_wave_aps_veg_dispatch_v1.json` | E0–E5 machine **done** | **@coder-mcp:** maintain E4 — **`ship:false`** until G4 keyframes · **@designer-mcp:** DMCP-LG5-KEYFRAME-QC-001, DMCP-VEG-ATLAS-SHIP-001 |

**Stale doc alert:** [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) still says “DRAIN P3 next” — **ignore**; machine queue is 24/24 done. Tails live in designer work plan Track A.

**APS evolution dispatch (still authoritative for veg):** [`aps_option_d_dispatch_orders_v1.md`](../tools/orchestrator/queues/aps_option_d_dispatch_orders_v1.md) · [`plan_aps_evolution_veg_capability_20260616_v1.md`](plan_aps_evolution_veg_capability_20260616_v1.md)

---

### Grammar work

| Surface | Location | Status |
|:---|:---|:---|
| **Tier exposure + build-set guards** | `aps_grammar_evolution_queue.json` (15 rows) | Machine **CLOSED** — 3 G1 grammars on disk |
| **Warehouse pilot / G4 keyframes** | `grammar_continuation_queue.json` → **MCP-PILOT-GRAMMAR-001** | **PAUSED** (headless bake rejected — need real keyframe render) |
| **WH-TRACK-B** | `grammar_continuation_queue.json` → WH-TRACK-B-PAUSE | **active** pause — does not block APS/veg/HUD |
| **Third archetype (civic G1)** | Plan Track C2 | **done** — DES-GRAM-ARCHETYPE-CIVIC-001 concept Q✓ · RON GRAM-CONTENT-005 |
| **Grammar panel UX** | Designer signoff registry | Signed — consumer rows in evolution queue **done** |

**@designer-mcp grammar picks when not on power:** GRAM-CONTENT-005 `civic_block_v1.ron` (@coder-mcp) · resume MCP-PILOT-GRAMMAR-001 only after operator G4 runbook (`pilot_grammar_001_g4_checklist_v1.md`).

**@coder-mcp grammar picks:** CMCP-GRAM-* facility tools (brief, validate, sweep) — industrial grammar **specs done**, tools not shipped.

---

### Veg art + tooling

**Authority:** [`vegetation_system_honest_status_v1.md`](vegetation_system_honest_status_v1.md) · [`coder_vegetation_full_chain_prompt_v1.md`](coder_vegetation_full_chain_prompt_v1.md)

| Phase | Queue rows | Status |
|:---|:---|:---|
| **LG-0 → LG-4 sim** | `coder_vegetation_drain_queue.json` seq 1–70 | **Done** — lib/headless witnesses green |
| **Operator / play** | VEG-C14-OPERATOR-CHECKLIST-001 | **blocked** on @operator G-PLAY veg checklist |
| **Art ship F-phase** | VEG-F01, VEG-F02 | **blocked** on @designer-mcp / @coder-mcp LG-5 atlas ship path |
| **LG-6 flowers** | VEG-G03-LG6-FLOWERS-001 | **deferred** (correctly) |
| **Minimap veg legend** | DES-MINIMAP-VEG-LEGEND-002 → CDR-B-VEG-MINIMAP-LEGEND-UI-001 | Design **open** · coder **ready when spec lands** |

**Parallel dispatch board (full veg wave):** [`parallel_wave_aps_veg_dispatch_v1.json`](../tools/orchestrator/queues/parallel_wave_aps_veg_dispatch_v1.json) — 80+ rows; use when `coder_vegetation_drain` looks “closed” but art/play isn’t.

**Honest one-liner:** sim harness is green; **player-visible ecology + shipped veg atlas** is not.

---

### Landscape (LG-5 expanded)

| Artifact | Truth |
|:---|:---|
| 16-tile teach batch | `debug_runs/art_pipeline/tile_landscape_expanded_live.json` — **green:true** |
| Production ship | `tile_tile_landscape_expanded_v1_live.json` — **`ship: false`** |
| Burn/scar/recovery states | **Not in** `_vegetation_variant_catalog.ron` yet — blocks burn atlas authoring |
| Keyframe QC | **DMCP-LG5-KEYFRAME-QC-001** — **done** PASS WITH NOTES (`dmcp_art_spine_hub_wave_live.json`) |
| HANDOFF G0 rule | `proceed_tile_ship: no` until Track B manual keyframes |

**@coder-mcp:** after utility manifest, **APS-EVO-E4** maintenance + do **not** flip `ship:true` without designer-mcp G4 sign-off.

**@designer-mcp:** DMCP-VEG-ATLAS-SHIP-001 → unblocks VEG-F01/F02 (keyframe QC closed).

---

### Fire (never “finished” — triage lane)

Fire is **split across sim ecology, render extract, operator play, and steward regress** — no single queue owns it.

| Slice | Doc / queue | Status |
|:---|:---|:---|
| **F1 ecology** (fuel, old-growth gate) | [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md) | **Done** — `fire_ecology_live.json` |
| **F2 extract** (projection graph instances) | `coder_active_queue.json` FIRE-F2-EXTRACT-001 | **Done** — witness in stage5_full_app |
| **F2 fuel spread + smoke bridge** | `fire_ecology_f1_todos.md` F2-03/04 · `post_drain_active_queue.json` | **Open** — triage not Stage 5 gate |
| **G-PLAY fire demo** | `post_drain_phase5_queue.json` G-PLAY-FIRE-001 | Machine **done** |
| **Steward fire regress** | SIM-STEWARD-FIRE-REGRESS-001 | **deferred** — after Phase 5 slices |
| **VFX / sparks / streaming** | `designer_signoff_registry.json` FIRE7-* · [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) | Mix of done design + open capture |
| **Stage 5 triage** | [`stage5_triage_backlog.md`](stage5_triage_backlog.md) T3 | Deferred depth |

**@coder A picks:** FIRE-F2-FUEL-SPREAD-001 · WSS-SMOKE-BRIDGE-001 (from [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md)).

**@sim-steward:** SIM-STEWARD-FIRE-REGRESS-001 when product slices stable.

**@operator:** G-PLAY fire/ecology rows — split `lib_contract_green` vs `operator_session_green` per veg honest status.

---

## Program status (what is active vs closed)

| Program | Status | Queue / plan |
|:---|:---|:---|
| **PLAN-POWER-GRID-ART-ASSETS-001** downstream | **ACTIVE** P0 | [`power_grid_art_downstream_queue.json`](../tools/orchestrator/queues/power_grid_art_downstream_queue.json) |
| **PLAN-DESIGNER-WORK-202606-001** | **ACTIVE** (multi-track) | [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md) |
| **PLAN-APS-UIUX-OVERHAUL-001** | **CLOSED** 24/24 | [`aps_uiux_overhaul_queue.json`](../tools/orchestrator/queues/aps_uiux_overhaul_queue.json) |
| **PLAN-APS-GRAMMAR-EVOLUTION-001** | **CLOSED** 15/15 | [`aps_grammar_evolution_queue.json`](../tools/orchestrator/queues/aps_grammar_evolution_queue.json) |
| **APS-OPTION-D-001** | **CLOSED** 23/23 | [`aps_option_d_agent_queue.json`](../tools/orchestrator/queues/aps_option_d_agent_queue.json) |
| **POST-DRAIN phase 2–4** | **CLOSED** | `post_drain_phase2/3/4_queue.json` |
| **POST-DRAIN phase 5** build-read | **BLOCKED** (design deps) | [`post_drain_phase5_queue.json`](../tools/orchestrator/queues/post_drain_phase5_queue.json) |
| **POST-DRAIN phase 6** | **DRAINED** | [`post_drain_phase6_coder_queue.json`](../tools/orchestrator/queues/post_drain_phase6_coder_queue.json) |
| **WH-TRACK-B** grammar continuation | **PAUSED** | [`grammar_continuation_queue.json`](../tools/orchestrator/queues/grammar_continuation_queue.json) |
| **Coder master / veg drain** | **CLOSED** | [`coder_master_drain_queue.json`](../tools/orchestrator/queues/coder_master_drain_queue.json) |

---

## @coder-mcp

**Rule:** one bpy/manifest phase per session on shared `app.py`; spec PASS ≠ ship — bpy + promote + `validate_asset_glb` required.

### PRIMARY (pick now)

| ID | Priority | Goal | Unblocks |
|:---|:---|:---|:---|
| **MCP-PWR-UTILITY-MANIFEST-001** | P0 | Utility batch manifest from [`batch_kit_utility_power_production_001.manifest.json`](../tools/mcp/schemas/examples/batch_kit_utility_power_production_001.manifest.json) | substation + transformer bpy rows |
| **MCP-PWR-NUCLEAR-BATCH-001** | P2 | Nuclear kit bpy + promote (spec **done**) | PWR downstream close |

### FALLBACK (same agent — if utility blocked on Blender/env)

| ID | Track | Notes |
|:---|:---|:---|
| **CMCP-FACILITY-NEEDS-PANEL-001** | Industrial E4 | Implements signed [`design_aps_facility_needs_v1.md`](design_aps_facility_needs_v1.md) |
| **CMCP-SITE-ZONE-VALIDATE-001** | Industrial E4 | Site zone grid validator per taxonomy spec |
| **CMCP-GRAMMAR-FACILITY-BRIEF-001** | Industrial E4 | Join grammar + catalog + chain |
| **CMCP-GRAM-SWEEP-PROCESS-001** | Industrial E4 | Process histogram in eval sweep |
| **APS-EVO-E4-ATLAS-EXPAND-001** | Landscape | Teach batch done; **ship:false** until G4 keyframes — see HANDOFF G0 rules |
| **OVR-P5-TAIL-001** | APS polish | status_atom tail from design system v1.1 |

### BLOCKED (re-check after PRIMARY Q✓)

| ID | Blocked by |
|:---|:---|
| MCP-PWR-SUBSTATION-BATCH-001 | MCP-PWR-UTILITY-MANIFEST-001 |
| MCP-PWR-TRANSFORMER-BATCH-001 | MCP-PWR-UTILITY-MANIFEST-001 |
| MCP-PWR-PROMOTE-SUBSTATION-001 | MCP-PWR-SUBSTATION-BATCH-001 |
| MCP-PWR-PROMOTE-TRANSFORMER-001 | MCP-PWR-TRANSFORMER-BATCH-001 |
| BUILD-READ-VISUAL-002 | BUILD-READ-SHAPE-001 (@designer-mcp pilot) |

**Regression:** `cd tools/mcp/python && python -m pytest -k aps -q`

---

## @designer-mcp

**Rule:** AssetSpec authority; critique before bpy; no Bevy HUD code.

### PRIMARY

Power-grid specs **done** (substation, transformer, nuclear). Next machine row:

| ID | Priority | Goal |
|:---|:---|:---|
| **MCP-PWR-NUCLEAR-BATCH-001** | P2 | Coordinate with @coder-mcp on nuclear bpy — or **QC after promote** rows |

### FALLBACK (plan backlog — no machine row yet)

| ID | Track | Deliverable | Status |
|:---|:---|:---|:---|
| **DMCP-VEG-ATLAS-SHIP-001** | B1 | G4/G5 art-ship sign-off when atlas registers | open |
| **DMCP-ATLAS-QC-PLAIN-002** | B2 | Plain-language QC copy v2 for warehouse/shopfront/bunker | open |
| **DES-STYLE-LANDSCAPE-RIparian-001** | C1 | Riparian/agri visual language | open |

**Closed this wave (2026-06-02):** DMCP-LG5-KEYFRAME-QC-001 · DMCP-TILE-ROWHOUSE-V2-001 · DMCP-MAT-PROFILE-PILOT-002 · DES-GRAM-ARCHETYPE-CIVIC-001 — witness [`dmcp_art_spine_hub_wave_live.json`](../debug_runs/art_pipeline/dmcp_art_spine_hub_wave_live.json).

### BLOCKED (re-check after promote)

| ID | Blocked by |
|:---|:---|
| DMCP-QC-SUBSTATION-001 | MCP-PWR-PROMOTE-SUBSTATION-001 |
| DMCP-QC-TRANSFORMER-001 | MCP-PWR-PROMOTE-TRANSFORMER-001 |
| BUILD-READ-PILOT-002 | BUILD-READ-PILOT-001 |

---

## @coder / @coder A / @coder B

**Territory:** A = infra/sim spine · B = UI/product/veg/minimap · C = weather (see [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json)).

### @coder B — PRIMARY

| ID | Priority | Goal | Inputs |
|:---|:---|:---|:---|
| **COD-ART-HUD-ICON-ATLAS-001** | P0 | `power_hud_atlas` PNG + RON + IconId in Bevy HUD | [`design_hud_power_icons_v1.md`](design_hud_power_icons_v1.md) |

### @coder B — FALLBACK

| ID | Track | Notes |
|:---|:---|:---|
| **CDR-B-VEG-MINIMAP-LEGEND-UI-001** | Track D | After **DES-MINIMAP-VEG-LEGEND-002** wire spec |
| **COD-SIM-HUD-BUILD-PICKER-001** | Track F | [`design_sim_hud_build_picker_v1.md`](design_sim_hud_build_picker_v1.md) signed |
| **COD-SIM-HUD-TRAY-BUILD-001** | Track F | Tray Build tab |
| **COD-SIM-HUD-POPUP-MIGRATE-001** | Track F | Popup tier migration |
| **COD-POWER-LINE-DRAW-001** | Track G | Power line draw/commit (design specs done) |
| **COD-POWER-OVERLAY-RENDER-001** | Track G | Map overlay states |

### @coder A — FALLBACK (drain clear — infra tail closed)

| ID | Track | Notes |
|:---|:---|:---|
| **VEG-C14-OPERATOR-CHECKLIST-001** | Veg | Blocked on operator checklist — pick phase-6 hardening or sim-steward triage |
| **OPS-F01 / WC-D04** | Infra slice 3 | [`infra_slice3_wc_d04_ops_f01_plan_v1.md`](infra_slice3_wc_d04_ops_f01_plan_v1.md) |
| Stage 5 regression | Spine | `cargo test -p proc_A_dine01 --lib stage5` |

### BLOCKED (cross-agent)

| ID | Owner | Blocked by |
|:---|:---|:---|
| BUILD-READ-* chain | coder / coder-mcp | BUILD-READ-DESIGN-001 (@designer) |
| VEG-F01/F02 atlas | coder_a | VEG-E08-PHASE-CLOSE-001 |

---

## @designer

**Rule:** specs + charters only — no Python/Tk/Rust implementation.

### PRIMARY (open in machine queue)

| ID | Priority | Program | Witness target |
|:---|:---|:---|:---|
| **DES-APS-PREVIEW-V2-001** | P1 | APS phase 2 | [`design_aps_preview_v2_spec_v1.md`](design_aps_preview_v2_spec_v1.md) |
| **DES-STYLE-INDUSTRIAL-WEST-001** | P1 | Style C1 | [`design_style_industrial_west_v1.md`](design_style_industrial_west_v1.md) — unblocks kit002 + factory grammar |
| **DES-APS-MAT-BROWSE-001** | P1 | Materials B3 | [`design_aps_mat_browse_v1.md`](design_aps_mat_browse_v1.md) |
| **DES-MINIMAP-VEG-LEGEND-002** | P2 | Track D | [`design_minimap_veg_legend_wire_v1.md`](design_minimap_veg_legend_wire_v1.md) → @coder B |

### FALLBACK (plan-only — not yet `open` in queue)

| ID | Track | Deliverable |
|:---|:---|:---|
| **DES-APS-INTERACTION-001** | A1 | Primary-action feedback + disabled-reason placement |
| **DES-APS-ONBOARD-SPEC-002** | A2 | First-10s onboarding path |
| **DES-APS-OPERATOR-RUBRIC-002** | A2 | Pixel walk checklist v2 (NEEDS-DISPLAY) |
| **DES-APS-PREVIEW-LADDER-001** | A3 | Preview fidelity G0→G4 ladder |
| **DES-STYLE-VICTORIAN-ROW-001** | C1 | Rowhouse style bible |
| **DES-STYLE-ISO-READ-001** | C1 | Global iso readability rules |
| **DES-STYLE-PACK-REGISTRY-001** | C2 | style_pack_id → bible → modules |
| **DES-ECOLOGY-PREVIEW-V2-001** | D | World preview ecology panel |
| **DES-BUILD-READ-HUD-002** | D | Grammar read HUD v2 |
| **DES-SIM-HUD-OPS-002** | F2 | Ops strip v2 (Phase F tail) |

### BLOCKED

| ID | Blocked by | Unblocks |
|:---|:---|:---|
| BUILD-READ-DESIGN-001 | operator / prior sign-off | entire build-read spine for @coder |

---

## @orchestrator-mcp

### PRIMARY

| ID | Goal |
|:---|:---|
| **ORCH-PWR-DOWNSTREAM-001** | Sequence downstream · spec≠ship gate · **on-call absorption only** (done 2026-06-02) |

### FALLBACK

| ID | Notes |
|:---|:---|
| **WH-TRACK-B-PAUSE** | Grammar continuation — **paused**; do not reopen without planner sign-off |
| On-call absorption | [`designer_oncall_absorption_v1.md`](../docs/archive/2026-06-src-dev/plans/designer_oncall_absorption_v1.md) |

### BLOCKED

| ID | Blocked by |
|:---|:---|
| PWR-ART-DOWNSTREAM-CLOSE-001 | promote substation + transformer + HUD atlas + nuclear batch |

---

## @planner / @planner-mcp

### FALLBACK (when G-PLAY blocked)

| ID | Notes |
|:---|:---|
| **PLAN-AUDIT-020** | Blocked on G-PLAY-01 operator |
| Queue seeding hygiene | [`plan_queue_seeding_v1.md`](plan_queue_seeding_v1.md) |
| Designer work plan maintenance | Keep [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md) synced with machine queues |

---

## @sim-steward / @operations-intelligence

Read-only triage when witnesses disagree or dual-writer drift:

| Trigger | Skill | Output |
|:---|:---|:---|
| Viewport/render drift | debug-intelligence | YAML routing packet → @coder |
| Pre-delete / cleanup | cleanup-completion-intelligence | classify obsolete vs incomplete |
| Lane close / proposal review | operations-intelligence | Q/C/E + complexity budget |

**Witness rollup:** `debug_runs/agent_ops/ops_report_latest.json` · `debug_runs/unified_witness_index.json`

---

## @operator

### PRIMARY (ready in post-drain queues)

| ID | Queue | Notes |
|:---|:---|:---|
| **G-PLAY-01** | phase2 / phase3 | Play acceptance — **NEEDS-DISPLAY** |
| **PERF-SHELL-001** | phase2 | Shell perf spot-check |

### FALLBACK

| ID | Notes |
|:---|:---|
| **G-PLAY-OPERATOR-01** | Veg/fire checklist v2 when **DES-G-PLAY-OPERATOR-V2-001** lands |
| **OVR-P6-OPERATOR-EYEBALL-001** | Closed — do not re-pick unless regression |
| APS operator rubric v2 | When **DES-APS-OPERATOR-RUBRIC-002** ready |

---

## Plan backlog by track (cross-agent pull list)

From [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md) — use when machine queues show zero `ready` for your role.

| Track | Focus | Key open IDs |
|:---|:---|:---|
| **A** APS polish | @designer → @coder-mcp | PREVIEW-V2, INTERACTION, ONBOARD, OPERATOR-RUBRIC |
| **B** Art spine | @designer-mcp | LG5 keyframe QC, rowhouse v2, mat pilot 002 |
| **C** Style / G1 | @designer + @designer-mcp | INDUSTRIAL-WEST, VICTORIAN-ROW, CIVIC archetype |
| **D** Sim product UX | @designer → @coder B | MINIMAP-VEG-LEGEND, ECOLOGY-PREVIEW |
| **E** Industrial grammar | all three | CMCP facility tools (specs **done**) |
| **F** Sim HUD phase 2 | @designer → @coder | BUILD-PICKER, TRAY, POPUP tiers (specs **done**) |
| **G** Power construction | @designer → @coder | Line draw/overlay (specs **done**) |
| **H** Power art | active downstream | utility bpy + HUD atlas |

---

## Queue file index (all machine queues)

| File | Role |
|:---|:---|
| [`agent_hub_queue_v1.json`](../tools/orchestrator/queues/agent_hub_queue_v1.json) | Auto-scan output — pick_now + blocked_fallback |
| [`designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) | @designer + @designer-mcp active assignments |
| [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) | @coder A/B/C territory + meta |
| [`power_grid_art_downstream_queue.json`](../tools/orchestrator/queues/power_grid_art_downstream_queue.json) | **ACTIVE** art downstream |
| [`mcp_active_queue.json`](../tools/orchestrator/queues/mcp_active_queue.json) | MCP lane history |
| [`grammar_continuation_queue.json`](../tools/orchestrator/queues/grammar_continuation_queue.json) | WH-TRACK-B (paused) |
| [`post_drain_phase5_queue.json`](../tools/orchestrator/queues/post_drain_phase5_queue.json) | Build-read (blocked) |
| [`coder_vegetation_drain_queue.json`](../tools/orchestrator/queues/coder_vegetation_drain_queue.json) | Veg program (closed, blocked tails) |
| [`queue_registry_v1.json`](../tools/mcp/schemas/queue_registry_v1.json) | Registered queue schemas |

---

## Anti-patterns

| Do not | Do instead |
|:---|:---|
| Idle when primary is blocked | Pick FALLBACK same agent |
| Mark Q✓ without exit_predicate witness | Read witness template on program plan |
| Self-certify pixel/operator items | NEEDS-DISPLAY → @operator |
| Re-open CLOSED programs (UIUX, grammar, option-D) | New program ID + planner sign-off |
| Retry Task subagent after usage error | Foreground @agent chat per AGENTS.md |

---

## Refresh command

```powershell
python tools/orchestrator/scripts/scan_queues_hub.py
```

Then reconcile drift into this doc if new programs land.
