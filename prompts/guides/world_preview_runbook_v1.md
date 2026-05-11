# World preview & raster pipeline — runbook `v1`

Version: `v1.0.1` (see §7 changelog)

> **Pair:** terrain orchestrator [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) §8b · matrix [`../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md`](../matrix/terrain_biome/composite_style_preview_integration_matrix_v1.md) · invalidation / multi-layer context **U7** [`../matrix/terrain_biome/runbook/u7_steps_v1.md`](../matrix/terrain_biome/runbook/u7_steps_v1.md).

## 1. Code map (current)

| Area | Path |
|:---|:---|
| Editor preview modules | `src/gui/editor/world_preview/` (`layers`, `render_raster`, `viewport`, `window`, …) |
| Full-world CPU raster | `render_raster::update_world_preview_texture` → `WorldPreviewTexture` |
| egui viewport | `window::display_world_preview`, `EditorViewport` |
| Optional tilemap overlay indices | `tilemap_bridge` + `bevy_tilemap_adapter` feature |

### 1.1 Execution dependency chain

Correct dependency direction (do not invert):

```text
terrain_unification_runbook_v1
    → u7_steps_v1
    → ChunkDirty / ChunkDependency (material + tuning invalidation)
    → world_preview_runbook_v1
    → chunk-dirty preview + compositing
    → GPU preview (Stage 4) later
```

Preview invalidation, **material** invalidation, **tilemap** invalidation, and **logistics / scenario overlay** invalidation should stay **aligned** on this graph—separate ad-hoc “invalidate everything” paths are a regression risk once map editor, logistics overlays, scenario markers, and debug visualizers coexist.

---

## 2. Explicit note — optimization strategy (read first)

**Current preview rasterization remains a full CPU buffer pass per update.** The `EditorViewport` / raster module split is **intentional infrastructure** for later:

- chunk-dirty partial redraw,
- composited preview layers, and
- eventual GPU-backed rendering.

**Optimization work should target the invalidation graph and layer boundaries first**, not micro-optimizing the current single-pass raster path.

That order matters because **premature** optimization here tends to be **discarded** once logistics, infrastructure, overlays, operational layers, scenario scripting, serialization, and the **dirty graph** (U7 and follow-ons) stabilize.

**CPU cost is acceptable for now if:**

- editor iteration stays responsive,
- chunk/world sizes stay bounded, and
- abstraction boundaries stay stable.

### 2.1 No gameplay authority (invariant)

**World preview is presentation-only.** It must **never** become authoritative gameplay state. All simulation authority remains **ECS / world data** (components, resources, saves).

This blocks long-term drift where overlays, tilemaps, GPU buffers, or preview caches accidentally become **parallel stores of truth**. Generated textures and egui/chrome are **views**; they are discarded and rebuilt from authority as needed.

---

## 3. Roadmap stages

### Stage 2 — Chunk-dirty textures

**Goal:** when chunk data changes, **re-raster only dirty chunks** and **upload sub-regions** of the preview atlas (or per-chunk textures composed in the viewport).

**Conceptual flow:**

```text
chunk changed
  → re-raster only dirty chunk
  → update sub-region texture
```

**Likely resources / queues (target sketch):**

| Name | Role |
|:---|:---|
| `ChunkDirty` | Already in U7 material pipeline — extend or mirror semantics for **preview** invalidation |
| `ChunkDependency` | Hash / version inputs per chunk; ties into **U7 invalidation graph** |
| `ChunkTextureCache` | GPU `Image` (or CPU buffer) per chunk or atlas slot; tracks revision |
| `DirtyRegionQueue` | Batched rectangles or chunk keys for upload / composite this frame |

**Tie-in:** [`u7_steps_v1.md`](../matrix/terrain_biome/runbook/u7_steps_v1.md) (partial rebuild, asset-driven dirty marking). Preview should **subscribe** to the same invalidation signals where possible instead of inventing a parallel graph.

#### Preview invalidation ownership (compact)

Use this table to avoid **“invalidate everything”** regressions when map editor, logistics overlays, scenario markers, and debug visualizers coexist. Refine rows as new systems land; the **principle** is narrow invalidation.

| Change source | Invalidates (preview) |
|:--|:--|
| `MaterialRegistry` | Terrain layer only |
| `RuleSet` | Terrain + resource |
| `TagRegistry` | Overlay + terrain |
| Road placement | Infrastructure layer |
| Power line edit | Infrastructure + overlay |
| Scenario objective markers | Overlay only |
| Camera move | **No** raster invalidation |

If a row is ambiguous, resolve in favor of **extra** invalidation for correctness first, then **narrow** once chunk keys and layer ownership are explicit in code.

#### Chunk texture coordinate authority (before implementation)

**One canonical type** for where a chunk’s preview texels live in an atlas (or logical slab):

```rust
pub struct ChunkTextureRect {
    pub atlas_origin: UVec2,
    pub size: UVec2,
}
```

**Separation of responsibilities:**

| Concern | Owner |
|:--|:--|
| Atlas layout, packing, slot assignment | **Cache / texture cache** (or dedicated atlas builder)—owns **`ChunkTextureRect`** |
| World/chunk raster contents | Raster systems (CPU today)—write texels **only** within the rect the cache assigned |
| Screen position, zoom, pan | **Viewport / `EditorViewport`** only—maps world ↔ screen; **does not** compute atlas layout |

