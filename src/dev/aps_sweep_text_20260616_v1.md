# APS TEXT & COPY sweep — 2026-06-16 `v1`

| Field | Value |
|:---|:---|
| **Dimension** | TEXT & COPY only (1 of 4 parallel dimension audits — layout / tab-design / visual-style owned by others) |
| **Reviewer** | `@designer` |
| **Method** | Static read of every user-facing string in `tools/mcp/art_pipeline_suite/*` + the MCP string producers it renders (`aps_validator_plain.py`, `aps_atlas_qc.py`, `aps_mat_auth_ui.py`, `aps_grammar_labels.py`, `aps_veg_extract_parity.py`, `landscape_state_labels.py`) |
| **State** | Tool is **runnable** today — the 6 files that were empty in the 2026-06-15 audit are restored and populated (boot preflight green). This sweep reviews the LIVE strings, not bytecode. |
| **Lens** | A NEW artist who does not know the codebase, schemas, gate IDs, or agent program names |
| **Scope of fix** | Design doc only — no production code edited |

This sweep **builds on** `design_aps_ux_review_20260615_v1.md` (launch/layout/a11y) and does not repeat its findings. Where that doc flagged a *label* (e.g. `-pk rename`, `ARCH-DNA + β v0`), I re-confirm against live code and note what changed.

---

## 0. Headline

The corpus is in good shape on *mechanics* (word-first status, plain-language P0/atlas QC, next-step callouts all live), but it **leaks engineering vocabulary into the chrome and panel titles** and is **terminologically inconsistent** for its three central nouns. A new artist meets, in the first three visible rows of the Buildings lane, the strings `assembly_snapshot`, `land_dna`, `topology_graph`, `keyframe_pack`, `material_profile`, `semantic_tags`, `(ARCH-MAT-001)`, `(APS-MAT-AUTH-UI-001)`, `(BUILD-SET)`, and `(APS-PREVIEW-001)` — none of which are explained on screen. The four prior fixes that improved *clarity of state* did nothing for *clarity of language*.

Two structural problems dominate:

1. **Program/gate IDs are printed in titlebars and labels.** ~12 LabelFrame titles end with a parenthetical engineering tag (`(ARCH-MAT-001)`, `(APS-PREVIEW-001)`, `(APS-MAT-002)`, `(BUILD-SET)`, `(APS-MAT-AUTH-UI-001)`, `(DMCP-E2 preset QC)` …). These are change-tracking IDs, not artist information. They are the single most repeated jargon offender.
2. **The same concept has three names.** The thing the artist builds is called `snapshot`, `assembly_snapshot`, `assembly`, and "the building" interchangeably — sometimes in the same panel. Same for *material* (`profile` / `material_profile` / `material profile` / `pilot`) and *module* (`module` / `part` / `placement` / `slot` / `cell`).

---

## 1. Findings table

Severity: **P0** = blocks comprehension or misleads about ship safety · **P1** = a new artist guesses wrong or stalls · **P2** = polish / consistency.

### 1a. Global chrome — flow bar, authority strip, pipeline pills, lane bar

