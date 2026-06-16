# PLAN-APS-ARTIST-TOOL-EXEC-001 — APS artist workflow (orchestrator assignment) `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-APS-ARTIST-TOOL-EXEC-001** |
| **Track** | **A — APS Product** (parallel to Track B keyframe proof) |
| **Parent** | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) · [`aps_authoring_tool_roadmap_v1.md`](aps_authoring_tool_roadmap_v1.md) |
| **Status** | **ACTIVE** — ready for `@orchestrator` assignment |
| **Date** | 2026-06-03 |
| **Rule** | **Keyframe ship / Blender bake does NOT block this plan.** Warehouse B2 runs in parallel. |

---

## Executive summary (for orchestrator)

**Goal:** Artists can build, tag, preview, and QC assets **without opening Blender** for everyday work. Metadata lands in `assembly_snapshot` (ARCH-MAT-001) and is visibly connected to engine consumption.

**Already on disk (do not re-assign):**

| ID | Deliverable |
|:---|:---|
| APS-PREVIEW-001 | Slot previews (module / material / combined / context) |
| APS-MAT-002 (partial) | Materials tab studio tree + preview modes |
| APS-ATLAS-PREVIEW-001 (v1) | Atlas tab packed atlas + cell strip + meta line |
| APS-UX-TOOLTIPS-001 (v1) | Core tooltips + metadata flow panels |
| APS-UX-PIPELINE-001 (v1) | Pipeline status bar |

**This plan:** finish APS UX, designer accessibility/readability pass, remaining previews, then optional Track B worker hook — **assigned by role below**.

---

## Assignment matrix

| Role | Owns | Does not own |
|:---|:---|:---|
| **@designer** | UX audit, IA, accessibility, tooltip copy, layout sign-off | Python/Tk implementation |
| **@designer-mcp** | Artist workflow critique, material/module QC criteria, pilot acceptance | ECS / Bevy |
| **@coder-mcp** | `tools/mcp/art_pipeline_suite/*`, preview render, atlas UI, witnesses | HUD / in-engine UI |
| **@coder** | `BUILD-WORKER-001`, Bevy preview worker enhancements, registry | APS Tk layout (unless coder-mcp blocked) |
| **@planner-mcp** | Schema/metadata contract checks in plan | Implementation |
| **@orchestrator** | Sequencing, queue rows, HANDOFF, no code | — |

---

## Phase 0 — Designer UX audit (gate before Phase 2–5 merge)

**ID:** `APS-UX-AUDIT-001`  
**Owner:** `@designer` (lead) + `@designer-mcp` (artist workflow)  
**Depends:** Run APS once (`python -m art_pipeline_suite.run`) with grammar snapshot loaded  
**Blocks:** Phase 5 sign-off; informs Phase 2–4 layout tweaks  

### Deliverables

1. **Heuristic review** (Nielsen-style + game-tools lens) per tab: Catalog, Assembly, Materials, Variants, Atlas  
2. **Accessibility & readability checklist:**
   - Contrast / font size (Consolas 8–9 vs Segoe labels)
   - Keyboard path (list → grid → apply material)
   - Tooltip density vs on-screen hints (avoid hover-only critical info)
   - Color-only status (●◐○) — add text labels where needed
   - Paned window minimum widths / scroll affordances
3. **Information architecture map:** what belongs on which tab; reduce duplicate controls  
4. **Artist journey doc:** first-time → generate warehouse → assign material → preview → variants → atlas QC (no Blender)  
5. **Recommended layout deltas** (wireframe or annotated screenshots) — max 2 pages  
6. **Sign-off row:** `tools/orchestrator/queues/designer_signoff_registry.json` entry `APS-UX-AUDIT-001`

### Acceptance

- Orchestrator can paste **≤10 bullet prioritized UX fixes** into coder-mcp queue  
- Designer explicitly scores: *clarity*, *discoverability*, *error recovery*, *accessibility* (1–10 each)

---

## Phase 1 — Recap & witness refresh (coder-mcp, ½ day)

**ID:** `APS-WITNESS-REFRESH-001`  
**Owner:** `@coder-mcp`  

| Task | Witness |
|:---|:---|
| Confirm APS-PREVIEW-001 green | `debug_runs/aps_preview_001_slot_live.json` |
| Document new modules | `debug_runs/aps_artist_tool_modules_live.json` (file list + pytest pass) |
| HANDOFF block | `tools/orchestrator/queues/HANDOFF.md` |

**Acceptance:** `pytest tools/mcp/python/tests/test_aps_preview_001.py test_aps_atlas_preview.py` green.

---

## Phase 2 — Catalog module preview (coder-mcp)

**ID:** `APS-PREVIEW-CATALOG-001`  
**Owner:** `@coder-mcp`  
**Depends:** Phase 0 IA note (slot for thumb in list)  
**Parallel:** Track B  

