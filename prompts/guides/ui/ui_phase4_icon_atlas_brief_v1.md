# UI Phase 4 — icon atlas brief `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | Design authority for **PLAN-UI-P4-ATLAS-001** |
| **Planner plan** | [`ui_phase4_icon_atlas_plan_v1.md`](ui_phase4_icon_atlas_plan_v1.md) |
| **Version** | `1.3.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` · **Phase 4.1 code** `@coder` |
| **Parent** | [`legacy_asset_reference_manifest_v1.md`](legacy_asset_reference_manifest_v1.md) |
| **Asset index** | [`tools/orchestrator/knowledge/ui_texture_assets.json`](../../../tools/orchestrator/knowledge/ui_texture_assets.json) |
| **Palette** | [`palette_v2_tokens.md`](palette_v2_tokens.md) · [`src/gui/style/palette.rs`](../../../src/gui/style/palette.rs) |
| **P4 host** | [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) § P4 · [`in_game_hud.rs`](../../../src/gui/in_game_hud.rs) `BuildRailRoot` |
| **Icon language** | [`ui_icone_design_spec.md`](ui_icone_design_spec.md) |
| **Petroleum tab** | [`petroleum_industry_ui_snippet_v1.md`](petroleum_industry_ui_snippet_v1.md) |

**Status:** **Phase 4.1 CODE DONE** (2026-05-23) · **P4-P5-01 DONE** (2026-05-25) · **UI-OH-P4-ART-001 SIGNED** (2026-05-25) · **P4-VEH-01 OPEN** (optional).  
**Witness:** `debug_runs/ui_shell_migration_live.json` → `phase4.icon_atlas_loaded`, `phase4.p5_br_tab_wired`.

---

## Quick reference — P4 rail (row 0)

**Source of truth:** [`ui_texture_assets.json`](../../../tools/orchestrator/knowledge/ui_texture_assets.json) → `phase4_icon_atlas`.  
**Cell size:** **32×32** px · **Atlas:** 256×128 (8×4 grid) · **Runtime:** collapsed build rail @ 48px width.

| Cell | `IconId` | `ToolContext` | Trace source (`cells` in JSON) | Silhouette read |
|:---|:---|:---|:---|:---|
| (0,0) | **RD** | `Roads` | `textures/misc/railroad_track.png` | Horizontal corridor / tie rhythm — **not** a building |
| (0,1) | **RL** | `Rail` | same as RD | Stronger **parallel rails** than RD; optional slight cant |
| (0,2) | **UT** | `Utilities` | `textures/power/tile_map_powerstuff_power_trasformer_oil_cooled_alpha.png` | **Stationary transformer** — fins + top bushings + conservator cylinder |
| (0,3) | **IN** | `Industry` | `legacy_manifest:factory_silhouette` | Chimney + box mass (e.g. `misc/wooden_buildings_01.png` crop) |
| (0,4) | **CV** | `Civil` | `legacy_manifest:civic_silhouette` | Low-rise cluster (e.g. `misc/cities.png` crop) — **not** vertical stack |

**Code map:** `tool_context_icon_id` in [`icon_atlas.rs`](../../../src/gui/hud/icon_atlas.rs) · witness `phase4.rail_icons: ["RD","RL","UT","IN","CV"]`.

**Do not ship for UT:** `textures/power/tile_map_rust_dev_utils_alpha.png` → **generator trailer** (wheels + hitch) — atlas cell **UT_MG** only (§5).

---

## 0. Code status snapshot (2026-05-23)

### Phase 4.1 — landed