**Invariants:**

- Viewport **never** computes atlas layout.
- Raster systems **never** infer screen coordinates.
- **Cache owns texture placement** (including future GPU atlas packing, sparse residency, paging, zoom-dependent mips).

#### Preview generation epoch (`PreviewGenerationEpoch`)

Likely resource for U7 / registry alignment:

```rust
#[derive(Resource, Default)]
pub struct PreviewGenerationEpoch(pub u64);
```

**Increment** (exact policy TBD) when global preview inputs change in ways that can invalidate many chunks at once, e.g.:

- Registry changes (`MaterialRegistry`, `TagRegistry`, …),
- Rule / `RuleSet` changes,
- Authoring tuning / noise / biome overlay relevant to preview,
- World seed / gen params identity used for preview (when applicable).

**Use:** chunk preview entries store `last_epoch`; if `chunk.epoch != global_epoch`, **mark_dirty(...)** (or equivalent) so async raster, GPU upload queues, and editor viewport stay synchronized without ad-hoc full-map clears.

---

### Stage 3 — Compositing layers

**Goal:** separate **terrain**, **resources**, **rail**, **road**, **power**, **overlay/debug** (and similar) so **overlay changes do not force a full terrain re-raster**.

**Direction:**

- Each logical layer owns its invalidation (dirty flags or region queues).
- Final preview = composited stack in the viewport (CPU blend today, GPU later).

#### Layer ordering invariant (z / composite order)

**Canonical draw order** (bottom → top). All preview / tilemap / compositor code should converge here even if some layers are **not implemented** yet—do **not** invent per-feature precedence.

| Layer | Composite z |
|:--|--:|
| Terrain | 0 |
| Resources | 10 |
| Road / rail | 20 |
| Power / utility | 30 |
| Scenario | 40 |
| Debug | 100 |

(Align with existing tilemap stack z-offsets in the adapter where possible; extend this table if a layer splits further.)

**Example future texture set (illustrative):**

```rust
pub struct EditorViewportTextureSet {
    pub terrain: Handle<Image>,
    pub infrastructure: Handle<Image>,
    pub logistics: Handle<Image>,
    pub overlays: Handle<Image>,
}
```

**Outcome:** monolithic “one buffer, one redraw” becomes **layered composition** with independent refresh.

---

### Stage 4 — GPU preview path

**Goal:** move hot paths off the CPU full-map fill where it pays off, after Stages 2–3 boundaries exist.

**Possible directions (non-exclusive):**

- Custom render graph node
- Compute shader for material / layer resolve
- Texture-array terrain atlas
- Bevy render sub-app integration
- Tilemap hybrid adapter (`U6`) + chunk mesh/material preview

**Alignment:** `U6` tilemap adapter, `U7` multi-layer / invalidation story, and the **layer split** from Stage 3.

---

## 4. Sequencing discipline

1. **Stabilize** invalidation semantics (`ChunkDirty` / `ChunkDependency` / world gen vs editor mutations).
2. **Introduce** chunk-scoped preview updates (Stage 2).
3. **Split** preview layers with separate handles and dirty tracks (Stage 3).
4. **Only then** invest in GPU path (Stage 4) so work lands on stable boundaries.

---

## 5. Presentation tokens & preview color authority (P1+)

Cross-cut with [`ui_design_language_plan_v1.md`](ui_design_language_plan_v1.md): preview chrome and **debug / overlay** colors should come from **`UiPalette`** (or derived shader constants later), not scattered `Color32::…`, so CPU preview and future GPU paths stay aligned.

**Replace** remaining ad-hoc colors in:

- World preview overlays / debug drawing,
- Dirty-chunk debug highlights,
- Scenario markers (where egui paints them),
- Tilemap diagnostics,
- Logistics overlays.

**Suggested helpers** (Rust API sketch—implement under `src/gui/style/` when migrating):

```rust
// Names illustrative; adapt to PreviewLayers / overlay enums in code.
pub fn preview_overlay_color(layer_bits: crate::gui::editor::world_preview::layers::PreviewLayers, palette: &UiPalette) -> Color32 { /* … */ }

pub fn preview_debug_layer_color(layer_id: u8, palette: &UiPalette) -> Color32 { /* … */ }

pub fn dirty_chunk_highlight(palette: &UiPalette) -> Color32 { /* e.g. accent_hot or warn */ }
```

Goal: one place to tune colors before they become **shader constants**, **LUTs**, or **material uniforms**.

---

## 6. Related docs

| Doc | Use |
|:---|:---|
| [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md) | Broader chunk scheduling; cites material `ChunkDirty` precedent |
| [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) | U7 rows, `ChunkDirty` bitmask |
| [`map_editor_runbook_v1.md`](map_editor_runbook_v1.md) | Editor UX; snapshot / serialization coupling |
| [`ui_design_language_plan_v1.md`](ui_design_language_plan_v1.md) | `UiPalette`, P1+ token migration, overlay color parity |

---

## 7. Changelog

| Date | Note |
|:---|:---|
| 2026-05-10 | Initial runbook: Stages 2–4, explicit “no premature micro-opt” note, U7 alignment. |
| 2026-05-10 | Invalidation ownership table, `ChunkTextureRect` authority, z-order invariant, `PreviewGenerationEpoch`, §2.1 no-gameplay rule, doc chain, P1+ token helpers sketch. |
