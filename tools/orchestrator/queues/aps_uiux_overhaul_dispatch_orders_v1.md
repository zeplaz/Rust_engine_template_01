# APS UI/UX Overhaul — dispatch orders v1

**Program:** PLAN-APS-UIUX-OVERHAUL-001  
**Drain playbook:** [`plan_aps_uiux_overhaul_drain_finish_v1.md`](../../src/dev/plan_aps_uiux_overhaul_drain_finish_v1.md) ★ **use this to close**  
**Queue:** [`aps_uiux_overhaul_queue.json`](aps_uiux_overhaul_queue.json)  
**Plan:** [`plan_aps_uiux_overhaul_20260616_v1.md`](../../src/dev/plan_aps_uiux_overhaul_20260616_v1.md)  
**Board:** [`aps_uiux_overhaul_agent_todos_v1.md`](../../src/dev/aps_uiux_overhaul_agent_todos_v1.md)

---

## Wave 3 — issued 2026-06-17

```text
Progress: 11/24 main queue (46%) · pytest -k aps: 161 passed

@coder-mcp         OVR-P3-LAYOUT-001 ★ PICK (app.py ONLY — blocks P4→P6)
@designer          OVR-DES-P45-SPINE-SPEC-001 ✓ + DES-OVR-P56-ONBOARD-OUTLINE-001 ✓
@designer-mcp      DMCP-OVR-P2-IMPL-AUDIT-001 ✓ + DMCP-OVR-P3-ACCEPT-RUBRIC-001 ✓
### @planner-mcp — **CLOSED 2026-06-17**

```text
DONE: PLAN-OVR-P3-GUARD-SPEC-001 · PLAN-OVR-HANDOFF-SYNC-001 · PLAN-OVR-P45-WITNESS-STUB-001
WIT:  debug_runs/mcp_planner_ovr_wave3_sign_live.json
```
@operations-intelligence  OPS-OVR-PROGRAM-SNAPSHOT-001
@sim-steward       STEWARD-OVR-APS-REGRESS-001
@coder A           CODER-A-VT5-TRIAGE-001 (Bevy — not APS)
@coder B           CODER-B-HUD-L5-POLISH-001 (sim HUD — not APS)
@operator          idle until P6 eyeball

Parallel queue: tools/orchestrator/queues/aps_uiux_overhaul_parallel_drain_v1.json
Witness: debug_runs/agent_ops/aps_uiux_overhaul_dispatch_live.json
```

### Copy-paste — one chat per agent

**@coder-mcp**
```text
OVR-P3-LAYOUT-001 — FILE LOCK app.py exclusive.
Authority: design_aps_uiux_layout_delta_v1.md + aps_design_system_v1.md
Work: chrome collapse, grid above fold, migrate padding literals, MIN 960×600 no h-scroll.
Add: test_aps_min_window_layout.py guard.
Exit: pytest -k aps green + commit → unblocks P4.
```

**@designer**
```text
OVR-DES-P45-SPINE-SPEC-001 → design_aps_uiux_spine_spec_v1.md
Plus: DES-OVR-P56-ONBOARD-OUTLINE-001 (outline while spine drafts)
Authority: aps_design_system_v1.md + design_aps_uiux_ia_sign_v1.md
```

**@designer-mcp**
```text
DMCP-OVR-P2-IMPL-AUDIT-001 — audit P2 vs copy pack + G0 ban-list
DMCP-OVR-P3-ACCEPT-RUBRIC-001 — layout accept rubric for P3 eyeball
```

**@planner-mcp**
```text
PLAN-OVR-P3-GUARD-SPEC-001 → plan_aps_uiux_p3_layout_guard_v1.md
PLAN-OVR-HANDOFF-SYNC-001 — reconcile HANDOFF vs queue
PLAN-OVR-P45-WITNESS-STUB-001 → plan_aps_uiux_p45_spine_witness_v1.md
```

**@sim-steward**
```text
STEWARD-OVR-APS-REGRESS-001 — run pytest -k aps after each coder-mcp phase Q✓
Witness: debug_runs/aps_uiux_overhaul_regress_watch_live.json
```

## Wave 1 — archived

```text
ORCH-OVR-001 ✓ DONE — witness debug_runs/agent_ops/aps_uiux_overhaul_dispatch_live.json
P0 GATE ENFORCED — OVR-P0-DESIGN-LOCK signed (aps_design_system_v1.md)

@coder-mcp         OVR-P1-TOKENS-001  ★ in_progress (aps_theme.py ONLY — no app.py)
@designer-mcp      DMCP-OVR-G0-AUDIT-001 ✓ DONE — witness dmcp_ovr_g0_audit_live.json (54 hits → P2)
@designer          OVR-DES-P2/P3/P4/P55 specs — ready (parallel spec)
@coder-mcp         FILE LOCK: ONE phase P1→P6 · app.py sole-writer P3/P4/P4.5 only
@coder-mcp         P2+ BLOCKED until prior phase Q✓ + commit
```

