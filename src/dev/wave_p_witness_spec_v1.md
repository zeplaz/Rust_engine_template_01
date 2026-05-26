# Wave P witness spec `v1` (PLAN-WAVE-P-WITNESS-SPEC-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WAVE-P-WITNESS-SPEC-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Status** | **SIGNED** |
| **Witness file** | [`debug_runs/wave_p_live.json`](../../debug_runs/wave_p_live.json) |
| **Writer** | `src/dev/wave_p_live_proof.rs` · lib tests in `world_preview/` |

---

## Profile

| Field | Value |
|:---|:---|
| `profile` | `WAVE_P_PREVIEW` |
| `source_system` | `wave_p_live_proof` |

---

## Required fields (rollup)

| Path | Type | Green when |
|:---|:---|:---:|
| `wave_p_green` | bool | `true` — product exit |
| `ui_wp_layout_002_green` | bool | D-04 slide sheet landed |
| `ui_wp_layout_d07_green` | bool | D-07 corner inset / minimap chrome rule |

---

## `ui_wp_layout_002` block

| Field | Meaning |
|:---|:---|
| `d04_unified_workspace` | Single workspace (no duplicate F8 window) |
| `d04_generator_sheet_open` | Sheet visible when test opens |
| `d04_sheet_body_wired` | Generator fields in sheet |
| `d04_map_dim_alpha` | Dim scrim 40–50% band (0–255) |
| `d04_sheet_width_px` | Sheet width (design 40–55% workspace) |
| `ui_wp_layout_002_green` | All D-04 checks pass |

---

## `ui_wp_layout_d07` block

| Field | Meaning |
|:---|:---|
| `d07_sidebar_minimap_removed` | No duplicate sidebar minimap in unified workspace |
| `d07_corner_inset_on_map` | Corner inset active |
| `d07_inset_side_px` | Inset extent (design ~140px) |
| `ui_wp_layout_d07_green` | D-07 checks pass |

---

## `wave_p_readiness` block

| Field | Meaning |
|:---|:---|
| `passes` | Stage 5 readiness rollup in preview context |
| `report.open_backlog_items` | `0` for exit |
| `report.composite_graph_sources` | Composite graph wired |
| `report.consumer_contract_ok` | Consumer contract |

---

## Coder witness IDs

| Queue ID | Lib test anchor |
|:---|:---|
| **UI-WP-LAYOUT-002** | `ui_wp_layout_002_writes_wave_p_live_json` |
| **UI-WP-LAYOUT-D07** | `ui_wp_layout_d07_*` tests |
| **COD-B-WP-WITNESS-001** | `cod_b_wp_witness_001_green` |

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib wave_p_live_proof ui_wp_layout_002 ui_wp_layout_d07
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | PLAN-WAVE-P-WITNESS-SPEC-001 — file created |
