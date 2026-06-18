# APS Site Preview IA `v1` — footprint + zone grid

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-SITE-PREVIEW-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E3-A |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_facility_site_zone_taxonomy_v1.md`](design_facility_site_zone_taxonomy_v1.md) · [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) |
| **Preview contract** | [`design_aps_uiux_preview_spec_v1.md`](design_aps_uiux_preview_spec_v1.md) |
| **Handoff** | CMCP-SITE-PREVIEW-PANEL-001 |
| **Verdict** | **PASS** |

```text
DES-APS-SITE-PREVIEW-001 Q✓
Two-level preview: building footprint (P0) + site zone grid (Layout view) — tier-gated
```

---

## 0. Problem

Artists see **building massing** in the footprint canvas but not **yard layout** (rail spur, utility ring, loading wing). Site zone pilots exist on disk (`*_site_v0.json`) with no APS surface.

**North star:** one workspace shows **where the hall sits on the site** — without forking preview channels or inventing zones in grammar.

---

## 1. Two-level model

| Level | Name | Fidelity chip | Data source | Existing widget |
|:---:|:---|:---|:---|:---|
| **L1** | Building footprint | **Layout view** | Grammar footprint / assembly placements | `FootprintCanvas` |
| **L2** | Site zone grid | **Layout view** | `site_zone_grid_v1` JSON | **NEW** `SiteZonePreviewCanvas` |

```text
┌─ Site preview (collapsible) ──────────────────────────────────────┐
│ [Layout view]  Site: logistics_rail_warehouse_site_v0              │
│ ┌─ L2 site grid (8×6 cells) ──────────────────────────────────┐ │
│ │ R R R R P P P P   legend: P L U R S K ·                      │ │
│ │ R . . . . . . .                                              │ │
│ │ W W W W W W . Y Y   ← L2 zones                               │ │
│ │ W W L L . . Y . .                                            │ │
│ │ . . S S . . . . .     ┌──┐                                   │ │
│ │                       │L1│ footprint inset (primary cells)  │ │
│ └───────────────────────└──┘───────────────────────────────────┘ │
│ Primary 12% · Loading 5% · Utility 8% · Rail spur north          │
└──────────────────────────────────────────────────────────────────┘
```

**L1 inset:** grammar footprint cells drawn **inside** L2 `primary` region — semi-opaque overlay (W/D/C/R/Y tokens unchanged).

---

## 2. Placement (Assembly workspace)

**Container:** `CollapsibleSection` titled **Site layout** — sibling below `FootprintCanvas` block inside center column (not a new tab).

**Default pack order (center column):**
1. Footprint grid (plan view) — existing
2. **Site layout** — this spec
3. Slot / assembly preview row — existing

**Landscape lane:** section **hidden**.

---

## 3. Tier exposure

| Grammar tier | Site layout section | L2 grid | Metrics row |
|:---|:---|:---|:---|
| **G0** | **hidden** | — | — |
| **G1** | **collapsed** | header + `○ Site layout unlocks at G2 — tune archetype family first` | — |
| **G2** | **collapsed** | body on expand; L2 + L1 inset | zone % on expand |
| **G3+** | **collapsed** default; artist may pin expanded | full | always on expand |

**Promoted at G2:** not a spine strip — collapsible only. **Pin** preference: `site_preview_pinned_v1` (bool).

**Amend to exposure matrix:** row **Site layout preview** — G0 hidden · G1 collapsed placeholder · G2+ collapsed with body.

---

## 4. Zone rendering (L2)

Use taxonomy tokens from [`design_facility_site_zone_taxonomy_v1.md`](design_facility_site_zone_taxonomy_v1.md) §5:

| Zone | Fill | Label on cell |
|:---|:---|:---|
| primary | `footprint_valid` α0.35 | — |
| loading | `footprint_valid` α0.35 | `Load` |
| utility | `yard_void` hatch | `Yard` |
| rail | `rail_void` + glyph | `Rail` |
| service | `service_outline` | `Svc` |
| parking | `parking_void` | `Park` |
| buffer | none | — |

**Legend row** (compact): `P L U R S K ·` — hover tooltips: Primary · Loading · Utility · Rail · Service · parKing · buffer.

**Cell size:** `cell_px = 20` default (smaller than footprint canvas 28) — fits 10×8 site in ~200px height.

---

## 5. Site selection logic

| Priority | Source |
|:---:|:---|
| 1 | `facility_binding.site_template` on active grammar |
| 2 | ARCH-DNA preset `site_zone_grid` path |
| 3 | Archetype default pilot map |

**Default map (until binding ships):**

| Archetype | Site JSON |
|:---|:---|
| `IndustrialWarehouse` / `RailEdge` | `logistics_rail_warehouse_site_v0.json` |
| `FactoryCluster` | `manufacturing_fabrication_hall_site_v0.json` |
| *(utility pilots)* | `power_substation_yard_site_v0.json` |
| *(tank / refinery)* | `fuel_depot_tank_farm_site_v0.json` |
| *(concrete steps)* | DMCP-PILOT-CONCRETE-SITE-001 paths when on disk |

**Header line:** `Site: {site_id}` · link `Open site JSON` (folder).

---

## 6. Metrics row (expanded G2+)

Single line under canvas:

```text
Primary {pct}% · Loading {pct}% · Utility {pct}% · Rail {yes|—} · Building fit {ok|warn}
```

| Metric | Rule |
|:---|:---|
| Zone % | from `metrics` block or computed |
| Building fit | L1 occupied cells ⊆ L2 primary ≥80% — warn `◐ partial fit — adjust footprint or site` |

Validator SZ-03…SZ-06 run on **Open** or **Refresh** — inline `✓ site valid` / `✗ site blocked — {rule}` via `status_atom`.

---

## 7. Four-state contract (preview spec)

| State | Site preview copy |
|:---|:---|
| loading | `⟳ Loading site layout…` |
| empty | `○ No site template for this archetype` |
| error | `◐ Site JSON invalid — run validate-report site_zone_grid` |
| result | L2 grid + fidelity chip **Layout view** |

Never black canvas.

---

## 8. Interaction (read-only v1)

| Action | v1 | Future |
|:---|:---:|:---:|
| Pan/zoom | — | P3 |
| Click zone cell | tooltip zone id | highlight in metrics |
| Edit zones in APS | **no** | designer-mcp JSON only |
| Sync from footprint | auto inset L1 on Generate | — |

**Focus:** artist **reads** site — edits via pilot JSON + validate CLI.

---

## 9. Relationship to other previews

| Surface | Scope | Do not merge |
|:---|:---|:---|
| Footprint canvas | Per-floor module tokens | Keep separate header |
| Assembly preview (P2) | 3D quick / browser | No zone overlay in 3D v1 |
| Atlas preview | Tile sheet | — |
| **Site layout** | Yard zones + footprint inset | This spec only |

**Inspector P3 highlight** (grammar rule → grid cells) applies to **L1 footprint only** — not L2 zones.

---

## 10. Layout & smoothness

| Rule | Spec |
|:---|:---|
| Collapsed height | header only ~28px |
| Expanded max height | 240px canvas + metrics — scroll inside section if MIN window |
| Section expand | vertical only — no horizontal notebook reflow |
| Debounce | 150ms on archetype change before site reload |

Ref: [`design_aps_uiux_layout_delta_v1.md`](design_aps_uiux_layout_delta_v1.md) AS-1 — Site layout counts toward **≤2** expanded grammar panels @ G2 launch.

---

## 11. Witness fields

```json
{
  "site_preview_visible": true,
  "site_preview_expanded": false,
  "site_id": "logistics_rail_warehouse_site_v0",
  "site_zone_grid_path": "assets/configs/buildings/pilots/logistics_rail_warehouse_site_v0.json",
  "l1_inset_fit_ok": true,
  "grammar_tier": "G2"
}
```

**Path:** `debug_runs/site_preview_panel_live.json` (after CMCP ships).

---

## 12. Verification

| Check | Method |
|:---|:---|
| G0 hidden | tier gate test |
| Rail warehouse shows R row | fixture JSON |
| L1 inset in primary | visual QA |
| MIN 960×600 no tab h-scroll | layout test |
| Validate inline | mock SZ-04 fail on smelter without utility % |

---

## 13. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** CMCP-SITE-PREVIEW-PANEL-001 · operator site-layout rubric