| Task | Detail |
|:---|:---|
| Module list thumb | GLB isolated thumb per row (reuse `try_render_glb_thumbnail_bytes`) |
| Browser preview link | Keep; thumb is faster path |
| Metadata hint | One line: “sidecar tags ≠ ship truth; assembly semantic_tags win” |
| Tooltip | `cat_metadata`, `cat_validate` — extend registry |

**Acceptance:** Select module → thumb visible in &lt;2s for indexed GLB; witness `debug_runs/aps_preview_catalog_live.json`.

---

## Phase 3 — Atlas preview v2 (coder-mcp + designer copy)

**ID:** `APS-ATLAS-PREVIEW-002`  
**Owner:** `@coder-mcp`  
**Designer input:** Phase 0 labels for grid overlay legend  

| Task | Detail |
|:---|:---|
| UV grid overlay | Draw `columns×rows` on packed atlas from `atlas_meta.json` |
| Cell ↔ grid highlight | Hover/click cell strip highlights grid cell |
| Validate atlas meta | Button → `validate_atlas_meta_v2` report in panel (plain language) |
| Register hint | “Next: `tile-atlas-register` / map stamp” with doc link |
| Open folder | Quick open PNG folder from preview |

**Acceptance:** Pilot folder `tile_warehouse_industrial_west_pilot_v1` shows 2-cell grid overlay; validator errors readable without JSON dump.

**Not in scope:** Keyframe render automation (Track B).

---

## Phase 4 — Tooltip & metadata completeness (coder-mcp + designer copy)

**ID:** `APS-UX-TOOLTIPS-002`  
**Owner:** `@coder-mcp` implements; `@designer` reviews copy  

| Surface | Keys to add |
|:---|:---|
| Variants | load/save, layer types, bake hint |
| Assembly | tag categories, footprint heatmap, grammar inspector rows |
| Catalog | batch/category filters, save metadata |
| Footprint canvas | cell colors, selection |
| Pipeline bar | each step explains prerequisite |

**Designer deliverable:** `prompts/designer_questions/aps_tooltip_copy_v1.md` — approved strings for `aps_tooltips.py`.

**Acceptance:** Every primary button on every tab has tooltip; designer copy doc merged.

---

## Phase 5 — UX polish from audit (coder-mcp + designer sign-off)

**ID:** `APS-UX-POLISH-001`  
**Owner:** `@coder-mcp`  
**Depends:** Phase 0 audit + Phase 4  

Implement **top 5** designer priorities only (orchestrator trims list). Typical candidates:

| Fix type | Example |
|:---|:---|
| Readability | Bump min font; status text beside glyphs |
| Discoverability | “Next step” callout on Assembly after generate |
| Error recovery | Validation failures → jump to field |
| Layout | Tab order; collapse rarely-used Atlas lod0 behind “Advanced” |
| Metadata panel | Default expanded on first visit; remember collapse state |

**Acceptance:** `APS-UX-AUDIT-001` designer sign-off **PASS** after polish.

---

## Phase 6 — Materials & assembly preview depth (coder-mcp)

**ID:** `APS-MAT-003` · `APS-PREVIEW-002b`  
**Owner:** `@coder-mcp`  

| ID | Task |
|:---|:---|
| APS-MAT-003 | Category tree: Industrial→Steel/Corrugated; Residential→Brick/Plaster (nested tree, not flat slash paths) |
| APS-PREVIEW-002b | Pipe assembly Bevy/browser PNG into slot **Placement context** thumb |
| APS-UX-GRAMMAR-WHY | Grammar inspector: human labels for rule_ids (designer-mcp glossary) |

**Depends:** Phase 0 IA  
**Acceptance:** 50+ profiles browsable without search; context thumb updates after “Preview assembly”.

---

## Phase 7 — Metadata authority enforcement UI (coder-mcp + planner-mcp)

**ID:** `APS-MAT-AUTH-UI-001`  
**Owner:** `@coder-mcp` · review `@planner-mcp`  

| Task | Detail |
|:---|:---|
| Snapshot diff hint | After Save: “N placements missing material_profile” |
| P0 gate plain language | Map validator codes → artist sentences |
| Engine path callout | Read-only panel: “Runtime reads: placement.material_profile → …” |
| Block misleading flows | Hide/disable “open Blender to assign materials” in APS copy |

**Ties to:** ARCH-MAT-001, [`arch_mat_001_material_authority_v1.md`](arch_mat_001_material_authority_v1.md)

**Acceptance:** New artist can answer “where does steel_panel_01 go?” from UI alone.

---

## Phase 8 — BUILD-WORKER-001 (coder / coder-mcp, Track B parallel)

**ID:** `BUILD-WORKER-001`  
**Owner:** `@coder` + `@coder-mcp`  
**Note:** Backend spine — **not** APS Tk, but unblocks honest materials in WRK  

| Task | Owner |
|:---|:---|
| Blender worker applies `material_profile` from snapshot at bake | `@coder-mcp` |
| Witness: baked GLB/PNG reflects snapshot materials | `@coder-mcp` |
| Bevy preview worker material bind parity | `@coder` |

**Does not block Phases 2–7.** Warehouse B2 keyframe still separate.

