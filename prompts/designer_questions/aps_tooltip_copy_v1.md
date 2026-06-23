# APS-UX-TOOLTIPS-002 — Approved tooltip copy `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-UX-TOOLTIPS-002 |
| **Owner** | `@designer` (copy) · `@coder-mcp` (wire + `aps_tooltips.py`) |
| **Source of truth** | This doc → merge into `tools/mcp/art_pipeline_suite/aps_tooltips.py` |
| **Date** | 2026-06-03 |
| **Verdict** | **APPROVED** — ready for coder-mcp merge |

**Rules:** Max ~360px wrap · plain language · name the authority (snapshot vs sidecar) · no hover-only critical paths without on-screen duplicate where P0.

---

## Global / chrome

| Key | Control | Approved copy | Wired |
|:---|:---|:---|:---:|
| `flow_send_assembly` | Flow → Send to Assembly | Copy the selected module's style pack into Assembly and open that tab. Set footprint, then Generate snapshot. | ✓ |
| `flow_bake_variants` | Flow → Bake variants | Save your assembly snapshot first. Then expand variant_set → tile_batch on Atlas tab. Does not open Blender. | ✓ |
| `flow_pack_atlas` | Flow → Pack atlas | Pack the PNG folder into a tile_map atlas (tilemapgen). Use after keyframe stills exist or tile batch output is ready. | ✓ |
| `pipeline_step` | Pipeline bar step label | Done = you have data for this step. Pending = visit the tab and complete the action shown in the hint line. | ✓ |
| `pipeline_catalog` | Pipeline · Catalog | Find and validate modules before assembly. Sidecar tags are hints only. | **new** |
| `pipeline_assembly` | Pipeline · Assembly | Snapshot is ship authority — materials and semantic tags live here. | **new** |
| `pipeline_materials` | Pipeline · Materials | Browse profiles; assign on Assembly after selecting a footprint cell. | **new** |
| `pipeline_variants` | Pipeline · Variants | Visual states for tile batch — requires saved assembly snapshot. | **new** |
| `pipeline_atlas` | Pipeline · Atlas | QC PNGs and packed atlas before registry. Keyframe in Blender is separate. | **new** |
| `meta_flow` | Metadata → engine checkbox | Read-only map: snapshot → worker preview → variant_set → atlas → engine. Expand when unsure where to edit. | ✓ |

---

## Catalog tab

| Key | Control | Approved copy | Wired |
|:---|:---|:---|:---:|
| `tab_catalog` | Tab | Find modules, check GLB health, edit sidecar hints. Sidecar tags are hints only — assembly snapshot is ship truth. | ✓ |
| `cat_batch_filter` | Batch combobox | Filter module list by production batch_id from the index. | **new** |
| `cat_category_filter` | Category combobox | Filter by module category (wall, roof, door, …). | **new** |
| `cat_refresh` | Refresh list | Reload module index from disk after external edits. | **new** |
| `cat_sidecar_truth` | Sidecar authority line | Sidecar tags and batch_id are module hints. Materials and semantic tags on assembly_snapshot win at runtime. | ✓ |
| `cat_metadata` | AssetSpec sidecar / notebook | Edit AssetSpec sidecar JSON. Index row is read-only. Assembly semantic_tags override sidecar at ship time. | ✓ |
| `cat_validate` | Validate GLB | Runs GLB tier and mesh checks. FAIL means fix the module before using it in assemblies or style packs. | ✓ |
| `cat_save_metadata` | Save metadata | Write sidecar JSON to disk. Does not update assembly_snapshot. | **new** |
| `cat_reindex` | Reindex library | Rebuild module index from folder scan — use after adding GLBs. | **new** |
| `cat_browser_preview` | Preview in browser | Opens isolated GLB in browser — slower than list thumb; good for mesh check. | **new** |
| `cat_trimesh` | 3D preview (trimesh) | Local mesh viewer — dev check only, not ship proof. | **new** |
| `cat_list_thumb` | Module list row | Select module — thumb shows isolated GLB when indexed. | **new** |

---

## Assembly tab

