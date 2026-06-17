# PLAN-APS-UIUX-OVERHAUL — drain & finish playbook `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Purpose** | Single doc to **drain and close** the APS UI/UX refactor — status, session prompts, exit gates |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) · [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) |
| **Queue** | [`aps_uiux_overhaul_queue.json`](../tools/orchestrator/queues/aps_uiux_overhaul_queue.json) |
| **Parallel** | [`aps_uiux_overhaul_parallel_drain_v1.json`](../tools/orchestrator/queues/aps_uiux_overhaul_parallel_drain_v1.json) |
| **Witness (close)** | `debug_runs/aps_uiux_overhaul_close_live.json` · profile [`plan_aps_uiux_overhaul_witness_v1.md`](plan_aps_uiux_overhaul_witness_v1.md) |

---

## 1. Snapshot — where we are

**Progress:** ~**13 / 24** main-queue rows done (**~54%**) · implementation **~40%** (P0–P2 shipped; P3–P6 not)

**Regression:** `cd tools/mcp/python && python -m pytest -k aps -q` → **161 passed** (baseline at open: 149)

### Done (design + early impl)

| Layer | Delivered |
|:---|:---|
| P0 gate | `aps_design_system_v1.md` |
| P1 tokens | `aps_theme.py` + font/style guards |
| P2 text | copy pack + `test_aps_no_jargon.py` |
| Designer specs | layout delta, IA sign, spine spec, preview spec, onboard outline, P2 audit, P3 rubric |
| Infra | queue registry, witness profile, G0 audit, file-lock dispatch |

### Not done (the finish line)

| Phase | Row | Blocker | Guard / proof |
|:---|:---|:---|:---|
| **P3** | OVR-P3-LAYOUT-001 | **★ NEXT** | `test_aps_min_window_layout.py` (missing) |
| P4 | OVR-P4-IA-001 | P3 | tab reorder, Stamp→Atlas, dead catalog path |
| P4.5 | OVR-P45-SPINE-001 | P4 | clickable pipeline spine |
| P5 | OVR-P5-STYLE-001 | P4.5 | `status_atom()` everywhere, tk→ttk |
| P5.5 | OVR-P55-PREVIEW-001 | P5 | 4 preview states, never black |
| P5.6 | OVR-P56-ONBOARD-001 | P5.5 + onboard spec | `test_aps_onboarding.py` |
| P6 | OVR-P6-CLOSE-001 | P5.6 | full pytest + WIT-HON witness |
| Human | operator eyeball → design sign → artist accept | P6 witness | NEEDS-DISPLAY |

**Designer parallel (now):** `OVR-DES-P56-ONBOARD-SPEC-001` → `design_aps_uiux_onboard_spec_v1.md` (does not block P3–P5)

---

## 2. Non-negotiable rules

1. **One `@coder-mcp` phase active** — commit between phases; never parallel on `app.py`.
2. **`app.py` exclusive writer:** P3, P4, P4.5 only (when that phase is active).
3. **Every coder-mcp session exit:** `pytest -k aps` + `test_aps_imports.py` + `test_aps_runtime_callbacks.py` green.
4. **Visual feel:** flag **NEEDS-DISPLAY** — no Q✓ on pixels without operator.
5. **Python 3.14** with Pillow — not `py -3.13`.
6. **Scope:** Tk UI/UX only — not grammar content quality, not Bevy registry (separate lanes).

---

## 3. Critical path (7 coder-mcp sessions)

```text
P3 layout → P4 IA → P4.5 spine → P5 style → P5.5 preview → P5.6 onboard → P6 close
    ↓
operator eyeball → @designer sign-off → @designer-mcp artist accept
```

**Estimated:** 7 focused `@coder-mcp` sessions + 1 operator walk + 2 sign-offs.

---

## 4. Session prompts — copy-paste per phase

Use **one chat per session** with `@coder-mcp`. Attach this file + the phase spec.

### Session A — P3 Layout ★ START HERE

