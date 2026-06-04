# ARCH-MATERIAL-AUTHORITY-001 — Material assignment in Assembly Snapshot / APS `v1`

| Field | Value |
|:---|:---|
| **Todo ID** | **ARCH-MATERIAL-AUTHORITY-001** |
| **Source** | [`prompts/planner_fix_auto_build.md`](../../prompts/planner_fix_auto_build.md) **L1315–L1700** |
| **Status** | **ACTIVE** — Phase A/B: [`plan_material_studio_phase_v1.md`](plan_material_studio_phase_v1.md) |
| **Date** | 2026-06-03 |

---

## What we kept getting wrong

Agents treated ship path as:

```text
Assembly → Open Blender → Assign materials → Viewport sanity → keyframe_render
```

Planner target (same doc, L1329–L1395):

```text
Assembly snapshot (module + material_profile + tags + lod per node)
    → Art Pipeline Suite (assign materials, preview variants, preview assembly)
    → MCP build blend (inherits snapshot)
    → Headless Blender worker (apply materials, render, export)
```

**Designer does not open Blender for materials.** Blender is a **compiler**, not the material editor.

**Pause** manual keyframe warehouse planning (L1700) until this authority move is in place — otherwise we harden a DCC workflow we already rejected.

---

## Authority model

| Layer | Owns |
|:---|:---|
| **Assembly snapshot** | `material_profile`, `variant_tags`, `semantic_tags`, `lod_policy` per placement |
| **APS** | Edit assignments, material library UI, live preview |
| **Blender worker** | Import GLB, apply profiles from snapshot, render, export |

Blend build **reads** snapshot; it does not invent materials.

---

## Immediate planner order (before PILOT-GRAMMAR-001 ship)

| ID | Owner | Task | Status |
|:---|:---|:---|:---:|
| **ARCH-MATERIAL-AUTHORITY-001** | @planner | This doc + queue/HANDOFF alignment | **active** |
| **APS-MATERIAL-BROWSER-001** | @coder-mcp | Material library with thumbnails (not combobox-only); click → apply to selected slot | **done** — [`material_browser.py`](../../tools/mcp/art_pipeline_suite/material_browser.py), witness [`aps_material_browser_live.json`](../../debug_runs/aps_material_browser_live.json) |
| **APS-PREVIEW-001** | @coder-mcp | Material browser thumbs (albedo/normal/roughness) | **partial** |
| **APS-PREVIEW-002** | @coder + @coder-mcp | Assembly preview (Bevy worker) | **done** |
| **APS-PREVIEW-003** | @coder-mcp | Variant preview on assembly state | **pending** |
| **PG-MATERIAL-GENERATION-001** | @coder + @coder-mcp | Grammar emits `material_profile` per placement | **done** |
| **PLAN-MATERIAL-STUDIO-001** | @planner-mcp | L1701+ Material Studio — [`plan_material_studio_phase_v1.md`](plan_material_studio_phase_v1.md) | **active** |
| **BUILD-WORKER-001** | @coder-mcp | Blender: snapshot material apply + render | **Phase B** |
| **MCP-PILOT-GRAMMAR-001** | @designer-mcp | G4 after Phase A+B — not Blender material UI | **blocked** |

---

## Done vs gap

| Item | State |
|:---|:---|
| ARCH-003 `material_profile` on placements (enrich from index) | **done** — `assembly.py` |
| APS combobox `list_material_profiles()` | **partial** — does not scale (L1514–L1539) |
| Rust/Python grammar `material_profile` from rules | **gap** — `building_grammar.rs` / `building_grammar.py` do not emit per placement |
| Material library (Assembly tab) | **partial** — thumbnails; no Materials tab / categories / sphere-wall previews |
| Pilot checklist Phase 2 “assign in Blender” | **superseded** — see [`pilot_grammar_001_g4_checklist_v1.md`](pilot_grammar_001_g4_checklist_v1.md) banner |

---

## Correct PILOT flow (when unblocked)

1. `generate(IndustrialWarehouse, industrial_west, seed)` → snapshot with `material_profile` on every placement.
2. APS: verify/override materials; preview assembly (Bevy worker).
3. `Save snapshot` → MCP `build-assembly` → headless apply + **keyframe_render** worker.
4. Designer G4 on **worker output** PNGs (manual keyframe = render step authority, not material editing in Blender UI).

---

## References

- [`aps_preview_004_bevy_worker_v1.md`](aps_preview_004_bevy_worker_v1.md)
- [`arch_assembly_graph_002_v1.md`](arch_assembly_graph_002_v1.md)
- [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md)