| Location `file:symbol` | Current text | Issue type | Sev | Proposed replacement |
|:---|:---|:---|:--:|:---|
| `app.py:_build_flow_bars` | `"All actions call rust_engine_mcp CLI/MCP — agents use the same APIs."` | Leaked internals — `rust_engine_mcp`, "CLI/MCP", "APIs" mean nothing to an artist; it is sitting permanently in the flow bar. | P1 | Drop it, or "Every button here runs the same tools the build pipeline uses." |
| `domain_router.py:AUTHORITY_BY_LANE` (buildings) | `"Ship truth: assembly_snapshot (materials + tags). Sidecar and atlas are inputs only."` | Jargon — `assembly_snapshot`, "Sidecar", "atlas" as bare nouns; "Ship truth" is in-house phrasing. Always-visible strip = high cost. | P1 | "What ships: the Assembly you save here (its materials + tags). Catalog data and atlas tiles only feed into it." |
| `domain_router.py:AUTHORITY_BY_LANE` (landscape) | `"Ship truth: landscape_grammar preset (land_dna + topology_graph). Bake via keyframe_pack only."` | Jargon overload — `landscape_grammar`, `land_dna`, `topology_graph`, `keyframe_pack` in one always-on line. | P0 | "What ships: the Landscape preset you select here. Tiles are baked through the keyframe step only." |
| `pipeline_pills.py:format_pill` | `"◐ {label} saved (QC not run)"` | Jargon — "QC" unexpanded; pills are the primary progress model. | P2 | "◐ {label} saved (not checked)" |
| `pipeline_status_bar.py:_set_lane_hint` (buildings) | `"Keyframe bake is behind Atlas — Assembly/Materials/Preview work without ship proof."` | Jargon — "behind Atlas", "ship proof" are opaque. | P2 | "You can build, assign materials, and preview without baking tiles. Tile bake happens on the Atlas step." |
| `pipeline_status_bar.py:_set_lane_hint` (landscape) | `"LG-5 atlas art-ship (G4/G5) is separate from schema/bake green."` | Leaked internals — `LG-5`, `G4/G5`, "green" are gate/phase codes. Worst jargon string in landscape chrome. | **P0** | "Final landscape tile art is signed off separately from passing the schema and bake checks." |
| `aps_tooltips.py:flow_bake_variants` | `"Expand variant_set → tile_batch and jump to Atlas (needs assembly + variants)."` | Jargon — `variant_set`, `tile_batch`. | P2 | "Turn your variant set into a tile job and open the Atlas step (needs an Assembly + a variant set)." |
| `aps_tooltips.py:tab_grammar` | `"Topology graph workspace — not building footprint."` | Jargon — "Topology graph"; the "not …" negative is confusing as a first read. | P2 | "Edit the landscape layout graph (roads, rings, patches). This is not the building footprint grid." |
| `aps_tooltips.py:tab_states` | `"Succession + burn + regrowth axes — vegetation_variant_catalog entries."` | Jargon — schema name `vegetation_variant_catalog`. | P2 | "Vegetation states over time — growth stages, fire, and regrowth." |

### 1b. Catalog tab

| Location `file:symbol` | Current text | Issue type | Sev | Proposed replacement |
|:---|:---|:---|:--:|:---|
| `catalog.py:SIDECAR_TRUTH` | `"Sidecar tags ≠ ship truth — assembly snapshot semantic_tags and material_profile win at runtime."` | Jargon — "Sidecar", `semantic_tags`, `material_profile`, "ship truth", "at runtime". Dense for a default-visible line. | P1 | "Tags here are hints only. The tags and materials you set in the Assembly are what actually ship." |
| `catalog.py:notebook.add` | `"AssetSpec sidecar"` | Jargon — "AssetSpec", "sidecar" are internal file-format names; tab label the artist must click. | P1 | "Module info (editable)" |
| `catalog.py:notebook.add` | `"Index entry"` | Ambiguous — "Index" unclear; pairs poorly with the tab beside it. | P2 | "Library record (read-only)" |
| `catalog.py:on_select` | `"{module_id} · job {rec.job_id} · {archetype}\nGLB: {path}\nGrid {grid} · dims {dim} · batch {batch}"` | Leaked internals — `job_id`, `GLB`, `batch` raw; no labels an artist parses. | P2 | Label the fields: "Size", "File", "Grid", "Batch"; drop `job` unless needed. |
| `catalog.py:on_validate` | `"Validation {status} · {verts} verts · {issues}"` | Minor — "verts" abbreviation; otherwise good (word + count). | P2 | "Validation PASS · 1,240 vertices · no issues" |
| `catalog.py` button | `"3D preview (trimesh)"` | Leaked internals — "trimesh" is a Python library name. | P2 | "Quick 3D preview" (move "trimesh/optional" to tooltip — already there). |

### 1c. Assembly tab (heaviest jargon density)