```text
Program: PLAN-APS-UIUX-OVERHAUL-001
Row: OVR-P3-LAYOUT-001
FILE LOCK: app.py, scrollable.py, footprint_canvas.py ONLY

Read:
- src/dev/aps_design_system_v1.md
- src/dev/design_aps_uiux_layout_delta_v1.md
- src/dev/plan_aps_uiux_p3_layout_guard_v1.md

Do:
- Collapse chrome bands; footprint grid above fold at 1280×800
- MIN window 960×600: no forced horizontal scroll; panes respect PANE_MIN_*
- Migrate hardcoded padding to GAP_/INSET_ tokens from aps_theme.py

Add guard: tools/mcp/python/tests/test_aps_min_window_layout.py

Exit:
- pytest tools/mcp/python/tests/test_aps_min_window_layout.py -q
- pytest tools/mcp/python/tests -k aps -q
- Mark queue row done; commit; hand off P4
```

### Session B — P4 IA

```text
Row: OVR-P4-IA-001
FILE LOCK: domain_router.py, app.py (tabs), catalog.py

Read: src/dev/design_aps_uiux_ia_sign_v1.md

Do:
- R1: fold Landscape Stamp into Atlas terminal state (5→4 pipeline keys)
- R2: remove dead landscape path in Buildings Catalog
- Buildings tab order: Catalog → Materials → Assembly → Variants → Atlas
- Unify material vocabulary across tabs (one canonical term set)

Exit:
- pytest tools/mcp/python/tests/test_aps_lane_tab_swap.py -q
- pytest tools/mcp/python/tests/test_aps_runtime_callbacks.py -q
- pytest -k aps -q
```

### Session C — P4.5 Pipeline spine

```text
Row: OVR-P45-SPINE-001
FILE LOCK: pipeline_status_bar.py, app.py (flow verbs)

Read: src/dev/design_aps_uiux_spine_spec_v1.md

Do:
- Clickable pipeline pills → navigate to owning tab
- ▣ marks current step; advance verbs gated on readiness
- No auto-tab-switch on lane change; narrate on_bake_variants

Exit:
- test_aps_runtime_callbacks.py (spine navigation paths)
- pytest -k aps -q
```

### Session D — P5 Style unification

```text
Row: OVR-P5-STYLE-001
Territory: aps_inline_feedback.py, status sites, tk→ttk where safe

Do:
- status_atom() everywhere: {glyph} {word} [— detail]
- Fix PASS-in-blue; ban ● material dialect
- Section component using tokens only

Exit:
- test_aps_style_tokens.py extended for status atom
- pytest -k aps -q
```

### Session E — P5.5 Preview

```text
Row: OVR-P55-PREVIEW-001
Read: src/dev/design_aps_uiux_preview_spec_v1.md

Territory: slot_preview_panel.py, assembly_preview_panel.py, atlas_preview_panel.py, aps_slot_preview.py

Do:
- 4 states: empty / loading / ready / error — never black/blank
- Fidelity labels; async update-on-select

Exit:
- runtime_callbacks green
- pytest -k aps -q
- Flag NEEDS-DISPLAY for operator pixel check
```

### Session F — P5.6 Onboarding

```text
Row: OVR-P56-ONBOARD-001
Read: src/dev/design_aps_uiux_onboard_spec_v1.md (must exist — @designer if missing)

Territory: metadata_flow_panel.py, state.py, panel empty states

Do:
- First-run: plain "how this works" — NOT auto-expanded schema diagram
- Friendly empty state per primary tab
- metadata_flow default collapsed; dismiss remembered in state.py

Add: test_aps_onboarding.py

Exit:
- pytest test_aps_onboarding.py -q
- pytest -k aps -q
- NEEDS-DISPLAY for first-run feel
```

### Session G — P6 Close

```text
Row: OVR-P6-CLOSE-001

Do:
- Full pytest -k aps green
- Refresh debug_runs/aps_uiux_overhaul_close_live.json per plan_aps_uiux_overhaul_witness_v1.md
- WIT-HON: validate-report witness_honesty debug_runs/aps_uiux_overhaul_close_live.json --compress 3
- Update queue: all P1–P5.6 rows done

Do NOT self-certify pixels — list NEEDS-DISPLAY rows for operator
```

---

## 5. Parallel work (while coder-mcp runs P3+)

| Agent | Row | Deliverable |
|:---|:---|:---|
| **@designer** | OVR-DES-P56-ONBOARD-SPEC-001 | `design_aps_uiux_onboard_spec_v1.md` |
| **@designer-mcp** | DMCP-OVR-P3-ACCEPT-RUBRIC-001 | Use rubric at P3 Q✓ eyeball |
| **@sim-steward** | STEWARD-OVR-APS-REGRESS-001 | pytest -k aps after each phase |
| **@orchestrator-mcp** | track file lock + queue row updates | `aps_uiux_overhaul_dispatch_live.json` |

