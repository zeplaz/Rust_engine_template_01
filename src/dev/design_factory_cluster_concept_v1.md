# FactoryCluster archetype concept `v1` — DMCP-GRAM-ARCHETYPE-FACTORY-001

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-GRAM-ARCHETYPE-FACTORY-001** |
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 · G1 |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Grammar** | [`factory_cluster_v1.ron`](../assets/configs/buildings/grammars/factory_cluster_v1.ron) |
| **Family spec** | [`design_grammar_archetype_family_g1_v1.md`](design_grammar_archetype_family_g1_v1.md) §2 |
| **Kit** | [`batch_kit_production_002.manifest.json`](../tools/mcp/schemas/examples/batch_kit_production_002.manifest.json) (shared industrial_west slots) |
| **Verdict** | **PASS** |

```yaml
order_critique:
  request_summary: "Second archetype concept — FactoryCluster vs IndustrialWarehouse"
  rules_audit:
    deterministic_output: pass
    no_bpy_in_spec: pass
  proceed: yes
```

---

## 1. Identity (artist-facing)

| Field | Value |
|:---|:---|
| **APS label** | Factory Cluster |
| **One line** | Parallel production bays with sawtooth roof rhythm |
| **Usage** | `factory` — zoning industrial, not civic |
| **District** | `manufacturing_row` · style pack `style_industrial_west` |

**Ban-list:** never show raw `FactoryCluster` in primary APS chrome — use label from [`grammar_labels_v1.json`](../assets/configs/buildings/grammars/grammar_labels_v1.json).

---

## 2. Silhouette read @ iso (64px footprint preview)

| vs `IndustrialWarehouse` | FactoryCluster |
|:---|:---|
| Single long hall + yard option | **Twin parallel halls** — two roof ridges |
| Door rhythm: one loading face | **Multiple bay doors** along long edge |
| Roof: gable or shed default | **Sawtooth default** (`roof_industrial`) |
| Detail density ~0.35 | **0.45** — more vents, chimneys, skylights |

**Default massing:** `double_hall` (40% weight) — artist should see **two parallel volumes** before detail props.

---

## 3. Massing strategies (concept picks)

| Strategy | Footprint | Artist read |
|:---|:---|:---|
| `double_hall` | 1.35 W:D rect | Two parallel bays — **signature** |
| `long_hall` | 1.85 W:D rect | Single stretched factory |
| `yard_complex` | yard interior | Courtyard + annex — flat roof wing |
| `l_shape` | L-shape | Office/service bar |

**Seed demos (deterministic QA):**

| seed | Expected mode |
|:---:|:---|
| 12001 | `double_hall` |
| 12002 | `long_hall` |
| 12003 | `yard_complex` |

---

## 4. Module kit binding (kit002 shared)

FactoryCluster reuses **kit_production_002** slots — no new module ids in G1:

| Slot | Module | Visual role |
|:---|:---|:---|
| `wall_1u` | `wall_steel_1u` | Bay repetition |
| `door_wide` | `door_warehouse` | Loading doors |
| `window_industrial` | `win_industrial_3u` | Clerestory band |
| `roof_industrial` | `roof_industrial_shed_2u` | Sawtooth plane |
| `prop_clutter` | `stack_chimney_1u` | Roof stacks |

**G2 gap (future):** extra clutter modules (`vent_louver_1u`, `pipe_run_2u`) — not G1 blockers.

---

## 5. Material / variant posture

| Layer | FactoryCluster bias |
|:---|:---|
| Age | More `weathered` (50%) than warehouse |
| Damage | Higher `abandoned` tail (20%) |
| Tags | `factory`, `street_facing`, `vent`, `chimney`, `skylight` |
| Tile batch | Shares `tile_batch_warehouse_industrial_west_production_v1` until factory-specific batch chartered |

---

## 6. APS exposure (G1)

| Surface | Copy |
|:---|:---|
| Archetype dropdown | **Factory Cluster** under group `Industrial` |
| Grammar panel | “Generate from a building style” → picks massing + district |
| Why tooltip | [`design_aps_grammar_why_copy_v1.md`](design_aps_grammar_why_copy_v1.md) — factory row sentence |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-02 |

```text
DMCP-GRAM-ARCHETYPE-FACTORY-001 Q✓ — G1 second archetype concept locked
```