| Key | Control | Approved copy | Wired |
|:---|:---|:---|:---:|
| `tab_assembly` | Tab | Build or load assembly_snapshot — the only place materials and semantic tags ship to the engine. | ✓ |
| `asm_grammar` | Use building grammar | When on, footprint size comes from grammar massing — manual W×D and floors are disabled. | ✓ |
| `asm_archetype` | Archetype combobox | Building archetype (IndustrialWarehouse, …) — drives grammar rule tables. | **new** |
| `asm_district` | District combobox | District style pack and default material profiles (e.g. industrial_west). | **new** |
| `asm_style_pack` | StylePack combobox | Module resolver style pack for plain (non-grammar) generate. | **new** |
| `asm_tier` | Tier combobox | Production tier for GLB validation — smoke vs production paths. | **new** |
| `asm_generate` | Generate snapshot | Grammar mode: archetype + district + seed. Plain mode: style pack + footprint. Both write a new snapshot. | ✓ |
| `asm_footprint_dims` | W×D / floors spinboxes | Footprint size when grammar is off. Grammar on: read-only from massing. | **new** |
| `asm_save` | Save snapshot | Save assembly_snapshot to disk. Required after material or tag edits. Engine reads this file — not Blender. | ✓ |
| `asm_save_reminder` | Next-step callout | Unsaved edits are lost if you switch tabs or close APS. Save before Bake variants or Preview assembly. | ✓ |
| `asm_load` | Load snapshot | Open existing assembly_snapshot JSON from staging or examples. | **new** |
| `asm_validate` | Validate | Check GLB paths, tiers, and required fields. Fix FAIL items before P0 gate or variant bake. | ✓ |
| `asm_p0` | P0 gate | Production gate: grammar chain + tier checks must pass before ship or tile bake. | ✓ |
| `asm_preview` | Preview assembly | Full assembly in Bevy worker or browser — slower than slot thumbs; confirm overall look. | ✓ |
| `asm_footprint` | Footprint canvas | Click a cell to select a placement slot. Colors show role density; selection drives material apply and slot previews. | ✓ |
| `asm_footprint_heatmap` | Footprint role colors | Cell tint = placement role density — not ship status. Select cell to edit tags and material. | **new** |
| `asm_material_lib` | Material browser (assign mode) | Step 1: click a footprint cell. Step 2: pick a profile. Step 3: Apply (or double-click profile). | ✓ |
| `asm_slot_preview` | Slot preview panel | Four reads: isolated module, material on primitives, combined, placement on footprint grid. | ✓ |
| `asm_tags` | Semantic tag categories | Tags filter engine placement rules (location, architecture, detail, condition). Saved per slot on snapshot. | ✓ |
| `asm_grammar_inspector` | Grammar inspector tree | Rule chain from snapshot — pinned overrides shown in bold when lineage fields present. | **new** |
| `asm_engine_path` | Material authority panel | Read-only: runtime reads placement.material_profile from this snapshot path. | **new** |
| `asm_iterate` | Iterate grammar (Phase 2) | Change one grammar layer without full seed reroll — Apply iteration when Phase 2 ships. | **new** |

---

## Materials tab

| Key | Control | Approved copy | Wired |
|:---|:---|:---|:---:|
| `tab_materials` | Tab | Browse and preview material profiles. Assign on Assembly tab after selecting a footprint cell. | ✓ |
| `mat_use_in_assembly` | Use in Assembly | Switch to Assembly tab with this profile highlighted — select a footprint cell, then Apply. | **new** |
| `mat_add_profile` | Add profile… | Register a new profile id in the material registry. | **new** |
| `mat_search` | Search field | Search by profile id, display label, category, or generator id. | ✓ |
| `mat_category` | Category combobox (legacy filter) | Filter flat category list — prefer category tree in studio layout. | ✓ |
| `mat_category_tree` | Categories tree | Browse Industrial → Steel, Residential → Brick, etc. Counts show profiles per node. | **new** |
| `mat_status` | Map status glyph + text | Ready = all maps present. Partial = some maps missing. Missing = generate or drop PNGs in profile folder. | ✓ |
| `mat_generate` | Generate selected | Create placeholder maps when PNGs are missing. Replace with authored textures, then Reload preview. | ✓ |
| `mat_generate_all` | Generate all missing | Batch-generate placeholders for every profile missing maps — dev bootstrap only. | **new** |
| `mat_open_folder` | Open texture folder | Open profile folder in Explorer — drop albedo/normal/metallic PNGs here. | **new** |
| `mat_open_registry` | Open registry JSON | Edit material_profiles registry — advanced; prefer Add profile dialog. | **new** |
| `mat_reload_preview` | Reload preview | Refresh thumb after replacing PNGs on disk. | **new** |
| `mat_apply` | Apply to selected slot (assign mode) | Writes material_profile on selected Assembly placement. Select footprint cell on Assembly first. | ✓ |
| `mat_preview_modes` | Preview modes panel | Sphere = quick read. Wall strip = facade scale. Building section = massing check before assign. | ✓ |

---

## Variants tab

