# Grammar archetype family G1 — content spec `v1` (GRAM-CONTENT-001)

| Field | Value |
|:---|:---|
| **ID** | **GRAM-CONTENT-001** |
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Date** | 2026-06-17 |
| **Owner** | `@designer-mcp` |
| **Authority** | [`arch_build_grammar_001_schema_v1.md`](arch_build_grammar_001_schema_v1.md) · [`building_grammar_v1.schema.json`](../../tools/mcp/schemas/building_grammar_v1.schema.json) |
| **Baseline** | [`industrial_warehouse_v1.ron`](../../assets/configs/buildings/grammars/industrial_warehouse_v1.ron) |
| **Verdict** | **PASS** — unblocks GRAM-CONTENT-002 |
| **No bpy** | Spec + RON-shaped data only |

```text
GRAM-CONTENT-001 Q✓
Target: archetype_count >= 3 → tier G1
Signed ids: IndustrialWarehouse (exists) + FactoryCluster + RailEdge
```

---

## 0. G1 bar

| Metric | Today | After 002–004 |
|:---|:---:|:---:|
| `*.ron` grammars | 1 | **3** |
| `list_archetype_ids()` | 1 | **≥3** |
| `grammar_set_tier()` | G0 | **G1** |
| APS archetype dropdown | 1 value + kit hint | **≥3** grouped choices; kit hint downgraded |

**Seed contract:** `generate(archetype_id, district_style, seed)` deterministic for all three — same inputs → same footprint_mode + rule_chain (existing evaluator).

---

## 1. Family map

```text
G1 industrial lineage (shared module kit — industrial_west slots)
├── IndustrialWarehouse  (warehouse)   — industrial_west     [EXISTS]
├── FactoryCluster       (factory)    — manufacturing_row   [NEW]
└── RailEdge             (warehouse)  — rail_yard_corridor  [NEW]
```

**Lineage grouping (APS G1 dropdown):** label group `Industrial` with three human labels from `grammar_labels_v1.json`.

---

## 2. Archetype A — `FactoryCluster` (NEW)

### Identity

| Field | Value |
|:---|:---|
| `grammar_id` | `factory_cluster_v1` |
| `archetype.id` | `FactoryCluster` |
| `archetype.usage` | `factory` |
| **Artist label** | Factory Cluster |
| **One-line** | Parallel production bays with sawtooth roof rhythm |

### Footprint bounds

| Field | min | max |
|:---|:---:|:---:|
| width | 5 | 14 |
| depth | 4 | 10 |
| floors | 1 | 2 |

### Massing strategies (≥2 — uses schema enum ids)

| id | weight | footprint_mode | width_depth_ratio | Role |
|:---|:---:|:---|:---:|:---|
| `double_hall` | **40** | `rect` | 1.35 | Twin parallel halls — default factory read |
| `long_hall` | **30** | `rect` | 1.85 | Single long bay |
| `yard_complex` | **20** | `yard_interior` | — | Courtyard + annex |
| `l_shape` | **10** | `l_shape` | — | Office/service wing |

**β bias (G2+):** high `βmod`, `βroof` — favors `double_hall` + `roof_industrial`.

### Roof / facade / detail / age

Mirror warehouse structure; deltas:

| Layer | Slot / tag |
|:---|:---|
| roof default | `roof_industrial` (sawtooth) |
| roof by_massing | `double_hall` → `roof_industrial`; `yard_complex` → `roof_flat` |
| facade tags | `exterior`, `industrial`, `factory`, `street_facing` |
| detail density | **0.45** (more roof clutter) |
| detail tags | `vent`, `pipe`, `chimney`, `skylight` |
| age weights | new 30 / weathered 50 / abandoned 20 |

### District style (≥1)

| id | style_pack_id | zoning | style_tags |
|:---|:---|:---|:---|
| `manufacturing_row` | `style_industrial_west` | `industrial` | `steel`, `sawtooth`, `brick`, `factory_row` |

**Material profiles:** same slot keys as `industrial_west`; reuse `steel_panel_01`, `roof_metal_01`, `glass_panel_01` (no new material ids in G1).

### Deterministic test seeds

| seed | district | Expected massing (weight pick) |
|:---:|:---|:---|
| 7 | `manufacturing_row` | `double_hall` |
| 42 | `manufacturing_row` | `long_hall` |
| 99 | `manufacturing_row` | `yard_complex` |

### Module kit gaps (document — do not block G1)

| Gap | Severity | Note |
|:---|:---:|:---|
| `chimney_stack_1u` | P2 | Detail prop — use `prop_clutter` until module exists |
| `skylight_band` | P2 | Roof detail — tag only in G1 |
| Sawtooth roof mesh variant | P1 | `roof_industrial` slot OK for G1 grammar |

---

## 3. Archetype B — `RailEdge` (NEW)

### Identity

| Field | Value |
|:---|:---|
| `grammar_id` | `rail_edge_v1` |
| `archetype.id` | `RailEdge` |
| `archetype.usage` | `warehouse` |
| **Artist label** | Rail Edge Warehouse |
| **One-line** | L-shaped hall along rail spur with loading wing + utility yard |
| **DNA preset link** | [`arch_dna_logistics_rail_warehouse_v0.json`](../../tools/mcp/schemas/examples/arch_dna_logistics_rail_warehouse_v0.json) |

