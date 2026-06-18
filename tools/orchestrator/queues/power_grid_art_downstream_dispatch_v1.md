# Power grid art — downstream dispatch v1

**Program:** PLAN-POWER-GRID-ART-ASSETS-001  
**Queue:** [`power_grid_art_downstream_queue.json`](power_grid_art_downstream_queue.json)  
**Witness:** [`plan_power_grid_art_downstream_witness_v1.md`](../../src/dev/plan_power_grid_art_downstream_witness_v1.md)  
**Board:** [`power_grid_art_downstream_agent_todos_v1.md`](../../src/dev/power_grid_art_downstream_agent_todos_v1.md)

---

## Mode: on-call absorption only (ORCH-PWR-DOWNSTREAM-001 · 2026-06-02)

```text
ORCH-PWR-DOWNSTREAM-001 ✓ DONE — witness debug_runs/agent_ops/power_grid_art_downstream_dispatch_live.json
SCOPE: sequence + spec≠ship gate ONLY — NO parallel wave picks issued

Standing queue FROZEN — activate slices on notify:
  @coder-mcp     MCP-PWR-* chain — orchestrator activation only
  @designer-mcp  DMCP-QC-* — after promote notify
  @designer      designer_oncall_absorption_v1.md triggers
  @coder B       COD-ART-HUD-ICON-ATLAS-001 — product notify only

Upstream specs DONE: substation · transformer · nuclear · HUD icons design
Rule: spec PASS ≠ ship — bpy + promote + validate_asset when slice runs
```

## Work order (deferred — activate on notify)

```text
WHEN ACTIVATED:
  @coder-mcp        MCP-PWR-UTILITY-MANIFEST-001 → batch → promote
  @designer-mcp     DMCP-QC-* after promote
  @coder B          COD-ART-HUD-ICON-ATLAS-001

CLOSE: PWR-ART-DOWNSTREAM-CLOSE-001 (after slices land)
```

## Work order (archived full-wave draft)

---

## Copy-paste orders

### @coder-mcp — utility bpy + promote

```text
PLAN-POWER-GRID-ART-ASSETS-001 downstream

1. MCP-PWR-UTILITY-MANIFEST-001
   Finalize batch_kit_utility_power_production_001.manifest.json
   Generate missing module AssetSpec stubs for substation composition
   validate-report mcp_spec on kit + transformer specs
   Witness: debug_runs/art_pipeline/mcp_pwr_utility_manifest_live.json

2. MCP-PWR-SUBSTATION-BATCH-001
   Authority: dmcp_spec_substation_yard_v1.md + kit_substation_yard_production_001.json
   bpy: constituent modules then kit assembly
   validate-report asset_glb on staging GLB
   Witness: mcp_pwr_substation_batch_live.json

3. MCP-PWR-TRANSFORMER-BATCH-001 (parallel with 2 if capacity)
   Authority: dmcp_spec_transformer_pad_v1.md
   Witness: mcp_pwr_transformer_batch_live.json

4. MCP-PWR-PROMOTE-SUBSTATION-001 / MCP-PWR-PROMOTE-TRANSFORMER-001
   promote + library-register --rebuild-all
   Update grid_substation / grid_distribution_transformer catalog ship paths
   validate-report asset_glb on assets/models/modules/.../model.glb

FORBIDDEN: Q✓ on spec-only · lod0 stub still authority · staging-only
```

### @designer-mcp — nuclear spec

```text
DMCP-SPEC-NUCLEAR-PWR-001
Input: design_nuclear_plant_massing_v1.md (PASS)
Deliverables:
  assets/staging/specs/kit_nuclear_pwr_production_001.json
  src/dev/dmcp_spec_nuclear_pwr_v1.md
6×6 site · containment_dome_pwr hero · module whitelist §3
Witness: debug_runs/art_pipeline/dmcp_nuclear_pwr_spec_live.json
NO bpy this wave — spec sign-off only
```

### @coder B — HUD atlas

```text
COD-ART-HUD-ICON-ATLAS-001
Input: design_hud_power_icons_v1.md
Deliver:
  assets/textures/ui/power_hud_atlas.png (20×20 rail, 16×16 chips)
  assets/configs/ui/power_hud_atlas.icon_atlas.ron
  Extend src/gui/hud/icon_atlas.rs OR new power_hud_icons.rs
  Wire build rail Utilities → Lines / Substation / Transformer icons
Verify: cargo test -p proc_A_dine01 --lib icon_atlas power_hud_icons
Witness: debug_runs/sim_hud_power_icons_live.json
NEEDS-DISPLAY: operator confirms rail icons readable
```

---

## Mega-prompt (orchestrator)

```text
Drain PLAN-POWER-GRID-ART-ASSETS-001 downstream wave.
Queue: tools/orchestrator/queues/power_grid_art_downstream_queue.json
Rule: spec PASS ≠ ship — bpy + promote + validate_asset required.

Parallel: manifest + nuclear spec + HUD atlas
Then: substation/transformer bpy batch → promote → QC
Close: power_grid_art_downstream_close_live.json + WIT-HON
```
