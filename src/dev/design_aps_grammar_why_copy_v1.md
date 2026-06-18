# APS Grammar "Why" Copy `v1` — APS-GRAM-P3-003

| Field | Value |
|:---|:---|
| **ID** | **APS-GRAM-P3-003** |
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`grammar_labels_v1.json`](../../assets/configs/buildings/grammars/grammar_labels_v1.json) · `aps_grammar_labels.py` |
| **Implements** | `APS-GRAM-P3-001` inspector tooltips · footprint highlight |
| **Verdict** | **PASS** |

```text
APS-GRAM-P3-003 Q✓
Unblocks: inspector detail column + P3 highlight affordance copy
```

---

## 0. Pattern

Inspector **Detail** column and grid hover use:

```text
{Human label} — {why sentence}
```

Source: `grammar_why_detail(rule_id)` — never raw `rule_id` alone in body text.

**P3 highlight:** on row select, footprint cells show banner:

```text
Highlighted: {human label} — {first clause of why}
```

---

## 1. Layer headings (inspector columns)

| Layer key | Heading (sentence case) |
|:---|:---|
| archetype | Building type |
| district_style | District |
| massing | Massing |
| footprint_mode | Footprint shape |
| roof | Roof |
| facade | Facade |
| detail | Detail |
| age | Age |

Retire: `Building:` / `Archetype:` engineer prefixes.

---

## 2. G1 archetype + district additions

Add to `grammar_labels_v1.json` + `GRAMMAR_LABELS` / `GRAMMAR_WHY`:

| rule_id | Human label | Why |
|:---|:---|:---|
| `FactoryCluster` | Factory cluster | Parallel production bays — favors double hall and modular roof clutter. |
| `RailEdge` | Rail edge warehouse | L-shaped hall along rail spur — loading wing and utility yard bias. |
| `manufacturing_row` | Manufacturing row | Factory-row steel palette; sawtooth roof emphasis. |
| `rail_yard_corridor` | Rail yard corridor | Logistics district along siding; wide doors and flat yards. |
| `SawtoothHall` | Sawtooth hall | (reserved G2 label) High roof modulation for north-light factories. |

---

## 3. Massing + footprint (existing — lock copy)

| rule_id | Why (verbatim for `GRAMMAR_WHY`) |
|:---|:---|
| `long_hall` | Wide shallow rectangle — main storage hall along street frontage. |
| `double_hall` | Two-bay depth; moderate width:depth ratio for split interior zones. |
| `l_shape` | L footprint — corner yard or loading wing. |
| `yard_complex` | Interior yard massing; flat roof bias, yard-facing modules. |
| `rect` | Standard grid rectangle placement. |
| `yard_interior` | Open yard cell pattern inside shell. |

---

## 4. Slot / placement hover (P3 extension)

When inspector row maps to placements, cell tooltip:

| massing_id | Tooltip |
|:---|:---|
| `long_hall` | `Main hall run — street-facing modules` |
| `double_hall` | `Inner bay — secondary module row` |
| `l_shape` | `Loading wing — yard-facing cells` |
| `yard_complex` | `Interior yard — open cells` |

If `placement_rule_id` missing on snapshot, show: `Placement — assign after regenerate`.

---

## 5. Empty / blocked inspector

| State | Copy |
|:---|:---|
| No snapshot | `Generate an Assembly to see the building style chain.` |
| No rule chain | `Run Ship check or regenerate — rule chain empty.` |
| Row selected, no cells | `No pieces match this step yet.` |

---

## 6. Implementation checklist

- [ ] `grammar_inspector.py` Detail column = `grammar_why_detail(rule_id)`
- [ ] `FootprintCanvas.highlight_for_rule(rule_id)` banner uses §4 templates
- [ ] `test_aps_grammar_labels.py` covers G1 ids
- [ ] `test_aps_no_jargon.py` — no `ARCH-DNA` in inspector strings

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |
