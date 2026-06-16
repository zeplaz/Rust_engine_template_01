# GRAMMAR-002-SLICE-001 — Facade + roof partial regen (thin plan) `v1`

| Field | Value |
|:---|:---|
| **ID** | **GRAMMAR-002-SLICE-001** |
| **Owner** | @planner-mcp |
| **Implement** | @coder (Rust) · @coder-mcp (Python/APS) |
| **Parent** | [`grammar_iter_001_spec_v1.md`](grammar_iter_001_spec_v1.md) · GRAMMAR-002 |
| **Status** | **PLAN READY** — after GRAMMAR-ITER-001 Phase 2 `massing` mode |
| **Date** | 2026-06-03 |

---

## Scope (thin slice only)

Enable **`iterate_grammar` modes `roof` and `facade`** for **IndustrialWarehouse** pilot — no new archetypes, no mesh-face massing.

**Out:** detail/age iteration content expansion, new module categories, Track B ship.

---

## Layer invalidation rules

| Mode | Touches placements | Preserves |
|:---|:---|:---|
| `roof` | All cells with `module_id` containing `roof` OR token `R` | Footprint, wall/door/window facade ring |
| `facade` | Perimeter `W`/`D`/`C` cells (not `R`, not interior) | Footprint dimensions, roof slots unless facade rule changes wall height |

**Material profiles:** Re-apply district `material_profiles` map for touched slots only ([`industrial_warehouse_v1.ron`](../../assets/configs/buildings/grammars/industrial_warehouse_v1.ron)).

---

## Grammar data additions (minimal)

Extend `building_grammar_v1` **facade** block (RON/JSON) — no schema version bump:

```ron
facade: (
    wall_slot: "wall_1u",
    door_slot: "door_wide",
    window_slot: "window_industrial",
    strategies: [
        (id: "factory_window_grid", weight: 50, window_density: 0.4),
        (id: "solid_industrial", weight: 30, window_density: 0.0),
        (id: "loading_bay", weight: 20, door_bias: "wide"),
    ],
),
roof: (
    default_slot: "roof_default",
    strategies: [
        (id: "roof_flat", weight: 40),
        (id: "roof_industrial", weight: 35),
        (id: "sawtooth", weight: 25),
    ],
    by_massing: [ ... existing ... ],
),
```

**Evaluator change:** pick `facade.strategies` / `roof.strategies` with `_mix_seed(seed, "facade")` — same pattern as massing.

---

## API overrides (already in request schema)

| mode | `overrides` | Behavior |
|:---|:---|:---|
| `roof` | `roof_slot` OR `roof_rule_id` | Pin slot; skip weighted strategy pick |
| `facade` | `wall_slot`, `door_slot`, `window_slot`, `facade_rule_id` | Re-resolve perimeter modules from style pack |

---

## Implementation slices

| Slice | Owner | Deliverable | Est. |
|:---|:---|:---|:---:|
| **G2S-1** | @planner-mcp | Example grammar JSON facade/roof strategies | **done** (this doc) |
| **G2S-2** | @coder | Rust: `apply_roof_layer` / `apply_facade_layer` on existing snapshot | 2–3h |
| **G2S-3** | @coder-mcp | Python parity + `grammar-iterate --mode roof|facade` | 2–3h |
| **G2S-4** | @coder-mcp | APS Iterate panel: roof/facade mode controls | 1–2h |
| **G2S-5** | @coder | Determinism tests + witness `grammar_002_roof_facade_live.json` | 1h |

**Depends:** GRAMMAR-ITER-001 Phase 2 (`massing` iterate) proves diff/lineage path.

---

## Acceptance

| Test | Pass |
|:---|:---|
| `roof` mode: only roof `node_id`s change between parent/child | diff.roof cells only |
| `facade` mode: wall/door/window modules change; roof unchanged | diff.facade cells only |
| `grammar_rule_chain.roof` / `.facade` updates | inspector shows new ids |
| Same seed + overrides → identical snapshot | determinism |

---

## Witness shape

```json
{
  "program_id": "GRAMMAR-002-SLICE-001",
  "modes": ["roof", "facade"],
  "archetype": "IndustrialWarehouse",
  "green": true
}
```

---

## Orchestrator paste

```text
GRAMMAR-002-SLICE-001 plan ready — assign after GRAMMAR-ITER-001-API massing green.

@coder G2S-2 roof/facade layer fns in building_grammar.rs
@coder-mcp G2S-3 Python iterate modes + G2S-4 APS controls
```
