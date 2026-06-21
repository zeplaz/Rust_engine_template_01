# Multi-parallel tracks — agent prompts `v1`

**Program:** [`plan_multi_parallel_tracks_v1.md`](plan_multi_parallel_tracks_v1.md)  
**Dispatch board:** [`multi_parallel_tracks_dispatch_v1.json`](../tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json)  
**Home queue (Q✓ authority):** [`multi_parallel_home_queues_v1.json`](../tools/orchestrator/queues/multi_parallel_home_queues_v1.json)  
**Wave-0 orders:** [`multi_parallel_tracks_wave0_orders_v1.md`](../tools/orchestrator/queues/multi_parallel_tracks_wave0_orders_v1.md)

---

## Session boot (all agents)

```text
node .claude/skills/agent-lang/driver.mjs boot <agent>
node .claude/skills/agent-lang/driver.mjs get-que <agent> --demand --minutes 60
```

**`get que`** is the default pull — returns next slice + up to 12 ready rows + optional hour-scale `demand_todos`. MCP: `get_que(agent, demand=true)`.

```text
node .claude/skills/agent-lang/driver.mjs doc src/dev/plan_multi_parallel_tracks_v1.md
```

**Pull rule:** Filter `owner=<you>` · `status=ready` · lowest `wave` · any track. Blocked on one track → **cross-drain** to another ready row (same owner). Never idle.

**Hour session:** `get-que <agent> --demand --minutes 60` → work `demand_todos[n]` in order · `slice-exec-brief <id>` per slice · WIT-HON → Q✓ after each.

**Close rule:** WIT-HON → witness → Q✓ **dispatch row + home queue row** (`agent-queue-update <id> done --note <witness>` — queue auto).

---

## @planner / @planner-mcp — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — sequencing + queue hygiene only (no implementation).

You do NOT set a global primary. You maintain the eight-track dispatch board and home queue sync.

Read first:
- src/dev/plan_multi_parallel_tracks_v1.md
- tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json
- tools/orchestrator/queues/multi_parallel_home_queues_v1.json
- src/dev/vegetation_system_honest_status_v1.md (honest gaps)
- ops-get-project-brief (build_set_health grammar_pilot_count)

This session — pick ONE:
1. QUEUE-SYNC-001 — Reconcile dispatch vs home_queue vs designer_active multi_parallel_ready; fix status drift (machine done ≠ witness green).
2. PLAN-APS-TRACK-A-INDEX-001 — Ensure DES-APS-INTERACTION/ONBOARD/PREVIEW-LADDER deliverable paths indexed in development_plan_index.md.
3. GRAMMAR-G4-GATE-001 — Document operator unblock path for MCP-PILOT-GRAMMAR-001 (LOCK-G4-OPERATOR) in pilot_grammar_001_g4_checklist_v1.md.
4. G-PLAY-ROLLUP-001 — Verify G-PLAY-01 sub-gates in master_chain_tensor_v1.json match post_drain queues.

Deliverable: plan delta or queue row status corrections — no Rust/Python/Tk.

Stop point: dispatch + home queue row counts match · no orphan ready without owner · HANDOFF lists multi-parallel not single primary.

Out of scope: bpy, Bevy ECS, APS panel implementation.
```

---

## @designer — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — spec authority only (no Python/Tk/Rust).

Pull board: tools/orchestrator/queues/multi_parallel_home_queues_v1.json
Filter: owner=designer · status=ready · wave=0 first

Wave-0 parallel picks (land ANY this session — no serial wait):
1. DES-APS-INTERACTION-001 → src/dev/design_aps_interaction_v1.md
   Primary-action feedback · disabled-reason placement · spine click affordance · list selection persist
   Ref: design_aps_smoothness_charter_v1.md · aps_design_system_v1.md status_atom

2. DES-APS-ONBOARD-SPEC-002 → src/dev/design_aps_onboard_spec_v2.md
   Expand design_aps_uiux_onboard_outline_v1.md → full first-10s path · replace MetadataFlowPanel contract

3. DES-APS-PREVIEW-LADDER-001 → src/dev/design_aps_preview_ladder_v1.md
   G0→G4 fidelity ladder (wireframe → massing → materials → variants)

4. DES-APS-MANUAL-FALLBACK-001 → src/dev/design_aps_manual_fallback_v1.md
   When manual footprint lane shows · deprecation banner copy

Cross-drain (same session if wave-0 specs waiting on review):
- DES-POWER-NODE-HOVER-001 (T6) — transformer/substation hover cards
- DES-SIM-HUD-OPS-002 (T5) — ops strip v2 spec
- VM-11-PREVIEW-AUDIT (T8) — preview vs main semantic audit

Wave-1 after INTERACTION + ONBOARD land:
- DES-APS-OPERATOR-RUBRIC-002 → operator pixel walk (NEEDS-DISPLAY)

Q✓: designer_active_queue.json multi_parallel_ready + multi_parallel_home_queues_v1.json
Witness: deliverable .md path (signed spec sections, not chat summary)

Do not: implement APS panels · mark Q✓ without deliverable file on disk
```

