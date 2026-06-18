# Facility site zone taxonomy `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-FACILITY-SITE-ZONE-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E1-B |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Schema** | `site_zone_grid_v1` · loader [`site_zone_grid.rs`](../construction/site_zone_grid.rs) |
| **Pilots** | `assets/configs/buildings/pilots/*_site_v0.json` (4 grids) |
| **Research** | [`design_industrial_process_research_v1.md`](design_industrial_process_research_v1.md) |
| **Verdict** | **PASS** |

```text
DES-FACILITY-SITE-ZONE-001 Q✓
Seven zone ids — canonical spellings match engine SiteZoneCell parser
```

---

## 0. Canonical zone ids

| Zone id | `SiteZoneCell` | Purpose | Overlay read |
|:---|:---|:---|:---|
| **primary** | `Primary` | Building footprint / production hall | solid fill `footprint_valid` |
| **loading** | `Loading` | Truck dock, batch queue, outbound | solid + label `Load` |
| **utility** | `Utility` | Stacks, tanks, substation yard, cooling | hatch `yard_void` |
| **rail** | `Rail` | Spur, siding, tipple | `parallel_lines` glyph |
| **service** | `Service` | Office, control, maintenance | outline `service_outline` |
| **parking** | `Parking` | Staff / fleet staging | light void `parking_void` |
| **buffer** | `Void` | Setback, greenfield, future expansion | no fill (`.`) |

**Spelling rule:** JSON `cells[]` use **lowercase** ids above. Unknown strings → `buffer` (void).

**Ban:** inventing zone ids outside this set without schema bump + designer amend.

---

## 1. Zone semantics

### primary
Occupied by main structure massing. **Metrics:** `primary_cells / site_cells` = `primary_pct_site`.

### loading
Must touch **site edge** that faces public road or internal haul road — validator warns if loading fully enclosed by primary.

### utility
Equipment that is **not** indoor production: substation pads, cooling, coal pile, tank dike. Heavy process steps (kiln, smelter) require **utility_pct_min** per research doc.

### rail
Optional except **coal plant** and **logistics_rail_warehouse** archetypes. Cells should form contiguous spur ≥2 cells.

### service
Small footprint — typically 1–4 cells. May abut primary or sit in buffer edge.

### parking
Staff / light fleet. Never required for mines; common on warehouse + fabrication.

### buffer
Default fill. Absorbs expansion. **Not** counted in `building_cells` metric.

---

## 2. Pilot inventory (v0 baseline)

| Pilot | Site JSON | W×D | Zones used | primary% |
|:---|:---|:---:|:---|:---:|
| `logistics_rail_warehouse_v0` | `logistics_rail_warehouse_site_v0.json` | 10×8 | all 7 | 12.5% |
| `manufacturing_fabrication_hall_v0` | `manufacturing_fabrication_hall_site_v0.json` | 8×6 | 6 (no rail) | 25% |
| `fuel_depot_tank_farm_v0` | `fuel_depot_tank_farm_site_v0.json` | 7×6 | 6 (no rail) | ~29% |
| `power_substation_yard_v0` | `power_substation_yard_site_v0.json` | 6×5 | 4 (utility ring) | ~27% |

**Reference layout:** rail warehouse = north rail + south parking + loading wing on primary — [`design_shape_rail_warehouse_pilot_v1.md`](design_shape_rail_warehouse_pilot_v1.md).

---

## 3. Required vs optional matrix (by archetype)

**Legend:** **R** required · **O** optional · **—** omit · **%** = minimum site % when R

| Zone | Logistics warehouse | Factory cluster | Process kiln/smelter | Tank farm | Substation yard |
|:---|:---:|:---:|:---:|:---:|:---:|
| **primary** | R 10% | R 12% | R 12% | R 15% | R 20% |
| **loading** | R 5% | O | R 5% (mixer/fab) | O | — |
| **utility** | O | O 10% | **R 20%** | **R 40%** | **R 50%** |
| **rail** | **R** spur | O | O (coal) | O | — |
| **service** | O | O | O | R 5% | R 5% |
| **parking** | R 5% | O | — | O | — |
| **buffer** | R 30% | R 25% | R 25% | R 20% | R 20% |

### Archetype → pilot lineage

| Archetype | Maps to grammar / pilot | Default site template |
|:---|:---|:---|
| **IndustrialWarehouse** | `logistics_rail_warehouse_v0` | §2 rail warehouse grid |
| **FactoryCluster** | `manufacturing_fabrication_hall_v0` | parking north, utility east |
| **RailEdge** | `logistics_rail_warehouse_v0` | same as warehouse — rail on long edge |
| **Tank farm / refinery** | `fuel_depot_tank_farm_v0` | utility wraps primary |
| **Grid yard** | `power_substation_yard_v0` | utility ring |
| **Concrete mine** | *new* | buffer-heavy, low primary |
| **Cement kiln** | *new* | utility south/west |
| **Concrete mixer** | *new* | loading on road edge |

---

## 4. Validator rules (CMCP-SITE-ZONE-VALIDATE-001)

| Rule id | Check | Severity |
|:---|:---|:---:|
| `SZ-01` | All cell ids ∈ taxonomy §0 | error |
| `SZ-02` | `width × depth == len(cells)` | error |
| `SZ-03` | `primary_pct_site` ≥ archetype minimum | warn |
| `SZ-04` | `utility_pct` ≥ minimum when role ∈ {kiln, smelter, refinery, substation} | warn |
| `SZ-05` | `loading` touches site perimeter | warn |
| `SZ-06` | `rail` contiguous ≥2 cells when present | warn |
| `SZ-07` | Orphan `service` not adjacent to primary or buffer edge | info |
| `SZ-08` | `zone_legend` keys match ascii_plan chars | error |

**Witness:** `debug_runs/site_zone_validate_live.json`

---

## 5. Overlay tokens (Bevy + APS preview)

Reuse pilot `zone_styles` tokens — do not invent per-site hex:

| Token | Use |
|:---|:---|
| `footprint_valid` | primary, loading |
| `yard_void` | utility |
| `rail_void` | rail |
| `service_outline` | service |
| `parking_void` | parking |
| `none` | buffer |

**APS site preview (G2+):** collapsed legend row — `P L U R S K ·` with hover tooltips per zone id.

---

## 6. Relationship to building footprint

```text
Building FootprintMatrix (grammar)  →  sits ON primary zone cells
Site zone grid                      →  superset — includes yards not in massing
```

**Rule:** `occupied_cells` ≤ `primary_cells` on site grid. Validator compares grammar footprint bbox vs primary region overlap ≥80%.

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |

**Unblocks:** CMCP-SITE-ZONE-VALIDATE-001 · DES-APS-SITE-PREVIEW-001 · concrete site pilots