### File-lock enforcement (ORCH-OVR-001)

```text
ACTIVE:  P1 OVR-P1-TOKENS-001
CHAIN:   P1 → P2 → P3 → P4 → P4.5 → P5 → P5.5 → P5.6 → P6
app.py:  exclusive writer ONLY when P3, P4, or P4.5 is active
FORBID:  parallel coder-mcp · skip phase · touch app.py out of phase
EXIT:    pytest -k aps + test_aps_imports + test_aps_runtime_callbacks each phase
```

## Wave 0 — P0 signed 2026-06-17 (archived)

```text
@designer          OVR-P0-DESIGN-LOCK  ✓ DONE — aps_design_system_v1.md
@planner-mcp       PLAN-OVR-QUEUE-REGISTRY-001 · PLAN-OVR-WITNESS-PROFILE-001
@coder-mcp         BLOCKED until P0 sign-off
@designer-mcp      DMCP-OVR-G0-AUDIT-001 after P0 draft
```

---

## Copy-paste orders

### @orchestrator-mcp
```text
Program PLAN-APS-UIUX-OVERHAUL-001.
Queue: tools/orchestrator/queues/aps_uiux_overhaul_queue.json
Gate: OVR-P0-DESIGN-LOCK must sign before OVR-P1-TOKENS-001.
Enforce sequential P1→P6 on @coder-mcp (app.py file lock).
NEEDS-DISPLAY items require operator eyeball — no Q✓ on pixels alone.
```

### @designer — P0 GATE ★
```text
OVR-P0-DESIGN-LOCK
Deliverable: src/dev/aps_design_system_v1.md
Consolidate: aps_sweep_text/layout/tabdesign/style/workflow_tooltips_vibe (20260616)
Must include: terminology glossary · full token spec · IA contract (Buildings reorder, Stamp→Atlas) · status atom · ban-list · voice rules
Sign-off blocks ALL @coder-mcp implementation.
```

### @planner-mcp — **CLOSED 2026-06-17**

```text
DONE: PLAN-OVR-QUEUE-REGISTRY-001 · PLAN-OVR-WITNESS-PROFILE-001
WIT:  debug_runs/mcp_planner_ovr_sign_live.json
```

### @coder-mcp — after P0
```text
Sequential phases ONLY — commit between each:
P1 OVR-P1-TOKENS-001 → P2 → P3 → P4 → P4.5 → P5 → P5.5 → P5.6 → P6
Every phase: pytest -k aps green + test_aps_imports + test_aps_runtime_callbacks
Interpreter: python 3.14 (Pillow) — not py -3.13
Visual fixes: flag NEEDS-DISPLAY — do not self-certify pixels
```

### @designer-mcp
```text
After P0 draft: DMCP-OVR-G0-AUDIT-001 (ban-list + voice audit)
After P6: DMCP-OVR-ARTIST-ACCEPT-001 (post-overhaul artist score)
```

### @operator
```text
After OVR-P6-CLOSE-001 witness: OVR-P6-OPERATOR-EYEBALL-001
Launch: python tools/mcp/art_pipeline_suite/run.py
Walk Buildings + Landscape at 1280×800 and MIN 960×600
```

---

## File-lock table (one active coder-mcp phase)

| Phase | Primary files | Next phase waits on commit |
|:---|:---|:---|
| P1 | `aps_theme.py`, guard tests | — |
| P2 | strings, `aps_tooltips.py`, `metadata_flow_panel.py` | P3 |
| P3 | `app.py` chrome, `scrollable.py`, `footprint_canvas.py`, panes | P4 |
| P4 | `domain_router.py`, `app.py` tabs, `catalog.py`, material sites | P4.5 |
| P4.5 | `pipeline_status_bar.py`, `app.py` flow verbs | P5 |
| P5 | `aps_inline_feedback.py`, status sites, tk→ttk | P5.5 |
| P5.5 | preview surfaces | P5.6 |
| P5.6 | `metadata_flow_panel.py`, `state.py`, empty states | P6 |

---

## Verification spine

```text
BLANG:PRE → boot <agent> → work → pytest -k aps → WIT-HON (P6) → Q✓
```

Baseline (must stay green): **149 passed** `pytest -k aps` at program start.