| Location `file:symbol` | Current text | Issue type | Sev | Proposed replacement |
|:---|:---|:---|:--:|:---|
| `assembly_panel.py:_build` intro | `"Assembly — snapshot is authority for materials & tags (not Blender)…keyframe tile bake is on Atlas tab."` | Jargon — "snapshot is authority", "keyframe tile bake"; "(not Blender)" assumes the artist worries about that. | P1 | "Assembly — what you set here (materials + tags) is what ships. Tile baking is on the Atlas step." |
| `assembly_panel.py:auth` LabelFrame | `"Material authority (APS-MAT-AUTH-UI-001)"` | **Gate ID in titlebar** + "authority" jargon. | **P0** | "Where materials come from" |
| `aps_mat_auth_ui.py:ENGINE_READ_PATH` | `"Runtime: placement.material_profile → material registry (assets/materials/textures/<id>/) → worker bake / Bevy preview bind → render extract. Assembly snapshot is authority…"` | **Worst jargon string in the app** — a code path printed verbatim: `placement.material_profile`, "registry", "worker bake", "Bevy preview bind", "render extract". | **P0** | "The material you pick is saved on each piece and used everywhere the building is drawn — preview, bake, and in-game. This Assembly is the source of truth, not Catalog or Blender." |
| `assembly_panel.py:grammar_set_panel` (via `grammar_build_set_panel.py`) | `"Grammar set (BUILD-SET)"` | **Program ID in titlebar** + "Grammar set" undefined. | P1 | "Building style set" (drop "(BUILD-SET)"). |
| `assembly_panel.py` checkbox | `"Use building grammar"` | Jargon — "grammar" is a generative-design term, not artist vocabulary. | P1 | "Generate from a building style (auto-place modules)" |
| `assembly_panel.py:iterate_section` | `"Iterate grammar (advanced)"` | Jargon — "Iterate grammar". | P2 | "Tweak one style layer (advanced)" |
| `assembly_panel.py:grammar_dna_section` & `grammar_dna_panel.py` title | `"Massing pressure (advanced)"` | Improved from `ARCH-DNA + β v0` (good — prior fix landed). Still: "Massing pressure" is jargon and the inner frames re-expose it. | P2 | "Building shape bias (advanced)" |
| `grammar_dna_panel.py` checkbox | `"Store ARCH-DNA + β in snapshot"` | **Worst surviving raw jargon** — `ARCH-DNA`, Greek `β`, `snapshot`. Prior audit flagged the section title; this *checkbox* still says it. | **P0** | "Save shape settings with this building" |
| `grammar_dna_panel.py` LabelFrame | `"ARCH-DNA (read-only from preset)"` | Raw jargon `ARCH-DNA`. | P1 | "Shape profile (from preset, read-only)" |
| `grammar_dna_panel.py` LabelFrame | `"Pressure field β (0–1)"` | Jargon — "Pressure field", `β`. | P1 | "Shape sliders (0–1)" |
| `grammar_dna_panel.py` label | `"DNA preset"` | Jargon — "DNA". | P2 | "Shape preset" |
| `assembly_panel.py` button | `"Generate snapshot"` | Inconsistent verb-noun — elsewhere it's "the Assembly". | P1 | "Generate Assembly" |
| `assembly_panel.py` button | `"P0 gate"` | **Leaked internals** — `P0`, "gate" are validator codenames on a primary button. | **P0** | "Ship check" (tooltip can keep "P0"). |
| `assembly_panel.py` button | `"Validate"` vs `"P0 gate"` | Ambiguous pair — two validate buttons with no hint of the difference (production schema vs P0+grammar). | P1 | "Check schema" and "Ship check" — and say which is required before saving. |
| `assembly_panel.py:on_save` block dialog | `"P0 gate failed — {action} anyway?"` / `"Proceed anyway? (Not recommended for ship/bake.)"` | Jargon — `P0 gate`. Message itself is good (actionable). | P1 | "Ship check failed — {action} anyway?" |
| `assembly_panel.py` LabelFrame | `"Selected slot — edit"` | Inconsistent noun — "slot" here, "cell" on the grid, "placement" in the list, "module" in the field. Four words, one thing. | P1 | "Selected piece — edit" (pick ONE noun, see §2). |
| `assembly_panel.py` label | `"Node id"` | Leaked internals — `node_id` is a schema key; artist never types or reads it meaningfully. | P2 | Hide it, or "Piece id (internal)". |
| `assembly_panel.py` label | `"LOD policy"` + values `lod0/production/hero` | Jargon — "LOD policy", `lod0`. | P1 | "Detail level" + values "rough / production / hero". |
| `assembly_panel.py:CollapsibleSection` | `"Semantic & variant tags"` | Jargon — "Semantic", "variant" as adjectives. | P2 | "Tags (look & state)" |
| `assembly_panel.py` button | `"Apply tags to slot"` / `"Apply to selected slot"` (material) | Inconsistent — "slot" again; two near-identical "Apply" buttons in different panes. | P2 | "Save tags to this piece" / "Use material on this piece". |
| `metadata_flow_panel.py:__init__` title | `"Metadata → engine (ARCH-MAT-001)"` | **Gate ID in titlebar** + "Metadata → engine" is dev framing. Appears on EVERY tab. | **P0** | "Where this data goes" (drop "(ARCH-MAT-001)"). |
| `metadata_flow_panel.py:_fill_content` (all blocks) | e.g. `"assembly_snapshot (AUTHORITY)\n material_profile → Blender/Bevy worker applies at bake/preview (not assigned in DCC)…"` | Jargon dump — `DCC`, "worker", "render extract", `grammar_rule_chain`, "drives generator seed path". This is the in-app explainer and is itself unreadable to a new artist. | **P0** | Rewrite as plain cause→effect (see §4 example). This panel is supposed to be the glossary; right now it needs a glossary. |
| `slot_preview_panel.py` title | `"Selected slot previews (APS-PREVIEW-001)"` | **Program ID in titlebar** + "slot". | P1 | "Selected piece previews" |
| `slot_preview_panel.py` hint | `"Previews unlock understanding — module isolated, material on wall+sphere, combined, placement highlighted…"` | Voice — "Previews unlock understanding" is marketing voice; rest is fine. | P2 | "Previews of the selected piece: module alone, its material, the two combined, and where it sits." |
| `grammar_inspector.py` columns | `layer / rule_id / detail / tags` headings | Leaked internals — `rule_id`, "layer" are generator schema columns. | P1 | "Step / Rule / Detail / Tags" (and resolve `rule_id` to a human label where possible). |
| `assembly_panel.py:_apply_material_profile` | `"Material {profile_id} applied — Save snapshot before bake"` | Inconsistent — "snapshot"; otherwise actionable. | P2 | "Material applied — save the Assembly before baking." |
| `assembly_panel.py:on_apply_slot` | `"Slot updated — run Validate before bake"` | Inconsistent — "Slot", "Validate". | P2 | "Piece updated — run Ship check before baking." |

