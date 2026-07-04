# BQ-K3 grammar enrichment charter `v1` — BQ-K3-GRAMMAR-001

| Field | Value |
|:---|:---|
| **ID** | **BQ-K3-GRAMMAR-001** |
| **Issue** | BQ-K3 |
| **Parent** | [`plan_building_quality_v1.md`](plan_building_quality_v1.md) § Phase K |
| **Date** | 2026-07-03 |
| **Owner** | `@designer-mcp` (data charter) → `@coder` (RON merge + BQ-H1 FacadeRule) |
| **Status** | **SIGNED** |
| **Verdict** | **PASS** — enrichment manifest on disk |
| **Manifest** | [`bq_k3_grammar_enrichment_v1.json`](../../tools/mcp/schemas/examples/bq_k3_grammar_enrichment_v1.json) |
| **Witness** | [`debug_runs/bq_k3_grammar_001_live.json`](../../debug_runs/bq_k3_grammar_001_live.json) |

```yaml
order_critique:
  request_summary: "+massing strategies, FacadeRule by_massing tables, age→APS mandate mapping"
  rules_audit:
    data_not_code: pass
    grammar_ron_ids_stable: pass
    feeds_bq_h1: pass
  blocked: false
  proceed: yes
  handoff: "@coder merges patches into civic_block/factory_cluster/rail_edge RON"
```

---

## 0. Scope

Enrich **CivicBlock**, **FactoryCluster**, and **RailEdge** grammars (IndustrialWarehouse already ships `facade.by_massing` — reference pattern).

**Not in scope:** Evaluator code, BlockFrame, v0 grammar retirement (BQ-H3).

---

## 1. Massing additions (+2 each archetype)

| Grammar | New strategies | footprint_mode |
|:---|:---|:---|
| civic_block_v1 | `t_block`, `u_courtyard` | t_shape, u_shape |
| factory_cluster_v1 | `stepped_row`, `t_loading` | rect, t_shape |
| rail_edge_v1 | `t_rail_spur`, `stepped_dock` | t_shape, rect |

Weights sum with existing strategies; `@coder` renormalizes if needed.

---

## 2. FacadeRule tables (BQ-H1 feed)

Each new massing id gets a `facade.by_massing` row:

- `door_rhythm` — street-facing policy hint
- `placement_tags` — additive semantic tags
- Optional slot overrides (`door_slot`, `window_slot`)

Pattern authority: `industrial_warehouse_v1.ron` § facade.by_massing.

---

## 3. Age / weathering → APS mandate tags

Extend age bands with `aps_mandate_tags` mapping:

| variant_tag | APS mandate_tags | condition_tags |
|:---|:---|:---|
| clean | surface_clean | new |
| weathered | surface_weathered, edge_wear | aged |
| abandoned | surface_abandoned, vandalism | derelict |
| damaged | surface_damaged, structural_stress | damaged |

Mapped in manifest; merged into grammar `age.bands` by `@coder`.

---

## 4. Acceptance (designer-mcp gate)

| # | Criterion |
|:---:|:---|
| K3-A | Manifest lists 3 grammars × ≥2 new massing each |
| K3-B | FacadeRule row per new massing id |
| K3-C | Age APS map covers 4 variant bands |
| K3-D | Roof by_massing hooks for new massing ids |

**RON merge** is `@coder` exit — charter gate is manifest completeness only.
