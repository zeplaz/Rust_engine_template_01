# Python asset tools — alignment runbook `v1`

> **STATUS:** Draft **v1**. Cross-cuts **ontology**, **asset authoring**, and **simulation-facing JSON**. Keeps the desktop **Asset Content Studio** (`src/utils/asset_tools/`) from drifting from Rust loaders and runbook taxonomies. **Sequence:** prefer advancing **S0** ([`asset_sim_ownership_matrix_v1.md`](../matrix/simulation_expansion/asset_sim_ownership_matrix_v1.md)) before **broad** S8 rewrites so the tool reflects **declared** engine ownership, not guessed lists.

Version: `v1.0.5`  
Audience: implementers and agents updating PyQt pages (`*_page.py`), `asset_config.py`, and exported JSON under `assets/`.

**Pairs with:** [`world_assets_tools_rulebook_v1.md`](world_assets_tools_rulebook_v1.md) (Bevy/world parity) · [`06_asset_content_studio_workflow_v1.md`](../designer_questions/tools_ui/spec/06_asset_content_studio_workflow_v1.md) (studio scope) · [`asset_system_audit_runbook_v1.md`](asset_system_audit_runbook_v1.md) (asset → owner matrix) · [`petroleum_industry_simulation_runbook_v1.md`](petroleum_industry_simulation_runbook_v1.md) (crude / fractions / refined vs carriers).

---

## 1. Purpose

The Python tool is not a second game design authority: it **reflects** committed schemas and **stable names** the engine understands. When simulation layers add registries (petroleum, concrete, power, transport), the tool must either:

- import enums and lists from a **single** Python module (`src/utils/asset_tools/src/config/asset_config.py` and siblings), or  
- generate that module from the same JSON/schema the Rust side uses, **once** such a pipeline exists.

Until generation exists, **`asset_config.py` is the editorial single source of truth** for tool comboboxes and normalization helpers; pages must not duplicate fat string lists.

---

## 2. Scope (in / out)

| In scope | Out of scope |
|:---|:---|
| `src/utils/asset_tools/**` (PyQt5 + Fluent) | Legacy `utils/asset_tools/` tree except migration notes |
| JSON shapes writers emit for `assets/configs/**`, `assets/config/**` | Rewriting the Bevy `AssetLoader` stack in one pass |
| Normalization of legacy field values on load (aliases) | Full JSON Schema CI for every asset type (future step pack) |
| Parity with **named** simulation concepts in domain runbooks | Gameplay balance numbers without designer sign-off |

---

## 3. Alignment principles

1. **One vocabulary per concept** — e.g. **crude / refinery fractions / `Refined_*` resources** vs **vehicle `fuel_type`** and **`fuel_class` as `FUEL_SOURCE_CATEGORY`** are different lists; do not merge them in UI copy or combo sources. See petroleum runbook and `asset_config.py` sections for `CRUDE_TYPES`, `REFINERY_FRACTIONS`, `RESOURCE_TYPES`, `FUEL_TYPES`, `FUEL_SOURCE_CATEGORY`.  
2. **Serialized keys stay stable** — e.g. keep JSON key `fuel_class` while the UI labels it “Fuel source category.”  
3. **Legacy strings** — map through `LEGACY_*_ALIASES` and `normalize_*` helpers when loading old files or pasted data; emit **canonical** names on save/export.  
4. **No orphan lists** — `power_page.py`, `vehicle_page.py`, `building_templates.py`, etc. should **import** shared tuples/dicts from `asset_config` (or a thin `taxonomy.py`) instead of hardcoding parallel arrays.  
5. **Verify after edits** — `python -m py_compile` on touched modules; manual smoke: launch the asset editor and walk affected tabs.

---

## 4. Staged work (implementation order)

Stages are **ordered** but may overlap with simulation expansion **S0–S7**. In the step-pack index, this is **S8** ([`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md)). **S0 before S8 (for engine-faithful taxonomies):** extend [`asset_sim_ownership_matrix_v1.md`](../matrix/simulation_expansion/asset_sim_ownership_matrix_v1.md) for the domains you are touching, then align `asset_config.py` / pages. **S1** (chunk scheduler) does not block S8 maintenance work and may run **in parallel** with S0 — see [`new_propsal_guide_may202608.md`](new_propsal_guide_may202608.md) §5. **S2** (weather) and **S8** may also run **in parallel** (Rust sim vs Python editor).

| Stage | Goal | Done when |
|:---:|:---|:---|
| **A0 — Inventory** | List every hardcoded domain list in `src/utils/asset_tools/src/pages/` and `config/`; note Rust/JSON consumers. | [`python_asset_tools_a0_inventory_v1.md`](../matrix/simulation_expansion/python_asset_tools_a0_inventory_v1.md) (maintain as lists move). |
| **A1 — Taxonomy pull-through** | Move strings into `asset_config.py` (or split module per domain); wire pages to import; add normalization for known legacy values. | No duplicated fuel/resource enums across vehicle/power/building pages; `py_compile` clean. |
| **A2 — Domain hooks** | Align power generator fuels, building/production hints, and transport-related fields with [`petroleum_industry_simulation_runbook_v1.md`](petroleum_industry_simulation_runbook_v1.md) + production specs where applicable. | Designer-facing doc lists same canonical names as tool exports. |
| **A3 — Guardrails (light)** | `scripts/check_resource_parity.py` — every Rust `ResourceType` variant must appear in `RESOURCE_TYPES`. | Script in `src/utils/asset_tools/scripts/`; documented in §6. |

**Stop condition:** If Rust renames an enum without updating this runbook and `asset_config.py`, treat as a **defect** in whichever PR introduced the drift.

---

## 5. Cross-links

| Doc | Role |
|:---|:---|
| [`new_propsal_guide_may202608.md`](new_propsal_guide_may202608.md) | Program index; lists this runbook in the catalog |
| [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md) | Layers; tool alignment supports **Ontology** + **Industry** authoring |
| [`matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md) | **S8** stage entry |
| [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md) | When placeholder tool pages mislead about shipped behavior |

---

## 6. Verify commands (minimal)

```text
python -m py_compile src/utils/asset_tools/src/config/asset_config.py
python -m py_compile src/utils/asset_tools/src/config/content_constants.py
python -m py_compile src/utils/asset_tools/src/pages/vehicle_page.py
python -m py_compile src/utils/asset_tools/src/pages/power_page.py
python src/utils/asset_tools/scripts/check_resource_parity.py
```

Add paths for any page touched in a PR. **CI:** [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs `cargo check` (default + `bevy_tilemap_adapter`), `check_resource_parity.py`, and the `py_compile` lines above on push / PR.

Full editor launch is manual until an integration test exists.

---

## 7. Document history

- **2026-05-06:** Initial `v1` — stages A0–A3; linked as simulation expansion **S8**.
- **2026-05-06:** `v1.0.5` — CI workflow documents verify commands ([`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)).