| Item | Status | Location |
|:---|:---|:---|
| Atlas loader + `IconId` enum | ✅ | [`src/gui/hud/icon_atlas.rs`](../../../src/gui/hud/icon_atlas.rs) |
| Custom RON asset loader | ✅ | Extension **`.icon_atlas.ron`** (`IconAtlasManifestLoader`) |
| Startup load | ✅ | `IconAtlasPlugin` → `load_icon_atlas_ui_system` on `Startup` |
| Plugin registration | ✅ | [`simulation_shell_phase2.rs`](../../../src/gui/hud/simulation_shell_phase2.rs) `HudSimulationShellPhase2Plugin` |
| P4 rail spawn | ✅ | [`in_game_hud.rs`](../../../src/gui/in_game_hud.rs) — `BuildRailToolIcon` + 32×32 `ImageNode` for row-0 tools |
| Sync / tint | ✅ | `sync_build_rail_from_strip_system` + `build_rail_icon_tint` |
| `ToolContext` → icon map | ✅ | `tool_context_icon_id` — **Mi / Ec / None** → text-only fallback |
| RON manifest on disk | ✅ | [`assets/configs/ui/icon_atlas_phase4.icon_atlas.ron`](../../../assets/configs/ui/icon_atlas_phase4.icon_atlas.ron) |
| Atlas PNG on disk | ⚠️ **placeholder kept** | [`assets/textures/ui/icon_atlas_phase4_v1.png`](../../../assets/textures/ui/icon_atlas_phase4_v1.png) — optional designer traced bake |
| Knowledge JSON | ✅ | [`ui_texture_assets.json`](../../../tools/orchestrator/knowledge/ui_texture_assets.json) `phase4_icon_atlas` |
| Unit tests | ✅ | `icon_atlas.rs` UV grid + `simulation_shell_phase2.rs` `phase4` witness payload |
| Witness field | ✅ | `UiShellMigrationWitness::icon_atlas_loaded` |

**Load paths (authoritative — use these, not §10 sketch names):**

```rust
// src/gui/hud/icon_atlas.rs
pub const ICON_ATLAS_TEXTURE_PATH: &str = "textures/ui/icon_atlas_phase4_v1.png";
pub const ICON_ATLAS_MANIFEST_PATH: &str = "configs/ui/icon_atlas_phase4.icon_atlas.ron";
```

**Runtime behavior (collapsed rail):**

- Row-0 tools (`Roads`…`Civil`): show atlas icon; hide short text label.
- Expanded left stack: icon **hidden**, full `"Rd roads"` text shown.
- `Military` / `Ecology`: text labels only (no `BuildRailToolIcon` child).

**Tint mapping (implemented in `build_rail_icon_tint`):**

| State | Token | Code |
|:---|:---|:---|
| Idle | `fg_muted` @ 72% alpha | `bevy_text_muted().with_alpha(0.72)` |
| Hover | `ink_magenta_bright` | `bevy_accent_hot()` on **icon** |
| Selected | `gold_bar` | `bevy_accent_gold()` on **icon**; slot border `bevy_accent_gold()`, bg `bevy_bg_vellum()` |

**Known gap vs §8:** slot **border** does not switch to `accent_hot` on hover (icon tint only). Track as **P4-F03** in Phase 4.2 if ops-strip F-03 parity required on build rail.

### Phase 4.2 — open

| Item | Owner | Notes |
|:---|:---|:---|
| Replace placeholder atlas with traced silhouettes | `@designer` | ✅ **UI-OH-P4-ART-001** — [`ui_oh_p4_art_signoff_record_v1.md`](../../../src/dev/ui_oh_p4_art_signoff_record_v1.md) |
| UT ≠ UT_MG blind acceptance | `@designer` + `@coder` | §5 — optional after art drop |
| Wire `P5_BR` to petroleum tray tab | `@coder` | **DONE** — `sync_petroleum_panel_tab_system`; witness `p5_br_tab_wired` |
| Wire `TRUCK` / `URAL` / `BUS` | `@coder` | logistics / convoy UI when panel exists |
| Hover border on build-rail slot | `@coder` | optional **P4-F03** |
| `Mi` / `Ec` atlas cells | `@designer` | row 0 cols 5–6 reserved |

---

## 1. Intent

Replace Phase 2A **text-only** build-rail affordances (`Rd` / `Rl` / `Ut` / …) with a **single 32×32-celled texture atlas** of **monochrome silhouettes** traced from legacy tile-map art. Icons must read at **48px rail width** ([`CONTEXT_RAIL_W_PX`](../../../src/gui/hud/simulation_shell_phase2.rs)) and stay **recognizable without hue** (shape + frame, per [`ui_icone_design_spec.md`](ui_icone_design_spec.md)).

**Authority:** Presentation only — icons reflect `ToolContext` from [`BuildStripState`](../../../src/construction/build_strip.rs); they do **not** select tools or mutate simulation.

---

## 2. Deliverables

| Artifact | Path | Status |
|:---|:---|:---|
| Baked atlas PNG | `assets/textures/ui/icon_atlas_phase4_v1.png` | ✅ Traced silhouettes (2026-05-25) |
| UV manifest | `assets/configs/ui/icon_atlas_phase4.icon_atlas.ron` | ✅ Matches §3.1 grid |
| Rust module | `src/gui/hud/icon_atlas.rs` | ✅ Phase 4.1 |
| Knowledge row | `tools/orchestrator/knowledge/ui_texture_assets.json` | ✅ `phase4_icon_atlas` block |
| Layout mock (optional) | `assets/ui/phase4/icon_atlas_phase4_layout_mock.png` | ✅ Copy of baked atlas |

