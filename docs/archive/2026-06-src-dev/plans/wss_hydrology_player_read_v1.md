# WSS hydrology — player readability by band `v1` (DESIGN-HYDRO-PLAYER-READ-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-HYDRO-PLAYER-READ-001** |
| **Version** | `1.1.0` |
| **Date** | 2026-05-28 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Feature pass** | [`wss_hydro_read_feature_pass_001.md`](wss_hydro_read_feature_pass_001.md) (**DESIGN-WSS-HYDRO-READ-001**, 2026-05-28) |
| **Prereq** | **WSS-DESIGN-GATE-001** PASS — [`wssr_design_signoff_v1.md`](wssr_design_signoff_v1.md) |
| **Plan** | [`wssr_plan_003_hydrology_runtime_v1.md`](wssr_plan_003_hydrology_runtime_v1.md) (**WSS-PLAN-003**) |
| **Parent readability** | [`wssr_readability_impact_v1.md`](wssr_readability_impact_v1.md) § Hydrology / Minimap |
| **Water VFX baseline** | [`water_surface_vfx_design_plan_v1.md`](water_surface_vfx_design_plan_v1.md) · [`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) |
| **Coder lane** | **WSS-HYDRO-RUNTIME-001** (B-H1) — slab authority + extract; **no** new ocean renderer |
| **No Rust** | Design table + channel policy only |

---

## Purpose

Tell players **what water should look like** at each zoom / surface so WSS hydrology slab migration does not regress closed **FX-WATER W1/W2** reads. Authority moves to `HydrologyState` in substrate; **presentation** stays on existing shader/particle vocabulary until extract rewires.

**Three-layer rule:**

```text
HydrologyState (L1 slab)     OWNS depth, masks, flow
HydrologyVisualExtract (L2)  DERIVES ribbons, foam hints, strategic band
gpu_water_* (L3)           CONSUMES extract — never owns depth
```

---

## Primary table — player read by band

| Product band | Typical views | **Player should see** | L1 source (WSS) | L3 presentation | Minimap |
|:---|:---|:---|:---|:---|:---|
| **Strategic** | Far map, minimap | **Color band only** — where wet vs dry; no river particles | `ocean_mask` + shallow/deep height band; compressed ribbon | D-W09: particles culled; teal/navy band persists | **Dim ribbon** @ ≤40% α — no W2 particles |
| **Operational** | Mid zoom | **River channels** as narrow darker strips; lakes as calm sheets | `river_mask` + `flow_velocity` + `water_depth` | Directional UV scroll (W1); sparse glints | Ribbon + band only |
| **Tactical** | `WorldMain` / `SimulationMap` | **Flow direction** + coast foam + lake glints — three water reads distinct | Full slab sample at cell; flood = depth pulse | W1 shaders + W2 particles (tactical only) | N/A (minimap separate) |
| **Cinematic / debug** | Focused camera | Full motion + foam at bends — highest fidelity | Same slab; optional debug depth overlay | Full particle budget | N/A |

**Designer rule:** If rivers read as lakes (same motion), fix **directional streak** before adding particles. If ocean reads as flat teal paint, fix **depth band + coast foam** before new shaders.

---

## Channel spec — ocean mask

| Element | Tactical | Strategic / minimap |
|:---|:---|:---|
| **Deep ocean** | Navy-teal base ([`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png)) | Single **desaturated** band — no swell shader detail |
| **Coast** | Shallow/deep boundary + **coast foam** pinpoints (W2) | 1px lighter edge on ribbon — no foam particles |
| **Authority** | `HydrologyState.ocean_mask` + `water_depth` gradient | L2 `HydrologyVisualExtract.coast_lines` count → witness `ocean_tile_count` slab-backed |
| **Forbidden** | Separate `OceanSystem` module or VFX-only tile counter | Particle shimmer on minimap |

**Witness policy:** `water_ocean_tiles` must trace to slab mask sum after B-H1 — not GPU-only count.

---

## Channel spec — river ribbon

| Element | Spec |
|:---|:---|
| **Width read** | **1–3 tiles** visual strip (may be 1 tile sim) — narrower than lakes |
| **Motion** | Directional UV scroll along `flow_velocity` / D8 downstream |
| **Particles** | Downstream micro-streaks (W2); centerline denser |
| **Bends** | Foam pinpoints at curvature |
| **Dry bed** | `water_depth ≈ 0` but `river_mask` persists — muted bed, no shimmer |
| **Flood** | Depth tint **pulse** + optional foam; hue **not** contamination amber |