### 1d. Materials tab

| Location `file:symbol` | Current text | Issue type | Sev | Proposed replacement |
|:---|:---|:---|:--:|:---|
| `materials_panel.py` intro | `"Material Studio — browse, generate, and edit profiles…Assign on the Assembly tab."` | Minor — "profiles" vs "materials"; otherwise clear. | P2 | "…browse, generate, and edit materials… Assign them on the Assembly step." |
| `material_preview_modes.py` title | `"Preview modes (APS-MAT-002)"` | **Program ID in titlebar.** | P1 | "Preview" |
| `material_library_widget.py:_select_profile` meta | `"id: {profile_id}\ncategory: {category}\ngenerator: {generator}  registry: {'yes' if in_registry else 'inferred'}\nmetallic: {x}  roughness: {y}"` | Jargon — "generator", "registry: inferred", lowercase keys. "inferred" is opaque. | P1 | Label rows clearly; replace "registry: inferred" with "Not yet registered". |
| `material_library_widget.py` button | `"Regenerate all pilots"` | Jargon — "pilots" (internal name for seed profiles); sits beside benign "Reload preview". Prior audit flagged the placement; the *word* is the text issue. | P1 | "Regenerate sample materials" (and separate it from Reload). |
| `material_library_widget.py` placeholder | `"GEN"` / `"ERR"` thumb labels | Cryptic — 3-letter codes on thumbnails. | P2 | "generating…" / "error". |
| `material_library_widget.py:_show_preview_image` | `"No albedo — click Generate selected"` | Jargon — "albedo". | P2 | "No color map yet — click Generate selected." |
| `material_library_widget.py:_maps_line` | `"albedo: yes  normal: yes  roughness: yes"` | Jargon — `albedo` / `normal` / `roughness` raw (PBR map names). Acceptable for material artists but inconsistent casing/format. | P2 | Keep map names (domain-correct) but title-case + spacing; consider "Color / Normal / Roughness". |
| `material_library_widget.py:_add_profile_dialog` | `"Profile id (e.g. steel_panel_02):"` | Minor — "Profile id"; consistent if §2 adopted. | P2 | "Material id (e.g. steel_panel_02):" |

