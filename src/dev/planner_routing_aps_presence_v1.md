# Planner routing — APS default presence corrections

| Field | Value |
|:---|:---|
| **From** | `@designer` · [`design_aps_default_presence_audit_v1.md`](design_aps_default_presence_audit_v1.md) |
| **To** | `@planner` |
| **Date** | 2026-06-02 |
| **Priority** | **P1** — blocks honest G4 expansion + operator APS reviews |

---

## Message

Planner — the APS grammar-evolution program is **planning against G0** while the repo **boots at G3**. Artist-facing exposure specs are fine; **plan tables, witness JSON, and guard summaries disagree with each other**. Please re-phase the next coder-mcp slices around **presence truth**, not new UI.

### Facts (measured 2026-06-02)

1. **`grammar_set_tier()` → G3** — 4 RON grammars, 4 archetypes, 5 districts, 4 DNA presets.
2. **`AssemblyPanel.refresh_grammar_tier_from_registry()`** applies G3 (DNA + iterate visible, kit hint hidden, set-health promoted).
3. **`grammar_set_brief` → green**, gaps none — but **`building_set_coverage`** and **`grammar_pilot_parity` → red** (`grammar_pilots: 0` vs brief’s 4). This is why tier stays G3 not G4.
4. **`aps_grammar_tier_gates_live.json` still says G0** with `kit_hint_visible: true` while `archetype_combo_count: 4` — violates designer anti-fake-green rule §10.3.
5. **Onboarding / 5-tab spine** still correct — no structural IA change needed.

### Plan doc edits (planner-owned)

| File | Action |
|:---|:---|
| [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) | Update § maturity “Example today” to **G3**; fix witness example JSON |
| [`plan_aps_grammar_evolution_witness_v1.md`](plan_aps_grammar_evolution_witness_v1.md) | Split **fixture G0** vs **live tier** witnesses; anti-fake-green uses live tier |

### Queue seeds (suggested)

**Machine queue:** [`tools/orchestrator/queues/aps_presence_correction_queue.json`](../../tools/orchestrator/queues/aps_presence_correction_queue.json)  
**Todo board:** [`aps_presence_correction_todos_v1.md`](aps_presence_correction_todos_v1.md)

| ID | Owner | Goal | Exit |
|:---|:---|:---|:---|
| **APS-GUARD-BRIEF-PARITY-001** | coder-mcp | Single pilot-count authority for brief · coverage · parity | `grammar_set_tier().reasons` empty at G4 bar OR honest red Set health |
| **APS-GRAM-TIER-GATES-LIVE-001** | coder-mcp | Refresh tier-gates witness from `refresh_grammar_tier_from_registry`, not `apply("G0")` | `aps_grammar_tier_gates_live.json` tier == `grammar_set_tier_live.json` |
| **DES-APS-SESSION-DUMP-001** | coder-mcp | Implement `aps_session_presence_live.json` bundle (§4.3 of audit) | WIT-HON green · operator one command |
| **DES-APS-ASSEMBLY-EMPTY-G2-001** | coder-mcp | Tier-aware assembly empty-state tail (designer copy in audit §1.3) | pytest string match |
| **OVR-APS-PRESENCE-OPERATOR-001** | operator | Run §4.2 dump before next rubric walk | Attach 3 JSON paths to HANDOFF |

**Do not open** new designer specs for tier matrix — [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) already PASS.

### Landscape expansion

[`design_landscape_lg5_expansion_matrix_v1.md`](design_landscape_lg5_expansion_matrix_v1.md) — **no designer amend**. Keep building G3 and landscape LG-5 on separate spine/domain labels (audit §3.2).

### Dependency order

```text
APS-GUARD-BRIEF-PARITY-001   (unblocks honest G4 + Set health copy)
        ↓
APS-GRAM-TIER-GATES-LIVE-001 + DES-APS-SESSION-DUMP-001
        ↓
Plan witness table refresh + operator pixel walk
```

---

## Attach for HANDOFF

- `debug_runs/grammar_set_tier_live.json` (tier G3)
- `debug_runs/grammar_set_brief_live.json` (green)
- `debug_runs/aps_grammar_tier_gates_live.json` (stale G0 — **do not trust until LIVE-001**)
- Designer audit: `src/dev/design_aps_default_presence_audit_v1.md`