---

## 3. Grid spec (32×32 cells)

**Cell size:** 32×32 px (logical).  
**Atlas size:** 8 columns × 4 rows = **256×128 px**.  
**Padding:** 1px clear gutter between cells in bake (export canvas may be 266×138 with gutters; UVs target inner 32×32 only).

### 3.1 Layout map

```text
     col:  0      1      2      3      4      5      6      7
        ┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┐
row 0   │  RD  │  RL  │  UT  │  IN  │  CV  │ rsv  │ rsv  │ rsv  │  ← P4 tool rail (ship)
        ├──────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┤
row 1   │UT_TX │UT_MG │ rsv  │ rsv  │ rsv  │ rsv  │ rsv  │ rsv  │  ← disambiguation (doc + QA)
        ├──────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┤
row 2   │TRUCK │ URAL │ BUS  │ rsv  │ rsv  │ rsv  │ rsv  │ rsv  │  ← vehicle silhouettes
        ├──────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┤
row 3   │P5_BR │ rsv  │ rsv  │ rsv  │ rsv  │ rsv  │ rsv  │ rsv  │  ← petroleum tab affordance
        └──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
```

**UV formula (coder):** `u0 = col * 32 / 256`, `v0 = row * 32 / 128`, size `32/256` × `32/128`.

### 3.2 ASCII layout mock (optional stand-in for PNG)

```text
┌─256px──────────────────────────────────────────────────────────┐
│ [RD road] [RL rail] [UT xfmr] [IN plant] [CV civic]  ·  ·  ·   │ 32px
│ [TX ok  ] [MG warn]  ·   ·   ·   ·   ·   ·                    │ 32px
│ [truck  ] [ural   ] [bus   ]  ·   ·   ·   ·                    │ 32px
│ [barrel ]  ·   ·   ·   ·   ·   ·   ·                           │ 32px
└────────────────────────────────────────────────────────────────┘
```

---

## 4. Tool rail icons (row 0) — P4 build rail

Maps to [`ToolContext`](../../../src/construction/build_strip.rs) short labels in [`sync_build_rail_from_strip_system`](../../../src/gui/hud/simulation_shell_phase2.rs).

| Cell | Code | `ToolContext` | Silhouette source (trace) | Trace rules |
|:---|:---|:---|:---|:---|
| (0,0) | **RD** | `Roads` | `assets/textures/misc/railroad_track.png` *or* road-corridor crop from legacy manifest | Horizontal corridor / tie rhythm; **not** a building footprint |
| (0,1) | **RL** | `Rail` | `assets/textures/misc/railroad_track.png` | Stronger parallel-rail read than RD; optional 15° cant |
| (0,2) | **UT** | `Utilities` | **`assets/textures/power/tile_map_powerstuff_power_trasformer_oil_cooled_alpha.png`** | §5 — **stationary transformer** (fins, bushings, conservator); **never** generator trailer |
| (0,3) | **IN** | `Industry` | Legacy manifest → factory / stack silhouette (e.g. `misc/wooden_buildings_01.png` mass crop) | Chimney + box mass; avoid civic dome shapes |
| (0,4) | **CV** | `Civil` | Legacy manifest → civic / block silhouette (`misc/cities.png` or buildings crop) | Low-rise cluster; distinct from IN vertical stack |

**Bake pipeline (designer):**

1. Load source PNG alpha.
2. **Trace** → single-channel silhouette (threshold alpha; no literal RGB in icon).
3. Fit to **24×24** content inside 32×32 cell (4px margin); center.
4. Add **1px `wire_magenta` frame** on cell edge (optional in atlas bake; coder may draw frame in UI instead — pick one owner, not both).

**Out of scope for row 0:** `Military`, `Ecology` — remain text or Phase 4.2 cells when product asks.

---

## 5. CRITICAL — transformer ≠ generator trailer

Two power PNGs in `assets/textures/power/` read as “grid gear” at thumbnail size. **Only the stationary transformer** may drive **UT** on the build rail.