### 1e. Variants tab

| Location `file:symbol` | Current text | Issue type | Sev | Proposed replacement |
|:---|:---|:---|:--:|:---|
| `variants_panel.py:_build` intro | `"variant_set_v1 — declarative layers (lighting, damage, material, fill). Bake via MCP variant_bake / tile_batch_run — no manual Blender."` | **Jargon-dense** — `variant_set_v1`, "declarative layers", `MCP`, `variant_bake`, `tile_batch_run`. First line of the tab. | **P0** | "Variant set — states of the same building (lighting, damage, fill). Bake them into tiles from here; no manual Blender." |
| `variants_panel.py` LabelFrame | `"Agent patch strip"` | **Leaked internals** — "Agent patch strip" is an AI-workflow term; an artist has no model for it. | P1 | "Ask AI for a variant (advanced)" |
| `variants_panel.py:on_request_agent` | `"Wrote {path} · paste into Cursor; apply via variant_set_patch after review."` | Leaked internals — "Cursor" (an editor), `variant_set_patch`. | P1 | "Saved a request. Review it, then click Apply patch." |
| `variants_panel.py` field default | `"add_warm_window_lights"` (intent default) | Minor — snake_case sample in a user field reads as code. | P2 | "add warm window lights" |
| `variants_panel.py:on_apply_patch` | `"Patch JSON must be a list or {patch:[...]}"` | Leaked internals — JSON shape error shown to artist. | P1 | "That patch text isn't valid — paste the JSON the AI returned." |
| `variants_panel.py` button | `"Save JSON"` / `"Save RON"` | Leaked internals — "RON" is a Rust file format; artist can't choose meaningfully. | P2 | "Save" + a format dropdown, or "Save (engine .ron)" with a tooltip. |
| `variants_panel.py:on_variant_select` | `"bake: {status} · {png or '—'}"` | Minor — lowercase "bake:"; fine but inconsistent with sentence-case elsewhere. | P2 | "Bake: pending" / "Bake: done · tile_map.png". |

### 1f. Atlas tab

| Location `file:symbol` | Current text | Issue type | Sev | Proposed replacement |
|:---|:---|:---|:--:|:---|
| `atlas_panel.py:_build` intro | `"Atlas — preview cells & packed tile_map here. Keyframe bake in Blender is a separate ship step…"` | Jargon — "cells", `tile_map`, "Keyframe bake", "ship step". | P1 | "Atlas — preview your tiles and the packed tile sheet. The keyframe bake in Blender is a separate step." |
| `atlas_panel.py` label | `"tile_batch_v1"` (field label) | **Leaked internals** — raw schema name as a field label. | P1 | "Tile job file" |
| `atlas_panel.py` checkbox | `"-pk rename"` | **Cryptic** — flag name as a label. Prior audit recommended "Rename keyframe PNGs (-pk)"; **NOT applied** — still raw. | **P0** | "Rename keyframe PNGs for packing" |
| `atlas_panel.py` button | `"Pack atlas (tilemapgen)"` | Leaked internals — "tilemapgen" tool name. | P2 | "Pack atlas" (tool name → tooltip). |
| `atlas_panel.py` label/combo | `"lod0 batch"` + `kit_lod0_NNN` values | Leaked internals — `lod0`; prior audit wanted lod0 behind "Advanced", and the label itself is opaque. | P1 | Group under "Advanced (engine smoke tests)"; label "Smoke-test batch". |
| `atlas_panel.py` combo | phase values `g0g1 / geometry / promote / full` | **Leaked internals** — `g0g1` is a pipeline phase code. | P1 | "schema only / geometry / promote / full" (or hide entirely behind Advanced). |
| `atlas_panel.py:_domain_banner` | `"Register target: _tile_atlas_index (buildings)"` | Leaked internals — `_tile_atlas_index` is an index filename. | P2 | "Registers to: Buildings tile index" |
| `atlas_panel.py:refresh_landscape_register` | `"Register FAIL — missing: pilot, expanded"` / `"Register PASS — N atlas row(s)"` | Jargon — "pilot", "expanded", "atlas row" without context; non-actionable on FAIL. | P1 | "Not registered yet — missing: sample tiles, full tiles" + "what to do" hint. |
| `atlas_panel.py:else` block | `"Blender GUI hidden — RUST_ENGINE_ART_DEBUG_GUI=1 for legacy debug buttons."` | **Leaked internals** — env var exposed to artists. | P1 | "Blender debug buttons are hidden (developer mode only)." |
| `atlas_preview_panel.py:_refresh` | `"atlas_meta: tile_id={tid} · grid {c}×{r} · {n} cells · Next: tile-atlas-register / map stamp (see tools/mcp/README.md)"` | Jargon — `atlas_meta`, `tile_id`, "tile-atlas-register", "map stamp", a README path. | P1 | "Atlas: {n} tiles · grid c×r. Next: register this atlas for the map." |
| `atlas_preview_panel.py` empty | `"(no tile_map_*.png — run Pack atlas)"` | Minor — glob pattern shown. | P2 | "No packed tile sheet yet — run Pack atlas." |

