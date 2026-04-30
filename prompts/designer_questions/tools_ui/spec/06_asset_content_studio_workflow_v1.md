# Asset Content Studio — workflow & tool architecture `v1`

**Audience:** artists, level designers, technical designers, **not** Rust engineers.  
**Goal:** add, edit, preview, and validate **schemas + JSON + tile maps** for power plants, buildings, vehicles, weapons, and props **without** modifying `src/**/*.rs`.

**Paired code:** desktop editor `src/utils/asset_tools/` (PyQt5 + Fluent). **Legacy PySide tree:** `utils/asset_tools/` is **deprecated** — see [`07_asset_editor_dual_chain_audit_v1.md`](07_asset_editor_dual_chain_audit_v1.md).

---

## 1. Principles

| Principle | Meaning |
|:---|:---|
| **Data outside the binary** | Tunables and blueprints live in JSON / Tiled under `assets/`. Rust loads them; rebalance without recompiling. |
| **One logical “content studio”** | The same app hosts domain pages (Power, Buildings, Vehicles, **World Gen**, **Transport** roads/rails, **future:** Weapons). Shared: path resolution, preview, validation, tile picking. Reusable widgets: `components/editor_widgets.py`. |
| **Sidecars over hardcoding** | Visual variants (lights on/off, full/empty, damage) reference **named** tile regions or texture regions in a small JSON sidecar next to the art source. |
| **Placeholders are first-class** | Every slot may point to a **placeholder** entry (solid color, simple icon, or shared “missing” tile) until art is ready. The game and tools accept the same schema. |

---

## 2. Repository layout (what artists touch)

| Area | Path | Used for |
|:---|:---|:---|
| Power plant definitions | `assets/config/power/plant_definitions.json` | Full `PlantDefinition` rows; regenerate via `utils/gen_plant_definitions_json.py` or edit in a **future** Power Definitions sub-pane. |
| Building configs | `assets/configs/buildings/*.json` | Building blueprints (existing pattern). |
| Building type index | `assets/configs/buildings/_building_types_index.json` | Tool-facing map of `BuildingType` / factories / mines vs `s_flagz.rs` + links to examples. |
| Road configs | `assets/configs/roads/*.json` | Road segment / surface data (see `example_road_segment_v1.json`). **Transport** page in editor. |
| Rail configs | `assets/configs/rails/*.json` | Track / gauge examples (see `example_rail_track_v1.json`). |
| Vehicle configs | `assets/configs/vehicles/*.json` | Vehicle templates. |
| Tiled / isometric | `assets/tiled/**/*.tmx`, `*.tsx` | Tile layers, tilesets, object layers for facings and variants. |
| World gen tuning overlay | `assets/config/world_gen_tuning.json` (optional) | `noise_sampling` + `biome_tuning`; example: `world_gen_tuning.example.json`. **World Gen** page in asset editor · [`08_world_gen_desktop_tool_v1.md`](../../tools_ui/spec/08_world_gen_desktop_tool_v1.md). |
| Tool path helper | `src/utils/asset_tools/src/repo_paths.py` | Stable `Path` objects from Python to the above (finds repo root via `Cargo.toml` + `assets/`). |

Weapons and military props: when JSON schemas land under `assets/configs/` (or `assets/game_entities/`), register them in `repo_paths.py` and add a **Weapons** nav page reusing the same patterns below.

---

## 3. Isometric tiles: facings, states, and “logical” workflow

### 3.1 Problem

Isometric entities need **multiple view directions** (e.g. 4 or 8 facings). Gameplay and UI also need **states** (full/empty, lights on/off, turn signals, doors open, damaged). Artist-friendly workflow must avoid hand-editing coordinates in Rust.

### 3.2 Recommended Tiled conventions

