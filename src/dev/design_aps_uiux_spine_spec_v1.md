# APS UI/UX Pipeline Spine Spec `v1` — OVR-DES-P45-SPINE-SPEC-001

| Field | Value |
|:---|:---|
| **ID** | **OVR-DES-P45-SPINE-SPEC-001** |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Phase** | P4.5 (pipeline spine) |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §4.4 |
| **Inputs** | [`design_aps_uiux_ia_sign_v1.md`](design_aps_uiux_ia_sign_v1.md) · [`design_aps_uiux_layout_delta_v1.md`](design_aps_uiux_layout_delta_v1.md) · [`design_aps_uiux_copy_pack_v1.md`](design_aps_uiux_copy_pack_v1.md) · [`aps_sweep_workflow_tooltips_vibe_20260616_v1.md`](aps_sweep_workflow_tooltips_vibe_20260616_v1.md) §1 |
| **Implements** | `OVR-P45-SPINE-001` |
| **Verdict** | **PASS** — spine interaction signed for `@coder-mcp` |

```text
OVR-DES-P45-SPINE-SPEC-001 Q✓
Unblocks: OVR-P45-SPINE-001 · OVR-DES-P56-ONBOARD-SPEC-001 (finalize after spine lands)
```

---

## 0. Problem statement

Today the artist sees **three competing flow models**:

| Band | What it does today | Failure |
|:---|:---|:---|
| Pipeline pills | Reports readiness | **Not clickable** — looks like a stepper, isn't one |
| Flow verbs | Runs actions | **Always enabled** — fail only as red hint at row end |
| Per-panel "Next:" | Assembly only | **Orphan guidance** — not lane-wide |

Landscape still exposes a **Stamp** pill with no tab (pre-P4). Flow verbs can run **hidden multi-step** work (`Bake variants`) without narration.

**Spine goal:** one authoritative band answers **where am I · what's done · what's next · how do I advance**.

---

## 1. Chrome placement (inherits P3)

Spine lives on **Row 2** after P3 merge. Do not add a third chrome row.

```text
┌─ Row 1 ─────────────────────────────────────────────────────────────┐
│ [ ▣ Buildings ] [ Landscape ]     Next: [primary verb]  [verb] [verb] │
├─ Row 2 ─────────────────────────────────────────────────────────────┤
│ What ships: …     Pipeline: [▣✓ pill][pill][pill][pill][pill]  hint │
└─────────────────────────────────────────────────────────────────────┘
```

| Element | Row | Notes |
|:---|:---:|:---|
| Lane chips | 1 | `▣` on active lane (same glyph family as current-step pill) |
| Flow verbs | 1 | **One primary** enabled; rest disabled with reason on click |
| Authority strip | 2 | Left; truncated with tooltip if needed |
| Pipeline pills | 2 | Clickable; `▣` marks active **tab** step |
| Lane hint | 2 | Right; copy from copy pack §1 |

**Retire:** standalone flow bar row, standalone authority row, engineer caveat under flow (`rust_engine_mcp…`).

---

## 2. Pipeline keys (locked — must match P4 IA)

Keys **=== tab keys**, same order.

### Buildings (5)

```text
catalog · materials · assembly · variants · atlas
```

### Landscape (4)

```text
presets · grammar · states · atlas
```

| Rule | Detail |
|:---|:---|
| No Stamp pill | Register is Atlas terminal sub-state: `✓ Atlas registered` |
| Tab click | `notebook.select(index_for_key)` |
| Lane switch | Rebuild pills + verbs; no cross-lane step bleed |

**Live code drift:** `PIPELINE_STEPS_BY_LANE` buildings order and tab labels must be corrected in P4 before spine wires click handlers.

---

## 3. Pill interaction (S1)

### 3.1 Click behavior

| Gesture | Result |
|:---|:---|
| Click pill | Select owning tab; set pill as **current** (`▣`) |
| Click current pill again | No-op (no flash / no re-fetch) |
| Click pill on other lane | **Forbidden** — pills are lane-scoped; switch lane first |

Bind: `<Button-1>` on pill frame **and** label. Cursor: `hand2` when hoverable.

### 3.2 Current-step marker `▣`

