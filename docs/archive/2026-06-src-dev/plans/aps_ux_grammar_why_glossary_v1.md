# APS-UX-GRAMMAR-WHY — Human labels for grammar rule_ids (Phase 6 prep)

| Field | Value |
|:---|:---|
| **Program** | `APS-UX-GRAMMAR-WHY` |
| **Owner** | `@designer-mcp` (glossary) · `@coder-mcp` (inspector wiring) |
| **Feeds** | `grammar_inspector.py` `detail` column · slot `_slot_why_var` hints |
| **Source grammar** | `assets/configs/buildings/grammars/industrial_warehouse_v1.ron` |
| **Date** | 2026-06-03 |

Use **Human label** in UI; keep **rule_id** as stable key for validators and snapshots.

---

## Archetype layer

| rule_id | Human label | Why (artist-readable) |
|:---|:---|:---|
| `IndustrialWarehouse` | Industrial warehouse | Long-span storage / logistics shell; drives footprint bounds and module kit. |

---

## District / style layer

| rule_id | Human label | Why |
|:---|:---|:---|
| `industrial_west` | Industrial West district | Sawtooth + steel palette; default material profile map for wall/roof/door slots. |

---

## Massing layer

| rule_id | Human label | Why |
|:---|:---|:---|
| `long_hall` | Long hall | Wide shallow rectangle — main storage hall along street frontage. |
| `double_hall` | Double hall | Two-bay depth; moderate width:depth ratio for split interior zones. |
| `l_shape` | L-shaped yard | L footprint — corner yard or loading wing. |
| `yard_complex` | Yard complex | Interior yard massing; flat roof bias, yard-facing modules. |

---

## Footprint mode (chain key `footprint_mode`)

| rule_id | Human label | Why |
|:---|:---|:---|
| `rect` | Rectangle fill | Standard grid rectangle placement. |
| `l_shape` | L-shape fill | L-shaped footprint mask on grid. |
| `yard_interior` | Interior yard | Open yard cell pattern inside shell. |

---

## Roof layer (slot ids in chain)

| rule_id | Human label | Why |
|:---|:---|:---|
| `roof_default` | Default roof | Fallback roof module slot when massing has no override. |
| `roof_industrial` | Industrial roof | Sawtooth / metal industrial roof — paired with `long_hall`. |
| `roof_flat` | Flat roof | Low-pitch flat cap — paired with `yard_complex`. |

---

## Facade layer (slot ids — shown when facade rule resolves)

| rule_id | Human label | Why |
|:---|:---|:---|
| `wall_1u` | Standard wall (1u) | Primary exterior wall module slot. |
| `window_industrial` | Industrial window | Factory-style glazing slot on street-facing runs. |
| `door_wide` | Wide bay door | Loading / vehicle door slot. |

---

## Detail layer

| rule_id | Human label | Why |
|:---|:---|:---|
| `prop_clutter` | Roof / yard clutter | Vents, pipes, platforms — density 0.35 prop scatter. |

---

## Age / condition layer

| rule_id | Human label | Why |
|:---|:---|:---|
| `new` | New build | Clean variant tags; high weight in fresh districts. |
| `weathered` | Weathered | Mix of clean + weathered variant states. |
| `abandoned` | Abandoned | Damaged / abandoned variant bias for edge lots. |

---

## Material strategy (derived display — not a rule_id)

When `age` + `district_style` present, inspector shows e.g. **Weathered · Industrial West** — maps to district `material_profiles` overrides in grammar RON.

---

## Implementation note for @coder-mcp

```python
# tools/mcp/python/rust_engine_mcp/aps_grammar_labels.py (suggested)
GRAMMAR_WHY: dict[str, str] = { "long_hall": "Long hall — wide shallow storage shell", ... }

def human_label(rule_id: str) -> str:
    return GRAMMAR_WHY.get(rule_id, rule_id.replace("_", " ").title())
```

Grammar inspector: populate `detail` from `human_label(rule_id)`; keep `rule_id` column for support/debug.
