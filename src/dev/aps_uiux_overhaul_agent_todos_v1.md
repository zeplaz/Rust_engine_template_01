# APS UI/UX Overhaul — agent todo board

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Plan** | [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/aps_uiux_overhaul_queue.json`](../tools/orchestrator/queues/aps_uiux_overhaul_queue.json) |
| **Dispatch** | [`aps_uiux_overhaul_dispatch_orders_v1.md`](../tools/orchestrator/queues/aps_uiux_overhaul_dispatch_orders_v1.md) |
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

## Wave 0 — START NOW (parallel)

| Agent | ID | Task |
|:---|:---|:---|
| **@orchestrator-mcp** | ORCH-OVR-001 | Issue program; enforce P0 gate + file locks |
| **@orchestrator-mcp** | ORCH-OVR-FILELOCK-001 | Publish dispatch orders |
| **@designer** | **OVR-P0-DESIGN-LOCK** | **aps_design_system_v1.md** — GATE |
| **@planner-mcp** | PLAN-OVR-QUEUE-REGISTRY-001 | Register queue in integrity registry |
| **@planner-mcp** | PLAN-OVR-WITNESS-PROFILE-001 | P6 witness envelope spec |

**@coder-mcp is BLOCKED until P0 sign-off.**

---

## @designer (10 rows — authority, no Python)

| Phase | ID | Depends | Deliverable |
|:---|:---|:---|:---|
| **P0** | **OVR-P0-DESIGN-LOCK** | sweeps | `aps_design_system_v1.md` |
| P2 | OVR-DES-P2-COPY-PACK-001 | P0 | `design_aps_uiux_copy_pack_v1.md` |
| P3 | OVR-DES-P3-LAYOUT-DELTA-001 | P0 | `design_aps_uiux_layout_delta_v1.md` |
| P4 | OVR-DES-P4-IA-SIGN-001 | P0 | `design_aps_uiux_ia_sign_v1.md` |
| P4.5 | OVR-DES-P45-SPINE-SPEC-001 | P0 + P4 sign | `design_aps_uiux_spine_spec_v1.md` |
| P5.5 | OVR-DES-P55-PREVIEW-SPEC-001 | P0 | `design_aps_uiux_preview_spec_v1.md` |
| P5.6 | OVR-DES-P56-ONBOARD-SPEC-001 | P0 + spine spec | `design_aps_uiux_onboard_spec_v1.md` |
| P6 | OVR-P6-DESIGN-SIGN-001 | P6 close + operator | `design_aps_uiux_overhaul_signoff_v1.md` |

**P0 must include:** terminology glossary · token spec · IA contract (Buildings reorder, Stamp fold) · status atom · voice rules · ban-list.

---

## @coder-mcp (10 rows — sequential)

| Phase | ID | Depends | Key files | New guards |
|:---|:---|:---|:---|:---|
| P1 | OVR-P1-TOKENS-001 | P0 | `aps_theme.py` | `test_aps_style_tokens.py`, font floor ≤8 |
| P2 | OVR-P2-TEXT-001 | P1 + copy pack | all panels, `aps_tooltips.py` | `test_aps_no_jargon.py` |
| P3 | OVR-P3-LAYOUT-001 | P2 + layout delta | `app.py`, `scrollable.py`, `footprint_canvas.py` | `test_aps_min_window_layout.py` |
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
| P0+ | DMCP-OVR-G0-AUDIT-001 | After P0 draft | G0 audit: ban-list + voice rules |
| P6 | DMCP-OVR-ARTIST-ACCEPT-001 | After design sign | Artist re-verdict post-overhaul |

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
| ORCH-OVR-001 | Program dispatch |
| ORCH-OVR-FILELOCK-001 | Per-phase file lock enforcement |

---

## @operator (1 row — human gate)

| ID | When | Task |
|:---|:---|:---|
| OVR-P6-OPERATOR-EYEBALL-001 | After P6 witness | Walk both lanes 1280×800 + MIN 960×600 |

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
[/PLAN-APS-UIUX-OVERHAUL-001] P0 @designer NOW · @coder-mcp blocked until aps_design_system_v1.md
```
