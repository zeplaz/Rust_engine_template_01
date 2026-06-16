# APS-ATLAS-LEGEND-001 — Atlas preview legend + plain errors `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-ATLAS-LEGEND-001 |
| **Owner** | `@designer` (copy + legend) · `@coder-mcp` (panel + validator UX) |
| **Panel** | `atlas_preview_panel.py` · `atlas_panel.py` |
| **Validator** | `rust_engine_mcp/aps_atlas_qc.py` · `validate_atlas_meta_v2` |
| **Date** | 2026-06-03 |
| **Verdict** | **APPROVED** |

---

## Problem

Artists QC packed atlases before `tile-atlas-register` but need:

1. **Readable UV grid** — what do grid lines and highlight mean?
2. **Plain-language validation** — not JSON schema dumps or raw `signature` strings.

---

## Panel layout (legend placement)

```text
┌─ Tile preview (packed atlas + cells) ─────────────────────────────────────┐
│ ┌ Packed atlas (UV grid) ─────┐  ┌ Selected cell ─────────────┐          │
│ │ [thumb + gray grid lines]    │  │ [large cell thumb]         │          │
│ │ tile_map_….png               │  │ variant: … · grid: … · uv …│          │
│ │ UV overlay: 4×2 cells        │  │                            │          │
│ │ Legend: ▬ grid  ■ selected   │  │                            │          │
│ └──────────────────────────────┘  └────────────────────────────┘          │
│ Source PNG cells  [thumb][thumb][thumb]…  ← click syncs grid highlight     │
│ atlas_meta: tile_id=… · grid 4×2 · N cells · Next: register…              │
│ ┌ Validate atlas meta result (plain language) ─────────────────────────┐ │
│ │ PASS: Atlas meta looks complete — safe to proceed toward register.     │ │
│ └────────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## UV grid visual spec

| Element | Spec | Artist label (on-panel) |
|:---|:---|:---|
| Grid lines | `#888888`, 1px | **Grid** — divides packed PNG into columns×rows from `atlas_meta.json` |
| Selected cell outline | `#0066cc`, 2px rectangle | **Selected** — matches clicked source PNG / `grid` in meta row |
| No atlas PNG | Placeholder text | “(no tile_map_*.png — run Pack atlas)” |
| No meta | Meta line | “atlas_meta: (missing) — pack/register writes atlas_meta.json” |
| Grid legend line | Below path, Segoe UI 8 | `Legend: Grid lines = UV cells · Blue outline = selected cell` |

**Interaction:** Click source PNG cell → set `_highlight = (col, row)` from meta `grid` → redraw atlas thumb with blue outline.

**When columns/rows = 0:** Hide grid overlay; show meta line “UV overlay unavailable — fix atlas_meta columns/rows”.

---

## Source cell strip

| State | Copy |
|:---|:---|
| Empty folder | “(no source PNGs in folder)” |
| Cell with meta | Show thumb + truncated name (14 chars) |
| Cell without meta row | Still selectable; meta line “no meta row” in log only — panel shows variant from filename |

---

## Validate atlas meta — result panel

### PASS (green `#006400` or primary label)

```text
PASS: Atlas meta looks complete — safe to proceed toward tile-atlas-register.
```

Optional second line (muted):

```text
Grid 4×2 · 8 cells indexed · facings OK
```

### FAIL (error `#8b0000` or error_text style)

Show **up to 4 sentences** inline (panel `_atlas_qc_var`); dialog may show up to 8.

Prefix: `FAIL:` then ` · `-joined sentences.

---

## Plain-language error catalog (authoritative)

Map validator `signature` / `kind` → artist sentence. Extend `aps_atlas_qc._PLAIN` to match this table.

| Signature / condition | Artist sentence | Fix hint (optional second line) |
|:---|:---|:---|
| *(missing file)* | No atlas_meta.json in this folder — run Pack atlas first. | — |
| `atlas_meta_v2_parse` | Could not read atlas_meta.json — check the file exists and is valid JSON. | Open folder and re-run Pack |
| `atlas_meta_v2_version` | Atlas meta must be schema version 2 (v1 greybox is frozen). | Regenerate meta with current tilemapgen |
| `atlas_meta_v2_jsonschema` | Atlas meta is missing required fields — compare with a known-good pilot meta. | See `tile_warehouse_industrial_west_pilot_v1` |
| `atlas_meta_v2_facings` | render_contract.facings must be 4 or 8 so tile lookup matches the rig. | Set facings to 4 or 8 in meta or rebake |
| `atlas_meta_v2_lookup_incomplete` | Some variant/facing/frame cells are missing from lookups — bake or pack before register. | Re-run tile batch + pack |
| *(unknown)* | Validation failed — see atlas_meta.json and log. | Include `field:` suffix when present |

**Field suffix rule:** Append `(field: {name})` only when it helps locate JSON key — not for every error.

---

## atlas_meta summary line (unchanged intent, copy lock)

```text
atlas_meta: tile_id={id} · grid {cols}×{rows} · {n} cells · Next: tile-atlas-register / map stamp (see tools/mcp/README.md)
```

---

## Accessibility

| Rule | Spec |
|:---|:---|
| PASS/FAIL | **Text prefix** PASS or FAIL — not color-only |
| Grid meaning | Legend line always visible when grid drawn |
| Selected cell | Text line `variant: … · grid: … · uv: …` duplicates visual highlight |

---

## Pilot acceptance

Folder: `assets/staging/tiles/tile_warehouse_industrial_west_pilot_v1`

| Check | Expected |
|:---|:---|
| Grid overlay | 2×2 or pilot cols×rows visible on packed PNG |
| Click cell | Blue outline moves |
| Validate | PASS sentence or mapped FAIL sentence — no raw JSON in panel |

Witness: `debug_runs/aps_atlas_preview_002_live.json` (`plain_language`, `uv_grid_overlay`).

---

## Sign-off

```text
APS-ATLAS-LEGEND-001 complete
Legend: grid #888 · selected #0066cc · on-panel legend line
Plain errors: 6 mapped + fallback
Sign-off: APPROVED
```
