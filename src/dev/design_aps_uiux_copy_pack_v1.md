# APS UI/UX Copy Pack `v1` — OVR-DES-P2-COPY-PACK-001

| Field | Value |
|:---|:---|
| **ID** | **OVR-DES-P2-COPY-PACK-001** |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Phase** | P2 (text overhaul) |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §1–§2 |
| **Inputs** | [`aps_sweep_text_20260616_v1.md`](aps_sweep_text_20260616_v1.md) |
| **Implements** | `OVR-P2-TEXT-001` · `test_aps_no_jargon.py` |
| **Verdict** | **PASS** — signed copy for `@coder-mcp` |

```text
OVR-DES-P2-COPY-PACK-001 Q✓
Unblocks: OVR-P2-TEXT-001
```

---

## 0. Rules (apply every replacement)

1. **Glossary is law** — [`aps_design_system_v1.md`](aps_design_system_v1.md) §1.
2. **Ban-list is hard** — §2; fail `test_aps_no_jargon.py`.
3. **Schema keys** may appear **only** in path-field tooltips and developer log lines — never in labels, buttons, or body copy.
4. **Status pattern:** `{glyph} {word}[ — {detail}]` — word readable without color.
5. **Two validators must read differently:** "Check schema" vs "Run ship check".

---

## 1. Global chrome

| Location | Replace | With |
|:---|:---|:---|
| `app.py` flow caveat | `All actions call rust_engine_mcp CLI/MCP — agents use the same APIs.` | `Every button here runs the same tools the build pipeline uses.` |
| `domain_router.py` buildings authority | `Ship truth: assembly_snapshot (materials + tags). Sidecar and atlas are inputs only.` | `What ships: the Assembly you save here (its materials + tags). Catalog data and atlas tiles only feed into it.` |
| `domain_router.py` landscape authority | `Ship truth: landscape_grammar preset (land_dna + topology_graph). Bake via keyframe_pack only.` | `What ships: the Landscape preset you select here. Tiles are baked through the keyframe step only.` |
| `pipeline_status_bar.py` buildings hint | `Keyframe bake is behind Atlas — Assembly/Materials/Preview work without ship proof.` | `You can build, assign materials, and preview without baking tiles. Tile bake happens on the Atlas step.` |
| `pipeline_status_bar.py` landscape hint | `LG-5 atlas art-ship (G4/G5) is separate from schema/bake green.` | `Final landscape tile art is signed off separately from passing the schema and bake checks.` |
| `pipeline_pills.py` saved state | `◐ {label} saved (QC not run)` | `◐ {label} saved (not checked)` |

---

## 2. Assembly tab

| Location | Replace | With |
|:---|:---|:---|
| Intro | `Assembly — snapshot is authority for materials & tags (not Blender)…` | `Assembly — what you set here (materials + tags) is what ships. Tile baking is on the Atlas step.` |
| LabelFrame title | `Material authority (APS-MAT-AUTH-UI-001)` | `Where materials come from` |
| `aps_mat_auth_ui.py` ENGINE_READ_PATH | *(code path block)* | `The material you assign here is saved on each piece. The game and the preview both read it from this Assembly — not from Catalog tags or the Blender viewport. So: assign here, save, and it shows up everywhere.` |
| LabelFrame | `Grammar set (BUILD-SET)` | `Building style set` |
| Checkbox | `Use building grammar` | `Generate from a building style (auto-place modules)` |
| Collapsible | `Iterate grammar (advanced)` | `Tweak one style layer (advanced)` |
| Collapsible | `Massing pressure (advanced)` | `Building shape bias (advanced)` |
| Checkbox | `Store ARCH-DNA + β in snapshot` | `Save shape settings with this building` |
| LabelFrame | `ARCH-DNA (read-only from preset)` | `Shape profile (from preset, read-only)` |
| LabelFrame | `Pressure field β (0–1)` | `Shape sliders (0–1)` |
| Label | `DNA preset` | `Shape preset` |
| Button | `Generate snapshot` | `Generate Assembly` |
| Button | `P0 gate` | `Run ship check` |
| Button | `Validate` | `Check schema` |
| Dialog | `P0 gate failed — {action} anyway?` | `Ship check failed — {action} anyway?` |
| LabelFrame | `Selected slot — edit` | `Selected piece — edit` |
| Label | `LOD policy` | `Detail level` |
| LOD values | `lod0` / `production` / `hero` | `rough` / `production` / `hero` |
| Collapsible | `Semantic & variant tags` | `Tags (look & state)` |
| Button | `Apply tags to slot` | `Save tags to this piece` |
| Button | `Apply to selected slot` (material) | `Use material on this piece` |
| Toast | `Material {id} applied — Save snapshot before bake` | `Material applied — save the Assembly before baking.` |
| Toast | `Slot updated — run Validate before bake` | `Piece updated — run Ship check before baking.` |

---

## 3. Metadata flow (every tab)

| Location | Replace | With |
|:---|:---|:---|
| Title | `Metadata → engine (ARCH-MAT-001)` | `Where this data goes` |
| Assembly block | `assembly_snapshot (AUTHORITY) / material_profile → Blender/Bevy worker…` | See §4 block copy below |
| Landscape block | *(Rust type path)* | See §4 block copy below |

### §4 — Metadata block copy (paste verbatim)

**Assembly:**
```text
What you save in this Assembly is the source of truth.
• Materials → used when the building is baked or previewed.
• Tags → drive how the engine filters and places pieces.
• Variant tags → expand into tile states later.
Run the Ship check before baking. Save after every material or tag change.
```

**Landscape:**
```text
The game looks at each vegetation patch's growth stage and fire state to pick the matching tile from the landscape atlas. Those states are authored in the catalog file here — not in Blender.
```