### Footprint bounds

| Field | min | max |
|:---|:---:|:---:|
| width | 5 | 12 |
| depth | 4 | 9 |
| floors | 1 | 2 |

### Massing strategies (≥2)

| id | weight | footprint_mode | width_depth_ratio | Role |
|:---|:---:|:---|:---:|:---|
| `l_shape` | **45** | `l_shape` | — | **Primary** — rail-edge L (pilot family) |
| `yard_complex` | **30** | `yard_interior` | — | Utility + expansion yard |
| `long_hall` | **20** | `rect` | 2.1 | Straight hall along track |
| `double_hall` | **5** | `rect` | 1.4 | Rare symmetric variant |

**β bias:** high `βyard`, `βsvc`, `βirr` per logistics_rail_warehouse preset — favors `l_shape` + `yard_complex`.

### Roof / facade / detail / age

| Layer | Slot / tag |
|:---|:---|
| roof default | `roof_industrial` |
| roof by_massing | `l_shape` → `roof_industrial`; `yard_complex` → `roof_flat` |
| facade door | `door_wide` (loading) |
| detail density | **0.30** |
| detail tags | `vent`, `pipe`, `platform`, `rail_buffer` |
| age weights | new 35 / weathered 45 / abandoned 20 |

### District style (≥1)

| id | style_pack_id | zoning | style_tags |
|:---|:---|:---|:---|
| `rail_yard_corridor` | `style_industrial_west` | `industrial` | `steel`, `rail`, `logistics`, `loading_dock` |

Optional second district (P2 — not required for G1):

| id | Notes |
|:---|:---|
| `logistics_north` | Clone of `rail_yard_corridor` with `rail` tag emphasis — defer to G2 |

### Deterministic test seeds

| seed | district | Expected massing |
|:---:|:---|:---|
| 11 | `rail_yard_corridor` | `l_shape` |
| 43 | `rail_yard_corridor` | `l_shape` (parity with PG-QUALITY pilot) |
| 88 | `rail_yard_corridor` | `yard_complex` |

### Module kit gaps

| Gap | Severity | Note |
|:---|:---:|:---|
| Dedicated `loading_dock` module | P1 | Use `door_wide` + yard void for G1 |
| Rail spur overlay (site) | P2 | SITE-v0 — not grammar file |
| Pilot matrix `logistics_rail_warehouse_l_6x5` | — | **Exists** — RailEdge generate should prefer `l_shape` footprint_mode |

---

## 4. RON deliverables (GRAM-CONTENT-002)

| File | Archetype |
|:---|:---|
| `assets/configs/buildings/grammars/factory_cluster_v1.ron` | FactoryCluster |
| `assets/configs/buildings/grammars/rail_edge_v1.ron` | RailEdge |

**JSON mirrors (required):**

| File |
|:---|
| `tools/mcp/schemas/examples/building_grammar_factory_cluster_v1.json` |
| `tools/mcp/schemas/examples/building_grammar_rail_edge_v1.json` |

**Validate:** `python -m rust_engine_mcp.cli validate-report arch_build_grammar <path> --compress 3` on each JSON mirror.

---

## 5. Human labels (GRAM-CONTENT-003)

Add to `grammar_labels_v1.json`:

```json
"archetypes": {
  "IndustrialWarehouse": { "label": "Industrial Warehouse", "usage": "warehouse" },
  "FactoryCluster": { "label": "Factory Cluster", "usage": "factory" },
  "RailEdge": { "label": "Rail Edge Warehouse", "usage": "warehouse" }
},
"district_styles": {
  "industrial_west": "Industrial West",
  "manufacturing_row": "Manufacturing Row",
  "rail_yard_corridor": "Rail Yard Corridor"
}
```

**APS rule:** combos show **label** only — never raw `FactoryCluster` in primary UI ([`aps_design_system_v1.md`](aps_design_system_v1.md) ban-list).

---

## 6. Tier witness (GRAM-CONTENT-004)

After 002+003+`APS-GRAM-TIER-001`:

**Path:** `debug_runs/grammar_set_tier_g1.json`

```json
{
  "tier": "G1",
  "archetype_count": 3,
  "archetype_ids": ["IndustrialWarehouse", "FactoryCluster", "RailEdge"],
  "kit_hint_downgraded": true,
  "building_set_coverage": "pass"
}
```

**Forbidden:** tier G1 with `archetype_count < 3`; dropdown still showing only `IndustrialWarehouse`.

---

## 7. Chain summary

```text
GRAM-CONTENT-001  ✓ this spec
    ↓
GRAM-CONTENT-002  @coder-mcp — RON + JSON mirrors
    ↓
GRAM-CONTENT-003  @coder-mcp — grammar_labels_v1.json
    ↓
GRAM-CONTENT-004  @coder-mcp — grammar_set_tier() → G1 witness
    ↓
APS-GRAM-TIER-002-REFRESH — kit hint off; 3+ dropdown values
```

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-17 |

**@coder-mcp:** implement §4 RON verbatim from §2–§3 tables; do not invent fourth archetype for G1.
