# AGENT-HUB-QUEUE-001 — unified pick board for all agents

```text
Generated: 2026-06-20 (refresh with scan_queues_hub.py)
Machine truth: tools/orchestrator/queues/agent_hub_queue_v1.json
Plans index:   src/dev/plan_designer_work_202606_v1.md · src/dev/development_plan_index.md
Handoff:       tools/orchestrator/queues/HANDOFF.md
```

## How to use this hub

1. **Session start:** read **`plan_multi_parallel_tracks_v1.md`** + dispatch board + `HANDOFF.md` — do not guess from memory.
2. **Pick model:** **no global primary.** Filter `multi_parallel_tracks_dispatch_v1.json` by `owner=<you>` + `status=ready` + lowest `wave` — pull from **any track**.
3. **Cross-drain:** blocked on track A → pick next `ready` row on track B (same owner). Never idle.
4. **Dual Q✓:** close dispatch row **and** home program queue row + witness.
5. **pick_now below** = auto-scan of home queues only — use dispatch board for parallel pull across tracks.
6. **Refresh:** `python tools/orchestrator/scripts/scan_queues_hub.py` after home queue edits.

```text
FILTER owner → wave-0 ready on ANY track → blocked? → cross-drain same owner → WIT-HON → dual Q✓
```

---

## Multi-parallel tracks (AUTHORITATIVE pull board)

**Plan:** [`plan_multi_parallel_tracks_v1.md`](plan_multi_parallel_tracks_v1.md)  
**Machine:** [`multi_parallel_tracks_dispatch_v1.json`](../tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json) (52 rows)  
**Wave-0 orders:** [`multi_parallel_tracks_wave0_orders_v1.md`](../tools/orchestrator/queues/multi_parallel_tracks_wave0_orders_v1.md)

| Track | ID | Focus | Wave-0 pull (examples) |
|:---|:---|:---|:---|
| **T1** | APS-STUDIO | Tk artist studio 9/10 | DES-APS-INTERACTION · **ASM-PREFAB-IA** · OVR-P5-TAIL |
| **T2** | GRAMMAR-SHIP | pilots + G4 + build-set | CODER-PILOT-REFACTOR · CMCP-GRAM-* · GRAM-CONTENT-005 |
| **T3** | VEG-SHIP | ecology art · ship:false honest | DMCP-VEG-ATLAS-SHIP · E4 expand · minimap legend |
| **T4** | FIRE-SIM | fuel spread · smoke bridge | FIRE-F2-FUEL-SPREAD · WSS-SMOKE-BRIDGE |
| **T5** | SIM-HUD | picker · tray · popup · theme | COD-SIM-HUD-BUILD-PICKER · TRAY · POPUP-MIGRATE |
| **T6** | POWER-UX | construction B/C/D (A closed) | DES-POWER-NODE-HOVER · COD-POWER-OVERLAY · ISLAND-HIGHLIGHT |
| **T7** | PLAY-ACCEPT | G-PLAY operator rollup | G-PLAY-01 · G-PLAY-OPERATOR-01 · PERF-SHELL |
| **T8** | INFRA-PERF | VM · perf triage | TRIAGE-PERF-SHELL · VM-09-V2 |

**Cross-track locks:** `LOCK-APP-PY` · `LOCK-G4-OPERATOR` · `LOCK-WITNESS-STAGE5` · `LOCK-BLENDER-BATCH` — skip locked row, pick next ready.

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

### Assembly prefab listing & module potential (YOUR CONCERN)

**Symptom:** Buildings/modules **exist on disk** (`_module_index` · promoted GLBs · utility kits) but Assembly feels like it **doesn't use them** — placement list is grammar-assigned only, module field is **read-only**, Catalog is a **separate tab**, and complexity sliders are **buried**.