---

## 4. Catalog

| Location | Replace | With |
|:---|:---|:---|
| Truth line | `Sidecar tags ≠ ship truth — assembly snapshot semantic_tags…` | `Tags here are hints only. The tags and materials you set in the Assembly are what actually ship.` |
| Inner tab | `AssetSpec sidecar` | `Module info (editable)` |
| Inner tab | `Index entry` | `Library record (read-only)` |
| Button | `3D preview (trimesh)` | `Quick 3D preview` |

---

## 5. Materials

| Location | Replace | With |
|:---|:---|:---|
| Intro tail | `…edit profiles…Assign on the Assembly tab.` | `…edit materials… Assign them on the Assembly step.` |
| LabelFrame | `Preview modes (APS-MAT-002)` | `Preview` |
| Meta line | `registry: inferred` | `Not yet registered` |
| Button | `Regenerate all pilots` | `Regenerate sample materials` |
| Thumb label | `GEN` / `ERR` | `generating…` / `error` |
| Empty | `No albedo — click Generate selected` | `No color map yet — click Generate selected.` |
| Maps row | `albedo: yes  normal: yes  roughness: yes` | `Color: yes · Normal: yes · Roughness: yes` |
| Dialog | `Profile id (e.g. …)` | `Material id (e.g. …)` |

---

## 6. Variants

| Location | Replace | With |
|:---|:---|:---|
| Intro | `variant_set_v1 — declarative layers…Bake via MCP variant_bake / tile_batch_run…` | `Variant set — states of the same building (lighting, damage, fill). Bake them into tiles from here; no manual Blender.` |
| LabelFrame | `Agent patch strip` | `Ask AI for a variant (advanced)` |
| Toast | `Wrote {path} · paste into Cursor; apply via variant_set_patch…` | `Saved a request. Review it, then click Apply patch.` |
| Error | `Patch JSON must be a list or {patch:[...]}` | `That patch text isn't valid — paste the JSON the AI returned.` |
| Buttons | `Save JSON` / `Save RON` | `Save` + format dropdown, or `Save (engine format)` with tooltip |

---

## 7. Atlas

| Location | Replace | With |
|:---|:---|:---|
| Intro | `Atlas — preview cells & packed tile_map here…` | `Atlas — preview your tiles and the packed tile sheet. The keyframe bake in Blender is a separate step.` |
| Field label | `tile_batch_v1` | `Tile job file` |
| Checkbox | `-pk rename` | `Rename keyframe PNGs for packing` |
| Button | `Pack atlas (tilemapgen)` | `Pack atlas` |
| Combo label | `lod0 batch` | `Smoke-test batch` (under Advanced) |
| Phase values | `g0g1 / geometry / promote / full` | `schema only / geometry / promote / full` |
| Banner | `Register target: _tile_atlas_index (buildings)` | `Registers to: Buildings tile index` |
| Register FAIL | `Register FAIL — missing: pilot, expanded` | `Not registered yet — missing: sample tiles, full tiles` |
| Debug | `Blender GUI hidden — RUST_ENGINE_ART_DEBUG_GUI=1…` | `Blender debug buttons are hidden (developer mode only).` |
| Preview status | `atlas_meta: tile_id=…` | `Atlas: {n} tiles · grid {c}×{r}. Next: register this atlas for the map.` |
| Empty | `(no tile_map_*.png — run Pack atlas)` | `No packed tile sheet yet — run Pack atlas.` |

---

## 8. Landscape tabs

| Location | Replace | With |
|:---|:---|:---|
| Presets LabelFrame | `Must-read (DMCP-E2 preset QC)` | `Preset summary` |
| Summary rows | `District read` / `Pressure headline` / `Topology summary` | `District` / `Disturbance level` / `Layout shapes` |
| Badge | `Teach` | `Example (not for ship)` |
| Grammar heading | `Grammar — topology graph ship truth (land_dna + topology_graph)` | `Grammar — the landscape layout graph (this is what ships).` |
| States heading | `States — succession + disturbance matrix` | `States — growth stages & fire` |
| Status | `○ blocked — no preset` | `Pick a preset first` |
| Status | `◐ await grammar` | `Generate grammar first` |
| Parity title | `Engine reads (veg extract parity)` | `Engine read check (vegetation)` |
| ENGINE_READ_PATH | *(Rust path)* | `The game reads vegetation state (growth + fire) to pick the right tile from the landscape atlas. Authored states live in the catalog file, not in Blender.` |
| Parity FAIL | `route to @coder resolver` | `Some states won't load in-game yet — flag this to engineering before publishing.` |

---

## 9. Slot / assembly preview chrome

| Location | Replace | With |
|:---|:---|:---|
| `slot_preview_panel.py` title | `Selected slot previews (APS-PREVIEW-001)` | `Selected piece previews` |
| Hint | `Previews unlock understanding — module isolated…` | `Previews of the selected piece: module alone, its material, the two combined, and where it sits.` |

---

## 10. Tooltip keys (`aps_tooltips.py`)

| Key | Replace gist | With |
|:---|:---|:---|
| `flow_bake_variants` | `variant_set → tile_batch` | `Turn your variant set into a tile job and open the Atlas step (needs an Assembly + a variant set).` |
| `tab_grammar` | `Topology graph workspace` | `Edit the landscape layout graph (roads, rings, patches). This is not the building footprint grid.` |
| `tab_states` | `Succession + burn + vegetation_variant_catalog` | `Vegetation states over time — growth stages, fire, and regrowth.` |

---

## 11. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**@coder-mcp:** implement table-for-table; do not paraphrase. Run `pytest tools/mcp/python/tests/test_aps_no_jargon.py` after pass.
