# APS UI/UX Overhaul — dispatch orders v1

**Program:** PLAN-APS-UIUX-OVERHAUL-001  
**Queue:** [`aps_uiux_overhaul_queue.json`](aps_uiux_overhaul_queue.json)  
**Plan:** [`plan_aps_uiux_overhaul_20260616_v1.md`](../../src/dev/plan_aps_uiux_overhaul_20260616_v1.md)  
**Board:** [`aps_uiux_overhaul_agent_todos_v1.md`](../../src/dev/aps_uiux_overhaul_agent_todos_v1.md)

---

## Wave 0 — issue now

```text
@orchestrator-mcp  ORCH-OVR-001 + ORCH-OVR-FILELOCK-001
@designer          OVR-P0-DESIGN-LOCK  ★ PICK (GATE — blocks all coder-mcp)
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

### @planner-mcp — parallel
```text
PLAN-OVR-QUEUE-REGISTRY-001 — register aps_uiux_overhaul_queue in queue_integrity
PLAN-OVR-WITNESS-PROFILE-001 — src/dev/plan_aps_uiux_overhaul_witness_v1.md for P6 close witness
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