---

## Phase 9 — Integration witness & orchestrator close

**ID:** `APS-ARTIST-TOOL-E2E-001`  
**Owner:** `@coder-mcp` + `@designer-mcp` sign-off  

**Script (artist path, no Blender):**

1. Catalog: validate module thumb  
2. Assembly: grammar generate → slot previews → assign material → Save  
3. Materials: browse tree → preview modes  
4. Variants: new from assembly  
5. Atlas: point at staging folder → cell preview → pack (if PNGs present) → meta validate  

**Witness:** `debug_runs/aps_artist_tool_e2e_live.json`  
**Designer-mcp:** PASS/FAIL on “would an artist ship this workflow?”

---

## Orchestrator queue rows (paste-ready)

```text
APS-UX-AUDIT-001        @designer + @designer-mcp   Phase 0 UX/accessibility audit — GATE
APS-WITNESS-REFRESH-001 @coder-mcp                  Phase 1 witness refresh
APS-PREVIEW-CATALOG-001 @coder-mcp                  Phase 2 catalog GLB thumbs
APS-ATLAS-PREVIEW-002   @coder-mcp                  Phase 3 UV grid + validator UX
APS-UX-TOOLTIPS-002     @coder-mcp + @designer      Phase 4 tooltip copy complete
APS-UX-POLISH-001       @coder-mcp                  Phase 5 top-5 audit fixes
APS-MAT-003             @coder-mcp                  Phase 6 nested material tree
APS-PREVIEW-002b        @coder-mcp                  Phase 6 assembly context thumb
APS-MAT-AUTH-UI-001     @coder-mcp                  Phase 7 metadata authority UI
BUILD-WORKER-001        @coder + @coder-mcp         Phase 8 parallel spine
APS-ARTIST-TOOL-E2E-001 @coder-mcp + @designer-mcp  Phase 9 integration witness
```

**Suggested order:**

```text
Phase 0 (designer) ──┬──► Phase 2,3,4 (coder-mcp, parallel)
Phase 1 (coder-mcp)─┘         │
                               ▼
                    Phase 5 (coder-mcp after audit)
                               │
                    Phase 6,7 (coder-mcp)
                               │
Phase 8 (coder, parallel)      ▼
                    Phase 9 E2E + designer-mcp sign-off
```

---

## Parallel lanes (NOT blocked by broken art or keyframe ship)

These are **out of this APS Tk program** but **orchestrator may assign anytime** — they do not wait on warehouse B2 or valid ship art.

| Lane | What it is | Owner | Relation to broken art |
|:---|:---|:---|:---|
| **Bevy assembly preview** | `bevy_preview_worker` + APS "Preview assembly" | `@coder` / `@coder-mcp` | **Already works** — previews snapshot/GLBs even when atlas ship is wrong |
| **Bevy artist QC HUD** (optional) | In-engine egui panel: load snapshot, stamp atlas, inspect materials in sim viewport | `@designer` + `@coder` | **Tool feature** — can show greybox/wrong art; proves metadata path |
| **Simulation product HUD** | `src/gui/in_game_hud.rs`, dock shell, PLAY-01 | `@designer` + `@coder` | Player chrome — orthogonal to MCP art pipeline |
| **Grammar depth** | GRAMMAR-001/002 massing/facade/roof | `@planner-mcp` + `@coder` Track C | Improves **future** buildings; does not fix current broken PNGs |

**Clarification:** "Non-goal" below means **not a task row inside this Tk APS plan** — not "forbidden work."

## Out of scope for *this* program only

| Item | Where it lives instead |
|:---|:---|
| Manual Blender keyframe 24-cell ship stills | Track B · MCP-PILOT-GRAMMAR-001 B2 |
| PostgreSQL agent telemetry | [`plan_agent_operations_intelligence_v1.md`](plan_agent_operations_intelligence_v1.md) Phase 4 |
| Simulation HUD shell / multiview infra hardening | [`post_stage6_active_todos.md`](post_stage6_active_todos.md) · `@designer` |
| Full grammar generator rewrite | Track C · [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |

---

## Risk register

| Risk | Mitigation |
|:---|:---|
| Designer audit delayed → coder-mcp builds wrong layout | Phase 2–4 only after Phase 0 **draft** (48h); full sign-off before Phase 5 |
| Tooltip overload | Designer caps at ~40 keys; critical paths also on-screen |
| Tk accessibility ceiling | Document known limits; prefer visible labels over hover-only |
| trimesh missing → thumbs empty | Placeholder + “install trimesh” hint in Catalog/Assembly |

---

## References

- APS code: `tools/mcp/art_pipeline_suite/`
- Tooltips: `aps_tooltips.py`
- UI boundary: [`prompts/guides/ui_boundary_guide_v1.md`](../../prompts/guides/ui_boundary_guide_v1.md)
- Material authority: [`arch_mat_001_material_authority_v1.md`](arch_mat_001_material_authority_v1.md)
- Orchestrator HANDOFF: [`tools/orchestrator/queues/HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md)
