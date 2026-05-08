# Fact vocabulary rulebook (v1)

**Status:** authoritative ontology draft for terrain **semantic tags** only.  
**Scope:** `TagRegistry` strings interned to `TagId` / `TagSet` on cells; material defs that **emit** fact tags.

---

## 1. Core principle

| Allowed | Forbidden |
|:---|:---|
| Statements about **what exists** physically or ecologically on/near the cell | Statements that **conclude** play for all agents, modes, or futures |
| Facts stable under unchanged worldgen output | Verdicts that belong in mobility, law, engineering, or weather systems |

**Examples of forbidden tag semantics**

- `traversable` / `non_traversable` — mode-dependent; **delete** from vocabulary.
- Anything meaning “blocked” / “allowed” for movement without naming **who** and **how**.

**Borderline tags (refactor)**

- `mineable` — often a **resource + technology + jurisdiction** conclusion. Prefer geological/ecological facts (e.g. `mineral_rich`, `outcrop_exposed`) and let extraction systems decide mineability.

---

## 2. Category taxonomy (Layer 1)

Tags MUST belong to exactly one **category** string in registry JSON (for tooling and designer mental model). Suggested categories:

| Category | Meaning |
|:---|:---|
| `topology` | Slope, relief, discontinuities (cliff, ridge, broken ground) |
| `surface` | Footprint interaction: texture, firmness, looseness, mud, rockiness |
| `hydrology` | Water state and soil moisture regime (not “nav”) |
| `ecology` | Vegetation structure / cover |
| `geology` | Substrate strength, stability, mineral signal |
| `moisture` | **Atmospheric / bucket class** (legacy: wet/dry) — prefer converging with hydrology where duplicate |
| `temperature` | Thermal class (legacy: hot/cold) — fact-like at coarse LOD |

Do **not** add a `nav` or `gameplay` category for cell tags. Navigation lives in **mobility rules** ([`mobility_profile_matrix_v1.md`](mobility_profile_matrix_v1.md)).

---

## 3. Initial fact tag set (v1)

**Topology**

- `steep`
- `cliff`
- `broken_ground`
- `ridge`

**Hydrology**

- `waterlogged`
- `flooded`
- `marsh`
- `shallow_water`
- `deep_water`

**Surface**

- `hard_surface`
- `loose_surface`
- `muddy`
- `rocky`

**Ecology**

- `dense_vegetation`
- `sparse_vegetation`
- `forest_canopy`

**Geology**

- `hard_bedrock`
- `unstable_subsurface`
- `mineral_rich`

**Migration from current example registry**

- Map legacy `wet` → consider `waterlogged` or keep `wet` as moisture **fact** only if defined as “elevated vadose / saturation signal”, not “bad for trucks”.
- Map `rock`, `sand`, `soil` — align with **material identity** first; facts may duplicate lightly until material defs own single source of truth.
- Remove `traversable`, `non_traversable`.

---

## 4. Materials emit facts

**Rule:** `MaterialDef` (registry) lists **fact tags** only. Example:

```json
{
  "material_id": "wet_clay",
  "tags": ["soft_surface", "waterlogged", "unstable", "muddy"]
}
```

RON **rules** match on fact combinations to pick presentation/material id — not to assign “passable”.

---

## 5. Authoring checklist

- [ ] Tag name reads like a **noun phrase** or **measurable condition**, not an order (“no_go”).
- [ ] Designer can explain the fact **without naming a unit type**.
- [ ] Two mobility profiles could reasonably disagree on outcome using the **same** tag set → good fact.
- [ ] If only one profile will ever care, still a fact if physical; else fold into **derived metric** ([`derived_metric_pipeline_v1.md`](derived_metric_pipeline_v1.md)).

---

## 6. Contracts

This rulebook is the **modding contract** for:

- `tag_registry` JSON
- material tag columns
- worldgen passes that **add/remove** tags (pass 2+)
- preview “fact overlay” modes (see mobility doc + preview matrix)

Version bumps: bump `schema_version` in registry when categories or required renames batch.
