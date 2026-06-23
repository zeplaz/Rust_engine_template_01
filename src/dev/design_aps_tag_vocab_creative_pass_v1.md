# APS tag vocabulary creative pass `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-TAG-VOCAB-CREATIVE-001** |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Code** | `rust_engine_mcp/aps_tag_vocabulary.py` · taxonomy v1 expansion |
| **Verdict** | **PASS (qualified)** — labels + hints shipped; operator tag rubric walk deferred |

```text
DES-APS-TAG-VOCAB-CREATIVE-001 Q✓
No more raw snake_case in APS tag pickers — every mandate tag has artist label + hint
```

---

## Problem

APS showed **engineer tag ids** in Variants mandate families (`cultural_survival`, `fire_frame_axis`) and Assembly variant tags (`clean`, `night`) with **no context** on select. Reaction event dropdown had human labels but no anchor guidance. Taxonomy felt **thin** for industrial / logistics reads.

## Three tag surfaces (clarified)

| Surface | Where | Authority |
|:---|:---|:---|
| **Semantic tags** | Assembly · per placement | `aps_tag_taxonomy_v1.json` — ships on snapshot |
| **Assembly variant tags** | Assembly · per piece | `COMMON_VARIANT_TAGS` — piece-level state hints |
| **Mandate tags** | Variants · per variant_key row | `TAG_FAMILIES` — reaction / tile session metadata |

**Not interchangeable** — copy must say which surface the artist is editing.

## Shipped fixes

| Fix | Detail |
|:---|:---|
| Human labels | Mandate + assembly variant checkboxes show artist labels |
| Context line | Variants tag row shows hint on toggle + reaction filter shows anchor suggestions |
| Tooltips | `var_mandate_tag:*`, `asm_semantic_tag:*`, `asm_variant_tag:*` |
| Dedup | Removed duplicate `night_off` in `TAG_FAMILIES.light` |
| Audit test | `test_aps_tag_vocabulary.py` — 100% mandate label coverage |
| Taxonomy +6 | `rail_adjacent`, `waterfront`, `utility`, `cooling_tower`, `transformer_yard`, `decommissioned` |

## Creative tier-2 backlog (not shipped)

| Tag | Category | Story |
|:---|:---|:---|
| `district_power_feed` | detail | Substation tie-in for power grid sim |
| `bilingual_signage` | detail | Pairs with language_ban reaction |
| `occupation_banner` | condition | Transitional governance read |
| `scaffold_wrap` | condition | Long-horizon construction |

Queue as **DES-APS-TAG-TIER2-001** after operator rubric signs tier-1.

## Operator test plan

1. Variants → toggle **Window glow** — context line names emissive read
2. Reaction filter → **Heritage site destruction** — line lists Burn origin, Heritage marker
3. Assembly → hover **Rail adjacent** — tooltip describes logistics frontage
4. Run `pytest tools/mcp/python/tests/test_aps_tag_vocabulary.py -q`

## Exit predicate

`tag_vocabulary_audit()["green"] == True` + no raw snake_case in Variants mandate UI.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-06-02 |