| Layer | What exists | What's missing |
|:---|:---|:---|
| **Index** | 100+ rows in `_module_index.json` — production + lod0 per `module_id` × `style_pack` | No in-Assembly browse filtered by **style-pack whitelist** ([`design_style_pack_registry_v1.md`](design_style_pack_registry_v1.md) signed but **not wired to picker UI**) |
| **Generate** | `assembly.generate_assembly_snapshot()` picks modules via grammar + `_resolve_module_row()` | Artist cannot **swap** a slot to another whitelisted prefab after generate |
| **Placement list** | `placement_list` + footprint grid — shows `module_id` per cell | **Not a prefab browser** — select cell → readonly `module_var` Entry |
| **Catalog tab** | `list_modules(batch_id, category)` full browse | **No "assign to selected slot"** bridge back to Assembly |
| **Sliders / push complexity** | `GrammarDnaPanel` β sliders (0–1) · `GrammarIteratePanel` (massing, W×D, modes) | Hidden in **collapsed "advanced"** sections; tier **G3 on disk** should expand DNA/iterate — easy to miss |
| **Source tier** | `production` vs `lod0` combobox (manual lane) | When production GLB missing, resolver **falls back to lod0** silently → looks like "failed potential" |
| **Set health** | `grammar_set_brief` · eval sweep · tier chip G3 | **Not G4** — `building_set_coverage` blocks full production-set claims |

**Code anchors:**
- Readonly module: `assembly_panel.py` ~L304–307 (`module_var` Entry `state="readonly"`)
- Resolver: `assembly.py` `_resolve_module_row()` — filters by `style_pack_id`, `development_tier`, `stylepack_visible`
- Sliders: `grammar_dna_panel.py` · iterate: `grammar_iterate_panel.py`
- Tier exposure: `apply_grammar_tier()` in `assembly_panel.py` — G3 → DNA/iterate **visible** when sections expanded

**Queue gap:** No row in `multi_parallel_tracks_dispatch_v1.json` for **Assembly module picker / prefab swap**. Related **done** work: PG-MODULE-AUDIT-002 (production corners/doors), APS-GRAM-P3-001 (inspector↔footprint), grammar evolution closed.

**Recommended picks (new — add to T1 APS-STUDIO when scheduling):**

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-APS-ASM-PREFAB-IA-001** | @designer | Slot module picker IA — whitelist filter, tier badge (prod/lod0/missing GLB), Catalog→slot handoff |
| **CMCP-ASM-MODULE-PICKER-001** | @coder-mcp | Replace readonly module Entry with searchable combobox · `library_search` · persist on snapshot |
| **CMCP-ASM-RESOLVE-HONEST-001** | @coder-mcp | When prod missing: inline reason + link to validate/promote — no silent lod0 fallback in UI label |

**Until those land — artist workarounds:**
1. Generate with **Source tier = production** (Setup → Manual override section when grammar off, or ensure grammar path passes `source_tier=production`).
2. At **G3**, expand **"Building shape bias"** + **"Tweak one style layer"** for β sliders and iterate modes.
3. Use **Catalog** to find module IDs · cross-check `_module_index` for production row + GLB on disk.
4. Run **Set health → Run sweep** on Assembly for massing spread before blaming prefabs.

---

### APS refactor (UI/UX overhaul + evolution)

**Two different programs — don’t merge them:**

| Program | Queue | Status | Your peeps pick |
|:---|:---|:---|:---|
| **UI/UX overhaul** (Tk chrome, tabs, design system) | `aps_uiux_overhaul_queue.json` | Machine **CLOSED** | **@designer:** DES-APS-PREVIEW-V2, INTERACTION, ONBOARD, OPERATOR-RUBRIC · **@coder-mcp:** status_atom tail (OVR-P5) |
| **Veg capability evolution** (domain router, landscape tab, LG-5) | `mcp_aps_evolution_queue.json` + `parallel_wave_aps_veg_dispatch_v1.json` | E0–E5 machine **done** | **@coder-mcp:** maintain E4 — **`ship:false`** until G4 manual · **@designer-mcp:** criteria done (`DMCP-VEG-ATLAS-SHIP-001`) |

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

