# Build readability + site composition `v1` (BUILD-READ-DESIGN-001 · SITE-v0-001 · WORLD-001)

| Field | Value |
|:---|:---|
| **Programs** | **BUILD-READ-DESIGN-001** · **BUILD-READ-SITE-v0-001** · **BUILD-READ-WORLD-001** |
| **Parent** | **⟨PLAN-BUILD-READABILITY-001⟩** |
| **Date** | 2026-06-13 |
| **Owner** | `@designer` (charter) · `@coder` (BUILD-READ-WORLD-002 · SITE-v0-002) |
| **Verdict** | **PASS** |
| **Guide** | [`prompts/guides/build_grammer2_exman.md`](../prompts/guides/build_grammer2_exman.md) §SITE-GRAMMAR · §ARCH-DNA EXAMPLE |
| **Exec** | [`plan_operator_build_readability_exec_001_v1.md`](plan_operator_build_readability_exec_001_v1.md) |
| **Grammar v0** | [`arch_build_grammar_v0_baseline_v1.md`](arch_build_grammar_v0_baseline_v1.md) |
| **Witness** | [`debug_runs/design_build_readability_live.json`](../debug_runs/design_build_readability_live.json) |
| **Unblocks** | **BUILD-READ-WORLD-002** · **BUILD-READ-SITE-v0-002** · **BUILD-READ-DESIGN-002** |

**No Rust in this doc.** Readability rules + site mock + scale table only.

---

## Mission

Fix **doll-house on green chunk**: buildings must read as **one layer inside a site**, not a rectangle filling the world tile. Primary structure occupies **15–40% of the site stub**; yard, service, and rail are **first-class void** at default sim zoom.

**Acceptance test:** *At default operational zoom, operator can name warehouse vs yard vs rail spur on the site stub overlay — primary footprint is visibly smaller than the site box.*

---

## 1. Readability brief (BUILD-READ-DESIGN-001)

### 1a. Screen height vs chunk (default sim)

Assume **1280×720** window, map hole ≈ **900×520 logical px**, default `zoom_alpha ≈ 0.42` (operational play).

| Element | Target screen height | Notes |
|:---|:---:|:---|
| **Full chunk** (visible tactical span) | **≥ 280 px** | Operator orients district |
| **Site stub box** (v0 overlay) | **120–200 px** | Dashed border; labels readable |
| **Primary structure** (committed mesh/tile) | **40–90 px** tall iso read | **15–40% of site box height** |
| **Ghost footprint (Adjust)** | **≥ 24 px** tile edge | Below = show **tile grid only**, hide mesh preview |
| **Single tile** | **≥ 8 px** | Under = merge into heat-style fill |

**Rule:** If primary iso mesh < 40 px tall at default zoom, prefer **iso tile stamp + outline** until BUILD-READ-WORLD-002 scale lever lands.

### 1b. Footprint partial-alpha language

| State | Visual | Copy |
|:---|:---|:---|
| **Valid** | Green fill α ≤ 0.35 + 2 px outline | `Valid placement` |
| **Risky** | Amber hatch (`footprint_risky_color`) | `Risky — overlap or terrain` |
| **Invalid** | Red hatch (`footprint_invalid_color`) | `Blocked — {reason}` |
| **Adjust locked** | Gold ring (per [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md)) | `Locked — Ctrl rotate · Shift scale` |

**Weighted raster:** partial-alpha cells use **same hue** as valid; α = `weight` (0.2–1.0). Shift+scale must **monotonically** change occupied cell count or max weight — never silent bbox-only scale.

### 1c. Mesh vs iso tile authority

| Condition | Draw |
|:---|:---|
| PG-2 extract **lod0+** mesh loaded for catalog id | Mesh + footprint outline |
| Production iso tile in registry | Tile stamp + state row |
| Fallback / greybox only | **Tile grid only** — no empty bbox |
| Ghost (pre-commit) | Footprint tiles only — never commit mesh |

**Authority:** `ConstructionVisualRequests` for ghost; post-commit via PG-2 / registry — not parallel egui mesh.

---

## 2. Site composition mock (BUILD-READ-SITE-v0-001)

### 2a. Meta-rule (locked)

```text
Site > building · Function > shape · Primary ≈ 15–40% of SITE (not chunk)
```

### 2b. v0 site stub — Industrial Rail Warehouse pilot

**Site bounds:** **10×8 world tiles** (80 cells) — overlay only; does not change commit bbox alone.

**ARCH-DNA preset:** `logistics_rail_warehouse_v0` — [`arch_dna_logistics_rail_warehouse_v0.json`](../tools/mcp/schemas/examples/arch_dna_logistics_rail_warehouse_v0.json)

```text
SITE 10×8 (N = rail / road edge)
┌─R─R─R─R─P─P─P─P─P─P─┐  R=rail spur  P=parking void
│ R · · · · · · · · · │
│ W W W W W W · Y Y Y │  W=primary hall  Y=utility yard
│ W W W L L · · Y · · │  L=loading wing
│ · · S S · · · · · · │  S=service block
│ · · · · · · · · · · │
└─────────────────────┘
```