- **One tileset** (or few) per entity family; consistent tile size (e.g. 64×32 isometric diamond).
- **Layers** (or **tile layers per variant**), named predictably, e.g.  
  `base`, `lights_on`, `lights_off`, `cargo_full`, `cargo_empty`, `turn_L`, `turn_R`, `damaged`.  
  Alternatively: **one layer** + **flip flags** only if your pipeline already supports it.
- **Object layer** `facing_markers`: optional rectangles naming `N`, `NE`, `E`, … mapping to column ranges in the tileset (for tools to parse).
- Export: **same TMX/TSX** paths referenced from a **sidecar JSON** the engine eventually loads (see §4).

### 3.3 Tool behavior (target)

1. **Import TMX/TSX** — parse with a small XML reader (no strict dependency required for v1: optional `tmx`/`tiled` PyPI package later).
2. **Tile grid viewer** — show the tileset image with grid overlay; click to record `(tile_id)` or `(x,y)` in GID space.
3. **Assign matrix** — UI table: rows = facings, columns = state (e.g. `default`, `lights_on`, `turn_left`). Cells store tile id or source rect.
4. **Emit** — write `assets/configs/visuals/<asset_id>_sprite_map.json` (name TBD) consumed by Bevy `AssetLoader` later; until then, tools + docs are the contract.

### 3.4 Variants (lights, signals, fill level)

Model as **orthogonal toggles** in data where possible:

- `visual_variant_groups`: e.g. `lighting: [off, on]`, `load: [empty, partial, full]`, `turn_signal: [none, left, right, hazard]`.
- Cartesian explosion is **optional** at export: either list only authored combinations or generate placeholder refs for missing pairs (clearly marked in preview).

---

## 4. Power plants: binding definitions to visuals

- **Simulation** row: `PlantDefinition.id` in `plant_definitions.json` (`definition_id` on `PowerPlant` in ECS).
- **Studio** row: optional `visual_profile_id` or inline `sprite_map` path in a **separate** JSON if you want to keep `plant_definitions.json` purely numeric; alternatively add a `visual` block to the plant row once schema is stable.
- **Workflow:** designer picks `definition_id` → tool shows key stats (read-only) → assigns tile map / texture profile → saves **visual sidecar** only.

---

## 5. Automation & placeholders

| Automation | Description |
|:---|:---|
| **Assign placeholder** | Button: fill all unset cells with repo-wide `placeholder_iso.png` or a colored procedural pixmap in preview only. |
| **Bulk facing copy** | Copy row `N` → mirror to `S` when art is symmetric (with flip flag in sidecar). |
| **Validate on save** | JSON Schema (optional): required fields, enum alignment with Rust `serde` names (`PascalCase` for `OperationalStatus`, etc.). |
| **Regen from script** | e.g. `utils/gen_plant_definitions_json.py` for bulk plant rows; tools **reload** file with user confirmation if on-disk changed. |

---

## 6. Desktop app structure (incremental)

Current nav: Home, Vehicles, Buildings, Textures, Power (`main_window.py`). **Suggested order of implementation:**

1. **Shared:** `repo_paths.py` (done) + “Open in Explorer” / path labels on each page.
2. **Power:** tab “Plant definitions” — list `plant_definitions.json` ids, view JSON in read-only tree, **future** edit + validate.
3. **Textures / Buildings:** wire file pickers to `repo_paths.buildings_configs_dir` and tile dir; show `TEXTURE_MAP_STATES` from `asset_config.py` as checklist vs sidecar.
4. **Tile picker widget** — reusable `TileGridPicker` used by Buildings, Vehicles, Power visual tab.
5. **Weapons** — duplicate Building page pattern with its schema folder.

---

## 7. What stays in Rust

- **Loading** definitions, spawning entities, gameplay systems.
- **No** per-asset hardcoding: new buildings/plants/weapons should be **new JSON + art**, not new `match` arms.

---

## 8. Review cadence

When adding a new content domain: update **`repo_paths.py`**, this doc §2 table, and `production_economy` / domain specs if economy-affecting.
