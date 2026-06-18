# APS Smoothness Charter `v1` — Tk-realistic interaction

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-SMOOTHNESS-001** |
| **Program** | APS UI/UX phase 2 — artist 9/10 target |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Depends** | [`design_aps_design_system_v11_delta_v1.md`](design_aps_design_system_v11_delta_v1.md) |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §0 north star |
| **Verdict** | **PASS** |

```text
DES-APS-SMOOTHNESS-001 Q✓
Professional = calm + predictable — not animation
```

---

## 0. Tk ceiling

No CSS transitions, no 60fps animations, no modal wizards. Smoothness = **immediate feedback**, **stable layout**, **explained disabled states**, **preserved focus**.

---

## 1. Primary action feedback

| Rule | Spec |
|:---|:---|
| **Inline ack** | On click: button → `⟳ {verb}…` + `state=disabled` until job ends |
| **Origin** | Ack at **button** — not log-only |
| **Completion** | Re-enable + `✓ {past tense}` toast adjacent OR status atom on same row |
| **Failure** | `✗ {verb} failed — {fix}` at button row |

**Applies to:** Generate Assembly · Run ship check · Pack atlas · Validate · Preview assembly · Bake variants.

---

## 2. Disabled-why (adjacent, not silent)

| Pattern | Copy placement |
|:---|:---|
| Flow verb disabled | `pipeline_status_bar` `_advance_blocked_var`: `✗ {reason}` |
| Flow verb prereq | Same row — `flow_prerequisite_message` text |
| Greyed button | Tooltip OR 1-line label **below** button group — never log-only |
| Combo empty | Empty-state from onboard spec §3 |

**Ban:** disabled control with no explanation within 1 row.

---

## 3. No layout jump

| Surface | Rule |
|:---|:---|
| Validation banner | Reserve **min height 24px** row; empty = invisible spacer |
| Kit hint (G0) | Fixed single line + tooltip for full text at MIN |
| Status log expand | Capped height — notebook keeps priority ([`design_aps_uiux_layout_delta_v1.md`](design_aps_uiux_layout_delta_v1.md) SH-5) |
| Collapsible expand | Body packs below header — no reflow of siblings sideways |
| Pipeline refresh | Pill width stable — text swap only, no repack of chrome |

---

## 4. Debounce & async

| Interaction | Delay | Behavior |
|:---|:---:|:---|
| Material search filter | **300ms** | Show `⟳ filtering…` in search row if >150ms |
| Profile card select → preview | **150ms** | `○ Rendering…` in preview pane |
| Assembly piece select → 2×2 thumbs | **0ms** show loading; worker async | stale job cancelled |
| Tab switch | **0** | Sync selection restore where possible |

**Ban:** UI freeze >100ms without `⟳` somewhere in the active panel.

---

## 5. Focus & selection persist

| Action | Rule |
|:---|:---|
| Tab switch same lane | Restore list selection + scroll position |
| Generate completes | Focus stays on Generate row; footprint gets visual flash (selection) |
| Modal `askyesno` | Return focus to invoking button |
| Lane switch | `clear_cross_lane_selection` — no cross-lane bleed |

---

## 6. Log & spine discipline

| Rule | Value |
|:---|:---|
| Status log default | collapsed |
| Status log max expanded | **5 lines** visible |
| Duplicate ack | If button shows `⟳`, log may omit duplicate start line |
| Spine | One `▣` current step — never two “you are here” indicators |

---

## 7. Grammar-tier smoothness

| Tier | Extra rule |
|:---|:---|
| G0 | Kit hint does not expand multi-line @ MIN |
| G1 unlock | Removing kit hint = `pack_forget` — **no** layout jump (reserve 0px) |
| G2+ | DNA panel expand = vertical only; no horizontal scroll |

Ref: [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) §10.

---

## 8. Verification

| Check | Method |
|:---|:---|
| Button ack | `test_aps_runtime_callbacks.py` — running state set |
| Disabled why | flow prereq returns non-empty when blocked |
| Layout | `test_aps_min_window_layout.py` @ 960×600 |
| Feel | **NEEDS-DISPLAY** operator rubric — “calm / predictable” row |

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**@coder-mcp:** implement §1–§6 in polish sprint; cite charter in commits.
