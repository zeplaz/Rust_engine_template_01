# APS-UX-AUDIT-001 — Art Pipeline Suite UX audit `v2` (post tag + preview pass)

| Field | Value |
|:---|:---|
| **Program ID** | **APS-UX-AUDIT-001** |
| **Prior** | [`design_aps_ux_audit_v1.md`](../docs/archive/2026-06-src-dev/plans/design_aps_ux_audit_v1.md) · [`design_aps_ux_review_20260615_v1.md`](design_aps_ux_review_20260615_v1.md) |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS WITH NOTES** |
| **Witness** | [`debug_runs/aps_ux_audit_v2_live.json`](../debug_runs/aps_ux_audit_v2_live.json) |

```text
APS-UX-AUDIT-001 v2 Q✓
Tool launches · Variants tag/preview pass landed · operator pixel walk still open
```

---

## Executive summary

APS **launches** and supports catalog → assembly → materials → variants → atlas without Blender for daily paths. Since v1/v1.5:

- **Variants:** live draft preview, layer context lines, human mandate tag labels, reaction-event anchor hints
- **Assembly + Variants:** generation trace strip + approve checkbox
- **Tags:** three-surface vocabulary (`aps_tag_vocabulary.py`) with pytest audit
- **Tooltips:** mandate/semantic/variant dynamic keys wired

**Remaining gap:** operator MIN-window pixel rubric + pipeline valid≠saved (F5 from June review).

---

## Scores (1–10)

| Dimension | v1 (06-03) | v2 (now) | Δ | Notes |
|:---|:---:|:---:|:---:|:---|
| **Clarity** | 6 | **7** | +1 | Tag labels + context lines on Variants; generation trace on Assembly |
| **Discoverability** | 5 | **7** | +2 | Apply vs preview distinction visible; reaction filter shows anchors |
| **Error recovery** | 7 | **7** | 0 | Unchanged — P0 dialogs + validate still strong |
| **Accessibility** | 4 | **6** | +2 | Mandate tags word-first; tooltip corpus expanded; footprint/swatch gaps remain |
| **Workflow efficiency** | 6 | **7** | +1 | Draft preview removes Apply guesswork on Variants |

**Composite:** **6.8** (was ~5.6) — clears plan target **7** after operator rubric + F5 pipeline validity.

---

## Top 10 issues (ranked)

| # | P | Issue | Owner |
|:---:|:---:|:---|:---|
| 1 | P1 | Pipeline bar ✓ = has-data not P0-valid | coder-mcp F5 |
| 2 | P1 | Assembly MIN 960×600 cram — advanced sections default expanded | coder-mcp F3 |
| 3 | P1 | Primary labels still 8px in catalog/footprint slots | coder-mcp F4 |
| 4 | P1 | No keyboard path footprint → apply material | coder-mcp |
| 5 | P2 | Footprint heatmap + swatch color-primary | coder-mcp F6 |
| 6 | P2 | Flow bar silent no-op into collapsed log | coder-mcp F7 |
| 7 | P2 | Atlas lod0 inline not under Advanced | coder-mcp |
| 8 | P2 | Tag tier-2 presets not authored (archetype bundles) | designer DES-APS-TAG-TIER2 |
| 9 | P2 | Operator tag rubric not executed | operator |
| 10 | P2 | `weather_penalty` construction badge deferred | coder C P2 |

---

## Top 5 coder-mcp fixes (actionable)

| # | Fix | Files |
|:---:|:---|:---|
| 1 | Pipeline **valid** vs **saved** on spine | `pipeline_status_bar.py`, `SuiteState.assembly_p0_passed` |
| 2 | Collapse ARCH-DNA + Iterate under Advanced @ Assembly | `assembly_panel.py` |
| 3 | Bump FONT_SMALL to 9px on primary labels | `aps_theme.py`, catalog, footprint |
| 4 | Archetype tag preset loader (tier-2) | `variants_panel.py` + taxonomy JSON |
| 5 | Bind remaining tooltip keys from copy pack | `aps_tooltips.py` per `aps_tooltip_copy_v1.md` |

---

## Accessibility checklist (v2)

| Item | Result |
|:---|:---|
| Status not color/glyph alone | **PARTIAL** — mandate tags + pipeline text improved; heatmap/swatch still weak |
| Critical actions not tooltip-only | **PASS** |
| Variants layer meaning without Apply | **PASS** — live draft + context line |
| Tag surfaces distinguished in copy | **PASS** — skill + tooltips |
| MIN 960×600 Assembly | **FAIL** — unchanged |
| Operator rubric @ MIN | **PENDING** — [`design_aps_tag_operator_rubric_v1.md`](design_aps_tag_operator_rubric_v1.md) |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS WITH NOTES** | 2026-06-02 |

Unblocks: operator pixel walk · APS polish tail · tier-2 tag presets
