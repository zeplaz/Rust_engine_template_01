# Water VFX — designer review record `WATER-DESIGN-001`

| Field | Value |
|:---|:---|
| **Review ID** | `WATER-DESIGN-001` |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Reviewer** | Design pass (Auto) |
| **Status** | **SIGNED — TUNE ROUND** (not track **CLOSED**) |
| **Build / commit** | `b2341a6` |
| **Brief** | [`prompts/guides/ui/vfx_post_implementation_review_v1.md`](../prompts/guides/ui/vfx_post_implementation_review_v1.md) § water |
| **Closure track** | [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) |
| **Design (W0)** | [`water_surface_vfx_design_plan_v1.md`](water_surface_vfx_design_plan_v1.md) **SIGNED** |
| **Mock** | [`assets/vfx/reference/water/water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) |
| **Witness** | [`debug_runs/stage5_full_app_live.json`](../debug_runs/stage5_full_app_live.json) |
| **Combined review** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) — **D-VFX** |

---

## Executive summary

**W1 + W2 first pass is real** (`water_w1_green`, tactical particles, river streaks in witness). **Player read is not design-complete:** ocean branch unproven (`water_ocean_tiles: 0`), bend/coast foam absent, river-vs-lake distinction at **strategic** zoom unverified in capture.

**Designer verdict:** ☑ **SIGNED — TUNE ROUND** — explicit coder tickets below. **Does not** mark **FX-WATER CLOSED** until witness exit + optional capture re-review.

---

## Prerequisites

| Gate | Required | Observed | Met |
|:---|:---|:---|:---:|
| P2 tactical proof | `water_tactical_zoom`, `zoom_alpha ≥ 0.65` | `0.85`, `tactical_witness_gates.all_green: true` | ☑ |
| W1 spine | `water_w1_green: true` | `true` | ☑ |
| W2 particles | `water_particle_rows > 0` at tactical | `96` | ☑ |
| River data | segments / tiles | `15` / `10241` | ☑ |
| Visual run | `--test visual` refreshed witness | `stage5_full_app_live.json` | ☑ |

---

## Captures

| File | Status | Use |
|:---|:---|:---|
| `assets/vfx/reference/review_captures/water_lake_tactical_20260524.png` | **PENDING** | Lake panel vs mock |
| `assets/vfx/reference/review_captures/water_river_tactical_20260524.png` | **PENDING** | River strip — must differ from lake |
| `assets/vfx/reference/review_captures/water_ocean_tactical_20260524.png` | **BLOCKED** | No ocean tiles in current map — see **WATER-DESIGN-002** |

**Procedure:** `cargo run -p proc_A_dine01 --release` → Simulation → tactical zoom (~40–70% map fill) → save PNGs under `review_captures/`.

**Interim evidence:** witness JSON + shader/code audit (this record). **Not** a substitute for final **PASS** on captures.

---

## Checklist — lake (D-W02, D-W06)

Compare vs mock **lake** panel ([`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) left).

| ID | Check | Evidence | Verdict |
|:---|:---|:---|:---|
| **D-W02** | Slow ripple / tonal variation — not flat teal slab | `water_shader_motion_always_on: true`; `water_overlay.wgsl` lake ripple branch; `water_lake_tiles: 13096` | **PASS** (code + witness) |
| **D-W06** | Lake glints optional — sparse pinpoint, not blobs | `water_particle_lake_glints: 72`; `water_particle_draw.wgsl` ≤2px step | **PASS** (witness) · capture pending for twinkle read |

| **Lake channel** | **PASS** (tactical) |

---

## Checklist — river (D-W01, D-W03, D-W07)

Compare vs mock **river** strip (center) — **must read as channel**, not lake teal.

| ID | Check | Evidence | Verdict |
|:---|:---|:---|:---|
| **D-W01** | River visible as channel | W1 `river_flow_scroll` in `water_overlay.wgsl`; catalog `water_river_segments: 15` | **TUNE** — data + shader landed; **strategic** side-by-side read **not** captured |
| **D-W03** | Directional flow along path | `water_particle_river_streaks: 24`; overlay UV scroll | **TUNE** — streaks in witness; confirm motion in PNG |
| **D-W07** | Downstream streaks + **bend foam** | Streaks ☑ · `water_particle_river_foam: 0` | **TUNE** → **WATER-W2-FOAM-001** |

