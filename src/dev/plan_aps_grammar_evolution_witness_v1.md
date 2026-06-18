# PLAN-APS-GRAMMAR-EVOLUTION — witness & exit gates `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Queue** | [`aps_grammar_evolution_queue.json`](../tools/orchestrator/queues/aps_grammar_evolution_queue.json) |
| **Rule** | **No Q✓** unless `exit_predicate.must` passes on witness JSON **and** guard pytest exists |

---

## Anti–fake-green rules

1. **Tier claims must match disk** — `archetype_count` in witness == `len(list_archetype_ids())`.
2. **UI gates must match tier** — `aps_grammar_tier_gates_live.json` scanned fields must agree with `AssemblyPanel` widget state at G0/G1.
3. **Content before G1 tier** — `grammar_set_tier_g1.json` forbidden until ≥3 `*.ron` grammars load.
4. **P3 link needs generate fixture** — `aps_grammar_p3_live.json` must include `rule_id_tested` + `cells_highlighted_count >= 1` from deterministic seed, not empty snapshot.
5. **Spine copy** — `aps_grammar_spine_tier_live.json` must diff G0 vs G1 assembly strings — identical copy = fail.
6. **WIT-HON** on all close witnesses — `validate-report witness_honesty`.

---

## Per-row witness contracts

### APS-GRAM-TIER-001

**Path:** `debug_runs/grammar_set_tier_live.json`

```json
{
  "tier": "G0",
  "archetype_count": 1,
  "district_count": 1,
  "reasons": ["archetype_count<3 for G1"],
  "grammar_files": ["industrial_warehouse_v1.ron"],
  "source": "grammar_set_tier()"
}
```

**Guard:** `tests/test_aps_grammar_tier.py` — asserts function exists, G0 today, reasons non-empty.

---

### APS-GRAM-TIER-002

**Path:** `debug_runs/aps_grammar_tier_gates_live.json`

```json
{
  "tier": "G0",
  "dna_panel_visible": false,
  "iterate_panel_visible": false,
  "build_set_expanded_default": false,
  "kit_hint_visible": true,
  "scanner": "test_aps_grammar_tier_gates.py"
}
```

**NEEDS-DISPLAY:** operator confirms ≤2 grammar panels expanded at launch.

---

### GRAM-CONTENT-001

**Deliverable only** — `src/dev/design_grammar_archetype_family_g1_v1.md` signed by @designer-mcp. No witness Q✓ without file path in queue row.

---

### GRAM-CONTENT-002

**Path:** `debug_runs/grammar_archetype_g1_live.json`

```json
{
  "archetype_count": 3,
  "archetype_ids": ["IndustrialWarehouse", "FactoryCluster", "RailEdge"],
  "ron_files_added": 2,
  "json_mirrors_added": 2,
  "validate_arch_build_grammar": "pass"
}
```

**Guard:** `tests/test_grammar_archetype_registry.py`

---

### GRAM-CONTENT-004 + tier G1

**Path:** `debug_runs/grammar_set_tier_g1.json`

```json
{
  "tier": "G1",
  "archetype_count": 3,
  "kit_hint_downgraded": true,
  "building_set_coverage": "pass"
}
```

---

### APS-GRAM-P3-001

**Path:** `debug_runs/aps_grammar_p3_live.json`

```json
{
  "inspector_click_highlights_grid": true,
  "rule_id_tested": "long_hall",
  "cells_highlighted_count": 4,
  "seed": 42,
  "archetype_id": "IndustrialWarehouse",
  "district_style": "industrial_west"
}
```

**Guard:** `tests/test_aps_grammar_inspector_link.py` — headless callback simulates row select → highlight state.

---

### APS-GRAM-TIER-004

**Path:** `debug_runs/aps_grammar_spine_tier_live.json`

```json
{
  "grammar_set_tier_present": true,
  "tier": "G1",
  "assembly_copy_tier_aware": true,
  "assembly_copy_g0_sample": "Generate from building type",
  "assembly_copy_g1_sample": "Tune shape bias; inspect rule chain",
  "atlas_warn_when_below_g4": true
}
```

**Guard:** `tests/test_aps_grammar_spine_tier.py`

---

### APS-GRAM-CLOSE-001

**Path:** `debug_runs/aps_grammar_evolution_close_live.json`

Rollup of all row witnesses + `pytest_aps: { passed, failed }` + `needs_display[]` operator verdicts.

---

## Guard tests to add (coder-mcp — fail until implemented)

| Test file | Proves |
|:---|:---|
| `test_aps_grammar_tier.py` | `grammar_set_tier()` API + G0 baseline |
| `test_aps_grammar_tier_gates.py` | Panel visibility at G0/G1 |
| `test_grammar_archetype_registry.py` | ≥3 archetypes after CONTENT-002 |
| `test_aps_grammar_inspector_link.py` | Inspector → footprint highlight |
| `test_aps_grammar_spine_tier.py` | Pipeline copy reads tier |

All must run under `pytest -k aps`.

---

## Cross-program (UIUX overhaul)

| UIUX row | Grammar row | Rule |
|:---|:---|:---|
| OVR-P45-SPINE-001 | APS-GRAM-TIER-004 | Spine implementation merges tier copy; witness must include `grammar_set_tier` |
| OVR-P2-TEXT-001 | GRAM-CONTENT-003 | Human labels for new archetypes |

Do not mark **OVR-P45** done if spine ignores tier.
