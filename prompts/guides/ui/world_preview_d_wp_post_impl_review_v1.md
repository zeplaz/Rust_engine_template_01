# World Preview — post-implementation review brief `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-D-WP-REVIEW-001** |
| **Owner** | `@designer` |
| **Status** | **COMPLETE** — record [`world_preview_d_wp_review_record_v1.md`](../../../src/dev/world_preview_d_wp_review_record_v1.md) **SIGNED PASS** |
| **Track** | **D-WP** — [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) |
| **Product full plan** | [`world_preview_product_full_plan_v1.md`](world_preview_product_full_plan_v1.md) (**PLAN-WP-DECISION-001**) |

---

## When to run

After **COD-B-WP-WITNESS-001** refreshes `debug_runs/wave_p_live.json` (D-04 + D-07 greens) and coder slices **UI-WP-LAYOUT-002** / **UI-WP-LAYOUT-D07** are landed.

**Not** a re-sign of D-01…D-12 worksheet choices — those stay on [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md).

---

## Designer workflow (~30–45 min)

```text
1. Read D-WP track rollup + worksheet §5           ~10 min
2. Open wave_p_live.json + stage5 world_preview   ~5 min
3. F8 manual: sheet open, corner inset, no float   ~10 min
4. Fill review record gap table + §11 checklist    ~10 min
5. SIGNED row → designer_signoff_registry.json     ~5 min
```

---

## Read list

| Doc | Why |
|:---|:---|
| [`world_preview_product_full_plan_v1.md`](world_preview_product_full_plan_v1.md) | Gate chain + review scope (**PLAN-WP-DECISION-001**) |
| [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) | Decision rollup D-01…D-12 |
| [`world_preview_d04_slide_sheet_spec_v1.md`](world_preview_d04_slide_sheet_spec_v1.md) | D-04 acceptance |
| [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) | D-01 invariant |
| [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) | Optional D-02 |
| [`debug_runs/wave_p_live.json`](../../../debug_runs/wave_p_live.json) | `ui_wp_layout_002_*` · `ui_wp_layout_d07_*` |
| [`assets/ui/world_preview/layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) | Parity reference |

---

## Pass criteria

| Channel | PASS when |
|:---|:---|
| **D-01** | Single workspace; no legacy float |
| **D-04** | Sheet + map dim; `ui_wp_layout_002_green: true` |
| **D-07** | Corner inset on map; sidebar overview removed; `ui_wp_layout_d07_green: true` |
| **Track honesty** | Record lists D-05…D-12 as deferred — not silent |

**FAIL** if D-01 regresses, sheet breaks pan/zoom, or review claims full mock parity without evidence.

---

## Deliverable

| Artifact | Path |
|:---|:---|
| Review record | [`src/dev/world_preview_d_wp_review_record_v1.md`](../../../src/dev/world_preview_d_wp_review_record_v1.md) |
| Track rollup update | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) v1.1+ |

---

## Copy-paste — @designer

```
Queue: DESIGN-D-WP-REVIEW-001
Read: prompts/guides/ui/world_preview_d_wp_post_impl_review_v1.md
      src/dev/world_preview_d_wp_review_record_v1.md (template)
Witness: debug_runs/wave_p_live.json → ui_wp_layout_002_green, ui_wp_layout_d07_green
Manual: F8 WorldGen → Parameters sheet → corner minimap on map
Sign: world_preview_d_wp_review_record_v1.md §11 SIGNED — PASS
Do NOT: reopen D-01…D-12 worksheet choices; mutate render_raster
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Brief for DESIGN-D-WP-REVIEW-001 |
