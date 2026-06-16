# Tools UX polish charter `v1` — interaction rework

| Field | Value |
|:---|:---|
| **Program** | **TOOLS-UX-POLISH-001** |
| **Date** | 2026-06-03 |
| **Owner** | `@designer` (charter) · `@coder-mcp` (APS Tk) · `@coder` (egui dev tools) |
| **Scope** | Art Pipeline Suite (Tk) · egui dev panels · map editor chrome |
| **Prior audit** | [`design_aps_ux_audit_v1.md`](design_aps_ux_audit_v1.md) — **PASS** on copy/a11y P0; **this doc** addresses **interaction polish** |
| **Verdict** | **CHARTER APPROVED** — phased rework recommended |

---

## Executive summary

Your read is accurate: the tools are **functionally complete** but **interaction quality lags professional authoring products**. Phase 5 fixed validation colors and status text; what remains is the “janky widget” layer — **blocking modals**, **UI-thread CLI work**, **nested scroll fights**, **layout overload**, and **inconsistent feedback**.

This charter translates industry desktop-tool UX patterns into a **concrete rework plan** for this repo. Goal: *feel like a shipped internal tool* (Substance/Blender panel discipline, VS Code async jobs, Figma-style non-blocking flows) — not a prototype Tk shell.

---

## Surfaces in scope

| Surface | Stack | Primary users | Polish priority |
|:---|:---|:---|:---:|
| **Art Pipeline Suite** | Tkinter + ttk | Artists, designer-mcp | **P0** |
| **Assembly snapshot QC** | egui (Bevy) | Devs, QA | P1 |
| **Diagnostics (F3)** | egui | Devs | P2 |
| **Map editor chrome** | egui (TEMP) | Designers, mappers | P2 — migrate to Bevy UI per existing note |

**Out of scope:** Player simulation HUD (SIM-HUD lane), MCP CLI-only workflows, Blender itself.

---

## Research — what “professional polish” means for authoring tools

