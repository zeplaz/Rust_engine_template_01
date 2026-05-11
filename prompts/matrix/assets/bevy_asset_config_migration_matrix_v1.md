# Bevy asset & config migration matrix `v1`

**STATUS:** ⏳ **Planned** — repo still has JSON samples under `assets/configs/`; RON migration not bulk-applied.

| Axis | State |
|:---|:---:|
| Decision | ✅ use **Bevy-aligned** config loading (RON preferred for hand edit) |
| Samples | ⏳ JSON → RON file-by-file |
| Loader code | ⏳ extend `AssetLoader` / `serde` paths |

> **Prompt use:** Pair `terrain_world/` + `production_economy/` + `factions/` READMEs for RON/config paths · confirm on-disk paths under `assets/` · cite loader/type names · `ASK:` for format rollout order.

---

## Locked intent

- **Best Bevy-supported config story** = **`ron`** + `serde` + optional `Asset` types (see Bevy book / 0.18 asset patterns).
- Keep **one manifest or index** per domain where useful (mirrors `ProductionManifest` pattern).

---

## Migration table

| Path pattern | Current | Target | Action |
|:---|:---|:---|:---|
| `assets/configs/buildings/*.json` | JSON | `*.ron` | convert + loader |
| `assets/configs/vehicles/*.json` | JSON | `*.ron` | convert + loader |
| `assets/configs/vehicles/vehicle_configs.json` | JSON array | `*.ron` or split files | 📎 |
| `assets/configs/production/*` | `.data` / mixed | typed RON or bin | audit |
| `assets/config/terrain/*.json` | — | committed examples + future registry | hybrid JSON tables (materials, tags) + RON rules 📎 [`material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) |

---

## Terrain registry assets (paired [`../../guides/bevy_asset_terrain_runbook_v1.md`](../../guides/bevy_asset_terrain_runbook_v1.md))

| Logical asset | Example on disk | Format | Target loader extension | `schema_version` | Loader / `Asset` |
|:---|:---|:---|:---|:---:|:---:|
| Material registry | `material_registry.example.json` · `material_registry.example.ron` (mirror) | JSON + RON | `.material_registry.{json,ron}` | required | **Applied** — `MaterialRegistry` + `MaterialRegistryLoader`; registered in [`MaterialUnificationPlugin`](../../../src/systems/terrain/material_plugin.rs); example RON parity tests; startup prefers `.ron` when present |
| Tag registry | `tag_registry.example.json` · `tag_registry.example.ron` (mirror) | JSON + RON | `.tag_registry.{json,ron}` | required | **Applied** — `TagRegistry` + `TagRegistryLoader`; registered in [`MaterialUnificationPlugin`](../../../src/systems/terrain/material_plugin.rs) with material + rules · **U5** |
| Material rules | `material_rules.example.ron` | RON | `.material_rules.ron` | required | **Applied** — `RuleSet` + `RuleSetLoader`; same registration site · **U5** |

**Loader / Asset:** All three terrain registry loaders register in [`MaterialUnificationPlugin::build`](../../../src/systems/terrain/material_plugin.rs) alongside `TerrainFamilyRegistryLoader`, `MobilityProfileRegistryLoader`, and `WorldProfileLoader`. Integration gate **A3** tracks matrix truth vs code ([`runbook/a3_steps_v1.md`](runbook/a3_steps_v1.md)).

**Rule:** extensions and formats **match** [`material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md) §2 — no drift without `ASK:`.

---

## Sub-questions

1. **Dual load** (accept JSON + RON during transition) — yes/no?
2. **Schema version** inside each file — `schema_version: u32` top-level?
3. **bevy_common_assets** or custom `AssetLoader` — 📎 after prototype sizing

---

## Designer linkage

- Hybrid saves: `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md`
- Terrain paired execution (registry loaders policy): [`../../guides/bevy_asset_terrain_runbook_v1.md`](../../guides/bevy_asset_terrain_runbook_v1.md) + `runbook/` under this folder
- Brush / tools: `prompts/designer_questions/terrain_world/terrain_tools_brushes_v1.md`
- Terrain **material / tag / rule** tables: [`terrain_world/material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md) · [`terrain_biome/material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md)