**@designer-mcp:** **VEG-F01** unblocked (criteria done) — actual `ship:true` still blocked on operator G4 (`landscape_expanded_g4_signoff.yaml`).

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
| **PLAN-POWER-GRID-CONSTRUCTION-UX-001** | **ACTIVE** Track B/C/D (A closed) | [`power_grid_construction_ux_queue.json`](../tools/orchestrator/queues/power_grid_construction_ux_queue.json) |
| **PLAN-POWER-GRID-ART-ASSETS-001** downstream | **CLOSED** (hub scan skips picks) | [`power_grid_art_downstream_queue.json`](../tools/orchestrator/queues/power_grid_art_downstream_queue.json) |
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
| **APS-EVO-E4-ATLAS-EXPAND-001** | P1 | Landscape teach batch maintenance · **ship:false** until G4 | veg atlas ship |
| **MCP-PWR-NUCLEAR-BATCH-001** | P2 | Nuclear kit bpy + promote (deferred — on-call only) | — |

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
| BUILD-READ-VISUAL-002 | BUILD-READ-SHAPE-001 (@designer-mcp pilot) |

**Closed (2026-06-20):** MCP-PWR-* utility chain · DMCP-QC-* · COD-ART-HUD-ICON-ATLAS-001 — witness [`power_grid_art_downstream_close_live.json`](../debug_runs/art_pipeline/power_grid_art_downstream_close_live.json).

**Regression:** `cd tools/mcp/python && python -m pytest -k aps -q`

---

## @designer-mcp

**Rule:** AssetSpec authority; critique before bpy; no Bevy HUD code.

### DMCP-QC (designer-mcp · post-promote artist QC)

| ID | Verdict | Witness | On-call tail |
|:---|:---|:---|:---|
| **DMCP-QC-SUBSTATION-001** | PASS WITH NOTES | [`dmcp_qc_substation_live.json`](../debug_runs/art_pipeline/dmcp_qc_substation_live.json) | G4 manual stills in [`dmcp_qc_substation_yard_v1.md`](dmcp_qc_substation_yard_v1.md) §4 |
| **DMCP-QC-TRANSFORMER-001** | PASS | [`dmcp_qc_transformer_live.json`](../debug_runs/art_pipeline/dmcp_qc_transformer_live.json) | 32px keyframe still optional |

**Refresh:** `python -m rust_engine_mcp.cli pwr-downstream-close-witness` · rollup [`power_grid_art_downstream_close_live.json`](../debug_runs/art_pipeline/power_grid_art_downstream_close_live.json)

**Rule:** QC rubrics = **@designer-mcp**; bpy/promote = **@coder-mcp**. Substation machine `green` is teach-tier (48v) — not full art-ship.

### PRIMARY

Power-grid art downstream **closed** (2026-06-20). On-call only:

| ID | Priority | Goal |
|:---|:---|:---|
| **MCP-PWR-NUCLEAR-BATCH-001** | P2 | Nuclear kit bpy + promote (deferred) |

### FALLBACK (plan backlog — no machine row yet)

**All fallback rows closed (2026-06-02).** Next @designer-mcp work is **on-call** only — see blocked/deferred below.

**Closed riparian (2026-06-02):** DES-STYLE-LANDSCAPE-RIparian-001 — witness [`dmcp_style_landscape_riparian_live.json`](../debug_runs/art_pipeline/dmcp_style_landscape_riparian_live.json).

**Closed open lane (2026-06-02):** DMCP-VEG-ATLAS-SHIP-001 · DMCP-ATLAS-QC-PLAIN-002 — witness [`dmcp_designer_mcp_open_lane_live.json`](../debug_runs/art_pipeline/dmcp_designer_mcp_open_lane_live.json).

**Closed art spine hub (2026-06-02):** DMCP-LG5-KEYFRAME-QC-001 · DMCP-TILE-ROWHOUSE-V2-001 · DMCP-MAT-PROFILE-PILOT-002 · DES-GRAM-ARCHETYPE-CIVIC-001 — witness [`dmcp_art_spine_hub_wave_live.json`](../debug_runs/art_pipeline/dmcp_art_spine_hub_wave_live.json).

### BLOCKED (re-check after promote)

| ID | Blocked by |
|:---|:---|
| BUILD-READ-PILOT-002 | BUILD-READ-PILOT-001 |

**Closed (2026-06-20):** DMCP-QC-SUBSTATION-001 · DMCP-QC-TRANSFORMER-001 · utility bpy/promote chain.

