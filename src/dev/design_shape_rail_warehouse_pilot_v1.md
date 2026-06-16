# Industrial Rail Warehouse — FootprintMatrix pilot `v1` (BUILD-READ-SHAPE-001)

| Field | Value |
|:---|:---|
| **Program** | **BUILD-READ-SHAPE-001** |
| **Owner** | `@designer-mcp` (spec) · `@coder` **BUILD-READ-SHAPE-002** |
| **Verdict** | **PASS** |
| **Parent** | [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) |
| **Guide** | [`prompts/guides/build_grammer2_exman.md`](../prompts/guides/build_grammer2_exman.md) §ARCH-DNA EXAMPLE |
| **DNA preset** | [`tools/mcp/schemas/examples/arch_dna_logistics_rail_warehouse_v0.json`](../tools/mcp/schemas/examples/arch_dna_logistics_rail_warehouse_v0.json) |
| **Witness** | [`debug_runs/design_shape_rail_warehouse_pilot_live.json`](../debug_runs/design_shape_rail_warehouse_pilot_live.json) |

**MCP lane:** data on disk only — no bpy · no new massing enum.

---

## Order critique (@designer-mcp)

```yaml
order_critique:
  request_summary: "Industrial Rail Warehouse L FootprintMatrix on disk for rotate/scale QA"
  concerns:
    - "Tray must not use from_size(w,d,true) for this pilot"
    - "DNA preset must not fork — link arch_dna_logistics_rail_warehouse_v0.json"
  rules_audit:
    no_ai_generated_images: pass
    deterministic_output: pass
    batch_processing: pass
    grid_alignment: pass
  blocked: false
  foresight_flags:
    - "SHAPE-002 tray wire uses _pilot_catalog.ron"
    - "VISUAL-002 tile batch references same matrix"
  proceed: yes
```

---
## Pilot identity

| Field | Value |
|:---|:---|
| **catalog_id** | `pilot:logistics_rail_warehouse_v0` |
| **mock shape id** | `logistics_rail_warehouse_l_6x5` |
| **grammar_id** | `industrial_warehouse_v1` |
| **preset_id** | `logistics_rail_warehouse_v0` |
| **Preferred massing** | `l_shape` → RailEdge candidate family |
| **Family** | `BuildingFamily::Logistics` / `SiteArchetype::Factory` |

---

## FootprintMatrix (authoritative for ghost)

**Grid:** 6 wide × 5 deep (row-major `cells`).

```text
Row 0 (rail edge):  1 1 1 1 1 1   ← main hall
Row 1:              1 1 1 0 0 0   ← hall + loading start
Row 2:              1 1 0 0 0 0   ← L leg
Row 3:              0 0 0 0 0 0
Row 4:              0 0 0 0 0 0
```

| Metric | Value |
|:---|:---:|
| Occupied cells | **11** |
| Bounding box | 6×5 = 30 |
| Occupancy vs bbox | **36.7%** |
| N occupied ≠ W×D | **11 ≠ 30** ✓ (rotate QA) |

**RON path (mock shapes):** [`assets/configs/buildings/_mock_shapes.ron`](../assets/configs/buildings/_mock_shapes.ron) — shape id `logistics_rail_warehouse_l_6x5`

**RON path (pilot catalog):** [`assets/configs/buildings/_pilot_catalog.ron`](../assets/configs/buildings/_pilot_catalog.ron) — pilot id `logistics_rail_warehouse_v0`

**JSON bundle (MCP / site):** [`assets/configs/buildings/pilots/logistics_rail_warehouse_pilot_v1.json`](../assets/configs/buildings/pilots/logistics_rail_warehouse_pilot_v1.json)

**Authority:** RON cells are **canonical** for Bevy tray load (`register_pilot_catalog_from_ron`); JSON mirrors for validate-report / site stub refs.

---

## ARCH-DNA + β (preset row — no duplicate)

Use existing **`arch_dna_logistics_rail_warehouse_v0.json`** — do not fork DNA.

| β (v0) | Value | Shape effect |
|:---|:---:|:---|
| βsym | .72 | RailEdge / symmetric hall |
| βyard | .93 | Site utility zone large |
| βsvc | .88 | Service block |
| βexp | .84 | Links Shift+scale |
| βmod | .92 | Module runs |

**Massing weight override** (from preset): favors `l_shape` + `yard_complex`.

---

## Site zones (data-only v0 — overlay in SITE-v0-002)

| Zone id | Role | Notes |
|:---|:---|:---|
| `primary` | warehouse hall | FootprintMatrix cells |
| `loading` | loading wing | Attached L leg — part of matrix |
| `utility` | utility yard | Void overlay |
| `rail` | rail spur | Infrastructure edge |

Site plan ASCII: [`design_build_readability_v1.md`](design_build_readability_v1.md) §2b.

---

## Tray / catalog wire (BUILD-READ-SHAPE-002)

1. Register `pilot:logistics_rail_warehouse_v0` from pilot JSON + mock shape cells.
2. Industrial submenu → **Rail Warehouse (pilot)** row.
3. Ghost uses matrix cells — not `from_size(w,d,true)`.
4. `mock_shapes_parity_green()` extended or pilot-specific witness.

---

## Validation gates (@coder / MCP consumer)

```bash
node .claude/skills/agent-lang/driver.mjs validate-report mcp_spec assets/configs/buildings/pilots/logistics_rail_warehouse_pilot_v1.json --compress 4
```

| Gate | Pass |
|:---|:---:|
| cells.len == width × depth | ✓ |
| occupied ≥ 8 | ✓ |
| occupied < width × depth | ✓ |
| preset_id links to arch_dna example | ✓ |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-06-13 |
| `@coder` | **BUILD-READ-SHAPE-002** done (tray + registry) | 2026-06-11 |

```text
BUILD-READ-SHAPE-001 complete
Matrix RON on disk (_mock_shapes.ron + _pilot_catalog.ron)
Witness: debug_runs/design_shape_rail_warehouse_pilot_live.json green
BLANG:Q✓ DESIGN-BUILD-READ-SHAPE-001
```
