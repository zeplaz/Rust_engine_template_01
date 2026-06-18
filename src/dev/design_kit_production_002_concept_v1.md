# kit_production_002 concept charter `v1` — DMCP-MODULE-KIT002-001

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-MODULE-KIT002-001** |
| **Batch** | `kit_production_002` |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Plan** | [`mcp_kit_production_002_unfreeze_v1.md`](../docs/archive/2026-06-src-dev/plans/mcp_kit_production_002_unfreeze_v1.md) |
| **G1 spec** | [`design_kit_production_002_g1_v1.md`](design_kit_production_002_g1_v1.md) |
| **Manifest** | [`batch_kit_production_002.manifest.json`](../tools/mcp/schemas/examples/batch_kit_production_002.manifest.json) |
| **Verdict** | **PASS WITH NOTES** — concept locked · G4 keyframes open |

```yaml
order_critique:
  request_summary: "Artist concept for warehouse industrial west kit002"
  rules_audit:
    tier_production: pass
    no_greybox_ship: pass
    deterministic_output: pass
  proceed: yes_with_notes
  blocker: "MCP-P2-KIT002-G4 manual keyframe stills"
```

---

## 1. Visual north star

**Lineage:** second **production** building kit after rowhouse victorian — **warehouse industrial west**.

| Pillar | Target |
|:---|:---|
| Silhouette | Long low mass + sawtooth or gable roof legible @ 64px tile |
| Rhythm | Repeated 1u steel bays · wide loading door · 3u window band |
| Weathering | Production PBR — not greybox flat |
| Grammar pairing | `industrial_warehouse_v1` **and** `factory_cluster_v1` share this kit |

**Feeds:** [`DES-STYLE-INDUSTRIAL-WEST-001`](plan_designer_work_202606_v1.md) style bible (parallel @designer).

---

## 2. Module roster (concept roles)

| Priority | module_id | Reads as… | Footprint job |
|:---:|:---|:---|:---|
| P0 | `wall_steel_1u` | Corrugated bay | Vertical repeat — defines width rhythm |
| P0 | `door_warehouse` | Roll-up / loading | Ground anchor — scale reference |
| P0 | `win_industrial_3u` | Clerestory band | Mid-wall light — breaks monotony |
| P1 | `corner_L` | Corner column | Turns footprint — reuse kit001 |
| P1 | `roof_industrial_shed_2u` | Sawtooth plane | Roof signature — **G2 bpy done** |
| P2 | `stack_chimney_1u` | Roof clutter | FactoryCluster detail density |

**Count:** 6 modules · all `development_tier: production` · `pbr_status: shipped`.

---

## 3. Manifest sketch (authority)

```text
kit_production_002
├── style_pack_id: style_industrial_west
├── grammar_ids: [industrial_warehouse_v1, factory_cluster_v1]
├── paired_tile_batch: tile_batch_warehouse_industrial_west_production_v1
├── paired_variant_set: variant_set_warehouse_industrial_west_production_v1
└── modules[6]: wall · door · window · corner · roof · stack
```

Absorbed batch: `kit_industrial_west_production_001` — **do not** fork duplicate module ids.

---

## 4. Tile / variant concept

| Axis | Warehouse kit002 states |
|:---|:---|
| Lighting | day / night_on |
| Damage | clean / weathered / damaged |
| Power | off / on (night windows) |
| Fill | full / half / empty yard |

**Variant graph:** `variant_graph_warehouse_industrial_west_v1` (ARCH-002).

**G4 minimum:** operator stills on **damage + night** cells before `proceed_ship: yes` ([`kit_production_002_g4_live.json`](../debug_runs/art_pipeline/kit_production_002_g4_live.json) currently **FAIL**).

---

## 5. FactoryCluster differentiation (same kit)

| Grammar | Massing | Kit emphasis |
|:---|:---|:---|
| `IndustrialWarehouse` | `long_hall`, `yard` | Single hall + yard door |
| `FactoryCluster` | `double_hall` | Twin bays + more `stack_chimney_1u` |

Same GLBs — different **placement grammar** and detail density.

---

## 6. Gate status

| Gate | Status | Witness |
|:---|:---|:---|
| G0 | ✓ | `warehouse_production_g0_rules.yaml` |
| G1 | ✓ | `design_kit_production_002_g1_v1.md` |
| G2 | ✓ | roof bpy + promotes |
| G3 | ✓ | `kit_production_002_g3_live.json` |
| G4 | **OPEN** | manual keyframes — `proceed_ship: no` |
| G5 | blocked | after G4 |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS WITH NOTES** | 2026-06-02 |

```text
DMCP-MODULE-KIT002-001 Q✓ — concept + manifest sketch · G4 remains operator lane
```
