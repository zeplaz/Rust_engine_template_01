# APS UI/UX Overhaul — agent todo board

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Plan** | [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/aps_uiux_overhaul_queue.json`](../tools/orchestrator/queues/aps_uiux_overhaul_queue.json) |
| **Drain playbook** | [`plan_aps_uiux_overhaul_drain_finish_v1.md`](plan_aps_uiux_overhaul_drain_finish_v1.md) ★ |
| **Prior** | APS-OPTION-D-001 **CLOSED** |
| **Rule** | **P0 gates all implementation** · coder-mcp **P1→P6 sequential** (shared `app.py`) · `pytest -k aps` every phase |

---

## Critical path

```text
@designer P0 design lock
    ↓
@coder-mcp P1 tokens → P2 text → P3 layout → P4 IA → P4.5 spine → P5 style → P5.5 preview → P5.6 onboard → P6 close
    ↓
@operator eyeball + @designer final sign + @designer-mcp artist accept
```

---

## Wave 3 — parallel drain (every agent)

**Queue:** [`aps_uiux_overhaul_parallel_drain_v1.json`](../tools/orchestrator/queues/aps_uiux_overhaul_parallel_drain_v1.json)

| Agent | ID | Task |
|:---|:---|:---|
| **@coder-mcp** | **OVR-P3-LAYOUT-001** | **★ PICK** — layout + app.py |
| **@designer** | OVR-DES-P45-SPINE-SPEC-001 | **✓ DONE** — [`design_aps_uiux_spine_spec_v1.md`](design_aps_uiux_spine_spec_v1.md) |
| **@designer** | DES-OVR-P56-ONBOARD-OUTLINE-001 | **✓ DONE** — [`design_aps_uiux_onboard_outline_v1.md`](design_aps_uiux_onboard_outline_v1.md) |
| **@designer-mcp** | DMCP-OVR-P2-IMPL-AUDIT-001 | **✓ DONE** — [`design_aps_uiux_p2_impl_audit_v1.md`](design_aps_uiux_p2_impl_audit_v1.md) |
| **@designer-mcp** | DMCP-OVR-P3-ACCEPT-RUBRIC-001 | **✓ DONE** — [`design_aps_uiux_p3_accept_rubric_v1.md`](design_aps_uiux_p3_accept_rubric_v1.md) |
| **@planner-mcp** | PLAN-OVR-P3-GUARD-SPEC-001 | **✓ SIGNED** 2026-06-17 |
| **@planner-mcp** | PLAN-OVR-HANDOFF-SYNC-001 | **✓ SIGNED** 2026-06-17 |
| **@planner-mcp** | PLAN-OVR-P45-WITNESS-STUB-001 | **✓ SIGNED** 2026-06-17 |
| **@operations-intelligence** | OPS-OVR-PROGRAM-SNAPSHOT-001 | **✓ DONE** |
| **@sim-steward** | STEWARD-OVR-APS-REGRESS-001 | **✓ DONE** — 163 passed |
| **@coder A** | CODER-A-VT5-TRIAGE-001 | VR-04 Bevy triage |
| **@coder B** | CODER-B-HUD-L5-POLISH-001 | Sim HUD lane 5 |
| **@operator** | — | P6 only |

---

## Wave 1 — archived

| Agent | ID | Status |
|:---|:---|:---|
| **@orchestrator-mcp** | ORCH-OVR-001 | **✓ DONE** — wave-1 issued |
| **@coder-mcp** | **OVR-P1-TOKENS-001** | **done** — §3 tokens + guards |
| **@designer-mcp** | DMCP-OVR-G0-AUDIT-001 | **✓ DONE** — [`design_aps_uiux_g0_audit_v1.md`](design_aps_uiux_g0_audit_v1.md) |
| **@designer** | OVR-DES-P2-COPY-PACK-001 | ready (parallel spec) |

## Wave 0 — P0 signed 2026-06-17

| Agent | ID | Status |
|:---|:---|:---|
| **@designer** | **OVR-P0-DESIGN-LOCK** | **✓ DONE** — [`aps_design_system_v1.md`](aps_design_system_v1.md) |

## Wave 0 — archived

| Agent | ID | Task |
|:---|:---|:---|
| **@orchestrator-mcp** | ORCH-OVR-FILELOCK-001 | Publish dispatch orders |
| **@planner-mcp** | PLAN-OVR-QUEUE-REGISTRY-001 | **✓ SIGNED** 2026-06-17 |
| **@planner-mcp** | PLAN-OVR-WITNESS-PROFILE-001 | **✓ SIGNED** 2026-06-17 |

---

## @designer (10 rows — authority, no Python)

| Phase | ID | Depends | Deliverable |
|:---|:---|:---|:---|
| **P0** | **OVR-P0-DESIGN-LOCK** | sweeps | `aps_design_system_v1.md` **✓** |
| P2 | OVR-DES-P2-COPY-PACK-001 | P0 | `design_aps_uiux_copy_pack_v1.md` **✓** |
| P3 | OVR-DES-P3-LAYOUT-DELTA-001 | P0 | `design_aps_uiux_layout_delta_v1.md` **✓** |
| P4 | OVR-DES-P4-IA-SIGN-001 | P0 | `design_aps_uiux_ia_sign_v1.md` **✓** |
| P4.5 | OVR-DES-P45-SPINE-SPEC-001 | P0 + P4 sign | `design_aps_uiux_spine_spec_v1.md` **✓** |
| P5.5 | OVR-DES-P55-PREVIEW-SPEC-001 | P0 | `design_aps_uiux_preview_spec_v1.md` **✓** |
| P5.6 | OVR-DES-P56-ONBOARD-SPEC-001 | P0 + spine lands | `design_aps_uiux_onboard_spec_v1.md` |
| P5.6 | DES-OVR-P56-ONBOARD-OUTLINE-001 | P0 | `design_aps_uiux_onboard_outline_v1.md` **✓** |
| P6 | OVR-P6-DESIGN-SIGN-001 | P6 close + operator | `design_aps_uiux_overhaul_signoff_v1.md` |

**P0 must include:** terminology glossary · token spec · IA contract (Buildings reorder, Stamp fold) · status atom · voice rules · ban-list.

---

## @coder-mcp (10 rows — sequential)

| Phase | ID | Depends | Key files | New guards |
|:---|:---|:---|:---|:---|
| P1 | OVR-P1-TOKENS-001 | P0 | `aps_theme.py` | `test_aps_style_tokens.py`, font floor ≤8 |
| P2 | OVR-P2-TEXT-001 | P1 + copy pack | all panels, `aps_tooltips.py` | `test_aps_no_jargon.py` **✓** |
| P3 | **OVR-P3-LAYOUT-001** | P2 + layout delta | `app.py`, `scrollable.py`, `footprint_canvas.py` | **★ PICK** — `test_aps_min_window_layout.py` |
| P4 | OVR-P4-IA-001 | P3 + IA sign | `domain_router.py`, `catalog.py`, tab order | lane isolation in runtime callbacks |
| P4.5 | OVR-P45-SPINE-001 | P4 + spine spec | `pipeline_status_bar.py`, `app.py` | pill→tab, no auto-switch |
| P5 | OVR-P5-STYLE-001 | P4.5 | `aps_inline_feedback.py`, tk→ttk | extend style tokens test |
| P5.5 | OVR-P55-PREVIEW-001 | P5 + preview spec | preview panels | labelled placeholder, never black |
| P5.6 | OVR-P56-ONBOARD-001 | P5.5 + onboard spec | `metadata_flow_panel.py`, `state.py` | `test_aps_onboarding.py` |
| P6 | OVR-P6-CLOSE-001 | P5.6 | witness refresh | full `pytest -k aps` + WIT-HON |

**Every phase:** keep `test_aps_imports.py` + `test_aps_runtime_callbacks.py` green.

---

## @designer-mcp (2 rows)

| Phase | ID | When | Task |
|:---|:---|:---|
| P0+ | DMCP-OVR-G0-AUDIT-001 | After P0 draft | G0 audit: ban-list + voice rules **✓** |
| P6 | DMCP-OVR-ARTIST-ACCEPT-001 | After design sign | **✓ DONE** 8/10 |

---

## @planner-mcp (2 rows — parallel wave 0)

| ID | Task |
|:---|:---|
| PLAN-OVR-QUEUE-REGISTRY-001 | Queue integrity registry |
| PLAN-OVR-WITNESS-PROFILE-001 | P6 witness field spec |

---

## @orchestrator-mcp (2 rows)

| ID | Task |
|:---|:---|
| ORCH-OVR-001 | Program dispatch | **done** 2026-06-02 · `$ref:debug_runs/agent_ops/aps_uiux_overhaul_dispatch_live.json` |
| ORCH-OVR-FILELOCK-001 | Per-phase file lock enforcement |

---

## @operator (1 row — human gate)

| ID | When | Task |
|:---|:---|:---|
| OVR-P6-OPERATOR-EYEBALL-001 | After P6 witness | **★ PICK** — structural pre-pass green; pixel walk pending |

---

## @coder A / @coder B

**No APS Tk scope** — idle unless G-PLAY / sim lanes reopen separately.

---

## Definition of done (program)

- [ ] P0 design system signed
- [ ] Token guards fail on literals (enforcement not vigilance)
- [ ] No jargon/gate IDs in visible strings or tooltips
- [ ] Buildings tabs: Catalog → Materials → Assembly → Variants → Atlas
- [ ] Landscape: Stamp folded into Atlas terminal state
- [ ] Clickable pipeline spine is sole "where am I / what's next"
- [ ] MIN 960×600 usable, footprint visible at 1280×800
- [ ] One status atom everywhere
- [ ] Preview surfaces: 4 states, never black
- [ ] First-run shows plain how-it-works, not schema diagram
- [ ] `pytest -k aps` green + operator eyeball + designer sign-off

```text
[/PLAN-APS-UIUX-OVERHAUL-001] wave-3 · P3 @coder-mcp NOW · P45 spine @designer parallel · planner wave-3 CLOSED · 11/24 main done
```