| Role | File | Atlas cell | Silhouette language (trace target) |
|:---|:---|:---|:---|
| **Oil-cooled transformer (AUTHORITATIVE for UT)** | `textures/power/tile_map_powerstuff_power_trasformer_oil_cooled_alpha.png` | **UT** (0,2) | **Fixed pad:** rectangular core, **vertical cooling fins**, **top bushings**, horizontal **conservator cylinder** on roof — **no wheels** |
| **Mobile generator trailer (FORBIDDEN for UT)** | `textures/power/tile_map_rust_dev_utils_alpha.png` | **UT_MG** (1,1) | **Towable:** box generator on **dual-axle trailer**, **V hitch**, **spare tire** on tongue — reads as field power cart, not substation |

```text
  UT (ship)                 UT_MG (QA only — DO NOT bind to Utilities)
 ┌─────────┐               ┌──○──┐  ← spare / hitch
 │ ▓▓ ▓▓   │  fins+bushings│     │
 │ ▓▓▓▓▓▓  │               │ GEN │
 └───▓─────┘  conservator  ○─────○  wheels
   no wheels
```

**Row 1 disambiguation cells (not bound to `ToolContext`):**

| Cell | Label | Purpose |
|:---|:---|:---|
| (1,0) **UT_TX** | Transformer trace reference | Gold-standard silhouette copied from transformer PNG |
| (1,1) **UT_MG** | Generator trailer — DO NOT SHIP | Same bake pipeline; designer QA + manifest `forbidden_alternate` |

**JSON guard (committed):** UT cell includes `"forbidden_alternate": "textures/power/tile_map_rust_dev_utils_alpha.png"`.

**Acceptance test (designer + coder):** At 48px rail width, blind review ≥4/5 labels **UT** as **transformer / grid**, not **generator trailer**.

---

## 6. Vehicle silhouettes (row 2)

**Rule:** Trace **alpha / empty-body** sheets — **not** full midday color tiles. Silhouette = occupancy shape only; runtime tints apply.

| Cell | Id | Source path | Notes |
|:---|:---|:---|:---|
| (2,0) | **TRUCK** | `textures/vehicles/civ_truck_01/tile_map_8_empty_miday.png` | Cab + bed massing |
| (2,1) | **URAL** | `textures/vehicles/ural_01/tile_map_ural_01_empty_midday.png` | Distinct cab height vs truck |
| (2,2) | **BUS** | `textures/vehicles/bus_01/tilemap_bus_01_alpha.png` | Long wheelbase, flat roof |

Use in: logistics inspector, convoy overlays, future tray chips — **not** P4 rail unless product promotes vehicles to rail.

---

## 7. Petroleum — P5 tab affordance (row 3)

| Cell | Id | Source | Host |
|:---|:---|:---|:---|
| (3,0) | **P5_BR** | `textures/misc/hjm-barrel_alpha.png` | Petroleum industry panel tab icon ([`petroleum_industry_ui_snippet_v1.md`](petroleum_industry_ui_snippet_v1.md)) |

**Read:** Single barrel, slight ¾ view; readable at 24px tab height. Pair with text label “Petroleum” / “P5” in tray — **icon + word**, not color alone.

---

## 8. Interaction tokens (selected / hover)

Map design names → [`UiPalette`](../../../src/gui/style/palette.rs) (see [`palette_v2_tokens.md`](palette_v2_tokens.md)).

| State | Design token | `UiPalette` field | Application on rail slot |
|:---|:---|:---|:---|
| **Idle** | `label_muted` | `fg_muted` | Silhouette tint ~70% white/cyan mono |
| **Hover** | `ink_magenta_bright` | `accent_hot` | 1px outer stroke on slot; optional silhouette brighten |
| **Selected** | `gold_bar` + `vellum` | `accent_gold` + `bg_vellum` | 2px left bar **or** full slot border gold; background `bg_vellum` |

**Implemented:** `sync_build_rail_from_strip_system` sets slot border/bg on selection; `build_rail_icon_tint` drives icon color on idle/hover/selected.

**Do not** use literal source art colors on icons — only tokens above.

---

## 9. `ui_texture_assets.json` (committed)

Current block in repo (`updated: 2026-05-23`):

