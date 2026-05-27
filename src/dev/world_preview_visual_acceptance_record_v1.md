# World Preview visual acceptance — `v1` (DESIGN-WP-VISUAL-ACCEPT-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-WP-VISUAL-ACCEPT-001** |
| **Coder queue** | **UI-WP-VISUAL-001** (Coder A **#6**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** |
| **D-WP review** | [`world_preview_d_wp_review_record_v1.md`](world_preview_d_wp_review_record_v1.md) **SIGNED — PASS** |
| **Witness** | [`debug_runs/wave_p_live.json`](../debug_runs/wave_p_live.json) |
| **Layout mock** | [`assets/ui/world_preview/layout_mock_v1.png`](../assets/ui/world_preview/layout_mock_v1.png) (1920×1080) |
| **Slide sheet ref** | [`assets/ui/world_preview/slide_sheet_spec_v1.png`](../assets/ui/world_preview/slide_sheet_spec_v1.png) |
| **Blockers** | [`visual_run_blockers.md`](visual_run_blockers.md) — operator `--test visual` optional |

**No Rust.** Designer **ACCEPT** for **UI-WP-VISUAL-001** on **lib-qualified** pipeline + signed layout decisions **D-02 / D-04 / D-07 / D-09** (constants).

---

## Executive summary

| Area | Verdict | Evidence |
|:---|:---:|:---|
| **Unified workspace (D-01)** | **PASS** | `ui_wp_layout_002_green`, `d04_unified_workspace` |
| **Map dominance (D-02)** | **PASS** | `ui_wp_layout_d02_opt_green`, fraction ≥ 0.65 @ HD baseline |
| **Slide sheet (D-04)** | **PASS** | `d04_sheet_body_wired`, dim α102 |
| **Corner inset (D-07)** | **PASS** | `ui_wp_layout_d07_green`, 140px inset |
| **Paper frame inset (D-09)** | **PASS (constant)** | `d09_paper_frame_inset_px: 12` — texture deferred WP-L1 |
| **GPU preview pipeline** | **PASS** | `ui_wp_pipeline_green`, `gpu_authoritative_surface: true` |
| **Full visual pixel audit** | **Optional** | Does not block qualified ACCEPT |

**Distinction:** Does **not** claim full mock parity (D-05…D-12 paper/motion assets). Operational chrome only.

---

## Visual compare — signed refs

| Decision | Ref asset / doc | Compare in running app |
|:---|:---|:---|
| **D-02** hero map ≥65% | `layout_mock_v1.png` central panel | World Preview @ 1280×720 / 1920×1080, sheet **closed** |
| **D-04** generator sheet | `slide_sheet_spec_v1.png` | Sheet open — dim map α102, width 400–720px |
| **D-07** corner overview | D-WP review § D-07 | Inset on map, not sidebar thumb |
| **D-09** asymmetry | Worksheet § D-09 | 12px paper inset constant wired; full offset polish deferred |

### D-02 acceptance (map dominance)

| Check | Pass when |
|:---|:---|
| Sidebar does not crowd hero map | `d02_map_area_fraction ≥ 0.65` in witness |
| HD baseline | `ui_wp_layout_d02_opt_green: true` |

### D-09 acceptance (paper / frame)

| Check | Pass when |
|:---|:---|
| Inset constant documented | `UI_WP_D09_PAPER_FRAME_INSET_PX = 12` |
| WP-L1 raster textures | **Deferred** — not required for this ACCEPT |

---

## Witness contract — `wave_p_live.json`

| Path | Expected |
|:---|:---|
| `/ui_wp_visual_001/green` | `true` |
| `/ui_wp_visual_001/lib_qualified` | `true` |
| `/ui_wp_visual_001/gpu_authoritative_surface` | `true` |
| `/cod_b_wp_witness_001_green` | `true` |
| `/wave_p_green` | `true` |
| `/ui_wp_layout_d02_opt_green` | `true` |
| `/ui_wp_layout_002_green` | `true` |
| `/ui_wp_layout_d07_green` | `true` |

```powershell
cargo test -p proc_A_dine01 --lib ui_wp_layout_002_writes_wave_p_live_json
cargo test -p proc_A_dine01 --lib cod_b_wp_witness_001
cargo test -p proc_A_dine01 --lib refresh_coder_a_ui_wp_wave_p_witness
```

**Operator optional:**

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
# WorldGen / World Preview chrome — manual compare to layout_mock_v1.png
```

---

## Playtest checklist (editor session)

| # | Pass | Fail |
|:---:|:---|:---|
| 1 | Single unified window — no duplicate preview shell | Second floating archive |
| 2 | Central map reads as hero (sheet closed) | Index dominates |
| 3 | Generator sheet slides without losing D-01 workspace | Detached panel |
| 4 | Corner inset visible on map at default zoom | Minimap thumb returns |
| 5 | Composite layers bind (biome + overlays) | Blank central panel |

---

## Out of scope (explicit)

| Item | Lane |
|:---|:---|
| D-05…D-08 paper textures | **UI-WP-LAYOUT-003** / WP-L1 |
| D-12 motion polish | **UI-WP-MOTION-001** (witness green ≠ full motion QA) |
| WP-L4 color key raster | **UI-WP-L4-001** |
| Stage 5 FULL_APP | Orthogonal spine gate |

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| ACCEPT without `wave_p_live.json` refresh | False green |
| Require pixel-perfect match to mock before lib green | Qualified ACCEPT allows constant-only D-09 |
| Collapse into Stage 5 regression failure | WP visual is infrastructure/product chrome |

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **PASS (qualified)** |
| Coder A | — | May close **UI-WP-VISUAL-001** — witness shows `visual_signoff_pending: false` |

**On-disk (2026-05-26):** `ui_wp_visual_001.green: true`, `lib_qualified: true` in [`wave_p_live.json`](../debug_runs/wave_p_live.json).

**Unblocks:** **UI-WP-VISUAL-001** · complements **DESIGN-D-WP-REVIEW-001** (operational PASS).

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-WP-VISUAL-ACCEPT-001** |