Synthesis from async UX literature ([LogRocket async patterns](https://blog.logrocket.com/ux-design/ui-patterns-for-async-workflows-background-jobs-and-data-pipelines/), [Stack Exchange blocking UI](https://ux.stackexchange.com/questions/43414/what-are-best-practices-for-blocking-ui-when-application-is-busy), agentic status surfaces) and desktop product norms:

### 1. Responsiveness contract

| Duration | Pattern | Anti-pattern |
|:---|:---|:---|
| **< 100ms** | Instant visual ack (button depress, selection highlight) | No feedback |
| **100ms – 2s** | Inline spinner on **trigger control only** + disable that control | Full-window modal |
| **2s – 60s** | **Job strip** / status drawer: step label, cancel, allow tab switch | `messagebox` + frozen window |
| **> 60s** | Background job + notification on complete; restore state on relaunch | Blocking until done |

**Rule:** Never call synchronous MCP/CLI on the Tk **main thread** without a busy shell. APS violates this today on Generate, Pack, Validate, tile batch.

### 2. Non-blocking feedback hierarchy

Replace modal-first with this stack (top = preferred):

```text
1. Inline status line (bottom bar — already exists, underused)
2. Inline panel result (validation label — now color-aware ✓)
3. Toast / non-modal banner (3–5s, dismissible)
4. Side job drawer (long ops — new)
5. Modal (only irreversible / data-loss)
```

**Current APS:** steps 4–5 used for routine success/failure (`messagebox.showinfo` on Generate OK, Atlas pack, etc.).

### 3. Progressive disclosure (density without scroll fatigue)

Professional panels use **three tiers**:

| Tier | Content | Default |
|:---|:---|:---:|
| **Primary** | One job + one outcome per viewport | Visible |
| **Secondary** | Inspectors, previews, filters | Visible or one accordion |
| **Advanced** | Agent patch, lod0 smoke, raw JSON | **Collapsed** |

**Current APS Assembly tab:** Scroll fatigue from always-expanded advanced sections — **fixed** via collapsibles (density-001). **Design reviews at 1280×800 @ 1080p**, not at minimum size.

Reference in-repo: **WorldGen UI** already shows the target pattern — progress bar + disabled controls while busy (`world_gen_ui.rs`).

**Viewport tiers:** Design target **1280×800** (default launch). Comfortable max **1440×900**. Minimum **960×600** = regression floor only. See [`aps_ux_professional_polish_rules_v1.md`](aps_ux_professional_polish_rules_v1.md) §0.

### 4. Scroll ownership (one scroller per viewport)

| Rule | Rationale |
|:---|:---|
| **One vertical scroll container per tab** | Nested `ScrollableFrame` + inner `Canvas` → wheel events fight |
| **Sticky section headers** inside scroll | User never loses context in tag pickers |
| **Focus-aware wheel routing** | Wheel scrolls the widget under cursor, not always outer tab |

**Current APS:** `app.py` wraps every tab in `ScrollableFrame`; Catalog list and Atlas cell strip add inner canvases without coordinated wheel delegation.

### 5. Interaction states (design system minimum)

Every clickable control should have explicit states:

| State | Visual |
|:---|:---|
| default | Base ttk / palette token |
| hover | Subtle fill (ttk theme or `#f5f5f5`) |
| active/pressed | Inset or darker fill |
| disabled | 50% opacity + `cursor=` |
| busy | Spinner glyph + “Running…” label on **same button** |
| error | Red border or left accent + text (not color-only) |

**Current APS:** busy state **missing**; disabled only during implicit freeze when main thread blocked.

### 6. Typography & spacing tokens

| Token | Use | Current drift |
|:---|:---|:---|
| **UI body** | Segoe UI **10–11pt** | Mixed 8–9 on meta lines |
| **Data mono** | Consolas **9pt** min for paths/JSON | Consolas **8** on maps, mesh, atlas meta |
| **Section gap** | 8px between groups, 16px between regions | Ad-hoc `pady=2/4/8` |
| **Wraplength** | Bind to parent width on `<Configure>` | Fixed 420 / 680 / 900 |

---

## Code-backed pain catalog (post Phase 5)

### P0 — “feels janky” (user-visible)

| # | Symptom | Root cause | Files |
|:---:|:---|:---|:---|
| J1 | Window **freezes** on Generate / Pack / Validate | Sync MCP on UI thread | `assembly_panel.py` `on_generate`, `atlas_panel.py` pack/batch |
| J2 | **Modal spam** after routine actions | Success = `messagebox.showinfo` | `assembly_panel.py`, `catalog.py`, `variants_panel.py`, `atlas_panel.py` |
| J3 | **Scroll doesn’t work** on module list / nested panels | Nested scroll without focus routing | `scrollable.py`, `catalog.py` |
| J4 | Assembly tab **endless scroll** | No accordion; grammar + tags always open | `assembly_panel.py` |
| J5 | Flow **Bake variants** jumps tabs + runs work silently | No busy state; weak save guard | `app.py` `on_bake_variants` |
| J6 | Preview / render **hangs** with no spinner | Inline render on select | `slot_preview_panel.py`, `assembly_preview_panel.py` |

### P1 — polish gaps

| # | Symptom | Files |
|:---:|:---|:---|
| P1 | QC panel: load blocks frame; spawn gives hint only | `assembly_snapshot_qc_ui.rs` |
| P1 | Diagnostics: one infinite scroll; sections buried in sim | `diagnostics_ui.rs` |
| P1 | Map editor: fixed window sizes; scroll vs zoom conflict | `map_editor/mod.rs` |
| P1 | Material cards: glyph + color + text (triple encode) | `material_library_widget.py` |
| P1 | Grammar tree still shows raw `rule_id` column | `grammar_inspector.py` |

### P2 — consistency

| # | Symptom |
|:---:|:---|
| P2 | Hardcoded hex colors scattered (`#0a4a7a`, `#ffffe0` tooltips) |
| P2 | `pack` + `grid` mixed in same panel |
| P2 | Platform-specific `os.startfile` without fallback copy |

---

## Target interaction model — APS

### Shell layout (wireframe)

```text
┌─ Flow bar ────────────────────────────────────────────────────────────────┐
│ [Send to Assembly] [Bake variants] [Pack atlas]     Pipeline: ○→○→✓→○→○ │
├─ Job strip (NEW — only when busy) ────────────────────────────────────────┤
│ ⟳ Packing atlas… step 2/3 · tilemapgen  [Cancel]                            │
├─ Tabs ────────────────────────────────────────────────────────────────────┤
│ Catalog | Assembly | Materials | Variants | Atlas                         │
│ ┌─ ONE scroll region per tab ─────────────────────────────────────────┐ │
│ │ [Primary actions]                                                     │ │
│ │ ▼ Inspector (collapsible)                                             │ │
│ │ ▼ Advanced (collapsed default)                                        │ │
│ └───────────────────────────────────────────────────────────────────────┘ │
├─ Status log (persistent, not truncated harshly) ──────────────────────────┤
│ Ready · last: catalog select: warehouse_wall_01                           │
└───────────────────────────────────────────────────────────────────────────┘
```

### Job strip spec (new component)

| Field | Behavior |
|:---|:---|
| Visibility | Shown when any background job running |
| Content | `{verb} {object}…` + optional step `2/3` + elapsed |
| Cancel | Sends cancel token to worker thread |
| Tab switch | **Allowed** — job continues; strip stays visible |
| Complete | Strip → green check 3s → hide; result also in status log |

**Implementation sketch (@coder-mcp):** `threading.Thread` + `queue.Queue` for MCP calls; main thread polls with `after(100, poll_jobs)`.

### Modal policy (replace matrix)

| Action | Today | Target |
|:---|:---|:---|
| Generate snapshot OK | Modal info | Inline validation PASS + status log |
| Generate P0 fail | Modal warning | Inline FAIL (red) + **Jump to field** link |
| Pack atlas OK | Modal | Job strip complete + atlas preview refresh |
| Validate GLB fail | Modal | Inline FAIL on catalog panel |
| Add material profile | Modal `grab_set` | Non-modal sheet or inline expand |
| Unsaved destructive exit | Modal confirm | **Keep modal** |

### Assembly tab IA rework

| Section | Default | Notes |
|:---|:---:|:---|
| Generate row + path | Open | Primary |
| Footprint + placement list | Open | Primary |
| Slot preview + material apply | Open | Primary |
| Semantic tags | **Collapsed** | Badge count when collapsed: `Tags (4)` |
| Grammar inspector | **Collapsed** | Show human label in header when collapsed |
| Validation result | Open | Already inline |

### Scroll fix pattern

1. **Remove** outer `ScrollableFrame` from tabs that have inner canvas lists **OR**
2. Implement **scroll focus chain**: on `<Enter>`, set `_active_scroll_target` on `ScrollableFrame`; wheel routes to active target only.

Prefer (2) — less layout churn.

---

## Target interaction model — egui dev tools

| Tool | Change |
|:---|:---|
| **QC HUD** | Load button → disabled + spinner; show preview PNG inline when spawn completes; footprint grid without spawn prerequisite |
| **Diagnostics** | Tab bar: Sim · Render · Weather · Construction · **Jobs**; move Assembly QC out of collapsed footer |
| **Map editor** | Single dock; busy flags on save/load/bake (mirror WorldGen); unify scroll/zoom modifier keys |

**Palette rule:** no raw hex in legend strings — use `UiPalette` tokens (QC grid already does this).

---

## Phased rework roadmap

| Phase | ID | Owner | Deliverable | Exit |
|:---:|:---|:---|:---|:---|
| **0** | **TOOLS-UX-CHARTER-001** | @designer | This doc | Charter approved |
| **1** | **APS-UX-ASYNC-001** | @coder-mcp | Job strip + thread wrapper for Generate/Pack/Validate | No UI freeze >500ms on those actions |
| **2** | **APS-UX-NONBLOCK-001** | @coder-mcp | Modal policy matrix; inline success/fail | ≤1 modal per user task on happy path |
| **3** | **APS-UX-SCROLL-001** | @coder-mcp | Scroll focus chain + catalog list wheel | Module list scrolls under cursor |
| **4** | **APS-UX-DENSITY-001** | @coder-mcp | Assembly accordions (tags, grammar) | **1280×800** primary path clear; **960×600** floor OK |
| **5** | **APS-UX-TOKENS-001** | @coder-mcp | `aps_ui_theme.py` — fonts, colors, spacing | No Consolas 8; wraplength binds to width |
| **6** | **EGUI-DEV-UX-001** | @coder | QC load/spawn feedback + diagnostics tabs | Witness JSON green |
| **7** | **TOOLS-UX-SIGNOFF-001** | @designer | Re-score interaction dimensions | PASS on responsiveness + scroll + modals |

**Dependency:** Phases 1–2 deliver the largest “jank” reduction; do not start token polish (5) before async (1).

---

## Acceptance tests (designer)

### APS — “professional polish” bar

| # | Test | Pass |
|:---:|:---|:---:|
| T1 | Generate snapshot: window stays responsive; button shows busy | |
| T2 | Pack atlas: user can switch to Materials tab while packing | |
| T3 | Catalog module list: wheel scrolls list when hover over list | |
| T4 | Assembly: tags + grammar collapsed by default @ **1280×800** | |
| T4b | Same path **usable** @ 960×600 floor (regression) | |
| T5 | Happy-path Generate: **zero** modals | |
| T6 | P0 fail: red inline text, no success-green flash | |
| T7 | Status log retains full last line (or “…expand log”) | |

### egui dev — bar

| # | Test | Pass |
|:---:|:---|:---:|
| T8 | QC load: button disabled + progress during parse | |
| T9 | Diagnostics: Assembly QC reachable in ≤2 clicks from open | |

---

## Score target (re-audit after Phase 1–4)

| Dimension | Audit v1 | Target v2 |
|:---|:---:|:---:|
| Clarity | 6 | 8 |
| Discoverability | 5 | 7 |
| Error recovery | 7 | 8 |
| Accessibility | 6 | 7 |
| Workflow efficiency | 6 | 8 |
| **Responsiveness (new)** | 3 | 8 |

---

## References

| Source | Apply to |
|:---|:---|
| In-repo WorldGen progress UI | APS job strip, map editor busy |
| [`design_aps_ux_audit_v1.md`](design_aps_ux_audit_v1.md) | Baseline + deferred items |
| [`aps_tooltip_copy_v1.md`](../prompts/designer_questions/aps_tooltip_copy_v1.md) | Bind after async (busy tooltips) |
| LogRocket async UI patterns | Job strip, partial failure summaries |
| VS Code / Blender panel discipline | Progressive disclosure, one primary task per pane |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **CHARTER APPROVED** | 2026-06-03 |

```text
TOOLS-UX-POLISH-001 charter complete
Priority: APS-UX-ASYNC-001 → APS-UX-NONBLOCK-001 (biggest jank win)
Handoff: @coder-mcp Phases 1–5 · @coder Phase 6
```