---

## @designer-mcp — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — AssetSpec / DMCP authority (no Bevy HUD · no Tk).

Pull board: multi_parallel_home_queues_v1.json · owner=designer-mcp

PRIMARY (P0 — unblocks veg art for whole team):
  DMCP-VEG-ATLAS-SHIP-001
  Deliverable: src/dev/dmcp_veg_atlas_ship_v1.md
  Criteria: G4/G5 ship sign-off · ship:false stays honest until criteria met
  Ref: design_landscape_lg5_expansion_matrix_v1.md · dmcp_art_spine_hub_wave_live.json
  Unblocks: VEG-F01 · VEG-F02 · player-visible ecology

ON-CALL (paused — do not start without operator):
  MCP-PILOT-GRAMMAR-001 — LOCK-G4-OPERATOR
  Runbook: pilot_grammar_001_g4_checklist_v1.md

Cross-drain while G4 blocked:
  DMCP-ATLAS-QC-PLAIN-002 (plain-language QC copy)
  DES-STYLE-LANDSCAPE-RIparian-001 (if in signoff registry ready)

Production rules: mcp-production-rules — no ship:true on landscape without G4 · keyframe_pack only

Q✓: dual write home queue + designer_active_queue multi_parallel_ready
Validate: validate-report mcp_spec on any new AssetSpec JSON
```

---

## @coder-mcp — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — tools/mcp/ only · one app.py editor per session (LOCK-APP-PY).

Boot:
  node .claude/skills/agent-lang/driver.mjs boot coder-mcp
  pytest tools/mcp/python/tests -k aps -q  (baseline)

Pull: multi_parallel_home_queues_v1.json · owner=coder-mcp · status=ready

Pick ONE territory per session:

OPTION A — T1 APS Studio (app.py territory):
  OVR-P5-TAIL-001 — status_atom migration all panels
  Ref: design_aps_design_system_v11_delta_v1.md
  Verify: pytest -k aps · test_aps_imports · test_aps_runtime_callbacks

OPTION B — T3 Veg/Landscape (no app.py):
  APS-EVO-E4-ATLAS-EXPAND-001 — 16 keyframe stills + teach batch refresh
  VEG-CATALOG-BURN-ROWS-001 — burn/scar rows in _vegetation_variant_catalog.ron
  G0: landscape_expanded_g0_rules.yaml — proceed_tile_ship: NO

OPTION C — T2 Grammar tools (CLI/MCP only):
  CMCP-GRAMMAR-FACILITY-BRIEF-001
  CMCP-SITE-ZONE-VALIDATE-001
  CMCP-GRAM-SWEEP-PROCESS-001
  GRAM-CONTENT-005 → assets/configs/buildings/grammars/civic_block_v1.ron

Cross-drain: if LOCK-APP-PY active → pick Option B or C same session.

Forbidden:
  ship:true on landscape without DMCP-VEG-ATLAS-SHIP-001
  ortho bake for production tiles
  spec-only Q✓ without bpy/validate where row requires

Q✓: grammar_continuation_queue.json row + multi_parallel_home_queues_v1.json
Interpreter: python 3.14 (Pillow) for art tools
```

---

## @coder — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — src/ sim + construction + HUD wire (not APS Tk).

Skills: bevy-simulation-grade (07-repo-authority-map first) · validation-first

Pull: multi_parallel_home_queues_v1.json · owner=coder · status=ready

Wave-0 parallel (pick one per session, cross-drain freely):

T5 Sim HUD:
  COD-SIM-HUD-EGUI-THEME-001 — UiPalette enforcement (spec: design_sim_hud_cohesion_charter_v1.md)
  COD-SIM-HUD-BUILD-PICKER-001 — rail-anchored picker (design_sim_hud_build_picker_v1.md)
  COD-SIM-HUD-TRAY-BUILD-001 — tray Build tab (design_sim_hud_tray_build_v1.md)

T2 Grammar:
  CODER-PILOT-REFACTOR-001 — remove warehouse-only Rust branches (grammar_continuation_queue)

T6 Power UX:
  COD-POWER-ISLAND-HIGHLIGHT-001
  COD-UTILITY-ACTIVATION-LINK-001
  COD-POWER-TOOL-RAIL-001

Regression: cargo test -p proc_A_dine01 --lib construction stage5 icon_atlas power_hud_icons

Hardening: coder_queue_hardening_rules_v1.md — witness exit_predicate required

