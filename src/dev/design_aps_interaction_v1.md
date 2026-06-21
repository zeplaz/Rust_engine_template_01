# APS interaction spec `v1` — primary feedback, disabled-why, spine affordance

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-INTERACTION-001** |
| **Program** | PLAN-MULTI-PARALLEL-TRACKS-001 · T1 APS-STUDIO |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_aps_design_system_v11_delta_v1.md`](design_aps_design_system_v11_delta_v1.md) · [`design_aps_smoothness_charter_v1.md`](design_aps_smoothness_charter_v1.md) |
| **Handoff** | OVR-P5-TAIL-001 · assembly/catalog panel polish |
| **Verdict** | **PASS** |

```text
DES-APS-INTERACTION-001 Q✓
Primary-action feedback · disabled-why adjacent · spine click affordance · selection persist
```

---

## 0. Scope

Tk-realistic interaction contract for **all Buildings + Landscape panels** — not new chrome, not animation.

**Out of scope:** Bevy sim HUD · MCP batch runners · theme replatform.

---

## 1. Primary-action feedback

| Action class | On click | On success | On failure |
|:---|:---|:---|:---|
| **Flow advance** (spine `▶`) | `⟳ {verb}…` on pill + disable | `✓` status atom + re-enable | `✗ {reason}` on same row |
| **Generate / Validate / Ship** | Button → `⟳ {verb}…` | `✓ {past}` adjacent OR toast 3s | `✗ {verb} failed — {fix}` |
| **Preview refresh** | `○ Rendering…` in preview pane | thumbs swap | `✗ Preview failed` inline |
| **Atlas pack / bake** | `⟳ Packing…` | spine step tick | banner + log line |

**Rule:** ack originates at the **control that was clicked** — never log-only for primary verbs.

---

## 2. Disabled-why (adjacent)

| Control | When disabled | Copy placement |
|:---|:---|:---|
| Spine `▶` | Prereq missing | `_advance_blocked_var`: `✗ {reason}` same row |
| Greyed button | Guard fail | Tooltip **or** 1-line caption **below** button group |
| Empty combo | No catalog rows | Empty-state from onboard spec §3 |
| Tier-locked panel | G&lt;n | Kit hint row: `Unlocks at G{n} — {human bar}` |

**Ban:** silent `state=disabled` on any primary verb.

---

## 3. Spine click affordance

```text
Catalog  →  Materials  →  Assembly  →  Variants  →  Atlas
  ▣           ○            ○            ○           ○
```

| State | Glyph | Click |
|:---|:---:|:---|
| **Current** | `▣` | No-op (already there) |
| **Complete** | `✓` | Jump tab + restore last selection |
| **Available** | `○` | Jump tab |
| **Blocked** | `○` + muted | Click shows `✗ {reason}` toast — no tab switch |

**Single current:** exactly one `▣` — never duplicate “you are here” in tab label and spine.

---

## 4. List selection persist

| Navigation | Rule |
|:---|:---|
| Tab switch Catalog ↔ Materials | Restore module id if still in list |
| Assembly piece select | Persist across preview state strip changes |
| Grammar inspector row | Highlight survives until new generate |
| Domain switch Buildings ↔ Landscape | Independent selection stores per domain |

**Restore failure:** select first row + caption `Previous selection unavailable`.

---

## 5. Panel-specific hooks

| Panel | Interaction note |
|:---|:---|
| `catalog_panel.py` | Row select → 150ms debounce preview ([`design_aps_preview_v2_spec_v1.md`](design_aps_preview_v2_spec_v1.md)) |
| `assembly_panel.py` | Slot click → immediate P1 thumb; combined P2 async |
| `pipeline_status_bar.py` | Spine is read/write — not decorative |
| `material_library_panel.py` | Assign applies on explicit `Apply` — not silent on blur |

---

## 6. Acceptance

| # | Check |
|:---:|:---|
| I1 | Every primary button shows inline `⟳` within 1 frame |
| I2 | Disabled spine step never silent |
| I3 | Tab return restores selection or states why not |
| I4 | No second “current step” indicator outside spine |
| I5 | `test_aps_onboarding.py` + interaction smoke pass |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
