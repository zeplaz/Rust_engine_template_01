# PLAN-APS-UIUX-OVERHAUL — P6 witness envelope `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Witness path** | `debug_runs/aps_uiux_overhaul_close_live.json` |
| **Owner** | `@planner-mcp` (PLAN-OVR-WITNESS-PROFILE-001) |
| **Honesty gate** | `validate-report witness_honesty debug_runs/aps_uiux_overhaul_close_live.json --compress 3` |

---

## Required top-level fields

| Field | Type | Rule |
|:---|:---|:---|
| `program_id` | string | `PLAN-APS-UIUX-OVERHAUL-001` |
| `status` | string | `pass` only when all phase guards green + operator eyeball recorded |
| `phases_complete` | array | `P0`…`P6` ids present |
| `pytest_aps` | object | `{ passed, failed, command }` from `pytest -k aps -q` |
| `guards` | object | per-phase guard test results (see below) |
| `needs_display` | array | rows flagged NEEDS-DISPLAY with operator verdict |
| `prior_program` | string | `APS-OPTION-D-001` |
| `regression_baseline` | number | `149` passed at program open |
| `_agent_meta` | object | env, commands, cross-links per `debug_run_envelope.rs` |

---

## Per-phase guard map (must appear in `guards`)

| Phase | Guard tests |
|:---|:---|
| P1 | `test_aps_font_floor.py`, `test_aps_style_tokens.py`, `test_aps_ux_polish_density_tokens.py` |
| P2 | `test_aps_no_jargon.py` |
| P3 | `test_aps_min_window_layout.py` |
| P4 | `test_aps_lane_tab_swap.py`, `test_aps_runtime_callbacks.py` |
| P4.5 | `test_aps_runtime_callbacks.py` (spine navigation) |
| P5 | `test_aps_style_tokens.py` (status atom) |
| P5.5 | preview callbacks (NEEDS-DISPLAY for pixels) |
| P5.6 | `test_aps_onboarding.py` |
| P6 | full `pytest -k aps` + WIT-HON |

---

## Always-on spine

```json
{
  "runtime_guards": [
    "tests/test_aps_imports.py",
    "tests/test_aps_runtime_callbacks.py"
  ]
}
```

Both must be `pass` at close.

---

## NEEDS-DISPLAY rows (human gate — no Q✓ on pixels alone)

| Queue ID | Owner |
|:---|:---|
| OVR-P55-PREVIEW-001 | operator eyeball |
| OVR-P56-ONBOARD-001 | operator eyeball |
| OVR-P6-OPERATOR-EYEBALL-001 | operator |

Witness `needs_display[]` entries: `{ id, verdict: pass|fail|notes, operator, at }`.

---

## Sign-off chain (blocks program close)

```text
OVR-P6-CLOSE-001 (coder-mcp witness)
  → OVR-P6-OPERATOR-EYEBALL-001
  → OVR-P6-DESIGN-SIGN-001
  → DMCP-OVR-ARTIST-ACCEPT-001
```

---

## Cross-links

- Plan: `src/dev/plan_aps_uiux_overhaul_20260616_v1.md`
- P0 authority: `src/dev/aps_design_system_v1.md` (when signed)
- Queue: `tools/orchestrator/queues/aps_uiux_overhaul_queue.json`
