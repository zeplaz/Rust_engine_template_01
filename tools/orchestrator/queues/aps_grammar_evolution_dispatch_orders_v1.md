# APS Grammar Evolution — dispatch orders v1

**Program:** PLAN-APS-GRAMMAR-EVOLUTION-001  
**Queue:** [`aps_grammar_evolution_queue.json`](aps_grammar_evolution_queue.json)  
**Plan:** [`plan_aps_grammar_evolution_v1.md`](../../src/dev/plan_aps_grammar_evolution_v1.md)  
**Witness:** [`plan_aps_grammar_evolution_witness_v1.md`](../../src/dev/plan_aps_grammar_evolution_witness_v1.md)  
**Board:** [`aps_grammar_evolution_agent_todos_v1.md`](../../src/dev/aps_grammar_evolution_agent_todos_v1.md)

**Rule:** Tier UI must match content on disk — **no fake green**.

---

## Work order (strict)

```text
1. APS-GRAM-TIER-001 + APS-GRAM-TIER-002   stop mashed UI
2. GRAM-CONTENT-001 → 002 → 003 → 004     fix sparse dropdowns → tier G1
3. APS-GRAM-TIER-002-REFRESH               re-gate UI at G1
4. APS-GRAM-P3-001                         inspector ↔ footprint
5. APS-GRAM-TIER-004                       spine copy matches tier
6. APS-GRAM-CLOSE-001                      WIT-HON rollup
```

---

## Wave 1 — issue now

```text
@orchestrator-mcp   ORCH-GRAM-001
@coder-mcp          APS-GRAM-TIER-001 ★ PICK
@coder-mcp          GRAM-CONTENT-002 ★ PICK (001 signed)
@designer-mcp       GRAM-CONTENT-001 ✓ DONE
@sim-steward        STEWARD-GRAM-WIT-HON-001 (watch tier honesty)
```

---

## Copy-paste orders

### @coder-mcp — Session 1 (TIER)

```text
PLAN-APS-GRAMMAR-EVOLUTION-001

Row 1: APS-GRAM-TIER-001
Implement grammar_set_tier() in rust_engine_mcp/grammar_build_set.py
Returns: { tier: G0-G4, reasons: [], archetype_count, district_count, ... }
Today: 1 ron file → tier G0, reasons explain gap to G1
Add: tests/test_aps_grammar_tier.py (must fail before impl)
Write: debug_runs/grammar_set_tier_live.json matching exit_predicate in witness profile
Exit: pytest test_aps_grammar_tier.py + pytest -k aps green

Row 2: APS-GRAM-TIER-002 (after 001 Q✓)
AssemblyPanel.apply_grammar_tier(tier) per plan exposure table
Add: tests/test_aps_grammar_tier_gates.py
At G0: hide DNA + iterate; kit hint on; build-set collapsed
Write: debug_runs/aps_grammar_tier_gates_live.json
NEEDS-DISPLAY: launch run.py — confirm ≤2 grammar panels expanded
```

### @designer-mcp — parallel

```text
GRAM-CONTENT-001
Deliverable: src/dev/design_grammar_archetype_family_g1_v1.md
Two new archetypes (FactoryCluster + RailEdge or signed alternates)
Each: massing strategies, districts, module gaps — deterministic seeds
Blocks GRAM-CONTENT-002 — no RON until spec signed
```

### @coder-mcp — Session 2 (CONTENT)

```text
GRAM-CONTENT-002 (after GRAM-CONTENT-001 signed)
Add *.ron + JSON mirror in grammars/ + schemas/examples/
list_archetype_ids() >= 3
validate-report arch_build_grammar each file
Witness: debug_runs/grammar_archetype_g1_live.json

GRAM-CONTENT-003 — labels in grammar_labels_v1.json
GRAM-CONTENT-004 — grammar_set_tier() → G1; debug_runs/grammar_set_tier_g1.json
APS-GRAM-TIER-002-REFRESH — kit hint off; 3+ dropdown values
```

### @coder-mcp — Session 3 (P3 + spine)

```text
APS-GRAM-P3-001
grammar_inspector.py: TreeviewSelect → assembly_panel → footprint_canvas highlight
Add FootprintCanvas.highlight_for_rule(rule_id) or equivalent
Fixture: generate IndustrialWarehouse/industrial_west seed=42; click long_hall row
Witness: debug_runs/aps_grammar_p3_live.json (cells_highlighted_count >= 1)

APS-GRAM-TIER-004
pipeline_status_bar.py + state: cache grammar_set_tier
Tier-aware Assembly/Atlas copy; atlas warn when tier < G4
Witness: debug_runs/aps_grammar_spine_tier_live.json
Guard: test_aps_grammar_spine_tier.py
```

### @sim-steward

```text
STEWARD-GRAM-WIT-HON-001
After each row Q✓: validate-report witness_honesty <witness> --compress 3
Block close if archetype_count in witness != len(list_archetype_ids())
```

---

## Definition of done

- [ ] `grammar_set_tier()` authoritative; G0 today, G1 after content
- [ ] Mashed UI fixed — panels tier-gated
- [ ] ≥3 archetypes in dropdowns with human labels
- [ ] Inspector click highlights footprint
- [ ] Pipeline spine copy tier-aware
- [ ] All guard tests green; close witness WIT-HON pass

```text
[/GRAMMAR-EVOLUTION] TIER-001 @coder-mcp NOW · CONTENT-001 @designer-mcp parallel
```