| Zone | Role | Cells | Overlay style |
|:---|:---|:---:|:---|
| **primary** | Warehouse hall | 6 | Valid green footprint |
| **loading** | Rail-edge wing | 4 | Valid green (attached) |
| **utility** | Yard / tanks | 12 | **Void** — amber dashed hatch, label `Yard` |
| **rail** | Spur | 4 | **Void** — grey rail glyph, label `Rail` |
| **service** | Utility bldg | 2 | Cyan outline, label `Svc` |
| **parking** | Edge | 10 | Dark grey void, label `Park` |
| **remainder** | Buffer / fence | 42 | Terrain base only |

**Primary + loading = 10 cells → 10/80 = 12.5%** (v0 pilot — lower bound; **WORLD-002** may bump iso draw scale so primary **reads** 15%+ without filling site).

**Occupancy check (design intent):** primary structure **never** uses full 10×8; max primary bbox **6×5** (30 cells) capped at **40%** (32 cells).

### 2c. Site zone colors (coder overlay BUILD-READ-SITE-v0-002)

| Zone id | Fill | Label |
|:---|:---|:---|
| `primary` | `footprint_valid_color` @ α0.35 | — |
| `loading` | same family | `Load` |
| `utility` | amber dashed hatch α0.15 | `Yard` |
| `rail` | `#888888` α0.25 + parallel lines | `Rail` |
| `service` | cyan outline | `Svc` |
| `parking` | `#444` α0.12 | `Park` |

**View-only:** overlay reads from site plan JSON — **no** new commit path.

---

## 3. Site-scale table (BUILD-READ-WORLD-001) — blocks WORLD-002

Coder picks **one primary lever** in BUILD-READ-WORLD-002; designer recommends **Option A first**.

| Lever | What changes | Player read | Risk |
|:---|:---|:---|:---|
| **A — Iso draw scale multiplier** | Post-commit building draw × **1.35–1.75** at operational zoom | Faster fix; primary reads larger **inside** same site | Must not break pick/ghost alignment |
| **B — Chunk sub-tile variation** | Terrain interior noise / biome micro-patches | World feels bigger; building unchanged | Art + streaming scope |

**Recommendation:** **A primary** for BUILD-READ-WORLD-002; **B defer** to terrain triage.

### 3a. Scale targets (WORLD-002 acceptance)

| Metric | Today (operator) | Target |
|:---|:---|:---|
| Primary % of **site stub** | ~95% (one rect = chunk) | **15–40%** occupied tiles |
| Primary % of **chunk** | ~95% | **≤ 45%** with site visible |
| Yard void visible at `zoom_alpha 0.42` | No | **Yes** — labeled |
| Rotate QA (non-square) | All squares | **L pilot** in tray ([`BUILD-READ-SHAPE-001`](design_shape_rail_warehouse_pilot_v1.md)) |

### 3b. WORLD-002 witness fields (for @coder)

```json
{
  "primary_pct_site_stub": 0.125,
  "primary_screen_px_height": 55,
  "site_stub_screen_px_height": 160,
  "lever": "iso_draw_scale_multiplier",
  "iso_draw_scale_multiplier": 1.5
}
```

---

## 4. Adjust mode HUD (BUILD-READ-DESIGN-002 preview)

Locked copy (extends [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md)):

| Surface | Text |
|:---|:---|
| Context strip (Adjust) | `BUILD · locked {x},{z} · Ctrl rotate · Shift scale · click to place` |
| Build rail tooltip | `Industry · Rail warehouse pilot — L footprint for rotate QA` |
| Site overlay legend | `Site stub · green=building · dashed=yard/rail` |

---

## 5. Acceptance (operator)

| # | Pass |
|:---:|:---|
| 1 | Lock ghost → Shift+scale → **tile count or weight area increases** |
| 2 | Ctrl+rotate on **L matrix** → occupied set rotates; bbox ≠ square fill |
| 3 | Site overlay shows **yard + rail** voids at default zoom |
| 4 | Primary reads **smaller than site box** (not full chunk) |
| 5 | Valid / risky / invalid colors match §1b |

---

## 6. Coder handoff

| Slice | Read | Do |
|:---|:---|:---|
| **BUILD-READ-SITE-v0-002** | §2c | View-only site stub overlay from plan JSON |
| **BUILD-READ-WORLD-002** | §3 | Option A iso multiplier; witness §3b |
| **BUILD-READ-DESIGN-002** | §4 | HUD copy wire |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-13 |
| `@coder` | pending WORLD-002 / SITE-v0-002 | — |

```text
BUILD-READ-DESIGN-001 + SITE-v0-001 + WORLD-001 complete
Unblocks: BUILD-READ-WORLD-002 · BUILD-READ-SITE-v0-002 · BUILD-READ-SHAPE-002
```
