# Art design inbound — project alignment `v1`

| Field | Value |
|:---|:---|
| **Source** | [`prompts/art_desgin_inbound.md`](../../prompts/art_desgin_inbound.md) |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` |
| **Status** | **SIGNED — alignment record** |
| **Rule** | Absorb **fit** ideas into existing signed docs; do **not** fork parallel sim/render authority |

**Principle (inbound):** Designer agent writes **structured specs** → tools produce assets → **validation** → engine RON/GLB. No `House_500.json`. No AI image as final albedo.

---

## 1. Fit matrix (inbound → repo)

| Inbound idea | Verdict | Authoritative doc / code |
|:---|:---:|:---|
| Module kit (walls/roofs/doors/windows), not finished buildings | **ABSORBED** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) · [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) |
| `StylePack` (Victorian, Industrial, Military, …) | **ABSORBED** | Same + PG-1 exec |
| `BuildingArchetype` + compact JSON spec | **ABSORBED** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) · `tools/mcp/schemas/asset_spec_v1.schema.json` |
| District **rules** (allowed buildings/roofs), not direct building gen | **ABSORBED** (this pass) | [`plan_settlement_hierarchy_exec_005_v1.md`](plan_settlement_hierarchy_exec_005_v1.md) · OG exec · `district_style_rules_v1.schema.json` |
| Organic growth: demand → queue, not player LMB every house | **ABSORBED** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) |
| Settlement hierarchy Town → District → Block | **ABSORBED** | [`plan_settlement_hierarchy_exec_005_v1.md`](plan_settlement_hierarchy_exec_005_v1.md) |
| Transport graph (not tile road booleans) | **ABSORBED** | [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) |
| GIS / Natural Earth references | **DEFERRED P8** | Roadmap Phase 8; Reference MCP metadata only until then |
| Validation (poly, pivot, grid, PBR, LOD, collision) | **ABSORBED** (designer + tools) | Module kit § Validation; [`plan_designer_mcp_art_toolchain_exec_001_v1.md`](plan_designer_mcp_art_toolchain_exec_001_v1.md) |
| `MaterialVariation` (5 textures × engine params) | **ABSORBED** (sim/render) | Procedural plan § Material variation |
| Tileable PBR 512/1024; landmarks 2048+ | **ABSORBED** | Module kit § Textures |
| Art Director MCP / Blender CLI / gltf-transform | **ABSORBED** | [`plan_designer_mcp_art_toolchain_exec_001_v1.md`](plan_designer_mcp_art_toolchain_exec_001_v1.md) · `tools/mcp/` |
| Real references (USGS, OSM, manuals) — no AI imagery | **ABSORBED** | Module kit § References; Reference MCP |
| Avoid “beautiful / cinematic / photoreal” prompts | **ABSORBED** | Module kit § Style guide |
| Foundation / utility / road prop modules | **PHASE 4b** | Module kit extension — after core 50 |
| Port / Railway **district** style bias | **TAGS** | `style_tags` + district rules — not separate StylePack IDs |
| Full Rust `mcp-daemon` crate in repo root | **DEFERRED** | Python `tools/mcp/` first; Rust validator optional ART-VAL-001 |
| Runtime infinite-world chunk streaming | **NOT ADOPTED** | Existing Stage 6 + WSS — do not duplicate inbound §4 |
| In-engine Material **graph** (Substance-style nodes) | **NOT ADOPTED** | Use Material Maker CLI + `MaterialVariation` at runtime |
| Growth emits MCP mesh jobs directly | **NOT ADOPTED** | Sim queues **construction plans**; art pipeline is designer/offline |

---

## 2. Gaps closed this alignment (2026-06-02)

| Gap | Fix |
|:---|:---|
| Inbound file not linked anywhere | This doc + index updates |
| District **style rules** RON shape undefined | `DistrictRecord.style_rules` in settlement exec; JSON schema |
| Validation constraints not in designer kit | Module kit v1.2 § Validation contract |
| `MaterialVariation` missing from sim types | Procedural architecture plan |
| Foundation/utility/road props only in inbound | Phase 4b charter in module kit |

---

## 3. Single content compiler (horizon)

```text
AssetSpec JSON (art-director)
  → Blender / Material Maker CLI (tools/mcp)
  → gltf-transform validate
  → assets/staging/
  → promote → assets/models/modules/ + RON sidecar
  → StylePack / BuildingArchetype (engine)
  → PG-2 assembly → RepresentationResult
```

**Growth sim never calls Blender.** It selects `ArchetypeId` + `StylePackId` from **district rules** and queues construction.

---

## 4. Witness / proof

| Lane | Witness |
|:---|:---|
| Art pipeline | `debug_runs/art_pipeline_validation_live.json` (when ART-VAL-001 lands) |
| Procedural sim | `construction_scaling_audit_001`, `construction_organic_growth_001` |
| Modules in engine | `assets/configs/buildings/_module_index.ron` |

---

## 5. Planner drain interaction

| Queue ID | Relationship |
|:---|:---|
| PLAN-SETTLEMENT-HIERARCHY-005 | `DistrictRecord.style_rules` |
| PLAN-ORGANIC-GROWTH-EXEC-001 | Filter proposals by district rules |
| PLAN-PROC-BUILD-EXEC-001 | StylePack + archetype loaders |
| PLAN-DESIGNER-MCP-ART-TOOLCHAIN-001 | Offline asset production |
| PLAN-CONSTRUCTION-SCALING-AUDIT-003 | Footprint grid before art attach |

**Deferred:** PLAN-HANABI-H-A2-PROD-CHARTER-001 · PLAN-WSS-POST-SPINE-001 · full MCP daemon split (Phase 4 of art exec).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Map art_desgin_inbound → signed construction/infra/art plans |
