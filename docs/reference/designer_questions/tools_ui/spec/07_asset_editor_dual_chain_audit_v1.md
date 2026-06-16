# Asset editor — dual tool-chain audit `v1`

**Canonical editor:** `src/utils/asset_tools/` (PyQt5 + qfluentwidgets, `main_window.py` → SplitFluentWindow).

**Legacy tree:** `utils/asset_tools/` (PySide6 + `asset_tool_.ui`, `main_entity_editor.py`).

**Purpose:** Record what each stack does, what is **broken** in legacy, and what was **ported** so the canonical path does not lose intended features.

---

## 1. Summary

| Aspect | Canonical (`src/utils/asset_tools`) | Legacy (`utils/asset_tools`) |
|:---|:---|:---|
| UI toolkit | PyQt5 + Fluent | PySide6 + loaded `.ui` |
| Entry | `run.py` → `main_window.main` | `run.py` → `main_entity_editor.main` (fails if it tried `main_window` first) |
| Status | **Maintained** — save/load JSON, nav pages | **Deprecated** — import/symbol errors; stubs |
| Repo paths | `src/.../repo_paths.py` | Hardcoded `/data/...` strings |

**Decision:** Use **only** `src/utils/asset_tools` for new work. Legacy remains as **reference** for ideas until deleted or archived.

---

## 2. Feature matrix

| Feature / idea | Canonical | Legacy | Notes |
|:---|:---:|:---:|:---|
| Nav: Home, Vehicles, Buildings, Textures, Transport, Power | ✅ | Partial | Legacy: toolbox, no Fluent nav |
| **Global “entity role” filters** (Building, Vehicle, Power, Transportable, Productive, Scenery) | ⚠️ constants only | ✅ UI checkboxes | Canonical: `config/content_constants.py` + Home “roles” caption; **no** mutual-exclusion tab locking yet |
| **Segment / faction** radio (SEGMENT_MEMBERSHIP) | ✅ per-page | ✅ global dock | Equivalent capability on building/vehicle/power forms |
| **Add asset class folder** → tabs of file lists | ✅ Home “Browse package” | ✅ `refresh_tabs` | Canonical: browse folder + list JSON/PNG (simplified) |
| **Substation / transformer / switch** power forms | ✅ `power_page.py` tabs | ⚠️ `power_pages.py` incomplete (`PowerUI`, bugs) | Canonical is the source of truth |
| **Building type** dropdown (Residencey, Warehouse, …) | ✅ `dynamic_building_page` | ⚠️ `BUILDING_TYPES` **undefined** — NameError | Legacy block is broken |
| **PowerUI** import from `buildings_pages` | N/A | ❌ **Wrong import** (`PowerUI` lives in `power_pages.py`) | Legacy does not run without fix |
| **Apartment / tile matrix** building UI | ? | ⚠️ partial in `buildings_pages` | Evaluate port if still needed |
| **Road / Rail** JSON workspace | ✅ `transport_page.py` + `assets/configs/roads|rails/` | ❌ stubs only | Legacy UI not runnable |
| **Voltage list** from `consts_for_entities.const` | ✅ Home + `integration/const_game_entities.py` | ✅ `populate_from_config` regex | Ported |
| **ClickableComboBox** (+ helpers) | ✅ `components/editor_widgets.py` | ✅ `subsystems.py` (PySide6) | PyQt5 port kept for reuse |
| **ArrowHideableSpinBox** | ✅ `apply_readonly_spin()` same module | ✅ class in legacy | Function-style port |
| **Dark theme** | ✅ `main_window` stylesheet | ✅ `style_sheet.py` | Equivalent |
| **FlatBuffers** tooling | ✅ `integration/flatbuffers.py` | ❌ | Canonical only |
| **`repo_paths`** (assets/config/power, tiled, …) | ✅ | ❌ | Canonical only |

Legend: ✅ done · ⚠️ partial/broken · ❌ absent.

---

## 3. Porting checklist (completed in this audit pass)

- [x] Document dual chain (this file).
- [x] `utils/asset_tools/README.md` → **deprecated**, points to canonical + §06/§07.
- [x] `config/content_constants.py` — `MASTER_ENTITY_FILTER_ROLES` aligned with legacy intent.
- [x] `repo_paths.game_entities_consts_file` + `integration/const_game_entities.py` — voltage line parse.
- [x] **Transport** page + road/rail example JSON + `_building_types_index.json`.
- [x] **PyQt5** `components/editor_widgets.py` (`ClickableComboBox`, regex populate, spin readonly helper).

### Optional later

- [ ] Mutual-exclusion “role” filters that enable/disable nav sections (legacy behavior).
- [ ] Remove or archive `utils/asset_tools` after one release cycle.

---

## 4. PyQt5 widget ports & transport assets (maintenance)

| Deliverable | Location |
|:---|:---|
| `ClickableComboBox`, label stack, regex populate, readonly spin helper | `src/utils/asset_tools/src/components/editor_widgets.py` |
| Roads / rails JSON workspace | `src/utils/asset_tools/src/pages/transport_page.py` |
| Example schemas on disk | `assets/configs/roads/example_road_segment_v1.json`, `assets/configs/rails/example_rail_track_v1.json` |
| Building-type index (tool ↔ `s_flagz.rs`) | `assets/configs/buildings/_building_types_index.json` |

### Before archival of `utils/asset_tools`

- [x] Ports above live under **canonical** `src/utils/asset_tools`.
- [x] Each surface domain has at least one **example JSON** and a **Rust source-of-truth** pointer in `meta` / index.
- [ ] Engine **load/save** for road/rail blobs (Rust) when gameplay needs it.
- [ ] Optional: `MappingTable` / `TreeComboBox` (legacy was incomplete — reimplement only if required).

---

## 5. Related

- [`06_asset_content_studio_workflow_v1.md`](06_asset_content_studio_workflow_v1.md) — artist workflow, Tiled, JSON.
- `src/utils/asset_tools/README.md` — how to run the canonical editor.
