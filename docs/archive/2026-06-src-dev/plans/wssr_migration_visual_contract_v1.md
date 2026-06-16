# WSS migration visual contract `v1` (WSS-DESIGN-GATE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-DESIGN-GATE-001** |
| **Deliverable** | 3 of 4 — migration visual contract |
| **Parent brief** | [`wssr_design_gate_brief_v1.md`](wssr_design_gate_brief_v1.md) |
| **Water reference** | [`assets/vfx/reference/water/water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) (**SIGNED**) |
| **Theme** | [`design_theme.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/design_theme.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |

---

## Purpose

Bind WSS substrate migration to **stable player-visible language**. Coders may refactor L1 authority if L3 presentation honors this contract and closed VFX witnesses remain green.

---

## Sim clipmap L0–L3 vs render clipmap

### Simulation clipmaps (authority — player does not see grid directly)

| Level | Effective role | Sim fields (subset) | Tick cadence |
|:---|:---|:---|:---|
| **L0** | Local tactical cell | smoke_density, fog, heat, ash, wind, visibility, toxic_hazard **sample** | Every advect tick |
| **L1** | Regional column / storm | Same family, coarser | Every advect tick |
| **L2** | Continental front | humidity, pressure, ash envelope, drought band | Slow tick |
| **L3** | Planetary climate background | pressure trend, climate moisture, long ash | Very slow / saveable seed |

**Focus rule:** L0/L1 origins re-center on sim focus (camera / player staging) — like terrain paging.

### Render clipmaps (derived — what player sees)

| Level | Upload | Player-visible effect |
|:---|:---|:---|
| **R0** | High-res subset of L0 | Tactical ground haze, local fog wisps, heat shimmer |
| **R1** | Decimated L0+L1 | Column smoke, rain shaft composite input |
| **R2** | L2 envelope | Strategic haze band, front edge |
| **R3** | L3 tint | Orbital / far zoom color grading only |

**Separation contract:**

```text
AtmosphereClipmapStack     OWNS sim truth (L0-L3)
AtmosphereRenderClipmap    DERIVES R0-R3 (may omit pressure, temporal filter)
gpu_weather_fire_field     CONSUMES R0-R1 primarily
Minimap compositor         CONSUMES R2-R3 compressed channels only
```

### Zoom band → visible stack

| Zoom band | Sim active | Render upload | Particles (L3) |
|:---|:---|:---|:---|
| **Tactical** (default play) | L0+L1 full | R0+R1 | Fire sparks ON (D-F09); water W2 ON |
| **Operational** (mid) | L0+L1+L2 | R1+R2 | Sparks fade; water particles reduce |
| **Strategic** (far) | L2+L3 | R2+R3 | Sparks OFF; water band only (D-W09) |
| **Orbital / preview** | L3 optional | R3 tint only | All particles OFF |

---

## Contamination — color + pattern language

**Not color-only.** Align with post-industrial archive ethos — stains, ink, registration marks.

| Channel | Base color (tactical) | Pattern | Label glyph |
|:---|:---|:---|:---|
| **airborne** | Amber-olive `rgb(180,160,90)` @ α≤0.35 | Horizontal **crosshatch** density ∝ concentration | ☁ plume icon in tile info |
| **soil** | Burnt umber `rgb(120,80,50)` | **Stipple** corner-weighted | ⛏ spill |
| **waterborne** | Oxidized teal-dark `rgb(40,100,110)` | **Flow-aligned streaks** (parallel to river ribbon) | ≈ runoff |
| **bioactive** | Sickly yellow-green `rgb(140,160,80)` | **Dot grid** | ⚕ biohazard (small) |
| **radiation** | Magenta drafting ink `rgb(200,80,160)` — **pigment not neon** | **Dashed contour** + crosshatch | ☢ fallback text |

### Coupling visuals

| Event | Visual |
|:---|:---|
| Toxic rain | Airborne → waterborne darkening streaks on hydrology ribbon |
| Washout | Soil stipple fade over N ticks; waterborne dilution |
| Industrial fire | Smoke column + airborne increment; ash_density sync |

**Accessibility:** every hazard type requires **pattern OR icon** in addition to hue.

---

## Ocean / river — hydrology-driven look

**Authority:** `HydrologyState` slab → `HydrologyVisualExtract` → existing W1/W2 GPU path.

**Visual contract unchanged from** [`water_surface_vfx_design_plan_v1.md`](water_surface_vfx_design_plan_v1.md) + [`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png):

| Water class | Base | Motion | Particles |
|:---|:---|:---|:---|
| **Lake** | Oxidized teal plane | Slow omnidirectional ripple (~0.03 Hz) | Sparse cyan/white glints |
| **River** | Narrower strip (1–3 tiles visual) | Directional UV scroll along flow | Downstream micro-streaks; bend foam pinpoints |
| **Ocean** | Deep teal/navy + horizon haze | Swell normal + subtle scanline | Coast foam at shallow/deep boundary |

**WSS migration adds:**

- `ocean_mask` / `river_mask` / `flow_velocity` from slab drive extract — **not** separate ocean renderer
- Runtime flood raises `water_depth` → deeper tint + optional foam hints — **same** particle vocabulary
- Dry river bed: mask persists, depth ≈ 0 — riparian ecology strip visible, no shimmer

**Strategic zoom (D-W09):** color band remains; particles culled — witness zero rows is correct.

---

## Smoke / dust — Layer A vs Layer B

### Smoke

| Layer | Owner | Persistence | Visual |
|:---|:---|:---|:---|
| **A — Sim** | `AtmosphereClipmapStack` smoke_density + thermal fold | Saveable | Drives hazard, sensors, AI; optional debug heatmap |
| **B — GPU** | `AtmosphereRenderClipmap` → `gpu_weather_fire_field` | Transient | Ground haze α≤0.45 tactical; column billboards; volumetric composite |

**When partial alpha vs field-only:**

| Condition | Treatment |
|:---|:---|
| Tactical zoom, local fire | Layer B partial alpha haze + sparks |
| Strategic zoom | Layer B **field-only** color shift (no billboards) |
| Toxic burn | Layer A increments `ContaminationState.airborne`; Layer B warm tint |
| Minimap | **Neither** — heat channel only |

### Dust

| Source | Sim write | Visual |
|:---|:---|:---|
| Erosion / desert wind | L0 `ash_density` advect | Ground haze (warm desaturate) |
| Vehicle / convoy kick | Event impulse → ash_density | Optional Hanabi puff (Layer 3, later) |
| Battlefield disturbance | ash_density + soil deposit | Strategic overlay envelope |
| Collapse / explosion | Short ash spike + soil | Single-frame puff OK (Hanabi candidate) |

**Not a silo:** no `DustSystem` module — transport via atmosphere field + contamination deposit.

---

## Hanabi (future) — event VFX style bounds

**Scope:** Layer 3 embellishment only — after `experiments/hanabi_validation/` PASS.

### Allowed

| Use | Style |
|:---|:---|
| Ember lift off fire front | Pinpoint orange-white, short life, low count |
| Local smoke wisp | Soft gray, **≤8** particles per event, no billboard stack |
| Explosion debris / collapse puff | Industrial dust — brown-gray, not cartoon |
| Spark accent (optional) | Only where gpu_particles already saturated — never duplicate tactical spark field |

### Forbidden

| Use | Why |
|:---|:---|
| World-scale rain/snow | Weather is field + composite |
| Smoke authority | Layer A sim owns density |
| Arcade muzzle flash stacks | Breaks industrial sim identity |
| Minimap / strategic zoom draws | Readability contract |
| Gameplay-affecting collision | L3 must not write L1 |

**Aesthetic bound:** particles feel like **material kick-up** in a documentary archive — not fantasy spell VFX. Prefer fewer, smaller, lower α than reference game trailers.

---

## Migration phases → visual checkpoints

| Phase | Visual checkpoint |
|:---|:---|
| W2-A slab types | No player-visible change |
| W2-B dual-write weather | No drift visible; witness metric only |
| W3-A hydrology hydrate | Rivers/oceans **look** unchanged; data backed by slab |
| W3-D hydro extract | Coast foam still matches target PNG |
| W4-B clipmap sim | Smoke columns track fire; no 128² seam artifacts |
| W4-C render clipmap | GPU field upload partial; tactical haze smooth |
| W4-D smoke stub removed | Tactical smoke visible without stub |
| H-A Hanabi spike | No main-app visual until merge approved |

---

## Regression visuals (must not regress)

| Witness | Visual row |
|:---|:---|
| `stage5_full_app_live.json` → `tactical_vfx_witness` | Sparks, smoke, water tactical |
| `fire_streaming_live.json` | Per-view fire isolation |
| `water_w1_green` / strategic water | D-W09 band |
| `construction_stage_live.json` | Ghost footprint tokens (R4) |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **SIGNED** | 2026-05-26 |