Distinct from status glyphs (`○ ◐ ✓ ✗`).

**Display template:** `{current?▣ :}{glyph} {StepLabel} {state_word}`

Examples:

```text
▣ ○ Catalog pending          ← user on Catalog, not started
  ✓ Materials valid          ← done, not current
▣ ◐ Assembly saved (not checked)
```

| Rule | Detail |
|:---|:---|
| Source of truth | Active notebook tab index → pipeline key |
| On tab change | Refresh `▣` without re-running readiness |
| On lane change | Recompute from new notebook + lane steps |

### 3.3 Status words (P2 copy)

Use [`design_aps_uiux_copy_pack_v1.md`](design_aps_uiux_copy_pack_v1.md) + [`design_aps_pipeline_pills_v1.md`](design_aps_pipeline_pills_v1.md) validity rules with these **user-visible** overrides:

| Internal key | Display word |
|:---|:---|
| `saved_qc_not_run` | `saved (not checked)` |
| `atlas_packed` | `packed (not checked)` |
| `presets_loaded` | `loaded (not checked)` |
| `grammar_saved` | `saved (not checked)` |

Landscape Atlas terminal when registered:

```text
✓ Atlas registered
```

Buildings Atlas terminal when registered:

```text
✓ Tiles registered
```

---

## 4. Flow verbs = spine advance (S2, S4)

### 4.1 Verb ↔ step mapping

| Lane | Verb key | Label (P2) | Advances step | Owning tab |
|:---|:---|:---|:---|:---|
| Buildings | `send_to_assembly` | Send to Assembly | `catalog` → `assembly` | Catalog |
| Buildings | `bake_variants` | Bake variants | `variants` | Variants |
| Buildings | `pack_atlas` | Pack atlas | `atlas` | Atlas |
| Landscape | `generate_grammar` | Generate grammar | `grammar` | Grammar |
| Landscape | `bake_states` | Bake states | `states` | States |
| Landscape | `pack_lg5_atlas` | Pack landscape atlas | `atlas` (+ register) | Atlas |

### 4.2 Enablement rules

| Rule | Detail |
|:---|:---|
| **One primary** | Exactly one verb `state=NORMAL` when prerequisites pass |
| Primary selection | First verb in lane order whose **target step** is the earliest incomplete step **at or after** current tab step |
| Disabled verbs | `state=DISABLED`; click shows prerequisite string inline (existing `flow_prerequisite_message`) |
| Wrong lane | Verbs hidden with lane frame — not disabled on wrong lane |

**Primary styling:** default button weight (not a new color token). Secondary verbs: same row, visually de-emphasized via `disabled` not smaller font.

### 4.3 Prerequisite strings (artist voice)

Reuse `aps_inline_feedback.flow_prerequisite_message` — ensure P2 copy pass removes internal gate names. Examples:

| Action blocked | Message |
|:---|:---|
| Send to Assembly, no module | `Select a module in Catalog first.` |
| Bake variants, assembly not checked | `Run Ship check on Assembly before baking variants.` |
| Pack atlas, no variants | `Create variant layers on the Variants tab first.` |
| Generate grammar, preset invalid | `Fix preset errors on Presets before generating grammar.` |

Show message in **Row 1** right of verbs (`FONT_HINT`, fail fg) — same slot as today `_flow_hint_var`.

### 4.4 Multi-step narration (S4 — `bake_variants`)

`Bake variants` may run **more than one** backend call. Required UX:

```text
⟳ Baking variants…
⟳ Building tile batch…
✓ Variants baked — open Atlas to pack tiles.
```

| Rule | Detail |
|:---|:---|
| Log + inline | Each sub-step appended to status log **and** transient Row-1 acknowledgement |
| No silent work | If >1 CLI invocation, artist sees ≥2 progress lines |
| On success | Variants pill → `valid`; **do not** auto-switch to Atlas |
| On partial fail | Variants pill → `fail`; verb stays primary with fail detail |

Same pattern for `pack_lg5_atlas` when register is a second step.

---

## 5. Advance-on-completion (S3)

When a step's readiness flips to `valid`:

| Do | Don't |
|:---|:---|
| Update pill glyph/word | Auto-switch notebook tab |
| Recompute **primary verb** for next incomplete step | Flash modal |
| Optionally pulse next pill border once (200ms) | Steal focus from in-progress edit |

**Next step line (Row 1):** when no verb is enabled, show muted hint:

```text
Next: Assign materials on the Materials tab.
```

Derive from first `pending` or `saved (not checked)` step **after** current. Hide when all steps `valid`.

**Retire:** `assembly_panel.next_step_var` flow copy — keep **contextual** material-assign hints only (not lane spine).

---

## 6. Tooltips (≤16 words)

| Key | Text |
|:---|:---|
| `pipeline_catalog` | `Pick a building module. Opens the Catalog tab.` |
| `pipeline_materials` | `Create and edit material profiles used on pieces.` |
| `pipeline_assembly` | `Place pieces, assign materials, run Ship check.` |
| `pipeline_variants` | `Set damage, lighting, and fill layers for baking.` |
| `pipeline_atlas` | `Pack tiles and register them for the game.` |
| `pipeline_presets` | `Choose a landscape preset and fix schema errors.` |
| `pipeline_grammar` | `Edit layout graph nodes and corridors.` |
| `pipeline_states` | `Author growth and fire states for vegetation.` |
| `spine_current` | `You are on this step.` |
| `spine_click` | `Go to this step.` |
| `flow_primary` | `Recommended next action for this lane.` |

---

## 7. Readiness model (coder reference)

Spine **reads** existing `PipelineStatusBar._refresh_*` signals — do not fork a second readiness resource.

| Buildings step | `valid` when (summary) |
|:---|:---|
| catalog | module selected |
| materials | all placements have material |
| assembly | ship check passed |
| variants | variant set present + valid |
| atlas | QC pass + registered |

| Landscape step | `valid` when (summary) |
|:---|:---|
| presets | preset validates |
| grammar | grammar saved on valid preset |
| states | succession/disturbance rows ready |
| atlas | packed + registered (stamp folded) |

---

## 8. Implementation checklist (`OVR-P45-SPINE-001`)

### `pipeline_status_bar.py`

- [ ] `tab_index_for_step(key) -> int` via `domain_router`
- [ ] Pill click → callback `on_spine_select_tab(key)` provided by `app.py`
- [ ] Render `▣` prefix on current step pill
- [ ] `hand2` cursor; bind click on pill frame + label
- [ ] `refresh_current_marker(active_tab_key: str)`
- [ ] Row-2 layout: authority left, pills center, hint right

### `app.py`

- [ ] Wire `on_spine_select_tab` → `notebook.select`
- [ ] Merge flow verbs to Row 1; single `_flow_hint_var`
- [ ] `refresh_spine_primary_verb()` on state change + tab change
- [ ] `on_bake_variants` / `pack_lg5_atlas` narration hooks
- [ ] Tab change handler updates spine current marker

### `domain_router.py`

- [ ] Buildings pipeline order = IA order (materials before assembly)
- [ ] Landscape pipeline: drop `stamp` key
- [ ] `step_for_tab_index(lane, index) -> key`
- [ ] `verify_option_d_ia_contract()` — 4 landscape keys

### Tests

| Guard | Asserts |
|:---|:---|
| `test_aps_runtime_callbacks.py` | pill click → tab index; each key maps to tab |
| `test_aps_spine_primary_verb.py` (new) | exactly one enabled verb per fixture state |
| `test_aps_spine_no_autoswitch.py` (new) | completing assembly does not change notebook index |
| IA guards | pipeline keys === tab keys per lane |

---

## 9. Acceptance (design)

- [ ] Artist can reach every tab by clicking its pill
- [ ] `▣` always matches active tab
- [ ] Exactly one primary verb when work remains; disabled verbs explain why
- [ ] No auto-tab-switch on step completion
- [ ] `Bake variants` never silent multi-step
- [ ] Landscape has **4** pills; Atlas shows registered terminal state
- [ ] No gate IDs / schema names in spine strings (P2 ban-list)

**Feel gate:** NEEDS-DISPLAY — operator walks both lanes after P4.5.

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |

```text
OVR-DES-P45-SPINE-SPEC-001 Q✓ — spine interaction locked for OVR-P45-SPINE-001
```
