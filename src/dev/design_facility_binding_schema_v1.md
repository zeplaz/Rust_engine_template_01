# Facility binding schema `v1` — DES-FACILITY-BINDING-001

| Field | Value |
|:---|:---|
| **Gate** | **DES-FACILITY-BINDING-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E2-A |
| **Owner** | `@designer-mcp` (draft) · `@designer` sign-off |
| **Date** | 2026-06-02 |
| **Authority** | [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) · [`industrial_supply_chains.json`](../assets/configs/industrial_supply_chains.json) |
| **Verdict** | **PASS** — optional grammar block; catalog is Layer 3 authority |

```yaml
order_critique:
  request_summary: "Join visual grammar to process catalog without inventing power/IO"
  rules_audit:
    catalog_authority: pass
    no_power_override: pass
    deterministic_output: pass
  proceed: yes
```

---

## 1. Problem

Building grammars today are **visual-only**. Supply chains and building catalog JSON hold **authoritative** `power_consumption`, `supply_chain_role`, and I/O tags — but APS iterate and grammar sweep cannot surface them without a join contract.

**Rule:** Layer 3 (catalog + chains) is authority. Grammar `facility_binding` **references** catalog rows — never overrides numeric power or resource lists.

---

## 2. Optional block on `BuildingGrammar`

```ron
facility_binding: (
    catalog_id: "concrete_mixer_plant",
    chain_id: "concrete_portland",
    supply_chain_role: "concrete_mixer",
    power_tier: "light",
    site_template_id: "concrete_mixer_plant_site_v0",
    program_axes: (
        storage: "medium",
        loading: "high",
        office: "low",
    ),
)
```

JSON mirror uses the same keys (see [`building_grammar_factory_cluster_v1.json`](../tools/mcp/schemas/examples/building_grammar_factory_cluster_v1.json)).

---

## 3. Field contract

| Field | Required | Authority | Notes |
|:---|:---:|:---|:---|
| `catalog_id` | yes | `assets/configs/buildings/<id>.json` | Must exist on disk |
| `chain_id` | yes | `industrial_supply_chains.json` | Chain containing the step |
| `supply_chain_role` | yes | chain step + catalog | Must match catalog `supply_chain_role` |
| `power_tier` | yes | **derived label** | `light` / `medium` / `heavy` / `grid` — see §4 |
| `site_template_id` | no | Layer 2 pilot | `site_id` in `site_zone_grid_v1` JSON |
| `program_axes` | no | designer charter | `storage` / `loading` / `office` / `service` / `expansion` — enum `low` \| `medium` \| `high` |

**Forbidden:** `power_consumption`, `produces`, `consumes` on grammar — tools load these from catalog at brief time only.

---

## 4. Power tier bands (label only)

| Tier | Designer units | Typical roles |
|:---|:---:|:---|
| `light` | 0–30 | mine, mixer, parking |
| `medium` | 31–80 | kiln, refinery, fab |
| `heavy` | 81–200 | smelter |
| `grid` | utility infra | plant, substation |

**Validator rule (CMCP):** recompute tier from catalog `power_consumption`; **fail** if grammar `power_tier` disagrees.

---

## 5. Schema patch (shipped)

| Artifact | Change |
|:---|:---|
| [`building_grammar_v1.schema.json`](../tools/mcp/schemas/building_grammar_v1.schema.json) | Optional `facility_binding` property + `$defs/facility_binding` |
| [`facility_binding_v1.schema.json`](../tools/mcp/schemas/facility_binding_v1.schema.json) | Standalone fragment for pilots / brief JSON |

Rust loader: **ignore** until `COD-FACILITY-BINDING-READ-001` — serde drops unknown fields today.

---

## 6. Tool join (when CMCP ships)

```text
grammar_id → facility_binding → catalog JSON → chain row → brief JSON
```

Witness target: `debug_runs/art_pipeline/grammar_facility_brief_live.json` (CMCP-GRAMMAR-FACILITY-BRIEF-001).

---

## 7. First consumers

| Grammar | `catalog_id` | `chain_id` | Site template |
|:---|:---|:---|:---|
| `factory_cluster_v1` | `concrete_mixer_plant` | `concrete_portland` | `concrete_mixer_plant_site_v0` |
| `rail_edge_v1` | `logistics_rail_warehouse` | `logistics_storage` | `logistics_rail_warehouse_site_v0` |
| `industrial_warehouse_v1` | `logistics_storage_warehouse` | `logistics_storage` | `logistics_storage_warehouse_site_v0` |

**Logistics chain:** `logistics_storage` in [`industrial_supply_chains.json`](../assets/configs/industrial_supply_chains.json) — storage/transfer only (no process I/O).

---

## 8. Sign-off

DES-FACILITY-BINDING-001 Q✓ — schema draft on disk · `@designer` IA sign-off for APS panels pending
