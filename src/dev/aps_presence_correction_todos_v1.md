# APS presence correction — agent todos `v1`

```text
⟦SYMLANG⟧⟐v1  ◈TODOS
⟨ID⟩ APS-PRESENCE-CORRECTION-001
Date: 2026-06-18
Status: **CLOSED** (2026-07-03)
Queue: tools/orchestrator/queues/aps_presence_correction_queue.json
Routing: src/dev/planner_routing_aps_presence_v1.md
Audit: src/dev/design_aps_default_presence_audit_v1.md
```

**Headline:** Live tier **G3** — brief/coverage/parity counts aligned · tier-gates live witness · session dump shipped. Operator rubric walk remains.

---

## Dependency order

```text
PLAN-APS-PRESENCE-PLAN-EDIT-001  (planner — parallel OK)
        ↓
APS-GUARD-BRIEF-PARITY-001       (coder-mcp P0)
        ↓
APS-GRAM-TIER-GATES-LIVE-001  +  DES-APS-SESSION-DUMP-001  (parallel)
        ↓
OVR-APS-PRESENCE-OPERATOR-001    (operator rubric walk)
```

---

## @planner

| ☐ | ID | Task | Deliverable |
|:---|:---|:---|:---|
| ☑ | **PLAN-APS-PRESENCE-PLAN-EDIT-001** | Update maturity "Example today" to **G3**; fix witness example JSON | `plan_aps_grammar_evolution_v1.md` |
| ☑ | *(same row)* | Split **fixture G0** vs **live tier** witnesses; anti-fake-green uses live | `plan_aps_grammar_evolution_witness_v1.md` |

---

## @coder-mcp

| ☐ | ID | Task | Exit |
|:---|:---|:---|:---|
| ☑ | **APS-GUARD-BRIEF-PARITY-001** | Unify pilot counts: brief · coverage · parity | `grammar_set_tier().reasons` empty at G4 OR honest red Set health |
| ☑ | **APS-GRAM-TIER-GATES-LIVE-001** | Live tier-gates witness from `refresh_grammar_tier_from_registry()` | `aps_grammar_tier_gates_live.json` tier == `grammar_set_tier_live.json` |
| ☑ | **DES-APS-SESSION-DUMP-001** | CLI `aps-session-presence-dump --write-witness` | `debug_runs/aps_session_presence_live.json` WIT-HON green |
| ☑ | **DES-APS-ASSEMBLY-EMPTY-G2-001** | Tier-aware empty label copy @ G2+ (designer **PASS**) | pytest string match in `assembly_panel.py` |

**Territory:** `grammar_build_set.py` · `cli.py` · `test_aps_grammar_tier_gates.py` · `assembly_panel.py`

**Regression:** `cd tools/mcp/python && python -m pytest -k aps -q`

---

## @designer

| ☐ | ID | Task | Exit |
|:---|:---|:---|:---|
| ☑ | **DES-APS-DEFAULT-PRESENCE-AUDIT-001** | Plan alignment audit — live G3 vs stale G0 | `design_aps_default_presence_audit_v1.md` **PASS (qualified)** |
| ☑ | **DES-APS-ASSEMBLY-EMPTY-G2-001** | Assembly empty tail @ G2+ | `design_aps_assembly_empty_g2_v1.md` **PASS** |

---

## @operator

| ☐ | ID | Task | Attach to HANDOFF |
|:---|:---|:---|:---|
| ☑ | **OVR-APS-PRESENCE-OPERATOR-001** | Operator lease attestation + G3 checklist | `aps_presence_operator_attestation_live.json` PASS 2026-07-03 |

**Commands:** see audit [`design_aps_default_presence_audit_v1.md`](design_aps_default_presence_audit_v1.md) §4.2

---

## Definition of done

- [x] Brief, coverage, and parity report the **same** pilot count
- [x] `aps_grammar_tier_gates_live.json` tier matches live registry (G3 today)
- [x] `aps_session_presence_live.json` bundles tier + brief + guards + ui_presence
- [x] `ui_presence.tier == grammar_set_tier.tier` (WIT-HON)
- [x] Operator HANDOFF has 3 witness paths attached (+ attestation witness)
- [x] Plan docs no longer claim G0 as "example today"
- [x] OVR-APS-PRESENCE-OPERATOR-001 Q✓ (operator lease 2026-07-03)

**Do not open** new designer tier-matrix specs — [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) already PASS.

```text
[/APS-PRESENCE-CORRECTION-001] CLOSED 2026-07-03 — APS-G4-COVERAGE-001 done · tier G4 · city G0 is next engine lane
```
