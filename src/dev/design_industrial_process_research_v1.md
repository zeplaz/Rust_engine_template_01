# Industrial process facility research `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-INDUSTRIAL-RESEARCH-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E1-A |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`industrial_supply_chains.json`](../../assets/configs/industrial_supply_chains.json) · per-step catalog under `assets/configs/buildings/` |
| **Plan** | [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) |
| **Verdict** | **PASS** |

```text
DES-INDUSTRIAL-RESEARCH-001 Q✓
Layer 3 (catalog + chains) is authority — this doc expresses Layer 2 site + Layer 1 visual cues only
```

---

## 0. Scope

Structured research brief for **three chain families** on disk today:

| `chain_id` | Steps | Designer focus |
|:---|:---:|:---|
| `concrete_portland` | 3 discrete + 1 legacy monolith | **P0 pilot** — DMCP-PILOT-CONCRETE-SITE-001 |
| `aluminum_primary` | 4 | Power step-up 22→200; buffer yards |
| *(utility)* | `utility_role` buildings | Not a supply chain — yard_complex grammar |

**Rejected:** inventing `power_consumption` in grammar · collapsing kiln+mixer into one APS archetype without catalog_id · footprints that contradict catalog `building_size_*`.

**MW note:** `electrical_from_power_units` maps designer units ÷ 100 → grid load ([`supply_chain.rs`](../economy/supply_chain.rs)). Research cites **designer units** from chain JSON.

---

## 1. Concrete Portland (`concrete_portland`)

**Display:** Concrete (Portland) · **concrete_type:** Portland

### 1.1 Chain layout (typical site spacing)

```text
[aggregate_mine] ──truck──► [cement_kiln] ──belt/pipe──► [concrete_mixer]
     light                      medium                      light
   open pit                  stack + kiln               batch silos
```

**Adjacency rules (designer):**

| From | To | Must | Rationale |
|:---|:---|:---:|:---|
| `aggregate_mine` | `cement_kiln` | within 2–4 tiles | limestone/gravel haul |
| `cement_kiln` | `concrete_mixer` | adjacent or 1-tile buffer | hot cement / clinker |
| any step | `grid_substation` | utility yard ≤ 3 tiles from kiln/mixer | 72+28 MW aggregate load |
| `concrete_mixer` | road/rail | loading zone on public edge | outbound concrete trucks |

### 1.2 Step briefs

#### `aggregate_mine` · `concrete_aggregate_mine`

| Field | Value |
|:---|:---|
| **Role** | `aggregate_mine` |
| **Catalog footprint** | 3×3 (9 cells) |
| **Power** | **18** units → **light** tier |
| **Produces** | Gravel, Limestone |
| **Consumes** | Labour, Diesel |
| **Typical site** | 8×6 minimum — primary pit + buffer + diesel service |
| **Zone requirements** | primary ≥25% site · utility 10–15% (crushers) · loading optional · buffer ≥30% |
| **Visual cues** | open cut read, conveyor to stockpile, low stack, haul road edge |
| **Grammar hint** | `FactoryCluster` or quarry stub modules · **not** warehouse L-shape |
| **Archetype pilot** | new `concrete_aggregate_mine_site_v0` (designer-mcp) |

#### `cement_kiln` · `concrete_cement_kiln`

| Field | Value |
|:---|:---|
| **Role** | `cement_kiln` |
| **Catalog footprint** | 4×3 (12 cells) |
| **Power** | **72** units → **medium** tier |
| **Produces** | Cement |
| **Consumes** | Limestone, Coal, Labour, Electricity |
| **Typical site** | 10×8 — kiln hall + coal yard + stack |
| **Zone requirements** | primary ≥15% · **utility ≥20%** (coal + stack setback) · loading 5–10% · buffer ≥25% |
| **Must adjacent** | utility yard on downwind side (design convention) |
| **Visual cues** | preheater tower, kiln tube, coal pile, vent stack module |
| **Grammar hint** | `FactoryCluster` · pipe_rack + stack modules |
| **Archetype pilot** | `concrete_cement_kiln_site_v0` |

#### `concrete_mixer` · `concrete_mixer_plant`

| Field | Value |
|:---|:---|
| **Role** | `concrete_mixer` |
| **Catalog footprint** | 3×2 (6 cells) |
| **Power** | **28** units → **light** tier |
| **Produces** | Concrete |
| **Consumes** | Cement, Water, Gravel, Chemicals, Labour, Electricity |
| **Typical site** | 8×6 — silos + batch plant + truck load |
| **Zone requirements** | primary ≥12% · **loading ≥10%** (truck queue) · utility 10% (silos) · parking 5% |
| **Must adjacent** | loading on road-facing edge; cement feed from kiln direction |
| **Visual cues** | elevated silos, batch canopy, truck lane |
| **Grammar hint** | `IndustrialWarehouse` loading wing OR compact `FactoryCluster` |
| **Archetype pilot** | `concrete_mixer_plant_site_v0` |

#### Legacy · `concrete_basic_production_plant` (integrated)

