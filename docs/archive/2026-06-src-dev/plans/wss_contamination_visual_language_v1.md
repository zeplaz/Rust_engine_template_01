# DESIGN-CONTAMINATION-001 — WSS contamination visual language `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-CONTAMINATION-001** |
| **Coder lane** | **A-W2** · **WSS-ATMOS-CLIPMAP-001** (AC-001..004) |
| **Planner** | [`wssr_plan_004_atmosphere_unification_v1.md`](wssr_plan_004_atmosphere_unification_v1.md) · [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) |
| **Baseline contract** | [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) § Contamination |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | Contamination overlay UI + F3 diagnostics before AC wiring |
| **No Rust** | Color, pattern, zoom, overlay channel names |

---

## Purpose

**`ContaminationState` is separate from `AtmosphereCell`.** Players must read **five channels** without confusing smoke, fog, or water VFX. This doc is the **implementation-facing** slice of the migration contract for A-W2.

**Forbidden:** merging toxin storage into atmosphere cells; neon arcade hazard colors; color-only discrimination.

---

## Channel vocabulary (sim → player)

| Channel | Sim field | Player noun | Primary read surface |
|:---|:---|:---|:---|
| **airborne** | `ContaminationState.airborne` | Plume / haze toxin | Tactical map tint + column hint |
| **soil** | `.soil` | Ground spill | Stipple on terrain |
| **waterborne** | `.waterborne` | Runoff / bloom | River ribbon + wet tint |
| **bioactive** | `.bioactive` | Organic hazard | Dot grid overlay |
| **radiation** | `.radiation` | Fallout / waste | Dashed contour |

**Atmosphere `toxic_hazard`:** derived **sample** for AI/sensors — **not** a sixth storage channel. Visual = warm desaturate haze only, no duplicate pattern.

---

## Tactical map overlay (primary)

Applied via strategic overlay / tile debug family — **not** egui-only.

| Channel | Fill RGBA | Pattern | α range |
|:---|:---|:---|:---|
| airborne | `(180,160,90)` | Horizontal crosshatch | 0.12–0.35 |
| soil | `(120,80,50)` | Stipple corner-weighted | 0.15–0.40 |
| waterborne | `(40,100,110)` | Flow-aligned streaks | 0.10–0.30 |
| bioactive | `(140,160,80)` | Dot grid 4px | 0.10–0.28 |
| radiation | `(200,80,160)` drafting ink | Dashed contour 6/4 | 0.12–0.32 |

**Intensity:** `α = lerp(α_min, α_max, saturate(concentration / channel_max))` per chunk cell.

**Stacking:** multiple channels on one cell → draw **patterns in fixed order** (soil → waterborne → airborne → bioactive → radiation) with **max 3** patterns visible; highest concentration channel wins icon.

---

## Glyph + label (tile inspector / F3)

| Channel | Glyph | Short label |
|:---|:---:|:---|
| airborne | ☁ | `Air` |
| soil | ⛏ | `Soil` |
| waterborne | ≈ | `Water` |
| bioactive | ⚕ | `Bio` |
| radiation | ☢ | `Rad` |

F3 row template (per chunk, when `RUST_ENGINE_WSS_DIAG=1`):

```text
AC plume={airborne:.2} soil={soil:.2} H2O={waterborne:.2} bio={bioactive:.2} rad={radiation:.2}
```

---

## Coupling events (motion)

| Event | Visual sequence |
|:---|:---|
| **Toxic rain** | Airborne crosshatch pulses 2 ticks → waterborne streaks strengthen along hydrology mask |
| **Washout** | Soil stipple fades 8 ticks; waterborne dilutes toward baseline |
| **Industrial fire** | Smoke column (Layer B) + airborne increment; optional warm tint sync |
| **Deposition** | Airborne high → soil stipple grows from plume footprint |

Animations are **field lerp**, not particle spam.

---

## Zoom bands

| Band | Contamination display |
|:---|:---|
| **Tactical** | Full patterns + glyphs on hover |
| **Operational** | Envelope only — dominant channel color field, patterns simplified to 2px noise |
| **Strategic** | Minimap heat channel `contamination_stress` scalar — **no** per-channel patterns |
| **Orbital** | Off or L3 climate tint only |

Align with [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) zoom table.

---

## Minimap compositor

| Rule | Spec |
|:---|:---|
| Channel | Single scalar `contamination_stress = max(airborne,soil,waterborne,bioactive,radiation)` normalized |
| Color ramp | Olive → umber → teal-dark (matches worst channel hue) |
| Polylines | **None** — heat only per construction/minimap rules |
| Fire overlay | Fire heat may sit above; contamination does not replace fire channel |

---

## Accessibility

| Rule | Spec |
|:---|:---|
| Pattern required | Every channel has pattern **or** glyph in inspector |
| Color-blind | radiation uses **dashed contour** even when hue visible |
| Motion | No flashing α > 0.5 Hz; pulse ≤ 0.15 Δα |

---

## Debug / witness (designer expectations)

| Witness / diag | Use |
|:---|:---|
| `wss_atmos_clipmap_001.contamination_types_wired` | All five vecs allocated on slab |
| `clipmap_l0_toxic_hazard_max` | Derived sample moves with plume |
| Overlay toggle | `contamination_debug_overlay` env — all channels @ 0.25 α for QA |

---

## Acceptance (designer)

1. Five channels use distinct pattern **and** glyph.
2. Tactical α ranges within table; no neon saturation.
3. Strategic/minimap uses scalar stress only.
4. F3 template matches `AC plume=…` line.
5. Toxic rain + washout sequences documented for coder stub ticks.

---

## Coder mapping

| Module | Lane |
|:---|:---|
| `src/substrate/atmosphere/contamination.rs` | A-W2 types |
| `src/systems/atmosphere/contamination_tick.rs` | tick + coupling |
| Strategic overlay stamp | read-only extract |

---

## Sign-off

| Role | Status | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-26 |
| `@coder` | **Unblocked** for A-W2 contamination UI | — |
