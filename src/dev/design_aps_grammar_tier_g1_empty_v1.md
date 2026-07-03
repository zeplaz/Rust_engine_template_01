# APS grammar tier G0/G1 empty states `v1` — DES-APS-GRAM-TIER-004

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-GRAM-TIER-004** |
| **Authority** | [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) §10 |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Handoff** | `@coder-mcp` — `grammar_tier_gate_snapshot()` wiring |
| **Verdict** | **PASS** |

```text
DES-APS-GRAM-TIER-004 Q✓
G0 kit hint · G1 unlock copy · mismatch guard rules
```

---

## Summary

Extract of §10 from tier exposure spec — canonical copy for G0 pilot vs G1 family seed. **Live tier today is G3** — G0/G1 copy still required for CI fixture matrix and regression tests.

### G0 kit hint (visible only @ G0)

`One building type in the kit for now — add grammar files under assets/configs/buildings/grammars/ to unlock more building types.`

### G1 unlock

- Hide kit hint when `archetype_combo_count >= 3`
- Optional toast: `✓ More building types available — pick a type and district, then Generate.`
- Tier chip: `G1 — family seed`

### Empty assembly

| Tier | Copy |
|:---|:---|
| G0–G1 | `No Assembly yet. Generate one from your building type.` |
| G2+ | `No assembly yet — Generate one to begin, then tune shape bias in the panels below.` |

See [`design_aps_assembly_empty_g2_v1.md`](design_aps_assembly_empty_g2_v1.md).

### Mismatch guard (witness)

| Field | Rule |
|:---|:---|
| `kit_hint_visible` | true **only** when `tier == G0` |
| `tier == G1` | implies `archetype_combo_count >= 3` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
