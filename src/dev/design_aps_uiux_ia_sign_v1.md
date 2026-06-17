# APS UI/UX IA Sign-off `v1` — OVR-DES-P4-IA-SIGN-001

| Field | Value |
|:---|:---|
| **ID** | **OVR-DES-P4-IA-SIGN-001** |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Phase** | P4 (tab design & IA) |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §4 |
| **Inputs** | [`aps_sweep_tabdesign_20260616_v1.md`](aps_sweep_tabdesign_20260616_v1.md) |
| **Implements** | `OVR-P4-IA-001` |
| **Verdict** | **PASS** — IA contract signed for `@coder-mcp` |

```text
OVR-DES-P4-IA-SIGN-001 Q✓
Unblocks: OVR-P4-IA-001 · OVR-DES-P45-SPINE-SPEC-001 (after P4 lands)
```

---

## 1. Lane structure — CONFIRMED (Option D live)

```text
LANE (persistent):  [ Buildings ]  [ Landscape ]
```

- Dual notebook (`OPTION_D_DUAL_NOTEBOOK = True`) — **do not** relabel one 5-tab row.
- `Ctrl+1` / `Ctrl+2`; `clear_cross_lane_selection` on lane switch.
- Lane chip + tint — not color alone.
- All lane-scoped chrome repaints on switch: lane bar · flow · authority · pipeline.

**3rd-lane rule:** +1 notebook + `*_BY_LANE` dict entries. Never nest notebooks. ≤6 tabs per lane.

---

## 2. Tab order — LOCKED

### Buildings (5)

```text
Catalog → Materials → Assembly → Variants → Atlas
```

| Tab | Owns | Authority |
|:---|:---|:---|
| **Catalog** | Module browse, module info, GLB check | module library (input) |
| **Materials** | Material library / studio | profiles (input) |
| **Assembly** | Footprint, pieces, tags, **material assign**, ship check | **Assembly** (ships) |
| **Variants** | Variant layers, bake prep | variant set (derived) |
| **Atlas** | Pack, QC, register | atlas (output) |

### Landscape (4)

```text
Presets → Grammar → States → Atlas
```

| Tab | Owns | Authority |
|:---|:---|:---|
| **Presets** | Preset browse, validate | preset (input) |
| **Grammar** | Layout graph editor | layout graph (ships) |
| **States** | Growth + fire matrix | catalog rows (derived) |
| **Atlas** | Pack + register + scope QC | atlas (output) |

---

## 3. Refinements — REQUIRED (R1–R4)

### R1 — Stamp folded into Atlas (P0)

**Problem:** Landscape pipeline had 5 steps incl. orphan `stamp` with no tab.

**Ruling:** **Fold.** Register/stamp is Atlas terminal state. Pipeline keys:

```text
Buildings:  catalog · materials · assembly · variants · atlas
Landscape:  presets · grammar · states · atlas
```

Update `domain_router.verify_option_d_ia_contract()` — **4** landscape keys, not 5.

Atlas terminal copy: `✓ Atlas registered` (landscape) / `✓ Tiles registered` (buildings).

### R2 — Delete Catalog landscape branch (P1)

**Problem:** `CatalogPanel._refresh_landscape_presets` unreachable after Option D.

**Ruling:** Remove landscape code path from Catalog. **Presets tab is sole preset reader.**

Guard: assert Catalog no longer imports `list_landscape_presets`.

### R3 — Atlas label disambiguation (P1)

Same class `AtlasPanel` serves both lanes — different ship gates.

| Lane | Tab label | Register banner |
|:---|:---|:---|
| Buildings | `Atlas` | `Registers to: Buildings tile index` |
| Landscape | `Atlas` | `Registers to: Landscape tile index` |

LG-5 G-scope QC must surface **per-gate** status — not one `register_green` ✓.

### R4 — Material authority (P0)

**One concept, three roles:**

| Tab | Role |
|:---|:---|
| Materials | Library — create/edit/preview |
| Assembly | Assignment — bind material to piece |
| Variants | Reference — **profile-id dropdown** (not free-text `wall_material`) |

Cross-link pattern: Assembly ↔ Materials (`_open_material_in_*`) is the model. Replicate for Variants → Materials ("edit this profile").

---

## 4. Pipeline ↔ tab contract

| Rule | Detail |
|:---|:---|
| Keys === tab keys | Same set, same order, per lane |
| Pills navigable | Click selects tab (P4.5 implements) |
| Flow verbs | Lane-scoped; prereq at button origin |
| No orphan steps | Every pill has a tab home |

Buildings flow today skips Materials on `Send to Assembly` — add implicit "profiles ready?" prereq or artist uses Materials tab first (order 3a makes this natural).

---

## 5. Implementation checklist (`OVR-P4-IA-001`)

- [ ] Re-order Buildings notebook tabs in `app.py` / `domain_router.py`
- [ ] Re-order `PIPELINE_STEPS_BY_LANE` buildings to match
- [ ] Drop landscape `stamp` pipeline key; fold register into Atlas
- [ ] Update `verify_option_d_ia_contract` to 4 landscape keys
- [ ] Strip Catalog landscape branch
- [ ] Variants material layer → profile dropdown from Materials catalog
- [ ] Atlas register banner lane-specific
- [ ] `test_aps_lane_tab_swap.py` green

---

## 6. Guard tests (new / updated)

| Guard | Asserts |
|:---|:---|
| `test_aps_lane_tab_swap.py` | Tab order matches signed IA |
| pipeline-keys === tabs | Per-lane key set + order |
| single preset reader | Catalog has no landscape presets |
| lane isolation | No cross-lane selection bleed |

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |

**Supersedes:** ad-hoc tab order in live code where it disagrees with §2. Live code is wrong until P4 lands.
