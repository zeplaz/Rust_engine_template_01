# DES-G-PLAY-OPERATOR-FLOW-001 — G-PLAY veg ecology checklist `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-G-PLAY-OPERATOR-FLOW-001** |
| **Parent** | [`play_scenario_acceptance_runbook_v1.md`](play_scenario_acceptance_runbook_v1.md) |
| **Supports** | VEG-C14 · [`plan_g_play_split_v1.md`](plan_g_play_split_v1.md) |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** — operator executes with base G-PLAY-01 |

---

## Addendum to G-PLAY-01 (veg slice)

Run **after** base checklist rows 1–8 pass. Adds **≤3 min** ecology read smoke.

| ☐ | # | Action | Pass | Fail |
|:---:|:---:|:---|:---|:---|
| ☐ | V1 | Pan map at **operational zoom** | ≥1 chunk shows topology tint OR ecology overlay enabled in diagnostics | Flat terrain only |
| ☐ | V2 | Open diagnostics **Ecology** section (collapsed in sim — expand once) | `ecology_rows_source` mentions live program | Missing / fixture-only |
| ☐ | V3 | Witness refresh | `topology_kind_count_visible >= 3` OR operator notes ≥3 kinds in legend | Single-color wash |
| ☐ | V4 | Optional fire lane | If fire active: scar/recovery readable vs clean canopy (word in legend) | Scar invisible |

## Witness pointers

- `debug_runs/landscape_grammar_lg4_preview_live.json`
- `debug_runs/stage5_full_app_live.json` ecology subsection when present

## Operator note template

```text
G-PLAY-01 veg addendum YYYY-MM-DD
V1-V3: PASS/FAIL — kinds seen: Network Patch Corridor …
Witness: landscape_grammar_lg4_preview_live.json green=true/false
```

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-06-16 |

Operator execution required for gate close (inherits G-PLAY-01 qualified pattern).
