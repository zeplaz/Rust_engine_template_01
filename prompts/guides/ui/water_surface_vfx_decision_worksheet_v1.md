# Water surface VFX — decision worksheet `v1`

**SIGNED 2026-05-24** · All **recommended** defaults accepted (legacy teal + pinpoint particles + river directional priority).

| Authority | [`water_surface_vfx_design_plan_v1.md`](../../../src/dev/water_surface_vfx_design_plan_v1.md) |
| **Target mock** | [`assets/vfx/reference/water/water_surface_target_v1.png`](../../../assets/vfx/reference/water/water_surface_target_v1.png) |
| **Legacy refs** | [`assets/vfx/reference/hydrology_ships/`](../../../assets/vfx/reference/hydrology_ships/) |

**Designer:** Design pass (recommended defaults) · **Date:** 2026-05-24

---

## §5 — D-W01…D-W10 (final)

| ID | Question | **Choice** | Summary |
|:---|:---|:---:|:---|
| **D-W01** | River visibility (W1) | **A** | Polyline overlay + directional shader |
| **D-W02** | Lake motion | **A** | Slow omnidirectional ripple |
| **D-W03** | River motion | **A** | UV scroll along flow dir (shader first) |
| **D-W04** | Ocean motion | **A** | Slow swell + horizon haze |
| **D-W05** | Particle primitive | **A** | Pinpoint / ≤2px (fire spark family) |
| **D-W06** | Lake particles | **B** | Sparse glints optional in W2 |
| **D-W07** | River particles | **A** | Downstream streaks + bend foam |
| **D-W08** | Ocean particles | **B** | Coast foam only |
| **D-W09** | Zoom fade | **A** | Fade all water particles when zoomed out |
| **D-W10** | Tech path | **A** | Custom WGSL (fire spine) |

**Overrides:** none.

---

## Transfer table (§11 authority doc)

| ID | Final |
|:---|:---|
| D-W01 | **A** |
| D-W02 | **A** |
| D-W03 | **A** |
| D-W04 | **A** |
| D-W05 | **A** |
| D-W06 | **B** |
| D-W07 | **A** |
| D-W08 | **B** |
| D-W09 | **A** |
| D-W10 | **A** |

---

## §11 checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | D-W01…D-W10 recorded | ☑ |
| 2 | `water_surface_target_v1.png` (lake vs river vs ocean) | ☑ |
| 3 | Palette tokens §6 (`water_teal`, `water_glint_cyan`, `foam_archival`) | ☑ |
| 4 | Density caps §7 (tactical 100% → strategic 0% particles) | ☑ |
| 5 | W1 shader scope signed (river overlay + lake/ocean motion) | ☑ |
| 6 | W2 particle scope signed (LakeGlint / RiverStreak / OceanFoam) | ☑ |

**Verdict:** ☑ **SIGNED**

---

## Unblocks

| Slice | Agent | First task |
|:---|:---|:---|
| **FX-WATER-SHADER-001** | `@coder` A | GPU hook + D-W02/D-W03/D-W04 verify — [`water_surface_vfx_coder_queue_v1.md`](water_surface_vfx_coder_queue_v1.md) § W1-A |
| **FX-WATER-SHADER-002** | `@coder` B | `water_w1_green` witness — § W1-B |
| **FX-WATER-PARTICLE-001** | `@coder` A | `water_particle*.wgsl` (D-W05, D-W10) after W1 green |
| **FX-WATER-PARTICLE-002** | `@coder` B | Emission profiles D-W06–D-W09 after W1 green |

**Spine rule:** hydrology owns topology — presentation only; no second terrain extract.

---

## Rationale (quick)

| ID | Why |
|:---|:---|
| D-W01 A | Rivers invisible because they share lake tiles — overlay fixes read |
| D-W03 A | Directional scroll sells flow without compute sim |
| D-W05 A | Consistent with signed fire pinpoint spark language |
| D-W09 A | Prevent particle soup at strategic zoom |
