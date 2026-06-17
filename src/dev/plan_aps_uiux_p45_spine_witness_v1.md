# PLAN-OVR-P45-WITNESS-STUB-001 — P4.5 pipeline spine witness `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-OVR-P45-WITNESS-STUB-001
Date: 2026-06-17
Status: **SIGNED** (@planner-mcp)
Parent: PLAN-APS-UIUX-OVERHAUL-001 · OVR-P45-SPINE-001
Profile: extends $ref:src/dev/plan_aps_uiux_overhaul_witness_v1.md guards.P4.5
```

**Goal:** Stub witness envelope for **navigation / readiness** fields — **no pixel Q✓**, no NEEDS-DISPLAY on colors.

**Witness path (P4.5 refresh):** `debug_runs/aps_uiux_p45_spine_live.json`

---

## Required fields (stub — fill at OVR-P45-SPINE-001 Q✓)

| Field | Type | Rule |
|:---|:---|:---|
| `program_id` | string | `PLAN-APS-UIUX-OVERHAUL-001` |
| `phase` | string | `P4.5` |
| `green` | bool | `true` only when navigation assertions pass |
| `lane` | string | `buildings` \| `landscape` |
| `active_step_id` | string | Current pipeline pill id |
| `step_clickable` | bool | User can advance via pill click |
| `flow_verb_bound` | string | Active flow bar verb matches step |
| `auto_tab_switch` | bool | **must be false** (no auto-switch on bake) |
| `readiness_gates` | object | `{ step_id: pass\|blocked\|warn }` per pill |
| `pytest_aps` | object | `{ passed, failed, command }` |
| `_agent_meta` | object | envelope v1 |

---

## Guard tests (machine)

| Test | Proves |
|:---|:---|
| `test_aps_runtime_callbacks.py` | Spine callbacks wired |
| `test_aps_pipeline_validity.py` | Pill states valid for lane |

**NEEDS-DISPLAY:** none for P4.5 — operator defers to P5.5 preview + P6 eyeball.

---

## Example stub (not ship target)

```json
{
  "_meta": { "schema": "debug_run_envelope_v1", "not_a_ship_target": true },
  "program_id": "PLAN-APS-UIUX-OVERHAUL-001",
  "phase": "P4.5",
  "green": false,
  "lane": "buildings",
  "active_step_id": "grammar",
  "step_clickable": true,
  "flow_verb_bound": "generate_grammar",
  "auto_tab_switch": false,
  "readiness_gates": { "grammar": "pass", "bake": "blocked" },
  "note": "stub until OVR-P45-SPINE-001"
}
```

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-17 | **SIGNED** — field contract only |
| **@designer** | — | **OVR-DES-P45-SPINE-SPEC-001** interaction truth |
| **@coder-mcp** | — | Refresh witness at **OVR-P45-SPINE-001** |

```text
⟦/PLAN-OVR-P45-WITNESS-STUB-001⟧  ΔWF→ OVR-DES-P45-SPINE-SPEC-001 · OVR-P45-SPINE-001
```
