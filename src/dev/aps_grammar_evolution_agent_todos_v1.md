# APS Grammar Evolution — agent todo board

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Plan** | [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) |
| **Queue** | [`aps_grammar_evolution_queue.json`](../tools/orchestrator/queues/aps_grammar_evolution_queue.json) |
| **Witness** | [`plan_aps_grammar_evolution_witness_v1.md`](plan_aps_grammar_evolution_witness_v1.md) |
| **Dispatch** | [`aps_grammar_evolution_dispatch_orders_v1.md`](../tools/orchestrator/queues/aps_grammar_evolution_dispatch_orders_v1.md) |
| **Rule** | Tier UI = content on disk · guard pytest + exit_predicate · no fake green |

---

## Execution todos (work order)

Prerequisite between **§1** and **§3**: `GRAM-CONTENT-001` → `002` → `003` → `004` (@designer-mcp + @coder-mcp).  
No `APS-GRAM-TIER-002-REFRESH` Q✓ until `grammar_set_tier_g1.json` green.

### §1 — APS-GRAM-TIER-001 + APS-GRAM-TIER-002 · stop mashed UI

**Owner:** @coder-mcp · **Blocks:** everything downstream

| ☐ | Task | Exit |
|:---:|:---|:---|
| ☐ | Implement `grammar_set_tier()` in `tools/mcp/python/rust_engine_mcp/grammar_build_set.py` | returns `tier` G0–G4 + `reasons[]` + live counts |
| ☐ | Add `tests/test_aps_grammar_tier.py` (fail before impl) | G0 today · reasons non-empty |
| ☐ | Write `debug_runs/grammar_set_tier_live.json` | `tier==G0` · `archetype_count==1` |
| ☐ | `AssemblyPanel.apply_grammar_tier(tier)` in `assembly_panel.py` | exposure table from plan |
| ☐ | G0 gates: hide DNA + iterate; kit hint on; build-set collapsed | scanner fields in witness |
| ☐ | Add `tests/test_aps_grammar_tier_gates.py` | panel visibility at G0 |
| ☐ | Write `debug_runs/aps_grammar_tier_gates_live.json` | `dna_panel_visible==false` · `kit_hint_visible==true` |
| ☐ | `pytest tools/mcp/python/tests -k aps -q` | all green |
| ☐ | **NEEDS-DISPLAY:** launch `run.py` — ≤2 grammar panels expanded | operator note in witness |

### §2 — GRAM-CONTENT-001 → 004 · sparse dropdowns → tier G1

**Owner:** @designer-mcp (001) + @coder-mcp (002–004) · **Blocks:** §3

| ☐ | Task | Exit |
|:---:|:---|:---|
| ☐ | **GRAM-CONTENT-001** spec signed | `src/dev/design_grammar_archetype_family_g1_v1.md` **✓** |
| ☐ | **Designer loop** before each content slice | `designer_grammar_iterate.ps1` → `next_actions` **✓** ([`design_grammar_iterate_tooling_v1.md`](design_grammar_iterate_tooling_v1.md)) |
| ☐ | **GRAM-CONTENT-002** add 2× `*.ron` + JSON mirrors | `list_archetype_ids() >= 3` — **PICK** |
| ☐ | `validate-report arch_build_grammar` each new file | `debug_runs/grammar_archetype_g1_live.json` |
| ☐ | **GRAM-CONTENT-003** human labels | `grammar_labels_v1.json` + `test_aps_grammar_labels.py` |
| ☐ | **GRAM-CONTENT-004** `grammar_set_tier()` → G1 | `debug_runs/grammar_set_tier_g1.json` |
| ☐ | Steward: `archetype_count` in witness == disk | WIT-HON on content witnesses |

### §3 — APS-GRAM-TIER-002-REFRESH · re-gate UI at G1

**Owner:** @coder-mcp · **Depends:** §1 Q✓ + §2 Q✓ (`GRAM-CONTENT-004`)

| ☐ | Task | Exit |
|:---:|:---|:---|
| ☐ | Re-call `apply_grammar_tier("G1")` after content lands | kit hint off |
| ☐ | Archetype combo shows ≥3 values | `archetype_combo_count >= 3` |
| ☐ | DNA/iterate still collapsed at G1 per exposure table | gates witness |
| ☐ | Write `debug_runs/aps_grammar_tier_g1_gates_live.json` | `tier==G1` · `kit_hint_visible==false` |
| ☐ | **NEEDS-DISPLAY:** dropdown not sparse; kit hint gone | operator note |
| ☐ | `pytest -k aps` green | regression |

### §4 — APS-GRAM-P3-001 · inspector → footprint

**Owner:** @coder-mcp · **Depends:** §1 TIER-002 + `GRAM-CONTENT-002`

| ☐ | Task | Exit |
|:---:|:---|:---|
| ☐ | `grammar_inspector.py`: `TreeviewSelect` → assembly panel callback | row select wired |
| ☐ | `FootprintCanvas.highlight_for_rule(rule_id)` (or equivalent) | highlight API |
| ☐ | Fixture: `IndustrialWarehouse` / `industrial_west` seed=42; click `long_hall` | cells highlight |
| ☐ | Add `tests/test_aps_grammar_inspector_link.py` | headless select → highlight state |
| ☐ | Write `debug_runs/aps_grammar_p3_live.json` | `rule_id_tested==long_hall` · `cells_highlighted_count>=1` |
| ☐ | `pytest -k aps` green | regression |

