# PLAN-STYLE-PACK-RON-001 — StylePack RON schema `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-STYLE-PACK-RON-001** |
| **Fleet** | **MCP-PLN-SP-001** · [`mcp_fleet_wave2_orders_v1.md`](mcp_fleet_wave2_orders_v1.md) |
| **Owner** | `@planner-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **SIGNED** |

---

## Summary

Style packs are **data-only** RON files mapping `StylePackId` → canonical `module_id` slots for PG-2 assembly. They do **not** embed meshes. Meshes resolve via `ProceduralModuleRegistry::resolve_module_id()` (lod0/production only; smoke excluded).

**Designer-mcp** authors pack files (**MCP-D0-SP-001**). **Coder** loads them in PG-1 (**MCP-PG-1-001**).

---

## File layout

| Path | Rule |
|:---|:---|
| `assets/configs/buildings/style_packs/style_<id>.ron` | One pack per file |
| `assets/configs/buildings/style_packs/_manifest.ron` | Optional index of pack ids (T2) |

**Wave 2 packs (7):** `style_victorian`, `style_modern`, `style_industrial_west`, `style_industrial_soviet`, `style_military`, `style_rural`, `style_colonial`.

**District tags (not separate files):** `port_district`, `railway_district` — bias via `style_tags` + [`district_style_rules_v1.schema.json`](../../tools/mcp/schemas/district_style_rules_v1.schema.json).

---

## RON shape (authoritative)

```ron
(
    schema_version: 1,
    style_pack_id: "style_victorian",
    label: "Victorian",
    usage_bias: ["residential", "commercial"],
    style_tags: ["brick", "residential", "pitched_roof"],
    // Slot map: grammar token + category → module_id (must exist in index as lod0+)
    slots: (
        wall_1u: "wall_brick_1u",
        wall_2u: "wall_brick_2u",
        door_default: "door_residential",
        window_1u: "win_single_1u",
        window_2u: "win_double_1u",
        roof_default: "roof_pitched_gable",
        roof_flat: "roof_flat",
        corner_outer: "corner_L",
        prop_clutter: "prop_vent",
    ),
    // Optional overrides when module not yet promoted — PG-2 hides slot, never smoke
    fallback_policy: "hide_slot",
)
```

---

## Field contract

| Field | Type | Required | Rule |
|:---|:---|:---:|:---|
| `schema_version` | u32 | yes | `1` |
| `style_pack_id` | string | yes | Matches filename `style_<suffix>.ron` |
| `label` | string | yes | HUD / debug |
| `usage_bias` | string[] | yes | Archetype filter hints |
| `style_tags` | string[] | yes | Subset match against module `style_tags` |
| `slots` | map | yes | Keys from **Slot keys** table; values = canonical `module_id` |
| `fallback_policy` | enum | yes | `hide_slot` \| `primitive_footprint` — **never** `smoke` |

### Slot keys (v1)

| Key | Category | Notes |
|:---|:---|:---|
| `wall_1u` | wall | Default facade bay |
| `wall_2u` | wall | Wide bay |
| `door_default` | door | Floor 0 |
| `door_wide` | door | Optional warehouse/garage |
| `window_1u` | window | |
| `window_2u` | window | |
| `window_industrial` | window | strip / 3u |
| `roof_default` | roof | |
| `roof_flat` | roof | |
| `roof_industrial` | roof | shed / sawtooth |
| `corner_outer` | corner_prop | L corner |
| `corner_inner` | corner_prop | T corner |
| `prop_clutter` | prop | vent, light, etc. |

**Validation:** every `slots.*` value must appear in `_module_index.ron` with `development_tier` ∈ {`lod0`, `production`} after resolve, or designer marks slot optional with `null` until batch lands.

---

## Loader contract (MCP-PG-1-001)

| API | Behavior |
|:---|:---|
| `load_style_packs(dir)` | Parse all `style_*.ron`; fail on duplicate `style_pack_id` |
| `StylePackRegistry::get(id)` | Returns pack or None |
| `StylePack::resolve_slot(key)` | Returns `module_id` string |
| `StylePack::module_ids()` | Iterator for witness manifest |

**Depends on:** `ProceduralModuleRegistry` initialized first (module index).

---

## JSON schema draft

**Path:** [`tools/mcp/schemas/drafts/style_pack_ron_v1.schema.json`](../../tools/mcp/schemas/drafts/style_pack_ron_v1.schema.json)

Used for designer-mcp `validate_report` stub (T2) and MCP-D0-SP-002 manifest — **not** engine runtime (engine uses RON serde).

---

## Witness (MCP-D0-SP-002)

**File:** `debug_runs/art_pipeline/style_packs_manifest_live.json`

```json
{
  "pack_count": 7,
  "pack_ids": ["style_victorian", "..."],
  "slots_per_pack": { "style_victorian": 12 },
  "unresolved_slots": [],
  "lod0_module_refs": 10,
  "green": true
}
```

---

## Gate checklist (PLN-SP-001)

| Gate | Criterion | Status |
|:---|:---|:---:|
| G0 | Schema doc SIGNED | **pass** |
| G1 | JSON schema draft in `schemas/drafts/` | **pass** |
| G2 | 7 pack RON files | **pending** — MCP-D0-SP-001 |
| G3 | Manifest witness green | **pending** — MCP-D0-SP-002 |
| G4 | PG-1 loader reads packs | **pending** — MCP-PG-1-001 |
| G5 | PG-2 distinct facades per pack | **pending** — MCP-PG-2-001 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | MCP-PLN-SP-001 — schema + loader contract |
