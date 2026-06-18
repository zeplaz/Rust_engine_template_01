# CivicBlock archetype concept `v1` — DES-GRAM-ARCHETYPE-CIVIC-001

| Field | Value |
|:---|:---|
| **Gate** | **DES-GRAM-ARCHETYPE-CIVIC-001** |
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 · G2 seed |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Tier** | **Concept seed** — RON deferred to GRAM-CONTENT-005 |
| **Verdict** | **PASS** (spec only) |

```yaml
order_critique:
  request_summary: "Third content family seed — civic/commercial low-rise (not industrial)"
  rules_audit:
    deterministic_output: pass
    no_bpy_in_spec: pass
    distinct_from_g1_industrial: pass
  proceed: yes
```

---

## 1. Identity (artist-facing)

| Field | Value |
|:---|:---|
| `grammar_id` (planned) | `civic_block_v1` |
| `archetype.id` | `CivicBlock` |
| `archetype.usage` | `civic` |
| **APS label** | Civic Block |
| **One line** | Stepped brick podium + colonnade band + flat civic roof |
| **Style pack** | `style_colonial` (primary) · `style_victorian` (secondary district) |
| **Zoning** | `commercial` · `civic` |

**Ban-list:** never show raw `CivicBlock` in primary APS chrome.

---

## 2. vs G1 industrial lineage

| | Industrial G1 | CivicBlock |
|:---|:---|:---|
| Roof | Shed / gable industrial | **Flat parapet** + optional cupola slot |
| Facade | Steel panel / wide warehouse door | **Brick band** + **storefront glazing** |
| Massing | long_hall · yard_complex | **stepped_block** · **corner_anchored** |
| Module kit | `kit_production_002` shared | **New kit seed** `kit_civic_commercial_001` (P2) |
| Facility binding | logistics / process | **Optional** `commercial_retail` catalog stub (P2) |

---

## 3. Footprint bounds (proposed)

| Field | min | max |
|:---|:---:|:---:|
| width | 3 | 8 |
| depth | 3 | 7 |
| floors | 1 | 3 |

---

## 4. Massing strategies (≥2)

| id | weight | footprint_mode | Role |
|:---|:---:|:---|:---|
| `stepped_block` | **45** | `rect` | Primary — podium step + upper setback |
| `corner_anchored` | **35** | `l_shape` | Corner shop + side wing |
| `row_infill` | **15** | `rect` | Narrow commercial infill |
| `plaza_setback` | **5** | `yard_interior` | Civic plaza void |

---

## 5. Roof / facade / detail

| Layer | Slot / tag |
|:---|:---|
| roof default | `roof_flat_parapet` |
| roof by_massing | `stepped_block` → `roof_flat_parapet`; `plaza_setback` → `roof_terrace` |
| facade window | `window_storefront_2u` |
| facade door | `door_glass_entry` |
| wall | `wall_brick_1u` |
| detail density | **0.28** |
| detail tags | `awning`, `sign_band`, `planter`, `lamp_post` |
| age weights | new 50 / weathered 40 / abandoned 10 |

---

## 6. District styles (≥1)

| id | style_pack_id | zoning | style_tags |
|:---|:---|:---|:---|
| `main_street_civic` | `style_colonial` | `commercial` | `brick`, `colonnade`, `storefront` |
| `town_hall_row` | `style_victorian` | `civic` | `brick`, `bay_window`, `parapet` |

---

## 7. Deterministic test seeds (for GRAM-CONTENT-005)

| seed | district | Expected massing |
|:---:|:---|:---|
| 17 | `main_street_civic` | `stepped_block` |
| 53 | `main_street_civic` | `corner_anchored` |
| 91 | `town_hall_row` | `row_infill` |

---

## 8. RON deliverable (deferred — @coder-mcp)

| File | Archetype |
|:---|:---|
| `assets/configs/buildings/grammars/civic_block_v1.ron` | CivicBlock |
| `tools/mcp/schemas/examples/building_grammar_civic_block_v1.json` | mirror |

**Do not** add fourth archetype to G1 tier witness — CivicBlock targets **G2** family expansion.

---

## 9. Sign-off

DES-GRAM-ARCHETYPE-CIVIC-001 Q✓ — civic/commercial seed locked · RON open
