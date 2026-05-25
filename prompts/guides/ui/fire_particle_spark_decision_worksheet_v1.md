# Fire pinpoint sparks — decision worksheet `v1`

**SIGNED 2026-05-24** · All **recommended** defaults accepted (legacy-style point sparks).

| Authority | [`fire_particle_spark_design_plan_v1.md`](../../../src/dev/fire_particle_spark_design_plan_v1.md) |
| Brief | [`fire_particle_spark_designer_brief_v1.md`](fire_particle_spark_designer_brief_v1.md) |
| **Legacy ref** | `C:\dev\razerz-master\shaderzglsl\elemental\compute_partical\` |
| **Stills** | [`assets/vfx/reference/elemental_sparks/`](../../../assets/vfx/reference/elemental_sparks/) |

**Designer:** Design pass (recommended defaults) · **Date:** 2026-05-24

---

## §5 — D-F01…D-F10 (final)

| ID | Question | **Choice** | Summary |
|:---|:---|:---:|:---|
| **D-F01** | Primary raster | **A** | Point sprites (WGSL: point list or ≤2px sharp quad equivalent) |
| **D-F02** | Motion | **A→B** | Phase A: twinkle only · Phase B: compute advection port |
| **D-F03** | Attractors | **A** | `FireVisualGpuInstance` centers — no new sim |
| **D-F04** | Lifetime visual | **A** | Ash → orange `age_intensity` mix (legacy frag) |
| **D-F05** | Twinkle | **A** | `sin(pos.x)` / `cos(pos.y)` personality |
| **D-F06** | Colors | **B** | Map legacy hex to `design_theme` / palette tokens |
| **D-F07** | Density | **A** | Many tiny points, low α — not fewer large blobs |
| **D-F08** | Blend | **A** | Additive hot cores + alpha embers |
| **D-F09** | Zoom | **A** | Fade sparks when zoomed out (strategic zoom) |
| **D-F10** | Smoke | **A** | Sparks render above smoke field |

**Overrides:** none.

---

## Transfer table (§11 authority doc)

| ID | Final |
|:---|:---|
| D-F01 | **A** |
| D-F02 | **A→B** |
| D-F03 | **A** |
| D-F04 | **A** |
| D-F05 | **A** |
| D-F06 | **B** |
| D-F07 | **A** |
| D-F08 | **A** |
| D-F09 | **A** |
| D-F10 | **A** |

---

## §11 checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | D-F01…D-F10 recorded | ☑ |
| 2 | Legacy ref stills in `assets/vfx/reference/elemental_sparks/` | ☑ |
| 3 | `fire_spark_target_v1.png` (blob vs pinpoint) | ☑ |
| 4 | Color key §6 (ash `#1D1D1E` → hot `#E67345` → palette tokens) | ☑ |
| 5 | Zoom/LOD: D-F09 A — fade sparks strategic zoom | ☑ |
| 6 | Blend D-F08 A confirmed | ☑ |
| 7 | Phase A look first; Phase B compute optional | ☑ |

**Verdict:** ☑ **SIGNED**

---

## Unblocks

| Slice | Agent | First task |
|:---|:---|:---|
| **FX-FIRE-SPARK-001** | `@coder` | `fire_particle_draw.wgsl` sharp falloff / point read (D-F01 A) |

**Spine rule:** single fire extract — edit shaders + sizing only.

---

## Recommended rationale

| ID | Why |
|:---|:---|
| D-F01 A | Legacy `gl_PointCoord` intent — pinpoint not blob |
| D-F02 A→B | Ship look without blocking on compute port |
| D-F04 A | Direct `mix(ash, orange, age_intensity)` from frag |
| D-F05 A | Position sin/cos twinkle from legacy frag |
| D-F07 A | Spark shower = many low-α points |
| D-F09 A | No particle soup at strategic zoom |