```json
"phase4_icon_atlas": {
  "atlas_png": "assets/textures/ui/icon_atlas_phase4_v1.png",
  "cell_px": 32,
  "grid": [8, 4],
  "manifest_ron": "assets/configs/ui/icon_atlas_phase4.icon_atlas.ron",
  "cells": {
    "RD": { "tool_context": "Roads", "source": "textures/misc/railroad_track.png" },
    "RL": { "tool_context": "Rail", "source": "textures/misc/railroad_track.png" },
    "UT": {
      "tool_context": "Utilities",
      "source": "textures/power/tile_map_powerstuff_power_trasformer_oil_cooled_alpha.png",
      "forbidden_alternate": "textures/power/tile_map_rust_dev_utils_alpha.png"
    },
    "IN": { "tool_context": "Industry", "source": "legacy_manifest:factory_silhouette" },
    "CV": { "tool_context": "Civil", "source": "legacy_manifest:civic_silhouette" },
    "UT_TX": { "source": "textures/power/tile_map_powerstuff_power_trasformer_oil_cooled_alpha.png", "qa_only": false },
    "UT_MG": { "source": "textures/power/tile_map_rust_dev_utils_alpha.png", "qa_only": true },
    "TRUCK": { "source": "textures/vehicles/civ_truck_01/tile_map_8_empty_miday.png" },
    "URAL": { "source": "textures/vehicles/ural_01/tile_map_ural_01_empty_midday.png" },
    "BUS": { "source": "textures/vehicles/bus_01/tilemap_bus_01_alpha.png" },
    "P5_BR": { "source": "textures/misc/hjm-barrel_alpha.png", "panel": "petroleum_p5_tab" }
  }
}
```

Parent [`legacy_asset_reference_manifest_v1.md`](legacy_asset_reference_manifest_v1.md) remains canonical for **disk paths** and art provenance; this JSON is orchestrator-facing index.

---

## 10. RON manifest (on disk)

File: `assets/configs/ui/icon_atlas_phase4.icon_atlas.ron` — loaded via custom extension `.icon_atlas.ron`.

```ron
(
  schema_version: 1,
  texture: "textures/ui/icon_atlas_phase4_v1.png",
  cell_size: (32, 32),
  atlas_size: (256, 128),
  icons: {
    "RD": (col: 0, row: 0),
    "RL": (col: 1, row: 0),
    "UT": (col: 2, row: 0),
    "IN": (col: 3, row: 0),
    "CV": (col: 4, row: 0),
    "UT_TX": (col: 0, row: 1),
    "UT_MG": (col: 1, row: 1),
    "TRUCK": (col: 0, row: 2),
    "URAL": (col: 1, row: 2),
    "BUS": (col: 2, row: 2),
    "P5_BR": (col: 0, row: 3),
  },
)
```

---

## 11. `@coder` handoff

### 11.1 Phase 4.1 — DONE (do not re-implement)

Exit criteria:

- [x] Atlas + RON load without panic in `BaseState::Simulation`.
- [x] P4 shows **RD RL UT IN CV** icon cells (placeholder art).
- [x] Selected/hover icon tint uses `accent_gold` / `accent_hot` (§8).
- [x] `cargo test -p proc_A_dine01 --lib` — `icon_atlas` + witness tests green.
- [x] `ui_shell_migration_live.json` includes `phase4` block.

**Verify locally:**

```powershell
cargo test -p proc_A_dine01 --lib icon_atlas
cargo test -p proc_A_dine01 --lib simulation_shell_phase2::tests::ui_p2a_001_live_witness_refresh
cargo run -p proc_A_dine01 -- --test frame
# Collapsed build rail: icons visible for Rd/Rl/Ut/In/Cv; Mi/Ec text-only
```

**Do not touch without reason:** `IconAtlasManifestLoader` extension list, `ICON_ATLAS_*_PATH` constants, or witness shape (downstream proofs depend on keys).

---

### 11.2 Phase 4.2 — next tasks (priority order)

#### P4-ART-01 — Swap atlas PNG — **DONE** (2026-05-25)

**Status:** Traced silhouette bake delivered — **UI-OH-P4-ART-001** **SIGNED — PASS**. Re-bake: `python tools/orchestrator/scripts/bake_icon_atlas_phase4.py`.

**Input:** Designer replaced `assets/textures/ui/icon_atlas_phase4_v1.png` with traced silhouettes (§4–§7). **Kept** 256×128 and cell indices — no RON change.

**Coder:** After art drop, run frame test + confirm `witness.icon_atlas_loaded` still true. Optional: add lib test comparing UT vs UT_MG cell hash or visual snapshot row.

#### P4-P5-01 — Petroleum tab icon — **DONE** (2026-05-25)

**Goal:** Show `IconId::P5Br` on petroleum industry panel tab ([`petroleum_industry_ui_snippet_v1.md`](petroleum_industry_ui_snippet_v1.md)).