| Key | Control | Approved copy | Wired |
|:---|:---|:---|:---:|
| `tab_variants` | Tab | Define visual states (day/night, damage, fill) from assembly. Bake prepares tile_batch — not a substitute for Save on Assembly. | ✓ |
| `var_load` | Load… | Open variant_set JSON or RON from disk. | **new** |
| `var_load_example` | Load example | Load pilot variant_set for warehouse industrial west. | **new** |
| `var_new_from_assembly` | New from assembly | Create variant_set from current assembly_id and snapshot variant_tags. | **new** |
| `var_save` | Save JSON / RON | Write variant_set to disk — required before tile batch expand. | **new** |
| `var_validate` | Validate | Check variant_set schema and assembly_id linkage. | **new** |
| `var_layers` | Layer controls (general) | Lighting, damage, fill, and material overrides become variant_key rows. Apply layers, then Save. | ✓ |
| `var_apply_layers` | Apply layers to selected | Commit dropdowns onto the selected variant row. Preview updates live while editing; Apply saves the row. | ✓ |
| `var_lighting` | Lighting combobox | day · night_off · night_on — drives tile lighting layer and Night preview chip. | ✓ |
| `var_power` | Power combobox | Grid story for reaction sessions — off · partial · on. | ✓ |
| `var_damage` | Damage state combobox | clean · dirty · damaged · ruined — visual damage layer. | ✓ |
| `var_fill` | Fill combobox | Occupancy overlay for sim tiles — not a geometry swap. | ✓ |
| `var_draft_preview` | Variant preview panel | Preview shows current controls. Draft strip until Apply commits the row. | ✓ |
| `gen_trace_approve` | Approve snapshot checkbox | Artist sign-off that this assembly is the parent for variant rows and tile bake. | ✓ |
| `gen_trace_edit_assembly` | Edit on Assembly | Switch to Assembly tab to change archetype, district, seed, or regenerate. | ✓ |
| `var_bake_hint` | Flow / Atlas link | After Save: Atlas tab → From variant set → Run tile batch. | **new** |

---

## Atlas tab

| Key | Control | Approved copy | Wired |
|:---|:---|:---|:---:|
| `tab_atlas` | Tab | QC source PNGs and packed atlas before registry. Keyframe capture in Blender is a separate step when stills are missing. | **new** |
| `atl_batch_json` | tile_batch path | tile_batch_v1 JSON — input to tile-batch-run. | **new** |
| `atl_batch_run` | Run tile batch | Expand variant_set to staging PNG folder via MCP pipeline. | **new** |
| `atl_folder` | PNG folder | Folder of per-variant PNG stills (keyframe or batch output). Preview cells before pack. | ✓ |
| `atl_keyframe_rename` | -pk rename checkbox | Rename keyframe PNGs to pack-friendly names before tilemapgen. | **new** |
| `atl_pack` | Pack atlas | Run tilemapgen on PNG folder. Produces tile_map_*.png and atlas_meta.json. | ✓ |
| `atl_preview` | Refresh preview | Reload packed atlas thumb, UV grid, and cell strip from folder. | ✓ |
| `atl_validate` | Validate atlas meta | Run atlas_meta v2 checks — plain-language result below. Fix FAIL before register. | **new** |
| `atl_qc` | Validate atlas meta (alias) | Check each cell for cropping, alpha, and variant_key naming before register. FAIL = do not ship. | ✓ |
| `atl_open_folder` | Open PNG folder | Open staging folder in Explorer for manual PNG edits. | **new** |
| `atl_cell_strip` | Source PNG cells | Click a cell to inspect variant_key, grid position, and UV — highlights matching atlas grid cell. | **new** |
| `atl_uv_grid` | Packed atlas + grid overlay | Gray grid = columns×rows from atlas_meta. Blue outline = selected cell. | **new** |
| `atl_lod0` | lod0 batch (advanced) | CI/smoke ortho batch — not ship art by itself. Collapse under Advanced when polish lands. | **new** |
| `atl_batch` | Smoke/register batch | CI ortho OR register-only path — not a substitute for keyframe stills on ship art. | ✓ |

---

## Merge checklist (@coder-mcp)

1. Add **new** keys to `TOOLTIPS` dict (this doc is authoritative).
2. Bind keys marked **new** in: `catalog.py`, `assembly_panel.py`, `material_library_widget.py`, `variants_panel.py`, `atlas_panel.py`, `atlas_preview_panel.py`, `pipeline_status_bar.py`.
3. Per-step pipeline tooltips: use `pipeline_catalog` … `pipeline_atlas` on each step label.
4. Do not change **Approved copy** without designer row in registry.

---

## Sign-off

```text
APS-UX-TOOLTIPS-002 copy doc complete
Keys: 36 wired + 42 new = 78 total
Sign-off: APPROVED
```