### §5 — APS-GRAM-TIER-004 · spine copy matches tier

**Owner:** @coder-mcp · **Depends:** §1 TIER-001 + §4 P3-001

| ☐ | Task | Exit |
|:---:|:---|:---|
| ☐ | Cache `grammar_set_tier()` in `state.py` / suite state | `grammar_set_tier_present` |
| ☐ | `pipeline_status_bar.py`: tier-aware Assembly step copy | G0 vs G1 strings differ |
| ☐ | Atlas step warns when `tier < G4` and ship check failed | not silent |
| ☐ | Add `tests/test_aps_grammar_spine_tier.py` | copy diff G0/G1 |
| ☐ | Write `debug_runs/aps_grammar_spine_tier_live.json` | `assembly_copy_tier_aware==true` |
| ☐ | Cross-check **OVR-P45-SPINE-001** does not ignore tier | witness includes `grammar_set_tier` |
| ☐ | **NEEDS-DISPLAY:** spine strings match tier at G0 and G1 | operator note |
| ☐ | `pytest -k aps` green | regression |

### §6 — APS-GRAM-CLOSE-001 + WIT-HON

**Owner:** @orchestrator-mcp + @sim-steward · **Depends:** §1–§5 all Q✓

| ☐ | Task | Exit |
|:---:|:---|:---|
| ☐ | Roll up row witnesses into `debug_runs/aps_grammar_evolution_close_live.json` | all slice paths linked |
| ☐ | `pytest -k aps` summary in close witness | `passed` / `failed` honest |
| ☐ | **STEWARD-GRAM-WIT-HON-001:** `validate-report witness_honesty` each witness | compress 3 |
| ☐ | Block close if tier claims ≠ `list_archetype_ids()` | anti–fake-green rule 1 |
| ☐ | Block close if G1 witnesses exist with `<3` archetypes | anti–fake-green rule 3 |
| ☐ | Collect `needs_display[]` operator verdicts | TIER-002 · REFRESH · TIER-004 |
| ☐ | Mark queue row `APS-GRAM-CLOSE-001` **done** | program close |

---

## Critical path

```text
TIER-001 → TIER-002 → CONTENT-001→004 → TIER-002-REFRESH → P3-001 → TIER-004 → CLOSE
         ↘ CONTENT-001 parallel with TIER-001/002
```

---

## @coder-mcp (7 implementation rows)

| Seq | ID | Status | Territory | Exit |
|:---|:---|:---|:---|:---|
| 1 | **APS-GRAM-TIER-001** | **✓ done** | `grammar_build_set.py` | `test_aps_grammar_tier.py` + tier witness |
| 2 | APS-GRAM-TIER-002 | **✓ done** | `assembly_panel.py` | `test_aps_grammar_tier_gates.py` |
| 3 | GRAM-CONTENT-002 | **✓ done** | `grammars/*.ron` | ≥3 archetypes |
| 4 | GRAM-CONTENT-003 | **✓ done** | `grammar_labels_v1.json` | human labels |
| 5 | GRAM-CONTENT-004 | **✓ done** | tier refresh | `grammar_set_tier_g1.json` |
| 6 | APS-GRAM-P3-001 | **✓ done** | inspector + footprint | `aps_grammar_p3_live.json` |
| 7 | APS-GRAM-TIER-004 | **✓ done** | `pipeline_status_bar.py` | spine tier copy |
| 8 | APS-GRAM-TIER-002-REFRESH | **✓ done** | assembly at G1 | G1 gates witness |
| — | APS-GRAM-CLOSE-001 | **✓ done** | rollup | `aps_grammar_evolution_close_live.json` WIT-HON pass |

---

## @designer-mcp

| ID | Status | Deliverable |
|:---|:---|:---|
| **GRAM-CONTENT-001** | **✓** | `design_grammar_archetype_family_g1_v1.md` |

---

## @designer

| ID | Status | Deliverable |
|:---|:---|:---|
| APS-GRAM-TIER-003 | **✓** | `design_aps_grammar_tier_wireframes_v1.md` |
| **DES-APS-GRAM-TIER-EXPOSURE-001** | **✓** | **`design_aps_grammar_tier_exposure_v1.md`** — **owns exposure per tier** |
| APS-GRAM-P3-003 | **✓** | `design_aps_grammar_why_copy_v1.md` |

---

## @orchestrator-mcp / @sim-steward

| ID | Owner | Task |
|:---|:---|:---|
| ORCH-GRAM-001 | orchestrator-mcp | **✓** Issue program |
| STEWARD-GRAM-WIT-HON-001 | sim-steward | **✓** Close witness WIT-HON passed |

---

## Definition of done

- [x] `grammar_set_tier()` returns honest G0/G1 from registry
- [x] G0: DNA/iterate hidden; kit hint visible
- [x] G1: ≥3 archetypes; kit hint off
- [x] Inspector row → footprint highlight (`long_hall` test)
- [x] Pipeline spine copy tier-aware
- [x] `pytest -k aps` green + close WIT-HON

```text
[/GRAMMAR-EVOLUTION] CLOSED_PENDING_OPERATOR — machine green 2026-06-18 · NEEDS-DISPLAY: TIER-002, REFRESH, TIER-004
```