**@coder A / @coder B:** no APS Tk scope — use parallel drain for Bevy lanes only.

---

## 6. Close ceremony (after Session G)

### Step 1 — Operator eyeball

```text
Row: OVR-P6-OPERATOR-EYEBALL-001
Launch: python tools/mcp/art_pipeline_suite/run.py
Walk: Buildings + Landscape at 1280×800 and MIN 960×600
Check: text, layout, tabs, spine, preview, onboarding vs aps_design_system_v1.md
Record verdict in witness needs_display[]
```

### Step 2 — Designer sign-off

```text
Row: OVR-P6-DESIGN-SIGN-001
Deliverable: src/dev/design_aps_uiux_overhaul_signoff_v1.md
Compare: north star + P0 design system + operator notes
```

### Step 3 — Artist accept

```text
Row: DMCP-OVR-ARTIST-ACCEPT-001
Deliverable: src/dev/design_aps_artist_ship_review_uiux_v1.md
Witness: debug_runs/art_pipeline/dmcp_ovr_artist_accept_live.json
Supersedes: prior 7/10 building-only score
```

---

## 7. Definition of done (program close)

- [ ] P3–P6 coder-mcp rows **done** with guard tests green
- [ ] Buildings tabs: Catalog → Materials → Assembly → Variants → Atlas
- [ ] Landscape: Stamp folded into Atlas
- [ ] Clickable pipeline spine is sole "where am I / what's next"
- [ ] MIN 960×600 usable; footprint visible at 1280×800
- [ ] One status atom everywhere; no jargon/gate IDs in visible strings
- [ ] Preview: 4 states, never black
- [ ] First-run: how-it-works, not schema dump
- [ ] `pytest -k aps` green + WIT-HON pass on close witness
- [ ] Operator eyeball recorded
- [ ] Designer sign-off + artist accept

---

## 8. Mega-prompt — drain to finish (orchestrator / parent chat)

Copy this into a **parent orchestrator chat** to drive the full drain:

```text
Mission: DRAIN AND CLOSE PLAN-APS-UIUX-OVERHAUL-001.

Authority:
- src/dev/plan_aps_uiux_overhaul_drain_finish_v1.md (this playbook)
- src/dev/aps_design_system_v1.md
- tools/orchestrator/queues/aps_uiux_overhaul_queue.json

Current: P0–P2 done. NEXT = OVR-P3-LAYOUT-001.

Rules:
- @coder-mcp: ONE phase per session, P3→P6 sequential, commit between each
- app.py: P3/P4/P4.5 only when active
- pytest -k aps + imports + runtime_callbacks every exit
- NEEDS-DISPLAY for pixels — operator at P6

Parallel now:
- @designer: OVR-DES-P56-ONBOARD-SPEC-001 (onboard spec — finish before Session F)

Execute Session A (P3) in @coder-mcp chat. When green, Session B (P4). Continue through Session G.
After P6 witness: operator eyeball → designer sign → designer-mcp artist accept.

Mark each queue row done. Refresh debug_runs/agent_ops/aps_uiux_overhaul_dispatch_live.json after each phase.
Do not reopen APS-OPTION-D unless regression fails.
```

---

## 9. Quick reference — spec map

| Phase | Spec |
|:---|:---|
| P0 | `aps_design_system_v1.md` |
| P2 copy | `design_aps_uiux_copy_pack_v1.md` |
| P3 layout | `design_aps_uiux_layout_delta_v1.md` · guard `plan_aps_uiux_p3_layout_guard_v1.md` |
| P4 IA | `design_aps_uiux_ia_sign_v1.md` |
| P4.5 spine | `design_aps_uiux_spine_spec_v1.md` |
| P5.5 preview | `design_aps_uiux_preview_spec_v1.md` |
| P5.6 onboard | `design_aps_uiux_onboard_spec_v1.md` (pending) · outline `design_aps_uiux_onboard_outline_v1.md` |
| P6 witness | `plan_aps_uiux_overhaul_witness_v1.md` |

---

```text
[/DRAIN-FINISH] ★ Session A @coder-mcp OVR-P3-LAYOUT-001 · @designer onboard spec parallel · 7 sessions to close
```