| **River channel** | **TUNE** |

---

## Checklist — ocean (D-W04, D-W08)

Compare vs mock **ocean** panel (right).

| ID | Check | Evidence | Verdict |
|:---|:---|:---|:---|
| **D-W04** | Deeper tone / swell / haze vs lake | Ocean branch in `water_overlay.wgsl`; witness `water_ocean_tiles: 0` | **TUNE** → **WATER-W1-OCEAN-001** + fixture (**WATER-DESIGN-002**) |
| **D-W08** | Coast foam at shallow/deep boundary only | `water_particle_coast_foam: 0` | **TUNE** → **WATER-W2-FOAM-001** (after ocean tiles exist) |

| **Ocean channel** | **TUNE** (unproven on current map) |

---

## Checklist — particles & zoom (D-W05, D-W09, D-W10)

| ID | Check | Evidence | Verdict |
|:---|:---|:---|:---|
| **D-W05** | Pinpoint glints/streaks (fire family) | `water_particle_rows: 96`; custom WGSL spine | **PASS** |
| **D-W09** | Particles fade strategic; shaders always on | Tactical: rows > 0, `motion_always_on: true` · Strategic band **not** exercised in this witness run | **TUNE** → **WATER-STRATEGIC-001** |
| **D-W10** | Custom WGSL, no Hanabi | `water_particle.wgsl` + `gpu_water_particle_draw.rs` | **PASS** |

| **Particles / zoom** | **TUNE** (strategic cull unproven in visual proof) |

---

## Overall verdict

| Channel | Verdict |
|:---|:---|
| Lake | **PASS** |
| River | **TUNE** |
| Ocean | **TUNE** |
| Particles / zoom | **TUNE** |
| **Water overall** | ☑ **TUNE** · ☐ PASS · ☐ FAIL |

**WATER-DESIGN-001 sign-off:** ☑ **SIGNED — TUNE ROUND** (tune tickets filed; blocks **FX-WATER CLOSED** until W1–W4 exit criteria green).

---

## Tune tickets → coder slices

| # | Channel | Issue | Slice | Owner |
|:---|:---|:---|:---|:---|
| **W-T01** | River | Ribbon read at **strategic** zoom — still lake-like at glance | **WATER-W1-RIVER-001** | A |
| **W-T02** | Ocean | `water_ocean_tiles: 0` — swell/haze unproven | **WATER-W1-OCEAN-001** | A |
| **W-T03** | River | Bend foam `water_particle_river_foam: 0` | **WATER-W2-FOAM-001** | B |
| **W-T04** | Ocean | Coast foam `water_particle_coast_foam: 0` | **WATER-W2-FOAM-001** | B |
| **W-T05** | Zoom | D-W09 strategic particle cull not in visual witness | **WATER-STRATEGIC-001** | B |
| **W-T06** | Witness | Foam + ocean gates in harness | **WATER-WITNESS-001** | B |
| **W-T07** | Polish | Pinpoint density / streak elongation after foam | **WATER-W2-TUNE-001** | A+B |

**Optional designer follow-up:** **WATER-DESIGN-002** — name one world-gen seed or unit fixture with deep-water band for W-T02 captures.

---

## Blocks / unblocks

| Item | Effect |
|:---|:---|
| **WATER-DESIGN-001 SIGNED (TUNE)** | Unblocks coder **WATER-W1/W2/WITNESS** slices; **does not** close FX-WATER track |
| **FX-WATER CLOSED** | Still requires § witness exit in [`water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) + upgrade to **PASS** (captures optional but recommended) |

---

## Sign-off table

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED — TUNE ROUND** | Witness + code audit; PNG captures pending |
| Coder | — | Acknowledged | Execute W-T01…W-T07 in closure plan order |

---

## Re-review trigger

Re-run **WATER-DESIGN-001** (bump record version) when:

1. `water_ocean_tiles > 0` **or** documented fixture waiver
2. `water_particle_river_foam > 0` and/or `water_particle_coast_foam > 0` (or documented “no bends/coast in fixture”)
3. `WATER-STRATEGIC-001` proves `water_particle_rows == 0` at strategic with `water_shader_motion_always_on: true`
4. Optional: three tactical PNGs saved — flip channel verdicts to **PASS** where mocks match

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Initial **WATER-DESIGN-001**; SIGNED TUNE ROUND; witness `b2341a6` |