### 1g. Landscape tabs (Presets / Grammar / States)

| Location `file:symbol` | Current text | Issue type | Sev | Proposed replacement |
|:---|:---|:---|:--:|:---|
| `landscape_presets_panel.py` LabelFrame | `"Must-read (DMCP-E2 preset QC)"` | **Program ID in titlebar** + "Must-read" is odd voice. | P1 | "Preset summary" |
| `landscape_presets_panel.py` rows | `"District read: —"` / `"Pressure headline: —"` / `"Topology summary: —"` | Jargon — "District read", "Pressure headline", "Topology". | P2 | "District", "Disturbance level", "Layout shapes". |
| `landscape_presets_panel.py:_ship_badge` | `"Ship"` / `"Draft"` / `"Teach"` | Ambiguous — "Teach" is opaque (means "teaching example, not a ship target"). | P1 | "Example (not for ship)" instead of "Teach". |
| `landscape_grammar_panel.py` heading | `"Grammar — topology graph ship truth (land_dna + topology_graph)"` | Jargon — `land_dna`, `topology_graph`, "ship truth". | P1 | "Grammar — the landscape layout graph (this is what ships)." |
| `landscape_grammar_panel.py:_show_node` | `"Scale: {scale_band}"`, `"(scaffold — no operators authored)"` | Jargon — "scale_band", "scaffold", "operators authored". | P2 | "Size: —", "(not set up yet)". |
| `landscape_states_panel.py` heading | `"States — succession + disturbance matrix"` | Jargon — "succession", "disturbance matrix". | P2 | "States — growth stages & fire" |
| `landscape_state_labels.py:status_display` | `"○ blocked — no preset"`, `"◐ await grammar"` | Voice — "blocked"/"await" terse; otherwise fine. | P2 | "Pick a preset first", "Generate grammar first". |
| `landscape_extract_parity_panel.py` title | `"Engine reads (veg extract parity)"` | Jargon — "veg extract parity". | P1 | "Engine read check (vegetation)" |
| `aps_veg_extract_parity.py:ENGINE_READ_PATH` | `"Runtime: SuccessionState + ActiveBurn → VegetationExtractFrame::BuildProfiles (rows[].variant_key) → landscape_chunk_atlas_stamp / LG-5 atlas index…"` | **Code path printed verbatim** — Rust type names. Mirror of the Assembly `ENGINE_READ_PATH` P0. | **P0** | "The game reads vegetation state (growth + fire) to pick the right tile from the landscape atlas. Authored states live in the catalog file, not in Blender." |
| `landscape_extract_parity_panel.py:refresh_parity` | `"FAIL — authored keys not consumable by engine resolver"` / `"Parity FAIL — block atlas promote; route to @coder resolver"` | **Leaked internals** — "@coder", "resolver", non-actionable for an artist (tells them to route to an engineer). | P1 | "Some states won't load in-game yet — flag this to engineering before publishing." |

---

## 2. Canonical terminology guide (the master decision)

Pick ONE word per concept and use it in every label, button, tooltip, and message. Recommended canon below; the right column lists the variants currently in the code that must be replaced.

