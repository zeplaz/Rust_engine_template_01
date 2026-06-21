# APS onboarding spec `v2` — first 10 seconds

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-ONBOARD-SPEC-002** |
| **Program** | PLAN-MULTI-PARALLEL-TRACKS-001 · T1 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Supersedes** | [`design_aps_uiux_onboard_spec_v1.md`](design_aps_uiux_onboard_spec_v1.md) (content retained; tier + grammar hooks added) |
| **Inputs** | [`design_aps_uiux_onboard_outline_v1.md`](design_aps_uiux_onboard_outline_v1.md) |
| **Handoff** | OVR-P56-ONBOARD-001 · `test_aps_onboarding.py` |
| **Verdict** | **PASS** |

```text
DES-APS-ONBOARD-SPEC-002 Q✓
First-10s path — welcome · lane · one action · spine teacher
```

---

## 0. First 10 seconds (success path)

| Second | Artist sees | System does |
|:---:|:---|:---|
| 0–2 | App opens · Buildings domain · welcome card | `onboarding_seen_v1` check |
| 2–4 | `Start on Catalog` affordance | Spine `▣ Catalog` |
| 4–6 | Catalog list + kit hint (G0) | First module pre-highlight if list non-empty |
| 6–8 | Metadata **collapsed** | No engineer diagram |
| 8–10 | Spine shows step 2 hint on hover | `Materials — assign looks` tooltip |

**Fail if:** modal blocks map · schema diagram visible · empty catalog without empty-state copy.

---

## 1. Welcome card (verbatim — unchanged from v1)

See v1 §1 Buildings + Landscape copy. Placement: inline below Row 2 chrome — **not modal**.

**Persistence keys:**

| Key | Meaning |
|:---|:---|
| `onboarding_seen_v1` | Buildings welcome dismissed |
| `onboarding_landscape_seen_v1` | Landscape lane card shown once |
| `aps_ui_prefs.json` | Stored under `debug_runs/` or user config path |

---

## 2. Lane chooser

| Domain | First tab | Spine step 0 label |
|:---|:---|:---|
| Buildings | Catalog | `Catalog` |
| Landscape | Presets | `Presets` |

**Switch domain:** preserve per-domain tab index in prefs.

---

## 3. Empty states (per panel)

| Panel | Empty copy |
|:---|:---|
| Catalog | `○ No modules in catalog — import or open staging folder` |
| Materials | `○ No profiles — Create profile to assign looks` |
| Assembly | `○ Generate a building first — use type + district above` |
| Presets (landscape) | `○ No presets — pick from registry or author new` |

---

## 4. Grammar tier hook (v2 delta)

When `grammar_set_tier()` returns **G0**:

| Surface | Onboard addition |
|:---|:---|
| Generate row | Kit hint: `Pilot kit — one archetype. More unlock as set matures.` |
| Advanced collapse | Label: `Setup & manual fallback` — not “grammar debug” |
| Spine | No extra walkthrough — spine remains teacher |

**G1+:** remove pilot sentence; show tier chip per [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md).

---

## 5. Dismiss + recall

| Action | Behavior |
|:---|:---|
| `Don't show again` | Set flags; never auto-show |
| Help → `Show getting started` | Re-open welcome (non-blocking) |
| First Landscape visit | One-time landscape card even if Buildings seen |

---

## 6. Guards

`test_aps_onboarding.py` asserts:

- Welcome strings present on first run fixture
- No `Ship truth` / `rust_engine_mcp` in welcome body
- Metadata collapsed default
- `Start on Catalog` selects tab 0

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
