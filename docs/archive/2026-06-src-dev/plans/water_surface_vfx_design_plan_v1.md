# Water surface VFX — design plan (master)

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **FX-WATER-SHADER-001** |
| **Status** | **SIGNED** (2026-05-24) — **FX-WATER-SHADER-001 unblocked** |
| **Worksheet** | [`water_surface_vfx_decision_worksheet_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/water_surface_vfx_decision_worksheet_v1.md) |
| **VFX architecture** | [`vfx_architecture_bevy_wgpu_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/vfx_architecture_bevy_wgpu_v1.md) |
| **Hydrology sim** | [`hydrology_v1.md`](../docs/reference/designer_questions/terrain_world/hydrology_v1.md) |
| **Visual refs** | [`assets/vfx/reference/water/water_surface_target_v1.png`](../../assets/vfx/reference/water/water_surface_target_v1.png) · [`assets/vfx/reference/hydrology_ships/`](../../assets/vfx/reference/hydrology_ships/) · [`assets/textures/tiles/water_v01_#800x400.png`](../../assets/textures/tiles/water_v01_#800x400.png) |

---

## Executive summary

Lakes, **rivers**, and oceans need **distinct motion + particle layers** so water reads alive under the archival UI — not flat teal tiles. **Rivers look “missing”** because gen already tags `RiverMarker` paths but visuals are identical to lakes (`ShallowWater` tint only). Fix = **river flow overlay + directional animation**, not new hydrology sim.

**SIGNED** — coder implements **W1 shaders first** (`FX-WATER-SHADER-001`), then **W2 particles** (`FX-WATER-PARTICLE-001`). **No second terrain extract.**

---

## Why rivers vanish today

| Layer | Lakes | Rivers | Oceans |
|:---|:---|:---|:---|
| **Gen data** | `LakeMarker` + basin cells | `RiverMarker` + path tiles | `deep_water_height_max` band |
| **Tile visual** | `apply_shallow_water_visual` → `ShallowWater` | **Same function** | Shallow/deep family by height |
| **Motion** | None | **None** | None |
| **Particles** | None | **None** | None |

Rivers exist in sim data; they **do not read as channels** because there is no narrower mask, flow direction, or streak animation.

---

## Target — three water reads + shared particle language

Use **pinpoint spark** vocabulary from [`fire_particle_spark_design_plan_v1.md`](fire_particle_spark_design_plan_v1.md) (many small, low α) — applied to water as **glints / streaks / foam**, not orange fire blobs.

**Mock:** [`water_surface_target_v1.png`](../../assets/vfx/reference/water/water_surface_target_v1.png)

### Lake (standing water)

| Channel | Spec |
|:---|:---|
| **Base** | Oxidized teal plane ([`ref_ship_on_flat_water_teal_20191213.png`](../../assets/vfx/reference/hydrology_ships/ref_ship_on_flat_water_teal_20191213.png)) |
| **Shader motion** | Slow omnidirectional ripple normal scroll (~0.03 Hz); subtle scanline optional |
| **Particles** | Sparse **surface glints** — white/cyan points, random twinkle, **no** directional bias |
| **Density** | Low — lakes are calm archive sheets |

### River (flowing — **priority fix**)

| Channel | Spec |
|:---|:---|
| **Base** | **Narrower** water strip on hydrology path (1–3 tiles wide visual, even if sim is 1 tile) |
| **Shader motion** | **Directional** UV scroll along D8 flow / path tangent; darker center, lighter edge |
| **Particles** | **Downstream streaks** — elongated micro-sparks; higher density in centerline |
| **Extra** | Bend **foam** pinpoints at curvature; optional industrial “flow ink” thread (dirty amber, very faint) |
| **Data source** | `HydrologyResult.rivers` paths + flow direction from accumulation gradient |

### Ocean / deep water

| Channel | Spec |
|:---|:---|
| **Base** | Deeper teal/navy; horizon **haze** (not flat plane) |
| **Shader motion** | Slow omnidirectional ripple + long **swell** normal; optional analog scanline sun reflection (subtle) |
| **Particles** | **Coast foam** line at shallow/deep boundary; distant spray at storm/weather hook |
| **Zoom** | Strategic zoom → reduce particles, keep color band (D-W09 A) |

---

## §5 — Signed decisions (2026-05-24)

| ID | Choice | Summary |
|:---|:---:|:---|
| D-W01 | **A** | River polyline overlay + directional shader (W1) |
| D-W02 | **A** | Lake slow omnidirectional ripple |
| D-W03 | **A** | River UV scroll along flow dir (shader first) |
| D-W04 | **A** | Ocean swell + horizon haze |
| D-W05 | **A** | Pinpoint / ≤2px (fire spark family) |
| D-W06 | **B** | Lake glints optional in W2 |
| D-W07 | **A** | River downstream streaks + bend foam |
| D-W08 | **B** | Ocean coast foam only |
| D-W09 | **A** | Fade all water particles when zoomed out |
| D-W10 | **A** | Custom WGSL (fire spine) — no Hanabi world fields |

**Overrides:** none.

---

## Architecture (Stage 5 safe)

```text
HydrologyResult / tile ShallowWater / DeepWater  (existing — authority)
        ↓
WaterSurfaceVisualCatalog  (NEW presentation resource)
  · lake_basins[], river_polylines[], ocean_mask
  · per-tile: kind = Lake | River | Ocean | None
  · river: flow_dir, path_id
        ↓
┌───────────────────┬────────────────────────┬─────────────────────┐
│ Tile water shader │ River ribbon / overlay   │ GPU water particles │
│ (scroll normal)   │ (flow UV pass)           │ (lake/river/ocean)  │
└───────────────────┴────────────────────────┴─────────────────────┘
        ↓
Map camera + minimap compositor (read-only tints)
```

| Layer | Owner | Notes |
|:---|:---|:---|
| Topology | `terrain/generation/hydrology/` | Do not duplicate |
| Water **kind** tag | Build at gen or from chunk matrix | Presentation only |
| Shader motion | `assets/shaders/water/` (new) | Phase W1 |
| Particles | Same spine as fire: compute expand + draw WGSL | Phase W2 |
| Weather coupling | Sample `ChunkWeather` for storm spray later | Optional W3 |

**Do not** use Hanabi for world-scale river fields ([`vfx_architecture_bevy_wgpu_v1.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/vfx_architecture_bevy_wgpu_v1.md)).

---

## Particle profiles (motion + look)

| Profile | Primitive | Motion | Color tokens |
|:---|:---|:---|:---|
| **LakeGlint** | Point / ≤2px | Random twinkle; drift ≤0.2 px/s | `water_glint_cyan`, white @ 20–40% α |
| **RiverStreak** | Short ribbon or stretched point | Advect along `flow_dir`; speed ∝ slope | `water_teal` + `dirty_amber` edge |
| **RiverFoam** | Cluster pinpoint | Spawn at path curvature peaks | `foam_archival` @ 50% α |
| **OceanFoam** | Line emitter at coast | Slow alongshore drift | `ecology_stain` + white foam |
| **OceanSpray** | Sparse upward streak | Wind from `ChunkWeather` | only when weather active (W3) |

Reuse **twinkle math** from legacy elemental frag (signed fire D-F05) at lower saturation.

---

## §6 Color key (signed)

Map legacy teal + archival foam to palette / shader constants (D-W05 A, fire D-F06 B pattern).

| Role | Hex / sample | Token / shader name |
|:---|:---|:---|
| Lake base | `#2a5a58` (oxidized teal from 2019 ref) | `water_teal` |
| Lake glint | `#5ee0dc` @ 30% α | `label_primary` / `water_glint_cyan` |
| River center | `#1e4544` | `water_river_deep` |
| River edge / streak | `#4a7878` | `label_muted` / `water_teal_edge` |
| Flow ink (optional) | `#e8c03a` @ 12% α | `gold_bar` / `dirty_amber` faint |
| Ocean deep | `#0f2828` | `water_ocean_deep` |
| Ocean haze | `#060808` @ 40% α | `panel_elevated` blend |
| Coast foam | `#c8b898` @ 50% α | `foam_archival` |
| Storm spray | `#5ee0dc` @ 25% α | `water_glint_cyan` (W3 only) |

**Tile baseline:** existing `water_v01` / `water_v02` textures remain; shader scroll **tints** on top — do not replace tile atlas in W1.

---

## §7 Density caps per zoom (signed — D-W09 A)

Shader motion **always on** at all zoom levels. Particles scale by zoom band (match fire `zoom_alpha` hook).

| Zoom band | Lake glints | River streaks + foam | Ocean foam | Shader motion |
|:---|:---:|:---:|:---:|:---:|
| **Tactical** (close) | 100% | 100% | 100% | 100% |
| **Operational** (mid) | 40% | 60% | 50% | 100% |
| **Strategic** (far) | **0%** | **0%** | **0%** | 100% |

**Per-chunk caps (W2):** max 8 lake glints · 24 river streaks · 12 foam pinpoints · 16 ocean foam per visible chunk at tactical zoom. Halve at operational; zero at strategic.

---

## Phases

| Phase | ID | Deliverable | Status |
|:---|:---|:---|:---:|
| **W0** | FX-WATER-DESIGN | Worksheet + target mock + **SIGNED** | ☑ |
| **W1** | FX-WATER-SHADER-001 | Tile water shader + river overlay + lake/ocean motion | **done** |
| **W1** | FX-WATER-SHADER-002 | Witness `water_w1_green` | **done** |
| **W2** | FX-WATER-PARTICLE-001 | WGSL spine | **done** |
| **W2** | FX-WATER-PARTICLE-002 | Emission profiles | **done** |
| **P2** | [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) | Proof + polish + tactical witness | **active** |
| **W3** | FX-WATER-WEATHER | Storm spray, wind ripples | deferred |
| **W4** | FX-WATER-RUNTIME | Event-driven flood/breach visuals (hydrology_v1) | deferred |

**MVP for “rivers not missing”:** **W1 river overlay pass** — ships before particles.

---

## §11 Designer sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | §5 **D-W01…D-W10** on worksheet | ☑ |
| 2 | Reference stills: lake / river / ocean target mocks | ☑ |
| 3 | River vs lake side-by-side mock (`water_surface_target_v1.png`) | ☑ |
| 4 | Particle density caps per zoom (§7) | ☑ |
| 5 | Palette tokens for water VFX (§6) | ☑ |
| 6 | Phase W1 vs W2 scope agreed | ☑ |

**Verdict:** ☑ **SIGNED**

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** | Recommended defaults; target mock + hydrology refs |
| Coder | 2026-05-24 | Acknowledged | **FX-WATER-SHADER-001** first — no duplicate extract |

---

## Coder handoff — **dual @coder active**

**Queue:** [`water_surface_vfx_coder_queue_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/water_surface_vfx_coder_queue_v1.md)

| Coder | Primary | Signed decisions |
|:---|:---|:---|
| **A** | FX-WATER-SHADER-001 → FX-WATER-PARTICLE-001 | D-W02–D-W04 A shaders; D-W05/D-W07/D-W10 W2 WGSL |
| **B** | FX-WATER-SHADER-002 → FX-WATER-PARTICLE-002 | D-W06 B optional; D-W07–D-W09 emission; D-W08 B coast only |

```
Lane: FX-WATER-SHADER-001 (A) or FX-WATER-SHADER-002 (B)
Read: water_surface_vfx_coder_queue_v1.md
Do NOT: second terrain extract; Hanabi; W2 until W1 witness green
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual stage5
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-24 | **SIGNED**; §5–§7; FX-WATER-SHADER-001 unblocked |
| v1.1.0 | 2026-05-24 | **SIGNED**; FX-WATER-SHADER-001 unblocked |
| v1.0.0 | 2026-05-24 | Initial plan — lake/river/ocean VFX split; river visibility gap |