| Concept (artist sees) | **Canonical word** | Variants found in live code (replace these) | Notes |
|:---|:---|:---|:---|
| The thing you assemble & save | **Assembly** | `snapshot`, `assembly_snapshot`, `assembly`, "the building", "the snapshot" | Keep `assembly_snapshot` ONLY in the file-path field tooltip. Everywhere else: "Assembly". |
| One placed building part | **Piece** | `slot`, `cell`, `placement`, `module`, `node` | Use "Piece" in editor UI. Keep "Module" for the *catalog kit item* (the reusable source asset). So: Catalog has **Modules**; an Assembly is made of **Pieces** (each piece uses a module). |
| The grid square in the footprint | **Cell** | (fine as-is) | "Cell" only for the grid square; clicking a cell selects a Piece. |
| A surface look | **Material** | `profile`, `material_profile`, `material profile`, `pilot` | "Material" in UI; `material_profile` only in path/registry tooltips. "pilot" → "sample material". |
| Texture maps | **Color / Normal / Roughness** | `albedo`, "maps" | "albedo" → "Color" in artist copy; keep PBR names only in the maps readout (domain-correct for material artists). |
| Auto-generation rules | **Building style** | `grammar`, `ARCH-DNA`, `β`, "massing", "DNA preset", "pressure field" | "Building style" / "shape settings"; never `grammar`, `ARCH-DNA`, `β`. |
| A state variation of a building | **Variant** | `variant_set`, `variant_set_v1`, `variant_key` | "Variant set" / "variant"; raw schema only in tooltips. |
| Packed tile sheet | **Atlas** | `tile_map`, `atlas_meta`, `tile_batch`, "cells" | "Atlas" for the sheet; "tile" for a cell; "tile job" for the batch file. |
| Validation that gates ship | **Ship check** | `P0`, `P0 gate`, "QC", "validate" (the strict one) | "Ship check" on the strict gate; "Check schema" on the schema-only one. |
| What ships / source of truth | **What ships** | "ship truth", "authority", "AUTHORITY" | "What ships:" lead-in; never "ship truth"/"authority" bare. |
| Landscape layout graph | **Layout graph** | `topology_graph`, "topology", "Topology summary" | "Layout graph"; "Layout shapes" for the summary. |
| Landscape preset content | **Landscape preset** | `landscape_grammar`, `land_dna`, "land DNA" | "Landscape preset"; `land_dna` → "disturbance settings". |
| Growth-over-time states | **Growth stage** | `succession`, "succession stage" | "Growth stage". |
| Post-fire states | **Regrowth** | `regrowth_macro`, "regrowth macro phase" | "Regrowth" / "regrowth stage". |

**Hard rule — no engineering IDs in visible text.** Strip every `(APS-…)`, `(ARCH-…)`, `(BUILD-SET)`, `(DMCP-…)`, `(LG-5)`, `(G0–G5)`, `(P0)`, `(v1)`, `(v2)` from LabelFrame titles, buttons, and labels. They belong in code comments and witness JSON, never on screen. (Count today: ~12 titlebars + several inline.)

---

## 3. Voice & tone rules

A new artist should never need a glossary to read a label. Apply these consistently:

1. **Sentence case for everything except proper nouns and tab labels.** Tabs stay Title-case single words (Catalog, Assembly, Materials, Variants, Atlas — already consistent). Buttons, labels, hints, messages: sentence case. (Today: mix of `Title Case`, `ALL CAPS` (`AUTHORITY`, `PASS`/`FAIL`), and `lowercase code`.)
2. **Buttons = imperative verb + canonical noun.** "Generate Assembly", "Save", "Run ship check", "Pack atlas". Avoid bare "Apply", "Validate", "P0 gate". Two buttons that do different things must read differently ("Check schema" vs "Run ship check").
3. **Status = word first, then glyph.** Already the pattern in materials/pipeline — extend it everywhere (atlas inline status, landscape states). Never glyph-only.
4. **Errors state the fix, in the artist's verbs.** Good models already exist (`aps_validator_plain.py` sentence + arrow hint). Bad models to fix: "Patch JSON must be a list…", "route to @coder resolver", "Register FAIL — missing: pilot, expanded". Every FAIL line needs a "what to do" clause.
5. **Never print code paths, type names, env vars, file globs, or tool names in body text.** `placement.material_profile → … render extract`, `VegetationExtractFrame::BuildProfiles`, `RUST_ENGINE_ART_DEBUG_GUI=1`, `tile_map_*.png`, "tilemapgen", "trimesh", "Cursor" → all move to tooltips at most, or are removed.
6. **No agent/program/gate identifiers in artist-facing copy.** "@coder", "DMCP-E2", "LG-5", "G4/G5", "BUILD-SET". The artist's mental model has no agents in it.
7. **Lead-in pattern for the two "where data goes" explainers** (`ENGINE_READ_PATH`, `metadata_flow_panel`): cause → effect in plain English, one sentence per hop, no arrows-of-symbols. (See §4.)
8. **Consistency of the "what ships" message.** Every authority strip / intro that asserts source-of-truth uses the exact phrase "What ships:" + the canonical noun. Right now it's "Ship truth", "authority", "AUTHORITY", "is authority" — four phrasings.