**Landed:** `image_node_for_id` in [`icon_atlas.rs`](../../../src/gui/hud/icon_atlas.rs) · `sync_petroleum_panel_tab_system` in [`simulation_shell_phase2.rs`](../../../src/gui/hud/simulation_shell_phase2.rs) · witness `phase4.p5_br_tab_wired`.

**Planner rollup:** [`ui_phase4_icon_atlas_plan_v1.md`](ui_phase4_icon_atlas_plan_v1.md) § P4.2a.

#### P4-VEH-01 — Vehicle row consumers

Wire `TRUCK` / `URAL` / `BUS` when logistics inspector chips exist (`HudInfoTab::Logistics` or convoy overlay). Read-only presentation.

#### P4-F03 — Build-rail hover border (optional)

In `sync_build_rail_from_strip_system`, when `Interaction::Hovered && !selected`, set `BorderColor::all(palette.bevy_accent_hot())` to match ops-strip F-03.

#### P4-MI-EC — Military / Ecology cells (deferred)

Add cols 5–6 row 0 in RON + extend `tool_context_icon_id` when product requests.

---

### 11.3 API reference (for consumers)

```rust
use crate::gui::hud::{
    IconAtlasUi, IconAtlasManifest, IconId,
    tool_context_icon_id, tool_context_uses_icon_atlas,
};

// Startup resource (always present after IconAtlasPlugin)
let atlas: Res<IconAtlasUi>;
let manifests: Res<Assets<IconAtlasManifest>>;

// Build ImageNode with UV sub-rect
if let Some(node) = atlas.image_node_for_tool(&manifests, ToolContext::Utilities) {
    // .with_color(tint) in sync system
}

// Manifest UV: IconAtlasManifest::cell_rect(IconId::Ut) → Bevy Rect in texel space
```

**Components:** `BuildRailToolSlot(ToolContext)` · `BuildRailToolIcon` · `BuildRailToolLabel`

**Systems:** `sync_build_rail_from_strip_system` in `Update` (simulation shell phase 2 plugin).

---

### 11.4 Out of scope (all phases)

- GPU minimap compositor ([`ui_phase3_gpu_minimap_m1_planner_v1.md`](ui_phase3_gpu_minimap_m1_planner_v1.md)).
- Procedural icon generation from ECS metadata.
- Gameplay / `BuildStripState` authority changes (presentation only).

---

## 12. Product notes (operator)

- **Minimap movable:** DQ-POST-06 — drag egui Minimap title bar; layout persists via `HudLayoutStore`. See [`ui_construction_playtest_v1.md`](../../../src/dev/ui_construction_playtest_v1.md) §1.
- **Build submenus in sim:** Construction catalog is **editor-only** (PLAY-01); sim build rail is mode icons only until UX-C.

---

## 13. Cross-links

| Doc | Role |
|:---|:---|
| [`legacy_asset_reference_manifest_v1.md`](legacy_asset_reference_manifest_v1.md) | Parent path index + art credit |
| [`ui_design_language_plan_v1.md`](../ui_design_language_plan_v1.md) | Token-first policy (P4 Bevy UI) |
| [`ui_phase4_icon_atlas_plan_v1.md`](ui_phase4_icon_atlas_plan_v1.md) | **PLAN-UI-P4-ATLAS-001** — track map + gates |
| [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) | F-03 hover / F-07 selected tab |
| [`ui_operational_direction_runbook_v1.md`](../ui_operational_direction_runbook_v1.md) | Left mode rail ergonomics |

---

## 14. Designer checklist

- [x] Grid layout + placeholder PNG committed (256×128).
- [x] UT / UT_MG cells distinct in manifest (cols differ — `icon_atlas` unit test).
- [x] **Replace placeholder** with traced **32×32** silhouettes per §4–§7 and JSON `phase4_icon_atlas.cells`.
- [ ] Transformer vs **generator trailer** blind test passed (§5) — **optional** @operator.
- [x] All row-0 icons readable at 48px rail, collapsed stack.
- [x] Vehicles distinguishable by silhouette at 32px.
- [x] P5 barrel recognizable beside tab label.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-23 | Initial grid + Phase 4.1 handoff |
| v1.1.1 | 2026-05-23 | Code snapshot; RON + witness |
| v1.4.0 | 2026-05-25 | **UI-OH-P4-ART-001** traced atlas + bake script |
| v1.3.0 | 2026-05-25 | P4-P5-01 closed; link PLAN-UI-P4-ATLAS-001 plan |
| v1.2.0 | 2026-05-24 | Quick ref RD/RL/UT/IN/CV; §5 transformer ≠ generator trailer |
