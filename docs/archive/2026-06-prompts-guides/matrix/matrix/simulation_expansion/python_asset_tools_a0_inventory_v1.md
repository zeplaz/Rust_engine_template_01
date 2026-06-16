# Python asset tools — A0 inventory (hardcoded taxonomy) `v1`

> **Runbook:** [`../../guides/python_asset_tools_alignment_runbook_v1.md`](../../guides/python_asset_tools_alignment_runbook_v1.md) · **Stage:** A0 — **Applied** (initial pass 2026-05-06).

Version: `v1.0.2`

---

## Purpose

Satisfy **A0 — Inventory**: every place `src/utils/asset_tools` defines **domain** string lists that should stay aligned with engine / JSON schemas. UI-only literals (dialog titles, sort column labels) are noted but not migration targets.

---

## Inventory table

| Location | Symbol / usage | Aligned? | Notes / consumer |
|:---|:---|:---:|:---|
| `config/asset_config.py` | `ASSET_TYPES`, `SEGMENT_MEMBERSHIP`, `RESOURCE_TYPES`, `FUEL_*`, petroleum lists, `CONCRETE_TYPES`, `POWER_*` (plant / grid / switch / status), … | **Source** | Canonical for editor comboboxes until codegen exists. |
| `pages/vehicle_page.py` | `VEHICLE_TYPES`, fuel combos | **Yes** | Imports from `asset_config`. |
| `pages/power_page.py` | Plant / distribution / switch enums | **Yes** | Imports `POWER_PLANT_TYPES`, `POWER_DISTRIBUTION_TYPES`, `SWITCH_*`, `OPERATIONAL_STATUSES` from `asset_config`. |
| `pages/building_page.py` | `RESOURCE_TYPES` | **Yes** | Imports from `asset_config`. |
| `pages/dynamic_building_page.py` | `RESOURCE_TYPES`, `SOUND_CLASSES` | **Yes** | Imports from `asset_config`. |
| `pages/texture_page.py` | `TEXTURE_MAP_STATES` | **Yes** | Imports from `asset_config` (`try` / fallback legacy list if import fails). |
| `config/content_constants.py` | `MASTER_ENTITY_FILTER_ROLES` | **Yes** | `tuple(ASSET_TYPES)` from `asset_config` (S8 ∥ S2, 202606). |
| *engine truth* | Rust `ResourceType` (`src/entities/types/p_enumz.rs`) vs Python `RESOURCE_TYPES` | **Drift** | Python list is richer (e.g. petroleum `Refined_*`); S0 matrix + S8 close the gap. [`asset_sim_ownership_matrix_v1.md`](../asset_sim_ownership_matrix_v1.md). |
| `pages/dynamic_building_page.py` | Sort combo: `Name`, `Category`, … | **N/A** | UI-only; not a simulation vocabulary. |
| `integration/const_game_entities.py` | Regex for power voltages in `.const` files | **N/A** | Reads game data; does not define ontology. |
| `pages/terrain_registry_pages.py`, `worldgen_page.py`, `transport_page.py` | JSON editors / paths; no hardcoded sim vocab lists | **N/A** | Code scan 2026-05-06 — no `addItems`/taxonomy arrays to centralize. |
| `scripts/check_resource_parity.py` | Rust `ResourceType` ⊆ `RESOURCE_TYPES` | **Guard** | Run: `python src/utils/asset_tools/scripts/check_resource_parity.py` |

---

## Next stages (handoff)

| Stage | Action |
|:---:|:---|
| **A1** | Unify `MASTER_ENTITY_FILTER_ROLES` with `ASSET_TYPES`; scan `terrain_registry_pages` / `transport_page` for new lists. (**Done:** `content_constants` → `ASSET_TYPES`.) |
| **A2** | Map `POWER_PLANT_TYPES` **Oil** / **Gas** to petroleum runbook vocabulary in docs (and later recipes), without breaking existing `plant_definitions.json`. |
| **A3** | **`scripts/check_resource_parity.py`** — fails CI if a Rust `ResourceType` is missing in Python. |

---

## Document history

- **2026-05-06:** Terrain / worldgen / transport pages scanned — no new lists; **A3** script added.