---

## @coder / @coder A / @coder B

**Territory:** A = infra/sim spine · B = UI/product/veg/minimap · C = weather (see [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json)).

### @coder B — PRIMARY

| ID | Priority | Goal | Inputs |
|:---|:---|:---|:---|
| **COD-POWER-OVERLAY-RENDER-001** | P1 | Compositor strokes by `VoltageClass` + state (Track B) | [`design_power_map_overlay_v1.md`](design_power_map_overlay_v1.md) |
| **COD-POWER-ISLAND-HIGHLIGHT-001** | P1 | Island boundary + dim unpowered | [`design_power_map_overlay_v1.md`](design_power_map_overlay_v1.md) |

### @coder B — FALLBACK (sim HUD polish tail)

| ID | Track | Notes |
|:---|:---|:---|
| **COD-SIM-HUD-BUILD-PICKER-001** | Track F | [`design_sim_hud_build_picker_v1.md`](design_sim_hud_build_picker_v1.md) signed |
| **COD-SIM-HUD-TRAY-BUILD-001** | Track F | Tray Build tab |
| **COD-SIM-HUD-POPUP-MIGRATE-001** | Track F | Popup tier migration |
| **CDR-B-VEG-MINIMAP-LEGEND-UI-001** | Track D | After **DES-MINIMAP-VEG-LEGEND-002** wire spec |

### @coder A — PRIMARY / FALLBACK

| ID | Track | Notes |
|:---|:---|:---|
| **COD-POWER-ISLAND-HIGHLIGHT-001** | Power Track B | [`power_grid_construction_ux_queue.json`](../tools/orchestrator/queues/power_grid_construction_ux_queue.json) |
| **COD-UTILITY-ACTIVATION-LINK-001** | Power Track D | Activation reads `UtilityConnection` |
| **COD-POWER-TOOL-RAIL-001** | Power Track D | Utilities rail → Lines entry |
| **VEG-C14-OPERATOR-CHECKLIST-001** | Veg | Blocked on operator checklist |
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
| **DES-POWER-NODE-HOVER-001** | P1 | Power Track B | [`design_power_node_hover_v1.md`](design_power_node_hover_v1.md) — **primary construction UX pick** |
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
| **ORCH-PWR-DOWNSTREAM-001** | **done** — lane closed 2026-06-20 · on-call absorption only |

### FALLBACK

| ID | Notes |
|:---|:---|
| **WH-TRACK-B-PAUSE** | Grammar continuation — **paused**; do not reopen without planner sign-off |
| On-call absorption | [`designer_oncall_absorption_v1.md`](../docs/archive/2026-06-src-dev/plans/designer_oncall_absorption_v1.md) |

### BLOCKED

| ID | Blocked by |
|:---|:---|
| — | PWR downstream **closed** |

**Closed:** PWR-ART-DOWNSTREAM-CLOSE-001 · COD-ART-HUD-ICON-ATLAS-001 · MCP-PWR utility chain.

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
| **G** Power construction | **ACTIVE** Track B | [`power_grid_construction_ux_queue.json`](../tools/orchestrator/queues/power_grid_construction_ux_queue.json) — Track A done |
| **H** Power art | **CLOSED** | utility bpy + HUD atlas — witness close live |

---

## Queue file index (all machine queues)

| File | Role |
|:---|:---|
| [`multi_parallel_tracks_dispatch_v1.json`](../tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json) | **AUTHORITATIVE** 8-track parallel pull board |
| [`multi_parallel_tracks_wave0_orders_v1.md`](../tools/orchestrator/queues/multi_parallel_tracks_wave0_orders_v1.md) | Wave-0 copy-paste agent orders |
| [`designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) | @designer + @designer-mcp active assignments |
| [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) | @coder A/B/C territory + meta |
| [`power_grid_art_downstream_queue.json`](../tools/orchestrator/queues/power_grid_art_downstream_queue.json) | **CLOSED** art downstream |
| [`power_grid_construction_ux_queue.json`](../tools/orchestrator/queues/power_grid_construction_ux_queue.json) | **ACTIVE** construction UX — Track A done |
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