---

## 4. Worked rewrite — the two `ENGINE_READ_PATH` strings + metadata-flow

These three are the densest jargon in the app and they are *meant* to be the explainers, so they are worth a precise rewrite. (Strings live in MCP modules I don't edit — this is the copy spec for whoever owns the overhaul.)

**Assembly `ENGINE_READ_PATH` (now a printed code path) →**
> "The material you assign here is saved on each piece. The game and the preview both read it from this Assembly — not from Catalog tags or the Blender viewport. So: assign here, save, and it shows up everywhere."

**`metadata_flow_panel` assembly block (now `assembly_snapshot (AUTHORITY) / material_profile → Blender/Bevy worker…`) →**
> "What you save in this Assembly is the source of truth.
> • Materials → used when the building is baked or previewed.
> • Tags → drive how the engine filters and places pieces.
> • Variant tags → expand into tile states later.
> Run the Ship check before baking. Save after every material or tag change."

**Landscape `ENGINE_READ_PATH` →**
> "The game looks at each vegetation patch's growth stage and fire state to pick the matching tile from the landscape atlas. Those states are authored in the catalog file here — not in Blender."

---

## 5. Severity rollup

- **P0 (10):** landscape pipeline hint (`LG-5/G4/G5`), landscape authority strip (`land_dna`/`topology_graph`/`keyframe_pack`), "Material authority (APS-MAT-AUTH-UI-001)" titlebar, Assembly `ENGINE_READ_PATH` code path, "Store ARCH-DNA + β in snapshot" checkbox, "P0 gate" button, "Metadata → engine (ARCH-MAT-001)" titlebar (every tab), metadata-flow content blocks, Variants intro (`variant_set_v1`/`MCP`/`tile_batch_run`), "-pk rename" checkbox, landscape `ENGINE_READ_PATH` code path. *(11 listed — `metadata_flow` title + content count as the same surface.)*
- **P1 (≈22):** terminology collisions (slot/cell/placement/module, snapshot/assembly, profile/material), all remaining `(PROGRAM-ID)` titlebars, "AssetSpec sidecar" tab, "Agent patch strip", "@coder resolver" message, lod0/`g0g1` phase codes, env-var exposure, "Use building grammar".
- **P2 (≈25):** casing/voice inconsistencies, abbreviations (verts, GEN/ERR, albedo), tool-name leaks moved to tooltips, glyph-vs-word polish.

---

## 6. Cross-dimension handoff note

This is the TEXT dimension only. Three findings touch adjacent dimensions and should be flagged to those owners, not actioned here:
- **Layout owner:** the metadata-flow panel rewrite (§4) shortens the block — re-check height after copy lands.
- **Tab-design owner:** the Catalog inner notebook tabs ("AssetSpec sidecar" / "Index entry") are tab labels — rename coordinated with tab-design.
- **Visual-style owner:** "word-first, glyph-suffix" status pattern (rule §3.3) assumes the glyph styling stays; confirm no glyph-only states are introduced.

Required engine hooks: none (copy-only). Diagnostics: none changed. Risk: renaming the `material_profile`/`snapshot` nouns in UI must NOT rename the JSON schema keys — keep raw schema names in path-field tooltips so artists can still match a filename.
