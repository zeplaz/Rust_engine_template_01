# Phase 4 icon atlas art — `UI-OH-P4-ART-001` `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UI-OH-P4-ART-001** |
| **Review ID** | **UI-OH-P4-ART** (aliases: **P4-ART-01**, **P4-ART**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Reviewer** | `@designer` |
| **Status** | **SIGNED — PASS** (traced silhouette atlas) |
| **Brief** | [`ui_phase4_icon_atlas_brief_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_brief_v1.md) v1.3.0 |
| **Planner plan** | [`ui_phase4_icon_atlas_plan_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase4_icon_atlas_plan_v1.md) |
| **Bake script** | [`tools/orchestrator/scripts/bake_icon_atlas_phase4.py`](../../tools/orchestrator/scripts/bake_icon_atlas_phase4.py) |
| **Atlas PNG** | [`assets/textures/ui/icon_atlas_phase4_v1.png`](../assets/textures/ui/icon_atlas_phase4_v1.png) |
| **Layout mock** | [`assets/ui/phase4/icon_atlas_phase4_layout_mock.png`](../assets/ui/phase4/icon_atlas_phase4_layout_mock.png) |
| **RON manifest** | [`assets/configs/ui/icon_atlas_phase4.icon_atlas.ron`](../assets/configs/ui/icon_atlas_phase4.icon_atlas.ron) (unchanged) |
| **Witness JSON** | `debug_runs/ui_shell_migration_live.json` → `phase4` block |
| **Wave 3 sheet** | [`ui_icon_atlas_sheet_v1.md`](ui_icon_atlas_sheet_v1.md) (`DESIGN-W3-P4-ATLAS-001` / **P4-ART-01**) |

---

## Executive summary

**Designer deliverable** for Phase 4 build-rail icon atlas: replace text-labeled **placeholder** with **monochrome traced silhouettes** per brief §4–§7.

**Verdict:** ☑ **SIGNED — PASS** — 256×128 atlas baked; row 0 **RD RL UT IN CV** + disambiguation **UT_TX / UT_MG** + vehicle row + **P5_BR** on grid; RON UV indices unchanged.

**Non-blocking:** §5 operator blind review (transformer vs trailer); runtime `icon_atlas_loaded` refreshes on sim load (lib tests green).

---

## Prerequisites

| Gate | Required | Observed | Met |
|:---|:---|:---|:---:|
| **PLAN-UI-P4-ATLAS-001** | Plan **DONE** | planner plan closed | ☑ |
| Phase 4.1 code | `IconAtlasPlugin` + rail UV | landed | ☑ |
| Grid spec | 8×4 @ 32px | 256×128 PNG | ☑ |
| Source paths | `ui_texture_assets.json` | all sources on disk | ☑ |
| UT ≠ UT_MG | Distinct cells + silhouettes | cols (2,0) vs (1,1) | ☑ |
| `cargo test --lib icon_atlas` | 6 tests | **ok** | ☑ |

**Prerequisite verdict:** ☑ **MET**

**Re-bake command:**

```powershell
python tools/orchestrator/scripts/bake_icon_atlas_phase4.py
cargo test -p proc_A_dine01 --lib icon_atlas
```

---

## Cell acceptance (designer)

| Cell | Source | Silhouette read | Verdict |
|:---|:---|:---|:---:|
| **RD** | `misc/railroad_track.png` | Horizontal corridor / ties | **PASS** |
| **RL** | same + 15° cant | Parallel rail emphasis | **PASS** |
| **UT** | oil-cooled transformer α | Fins + pad mass, **no wheels** | **PASS** |
| **UT_TX** | transformer reference | Matches **UT** | **PASS** |
| **UT_MG** | generator trailer α | Box + hitch read (QA only) | **PASS** |
| **IN** | `wooden_buildings_01.png` | Vertical industrial mass | **PASS** |
| **CV** | `cities.png` | Low-rise cluster | **PASS** |
| **TRUCK** | civ truck empty sheet | Cab + bed | **PASS** |
| **URAL** | ural empty sheet | Taller cab vs truck | **PASS** |
| **BUS** | bus α sheet | Long wheelbase | **PASS** |
| **P5_BR** | `hjm-barrel_alpha.png` | Barrel ¾ | **PASS** |

**§5 blind test:** deferred to **@operator** — silhouettes visually distinct at thumbnail; not a coder gate.

---

## §11 Designer sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | Read brief §4–§7 + icon language spec | ☑ |
| 2 | Traced atlas replaces placeholder (same path + RON) | ☑ |
| 3 | UT uses transformer source; UT_MG uses forbidden alternate | ☑ |
| 4 | Row-0 readable at 48px rail (white mono + runtime tint) | ☑ |
| 5 | Layout mock committed | ☑ |
| 6 | Does **not** wire P4-VEH-01 consumers (coder) | ☑ |

**Verdict:** ☑ **SIGNED — PASS**

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-25 | **SIGNED — PASS** |

---

## Unblocks

| Lane | Owner | Notes |
|:---|:---|:---|
| **P4-F03** | `@coder` | optional build-rail hover border |
| **P4-VEH-01** | `@coder` | wire TRUCK/URAL/BUS when logistics UI exists |
| **P4-MI-EC** | `@designer` + `@coder` | row 0 cols 5–6 when product asks |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **UI-OH-P4-ART-001** traced atlas + bake script |
