# PLAN-APS-GRAMMAR-EVOLUTION — witness & exit gates `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Queue** | [`aps_grammar_evolution_queue.json`](../tools/orchestrator/queues/aps_grammar_evolution_queue.json) |
| **Presence correction** | [`aps_presence_correction_queue.json`](../tools/orchestrator/queues/aps_presence_correction_queue.json) |
| **Date** | 2026-06-07 · **G3 reconcile** 2026-06-21 |
| **Rule** | **No Q✓** unless `exit_predicate.must` passes on witness JSON **and** guard pytest exists |

---

## Live vs fixture witnesses (mandatory split)

| Path | Purpose | Trust for cold-start? |
|:---|:---|:---:|
| `debug_runs/grammar_set_tier_live.json` | `grammar_set_tier()` on disk | **Yes** |
| `debug_runs/aps_grammar_tier_gates_live.json` | UI exposure after `refresh_grammar_tier_from_registry()` | **Yes** |
| `debug_runs/aps_session_presence_live.json` | Bundled tier + brief + guards + ui_presence | **Yes** |
| `debug_runs/aps_grammar_tier_gates_g0_fixture_live.json` | `apply_grammar_tier("G0")` matrix test only | **No** |
| Historical `grammar_set_tier_g1.json` etc. | Milestone snapshots | Audit only |

**Anti-fake-green:** never cite G0 fixture paths when reporting “what APS boots with today.” Live tier on disk is **G3** (2026-06-21).

---

## Anti–fake-green rules

1. **Tier claims must match disk** — `archetype_count` in witness == `len(list_archetype_ids())`.
2. **UI gates must match live tier** — `aps_grammar_tier_gates_live.json` fields must agree with `grammar_set_tier_live.json` tier **and** `AssemblyPanel.refresh_grammar_tier_from_registry()` at cold start. G0 fixture path is CI-only.
3. **Content before G1 tier claim** — `grammar_set_tier_g1.json` milestone forbidden until ≥3 `*.ron` grammars load (historical — bar met).
4. **P3 link needs generate fixture** — `aps_grammar_p3_live.json` must include `rule_id_tested` + `cells_highlighted_count >= 1` from deterministic seed, not empty snapshot.
5. **Spine copy** — `aps_grammar_spine_tier_live.json` must diff G0 vs G1 assembly strings — identical copy = fail.
6. **WIT-HON** on all close witnesses — `validate-report witness_honesty`.
7. **Bundled presence** — `ui_presence.tier` in `aps_session_presence_live.json` must equal `grammar_set_tier.tier` or WIT-HON fail.

---

## Per-row witness contracts

### APS-GRAM-TIER-001

**Path:** `debug_runs/grammar_set_tier_live.json`

**Live example (2026-06-21 — G3):**

```json
{
  "tier": "G3",
  "archetype_count": 4,
  "district_count": 5,
  "grammar_files": [
    "civic_block_v1.ron",
    "factory_cluster_v1.ron",
    "industrial_warehouse_v1.ron",
    "rail_edge_v1.ron"
  ],
  "reasons": ["building_set_coverage not green for G4"],
  "source": "grammar_set_tier()",
  "preset_count": 4,
  "f_axis_count": 4
}
```

**Guard:** `tests/test_aps_grammar_tier.py` — asserts function exists; tier matches disk; reasons non-empty when below G4.

**Historical G0 baseline** (pilot-era only — do not use as “today”):

```json
{
  "tier": "G0",
  "archetype_count": 1,
  "reasons": ["archetype_count<3 for G1"]
}
```

---

### APS-GRAM-TIER-002 — live + G0 fixture

#### Live (cold-start truth)

**Path:** `debug_runs/aps_grammar_tier_gates_live.json`

**Live example (G3 today):**

```json
{
  "tier": "G3",
  "grammar_set_tier": "G3",
  "dna_panel_visible": true,
  "iterate_panel_visible": true,
  "build_set_expanded_default": true,
  "kit_hint_visible": false,
  "archetype_combo_count": 4,
  "green": true,
  "grammar_set_tier_match": true,
  "scanner": "grammar_build_set.write_aps_grammar_tier_gates_live_witness"
}
```

**Exit predicate:** `tier` == `grammar_set_tier` == value in `grammar_set_tier_live.json`.

**Guard:** `test_write_aps_grammar_tier_gates_live_witness` — calls `refresh_grammar_tier_from_registry()`; writes live path only.

#### G0 fixture (CI matrix only)

**Path:** `debug_runs/aps_grammar_tier_gates_g0_fixture_live.json`

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

**Guard:** `test_write_aps_grammar_tier_gates_witness` — must **not** overwrite `aps_grammar_tier_gates_live.json`.

**NEEDS-DISPLAY:** operator confirms G3 launch — DNA + iterate visible, kit hint hidden.

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
  "tier": "G3",
  "assembly_copy_tier_aware": true,
  "assembly_copy_g0_sample": "Generate from building type",
  "assembly_copy_g3_sample": "Tune layers; inspect rule chain",
  "atlas_warn_when_below_g4": true
}
```

**Guard:** `tests/test_aps_grammar_spine_tier.py`

---

### DES-APS-SESSION-DUMP-001 (bundled presence)

**Path:** `debug_runs/aps_session_presence_live.json`

Rollup: `grammar_set_tier` + `grammar_set_brief` + `g4_guards` + `ui_presence` + `expansion`. Schema: [`design_aps_default_presence_audit_v1.md`](design_aps_default_presence_audit_v1.md) §4.3.

**Exit:** `ui_presence.tier` == `grammar_set_tier.tier`; WIT-HON green; CLI `aps-session-presence-dump --write-witness`.

---

### APS-GRAM-CLOSE-001

**Path:** `debug_runs/aps_grammar_evolution_close_live.json`

Rollup of all row witnesses + `pytest_aps: { passed, failed }` + `needs_display[]` operator verdicts.

---

## Guard tests to add (coder-mcp — fail until implemented)

| Test file | Proves |
|:---|:---|
| `test_aps_grammar_tier.py` | `grammar_set_tier()` API + live tier matches disk |
| `test_aps_grammar_tier_gates.py` | G0 fixture path + **live** witness from registry refresh |
| `test_grammar_archetype_registry.py` | ≥3 archetypes after CONTENT-002 |
| `test_aps_grammar_inspector_link.py` | Inspector → footprint highlight |
| `test_aps_grammar_spine_tier.py` | Pipeline copy reads tier |

All must run under `pytest -k aps`.

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.1.0 | 2026-06-21 | G3 live examples; live vs G0 fixture split; bundled presence witness · `PLAN-APS-PRESENCE-PLAN-EDIT-001` |
| v1.0.0 | 2026-06-07 | Initial witness profile |

---

## Cross-program (UIUX overhaul)

| UIUX row | Grammar row | Rule |
|:---|:---|:---|
| OVR-P45-SPINE-001 | APS-GRAM-TIER-004 | Spine implementation merges tier copy; witness must include `grammar_set_tier` |
| OVR-P2-TEXT-001 | GRAM-CONTENT-003 | Human labels for new archetypes |

Do not mark **OVR-P45** done if spine ignores tier.
