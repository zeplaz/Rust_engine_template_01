# Build readability D-001…D-004 + variant matrix `v1`

| Field | Value |
|:---|:---|
| **Programs** | **BUILD-READ-D-001** · **BUILD-READ-D-002** · **BUILD-READ-D-003** · **BUILD-READ-D-004** |
| **Parent** | **⟨PLAN-BUILD-READABILITY-001⟩** · **⟨PLAN-PROC-TILE-PROD-001⟩** (TILE-PROD-004) |
| **Date** | 2026-06-14 |
| **Owner** | `@designer` (charter) · `@designer-mcp` (matrix YAML) · `@coder-mcp` (bake) |
| **Verdict** | **PASS** |
| **Pilot** | `logistics_rail_warehouse_v0` · L-footprint 6×5 |
| **Witness** | [`debug_runs/design_build_read_d001_d004_live.json`](../debug_runs/design_build_read_d001_d004_live.json) |
| **Matrix** | [`debug_runs/art_pipeline/variant_matrix_rail_warehouse_pilot_v1.yaml`](../debug_runs/art_pipeline/variant_matrix_rail_warehouse_pilot_v1.yaml) |

**No Rust.** Four readability dimensions + variant matrix contract for pilot iso tiles.

---

## Chart — four dimensions

```text
D-001 Screen scale ──▶ D-004 Variant matrix
D-002 Footprint UX  ──▶     (sim → variant_key → 128px read)
D-003 Site 15–40%   ──▶
         ▲
    WORLD-002 iso ×1.35–1.75
```

---

## D-001 — Screen scale + draw authority

**Source:** [`design_build_readability_v1.md`](design_build_readability_v1.md) §1

| Element | Target @ 1280×720 · α≈0.42 |
|:---|:---|
| Chunk tactical span | ≥ 280 px |
| Site stub box (10×8) | 120–200 px |
| Primary structure | **40–90 px** iso height (**15–40%** of site box) |
| Ghost tile edge | ≥ 24 px |
| Single tile | ≥ 8 px |

**Draw authority:**

| Condition | Draw |
|:---|:---|
| Production iso tile in registry | **Tile stamp** + outline |
| PG-2 lod0+ mesh + no atlas | Mesh + footprint outline |
| Primary < 40 px tall | **Tile grid only** — no empty bbox |
| Ghost (pre-commit) | Footprint tiles only |

**WORLD-002:** `iso_draw_scale_multiplier` **1.35–1.75** — primary must **read** inside site, not fill chunk.

---

## D-002 — Footprint + Adjust UX

**Sources:** [`design_build_readability_v1.md`](design_build_readability_v1.md) §1b · [`design_build_toolbox_hud_v1.md`](design_build_toolbox_hud_v1.md)

| State | Visual | Strip copy |
|:---|:---|:---|
| Valid | Green α≤0.35 + 2px outline | — |
| Risky | Amber hatch | ` · risky overlap` |
| Invalid | Red hatch | ` · blocked: {reason}` |
| Adjust locked | Gold ring | `locked {x},{z} · Ctrl rotate · Shift scale` |

**Weighted raster:** partial cells use valid hue; α = weight (0.2–1.0).

---

## D-003 — Site composition (15–40%)

**Sources:** [`design_build_readability_v1.md`](design_build_readability_v1.md) §2 · [`logistics_rail_warehouse_site_v0.json`](../assets/configs/buildings/pilots/logistics_rail_warehouse_site_v0.json)

```text
Rule: Site > building · Primary ≈ 15–40% of SITE (not chunk)
Pilot: 10×8 site · 10 building cells = 12.5% (lower bound OK with WORLD-002 scale)
```

| Zone | Overlay | Label |
|:---|:---|:---|
| primary + loading | `footprint_valid` | — / `Load` |
| utility | amber dashed | `Yard` |
| rail | grey parallel lines | `Rail` |
| service | cyan outline | `Svc` |
| parking | dark void | `Park` |

**View-only** — overlay from site JSON; no commit mutation.

---

