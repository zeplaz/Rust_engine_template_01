# Design — Assembly → tile coupling (Object-Plus fidelity) `v1`

| Field | Value |
|:---|:---|
| **ID** | **DESIGN-ASSEMBLY-TILE-001** |
| **Owner** | `@planner` + `@planner-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **SIGNED** |
| **Triggers** | User: art lacks Object-Plus polish; tiles must come from **assembled** buildings; procedural rules must track real-world variants |

---

## Problem (current gap)

| Today | Gap |
|:---|:---|
| MCP ships **lod0 module GLBs** (silhouette-first) | Not Object-Plus-level polish, materials, or entourage |
| PG-2 plans **slot assembly** in-engine | No **authoritative 3D assembly scene** for art bake |
| Tile lane (`tile_batch_v1`) specs **floor/state variants** | Bakes can proceed **without** a resolved building instance |
| Procedural W/D/C grammar | Not yet **locked** to reference archetypes before tile export |

**Risk:** baking tiles from raw `module_wall` cubes produces generic iso sprites that do not match what players see in tactical 3D — and cannot represent real-world building variants.

---

## Authoritative pipeline (coupled)

```text
(1) MODULE PARTS     MCP module kit — lod0 silhouette → production polish (PBR, refs)
        ↓
(2) STYLE + GRAMMAR  StylePack slots + W/D/C footprint + archetype (real-world bias)
        ↓
(3) 3D ASSEMBLY      PG-2: instances modules into one building transform (Blender or engine staging)
        ↓
(4) ASSEMBLY SNAP     assembly_id, style_pack_id, footprint hash, module_id list, seed
        ↓
(5) TILE BAKE        tile_batch_run: ortho bake **assembled scene** (not per-module)
        ↓
(6) ATLAS + INDEX    _tile_atlas_index.ron — keys reference assembly_id + variant axes
        ↓
(7) RUNTIME          Map/tactical: sim state → variant → atlas UV (RepresentationResult)
```

**Rule:** Step **(5) is forbidden** until **(3–4)** pass G4 sign-off for that archetype × style pack.

---

## Coupling contracts

### A. Module tier (polish ladder)

| Tier | Role | Object-Plus bar |
|:---|:---|:---|
| `smoke` | MCP spine only | **Never** bake, never StylePack |
| `lod0` | PG-2 + assembly staging | Silhouette + readable grammar — **not** final tile art |
| `production` | Tile bake source | PBR shipped, reference citation, ≤2k tris/slot, real-world proportion |

Tile batches for **shipping art** must declare `source_tier: production` or `assembly_snapshot_ref` pointing at a production-tier assembly.

### B. Assembly snapshot (new artifact — between PG-2 and tile)

**Path (proposed):** `assets/staging/assemblies/<assembly_id>.json`

| Field | Purpose |
|:---|:---|
| `assembly_id` | `"{archetype}_{style_pack}_{footprint_hash}_s{seed}"` |
| `style_pack_id` | e.g. `style_victorian` |
| `footprint` | width, depth, W/D/C token grid |
| `module_placements` | `{ module_id, transform, slot_key }[]` |
| `reference_tags` | real-world survey / OSM / manual ref ids |
| `procedural_rules_version` | grammar + StylePack schema version |

**Blender:** import resolved GLBs → apply transforms → single collection → `tile_ortho_bake` camera rig (link lights/camera from `utils/Light_keysshotsetup.blend` — see [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md)).

**Engine:** PG-2 extract produces the same manifest for witness (`procedural_assembly_live.json`).

### C. Tile batch (amend `tile_batch_v1`)

Add **required** when baking building tiles (not terrain filler):

```json
"assembly_ref": {
  "assembly_id": "rowhouse_victorian_4x3_a1b2_s42",
  "style_pack_id": "style_victorian",
  "source_tier": "production"
}
```

Terrain/floor tiles (factory floor, dirt) may omit `assembly_ref`; **building iso tiles may not**.

### D. Procedural rules ↔ real world

| Coupling | Care rule |
|:---|:---|
| Archetype | `usage_bias` must match footprint grammar (doors on floor 0, industrial sawtooth only on warehouse archetypes) |
| StylePack | Slot ids map to **canonical** module_ids — no ad-hoc mesh names |
| Reference | Designer cites ref id in assembly snapshot; validator rejects bake without `reference_tags` on production tier |
| Variant axes | `damage` / `power` / `fill` reflect **sim state** on the **assembled** building, not the module catalog |

---

## Wave 2 impact (orchestrator)

| Lane | Change |
|:---|:---|
| **ART** | Continue lod0 for PG-2 wiring; schedule **production** pass before tile bake |
| **DATA** | StylePack RON — slots only; no tile PNGs |
| **ENGINE** | PG-2 must emit `assembly_snapshot` manifest before tile lane G3 |
| **TILE** | MCP-T0-002 schema adds `assembly_ref`; MCP-T2 bake uses `tile_ortho_bake` on assembly collection |
| **designer-mcp** | G4 sign-off includes “assembly matches reference silhouette” rubric |

**Frozen until assembly contract ships:** building iso tile bake (terrain/floor tiles may still schema-validate).

---

## Phases

| Phase | Deliverable | Owner |
|:---|:---|:---|
| **AT-1** | `assembly_snapshot` JSON schema + witness keys | planner-mcp |
| **AT-2** | PG-2 exports assembly manifest from footprint + StylePack | coder |
| **AT-3** | Blender `tile_ortho_bake` from assembly collection | coder-mcp — **MCP-AUTO-004** |
| **AT-4** | `tile_batch_v1` + `_tile_atlas_index.ron` keyed by assembly_id | coder-mcp — **MCP-AUTO-007/008** |
| **Exec** | Full automation plan | [`plan_tile_pipeline_automation_exec_v1.md`](plan_tile_pipeline_automation_exec_v1.md) |
| **AT-5** | Production module tier + Object-Plus validation gate | designer-mcp + tier validator |

---

## Acceptance (program level)

1. Same seed + style pack + footprint → **identical** assembly snapshot hash.
2. Tactical 3D mesh set == module list in assembly snapshot (lod0+ only).
3. Iso tile atlas pixel diff == 0 for re-bake of same assembly + variant axes.
4. Victorian vs industrial **same footprint** → **different** tile atlas cells (StylePack coupling proven).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Initial coupling — modules → assembly → tiles |