**Strategic zoom:** ribbon collapses to **dark thread** on desaturated teal band (D-W09 A).

---

## Channel spec — lake / standing water

| Element | Spec |
|:---|:---|
| **Base** | Oxidized teal plane (legacy ref ship palette) |
| **Motion** | Slow omnidirectional ripple (~0.03 Hz) |
| **Particles** | Sparse surface glints — **no** directional bias |
| **vs river** | Wider, calmer, no centerline streaks |

---

## Minimap compositor policy (locked)

```text
Stack (bottom → top):
  terrain base (muted)
  hydrology ribbon (dim, max 40% alpha)
  fire heat (R channel only)
  routing congestion (M3)
  unit markers (M3)
  fog of war / EW (M4)
```

| Allowed | Forbidden |
|:---|:---|
| Desaturated **StrategicWaterRibbon** from hydrology extract | W2 particles |
| Single-pixel coast hint on ribbon edge | Live flood animation |
| Wet/dry tint from `ocean_mask` aggregate | Full L0 depth field upload |

**Alpha cap:** `hydrology_minimap_alpha_max = 0.40`

---

## Per-view exceptions

| View | Treatment |
|:---|:---|
| **World Preview** | Static gen-time coast/river tint only — no runtime flood animation ([`wssr_readability_impact_v1.md`](wssr_readability_impact_v1.md)) |
| **Construction ghosts** | Hydrology tint **below** ghost footprint; flood deep-solve after commit only |
| **Contamination waterborne** | Teal-darkening streak **parallel** to river ribbon — distinct from flood pulse hue |

---

## Regression guards (do not break)

| Closed track | Witness / policy |
|:---|:---|
| FX-WATER W1/W2 | `water_w1_green`, `water_w2_foam_001_green`, `water_witness_rollup_green` |
| D-W09 strategic cull | Zero water particle rows at strategic zoom = **PASS** |
| D-W01–D-W04 signed decisions | River overlay + directional shader first |
| [`water_surface_target_v1.png`](../assets/vfx/reference/water/water_surface_target_v1.png) | Three-class read: lake / river / ocean |

---

## Acceptance cues (playtest)

| Band | Pass | Fail |
|:---|:---|:---|
| Strategic | Player names **wet regions**; minimap not noisy | Sparkles or foam on minimap |
| Operational | Rivers traceable as **channels** | Rivers indistinguishable from lakes |
| Tactical | Flow direction obvious; coast foam visible | Flat teal with no motion |
| Flood event | Water spreads read as **water** not toxin | Same color as contamination plume |

---

## Coder handoff (B-H1 / WSS-HYDRO-RUNTIME-001)

```
Read:  docs/archive/2026-06-src-dev/plans/wss_hydrology_player_read_v1.md
       docs/archive/2026-06-src-dev/plans/wssr_plan_003_hydrology_runtime_v1.md
       docs/archive/2026-06-src-dev/plans/water_surface_vfx_design_plan_v1.md
Touch: world_substrate hydrology hydrate, HydrologyVisualExtract, gpu_water_* bridge
Do:    ocean_mask + river_mask in slab; extract feeds existing W1/W2; minimap ribbon only
Do NOT: OceanSystem; FluidDomain; minimap particle draw; disable D-W09 for witness green
Verify: cargo test -p proc_A_dine01 --lib hydrology wss_substrate
Witness: wss_substrate_live.json hydrology_hydrated; stage5 water_* rows unchanged
```

| Policy constant (suggested) | Value |
|:---|:---|
| `hydrology_minimap_alpha_max` | `0.40` |
| `hydrology_strategic_particle_cull` | `true` (D-W09 parity) |
| `river_visual_width_tiles` | `1..3` |

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **SIGNED** |
| Coder | — | **Unblocks B-H1** |

**Unblocks:** **WSS-HYDRO-RUNTIME-001** hybrid assessment + slab hydrate slice.

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-HYDRO-PLAYER-READ-001** initial SIGNED |
| v1.1.0 | 2026-05-28 | **DESIGN-WSS-HYDRO-READ-001** feature HUD strings — [`wss_hydro_read_feature_pass_001.md`](wss_hydro_read_feature_pass_001.md) |