## D-004 — Variant matrix + player read

**Purpose:** Map sim context → `variant_key` → **128px iso readability** at operational zoom (TILE-PROD-004 / G4-5).

**Matrix file:** [`variant_matrix_rail_warehouse_pilot_v1.yaml`](../debug_runs/art_pipeline/variant_matrix_rail_warehouse_pilot_v1.yaml)

### D-004a — Sim → variant_key (pilot v1)

| Sim inputs | `variant_key` | Player must read @ α0.42 |
|:---|:---|:---|
| Operational · day · power on | `clean_day` | L-hall + rail-edge roof; yard void beside hall |
| Operational · night · power off | `clean_night_off` | Dark mass; no window glow |
| Operational · night · power on | `clean_night_on` | **T4** — loading-bay / office emissive dots |
| UnderConstruction | `under_construction_01` | Scaffold / open bay; site overlay `under_construction` |
| Damaged (defer v1.1) | `damaged_day` | **T5** — wear distinct from clean |
| Fire heat ≥ threshold (defer) | `burning_00`…`03` | **T6** — facade flame, not orange wash |

**Resolver:** [`tile_variant_resolver.rs`](../construction/procedural/tile_variant_resolver.rs) · catalog [`_variant_catalog.ron`](../assets/configs/buildings/_variant_catalog.ron)

### D-004b — Required keys (pilot ship floor)

| Tier | Keys | Count |
|:---|:---|:---:|
| **v1 required** | `clean_day`, `clean_night_off`, `clean_night_on`, `under_construction_01` | 4 |
| **v1.1** | `damaged_day`, `damaged_night_on` | +2 |
| **fire lane** | `burning_00`…`burning_03` (pilot subset) | +4 |

**Bake:** `keyframe_pack` · seed `440013` · [`tile_rail_warehouse_pilot_v1.json`](../assets/staging/specs/tile_rail_warehouse_pilot_v1.json)

### D-004c — 128px G4 rubric (pilot)

| Gate | Pass when |
|:---|:---|
| **G4-3** | `clean_day`, `clean_night_on`, `under_construction_01` reviewed @ 128px |
| **G4-5** | `clean_night_on` emissive readable without zoom (T4) |
| **G4-site** | Primary silhouette **smaller than site dashed box** in still review |
| **G4-L** | L-footprint leg visible — not square bbox fill |

Sign-off YAML: [`rail_warehouse_pilot_production_signoff.yaml`](../debug_runs/art_pipeline/rail_warehouse_pilot_production_signoff.yaml)

### D-004d — Mesh vs tile at commit

| After commit | Draw |
|:---|:---|
| Atlas hit for `pilot:logistics_rail_warehouse_v0` | **Iso tile stamp** (suppress PG-2 mesh) |
| No atlas / lod0 only | Footprint outline + greybox warning in debug |
| Site overlay on | Zone hatch **under** tile stamp |

---

## Acceptance (operator + G4)

| # | Pass |
|:---:|:---|
| 1 | D-001: primary reads **inside** site box at default zoom |
| 2 | D-002: valid/risky/invalid + Adjust ring on L matrix |
| 3 | D-003: yard + rail labels visible on site overlay |
| 4 | D-004: 4 required variant keys in matrix YAML |
| 5 | D-004: G4-3 minimum stills pass @ 128px (post bake) |

---

## Handoff

| Slice | Owner | Do |
|:---|:---|:---|
| **BUILD-READ-VISUAL-002** | @coder-mcp | Bake 4 keys from matrix |
| **BUILD-READ-VISUAL-001** | @coder | Stamp + suppress mesh when atlas hit |
| **BUILD-READ-SITE-v0-002** | @coder | Overlay from site JSON |
| **TILE-PROD-004** | @designer-mcp | G4 sign-off after stills land |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-14 |
| `@designer-mcp` | **PASS** (matrix on disk) | 2026-06-14 |
| `@coder-mcp` | pending bake | — |

```text
BUILD-READ-D-001…D-004 complete
Variant matrix + readability contract for rail warehouse pilot
```