Q✓: multi_parallel_home_queues_v1.json + power_grid_construction_ux_queue.json where applicable
WIT-HON: validate-report witness_honesty <witness> --compress 3
```

---

## @coder A — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — ecology · fire · infra A-half.

Pull: multi_parallel_home_queues_v1.json · owner=coder_a · status=ready

Wave-0:
  FIRE-F2-FUEL-SPREAD-001 — fuel-linked spread (plan_fire_f2_extract_exec_001_v1.md)
  FIRE-F2-READINESS-ALIGN-001 — fire_inst vs sim heat (fire_ecology_f1_todos.md F2-04)

Wave-1:
  CDR-A-VISUAL-SMOKE-ECO-001 — visual ecology capture (LOCK-WITNESS-STAGE5 — solo writer)
  VEG-F01-ATLAS-SHIP-001 — after DMCP-VEG-ATLAS-SHIP-001
  VM-10-MINIMAP-LOCKSTEP — stage5_triage_backlog.md

Cross-drain with T4/T3/T8 — not power overlay (coder B) or APS Tk.

Honest status: vegetation_system_honest_status_v1.md — lib green ≠ operator_session_green

Regression: cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology stage5

Q✓: multi_parallel_home_queues_v1.json + coder_vegetation_drain_queue.json if row exists there
```

---

## @coder B — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — product HUD · minimap · perf triage.

Pull: multi_parallel_home_queues_v1.json · owner=coder_b · status=ready

Wave-0 parallel:
  COD-SIM-HUD-POPUP-MIGRATE-001 (T5) — popup tier migration · design_sim_hud_popup_tiers_v1.md
  COD-POWER-OVERLAY-RENDER-001 (T6) — compositor strokes · design_power_map_overlay_v1.md
  CDR-B-VEG-MINIMAP-LEGEND-UI-001 (T3) — minimap_topology_legend_live.json
  TRIAGE-PERF-SHELL (T8) — frame wall / egui cost · stage5_triage_backlog.md

Wave-1:
  COD-SIM-HUD-OPS-002 — after DES-SIM-HUD-OPS-002 spec
  COD-SIM-HUD-CURSOR-001
  OPS-F01-WC-D04-001 — infra_slice3_wc_d04_ops_f01_plan_v1.md

Territory: src/gui/ — not tools/mcp/

Regression: cargo test -p proc_A_dine01 --lib minimap_compositor des_build_read

Cross-drain: if overlay blocked → popup migrate or perf triage same session
```

---

## @operator — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — T7 PLAY-ACCEPT only · NEEDS-DISPLAY.

Pull: multi_parallel_home_queues_v1.json · owner=operator · status=ready

Order (rollup leverage — unblocks veg + product truth):
  1. G-PLAY-01 — plan_g_play_close_001_checklist_v1.md
  2. G-PLAY-OPERATOR-01 — veg/fire/ecology operator checklist v2
  3. PERF-SHELL-001 — shell perf spot-check
  4. OPS-VT5-OPERATOR-001 — VT-5 flicker visual confirm (visual_run_blockers.md)

When DES-APS-OPERATOR-RUBRIC-002 lands:
  5. APS pixel walk — MIN window 960×600 · landscape lane · preview thumbs

You are the sole authority for operator_session_green and pixel sign-off.
No agent self-certifies NEEDS-DISPLAY items.

Witness: note session date + checklist section in queue Q✓ note field.
```

---

## @orchestrator / @orchestrator-mcp — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — sequencing only · never write production code.

Issue wave-0 to ALL owner lanes in parallel (multi_parallel_tracks_wave0_orders_v1.md).
Do NOT declare a global primary (power, APS, or other).

Monitor track close rows (wave-2):
  APS-STUDIO-CLOSE-001
  GRAMMAR-SHIP-CLOSE-001
  VEG-SHIP-CLOSE-001
  SIM-HUD-PHASE2-CLOSE-001

When closing a track: WIT-HON rollup witness · update HANDOFF · run scan_queues_hub.py

Route ECS/viewport drift to @sim-steward · art spec critique to @designer-mcp before bpy.

Read: ops-get-project-brief · handoff-brief · agent-queue-board
```

---

## @sim-steward — paste prompt

```text
Lane: PLAN-MULTI-PARALLEL-TRACKS-001 — triage + witness honesty · implement only when routed.

Pull when acting:
  VM-09-V2-INVERT-BRIDGE (T8) — route fix to @coder after YAML packet
  SIM-STEWARD-FIRE-REGRESS-001 (T4) — after FIRE-F2-FUEL-SPREAD-001
  VEG-SHIP-CLOSE-001 audit — vegetation_program_close honest vs operator_session_green

Skills: debug-intelligence · cleanup-completion-intelligence · bevy-simulation-grade

Shift A→B→C on witness drift. Never Q✓ dishonest rollup.
```

---

## Quick cross-drain matrix

| Blocked on… | Pull instead (same owner) |
|:---|:---|
| app.py lock (coder-mcp) | E4 landscape · CMCP-GRAM CLI tools |
| G4 warehouse (designer-mcp) | DMCP-VEG-ATLAS-SHIP |
| Power overlay deps (coder B) | SIM-HUD popup · minimap legend · PERF-SHELL |
| G-PLAY waiting (operator) | Coders continue all non-T7 tracks |
| DMCP veg ship waiting (coder A F01) | Fire fuel spread · visual smoke prep |

```text
⟦/multi_parallel_agent_prompts_v1⟧  ΔWF→ boot · filter owner · wave-0 · cross-drain · dual Q✓
```