| Field | Value |
|:---|:---|
| **Note** | Prefer discrete steps for new layouts |
| **Power** | 50 → **medium** |
| **Footprint** | treat as kiln+mixer monolith 5×4 design target |
| **APS** | show deprecation banner if selected: `○ legacy monolith — split kiln and mixer for new sites` |

### 1.3 Concrete chain site template (DMCP handoff)

| Step | Site grid W×D | primary% | loading% | utility% | rail |
|:---|:---:|:---:|:---:|:---:|:---:|
| aggregate_mine | 8×6 | 25% | 0% | 12% | optional spur |
| cement_kiln | 10×8 | 15% | 8% | 22% | coal rail optional |
| concrete_mixer | 8×6 | 12% | 12% | 10% | — |

---

## 2. Aluminum primary (`aluminum_primary`)

**Display:** Aluminum primary (Hall–Héroult chain)

### 2.1 Chain layout

```text
[bauxite_mine] ──► [alumina_refinery] ──► [aluminum_smelter] ──► [aluminum_fabrication]
    light              medium                  HEAVY                  medium
```

**Power asymmetry:** 22 → 85 → **200** → 48 — smelter dominates grid planning.

### 2.2 Step briefs

| Role | catalog_id | W×D | Power | Tier | Site emphasis |
|:---|:---|:---:|:---:|:---|:---|
| `bauxite_mine` | `aluminum_bauxite_mine` | 4×4 | 22 | light | open pit + buffer 35% |
| `alumina_refinery` | `aluminum_alumina_refinery` | 5×4 | 85 | medium | utility 25% (red mud, tanks) |
| `aluminum_smelter` | `aluminum_smelter1` | 4×3 | **200** | **heavy** | **utility ≥30%** cooling + potline yard |
| `aluminum_fabrication` | `aluminum_fabrication_plant` | 3×3 | 48 | medium | loading 15% coil/finish |

**Adjacency:**

| From | To | Rule |
|:---|:---|:---|
| refinery | smelter | alumina feed — prefer ≤2 tiles |
| smelter | substation | **required** within 4 tiles — heavy tier |
| smelter | fabrication | optional buffer yard 2–3 tiles |
| mine | refinery | truck haul — not co-located |

**Visual cues:** refinery = tank farm + pipe maze; smelter = potline roof + cooling towers; fab = crane bay + loading dock.

**Grammar hints:** smelter → `FactoryCluster` + dedicated cooling modules; mine → quarry read; refinery → `fuel_depot_tank_farm` pilot lineage.

**Deferred detail:** DES-INDUSTRIAL-RESEARCH-002 extends §2 with full site JSON templates.

---

## 3. Utility yards (`utility_role` — not supply chains)

| catalog_id | utility_role | W×D | Power in | Power out | Tier |
|:---|:---|:---:|:---:|:---:|:---|
| `grid_substation` | `substation` | 4×3 | 8 | — (transfer) | **grid** |
| `grid_distribution_transformer` | `transformer` | 2×2 | 4 | — | **grid** |
| `utilities_coal_plant` | `power_plant` | 6×5 | 12 | **650** gen | **grid** |

**Site pattern:** utility ring around primary equipment — see [`power_substation_yard_site_v0.json`](../../assets/configs/buildings/pilots/power_substation_yard_site_v0.json).

| Zone | Substations | Coal plant |
|:---|:---:|:---:|
| primary | transformer pads | turbine hall |
| utility | **≥60%** yard | coal pile + cooling |
| service | control shack | water intake |
| rail | — | coal spur **required** |

**Grammar hint:** `power_substation_yard_v0` ARCH-DNA pilot · coal plant = separate `yard_complex` archetype (DES-INDUSTRIAL-RESEARCH-003).

---

## 4. Cross-chain comparison

| Metric | Concrete (3-step) | Aluminum (4-step) |
|:---|:---|:---|
| Peak step power | 72 (kiln) | **200** (smelter) |
| Dominant zone | utility @ kiln | utility @ smelter |
| Loading critical | mixer outbound | fabrication outbound |
| Rail typical | optional coal | coal + bauxite optional |
| APS archetype | `FactoryCluster` + warehouse wing | `FactoryCluster` + tank farm |

---

## 5. Machine-readable handoff (designer-mcp)

Each step exports one row for `facility_research_brief_v1.json` (future schema):

```json
{
  "chain_id": "concrete_portland",
  "role": "cement_kiln",
  "catalog_id": "concrete_cement_kiln",
  "footprint_cells": { "w": 4, "d": 3 },
  "power_consumption": 72,
  "power_tier": "medium",
  "site_zone_requirements": {
    "primary_pct_min": 0.15,
    "utility_pct_min": 0.20,
    "loading_pct_min": 0.05,
    "buffer_pct_min": 0.25
  },
  "must_adjacent_to": ["utility"],
  "visual_cues": ["stack", "pipe_rack", "coal_yard"],
  "grammar_archetype_hint": "FactoryCluster"
}
```

**@designer-mcp:** DMCP-PILOT-CONCRETE-SITE-001 consumes §1.3 table + §5 rows for kiln/mine/mixer only.

---

## 6. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** DES-FACILITY-BINDING-001 · DMCP-PILOT-CONCRETE-SITE-001 · DES-APS-FACILITY-NEEDS-001 (with POWER-TIER)
