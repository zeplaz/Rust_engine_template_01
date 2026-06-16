# DESIGN-SMOKE-LAYER-AB-001 — Smoke Layer A (sim) vs Layer B (GPU) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-SMOKE-LAYER-AB-001** |
| **Coder lane** | **A-V3** · **WSS-SMOKE-BRIDGE-001** |
| **Related** | **A-W4** smoke stub removal · **A-W2** clipmap fold |
| **Planner** | [`wssr_plan_004_atmosphere_unification_v1.md`](wssr_plan_004_atmosphere_unification_v1.md) § Smoke Layer A/B |
| **Baseline** | [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) § Smoke/dust |
| **Prior art** | [`base_fire2_smoke.md`](../docs/archive/2026-06-prompts-guides/runbooks/guides/base_fire2_smoke.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | Stub → `ChunkSmokeField` → extract → render clipmap path |
| **No Rust** | Layer contract + α rules + debug labels |

---

## Purpose

End **`fire_visual_emit_smoke_stub`** ambiguity. Coders must know **what is saveable sim** vs **what is transient GPU** — and what the player sees at each zoom.

```text
Layer A (sim)  →  persistence, hazard, AI, fold into L0/L1 smoke_density
Layer B (GPU)  →  haze, columns, volumetric composite — DERIVED only
Layer 3        →  Hanabi wisps (future, non-authoritative)
```

---

## Layer A — Simulation (authoritative)

| Property | Spec |
|:---|:---|
| **Owners** | `ChunkSmokeField` (chunk ECS, transitional) → fold into `AtmosphereClipmapStack` L0 `smoke_density` |
| **Persistence** | Saveable; survives save/load |
| **Units** | Density ∈ [0, 1] per cell (normalized column mass) |
| **Writers** | Fire thermal fold, smolder decay, explosion impulse (event), toxic burn coupling → `ContaminationState.airborne` |
| **Readers** | Advect L0/L1, sensors, AI visibility sample, ecology stress |
| **Player direct view** | **Optional** debug heatmap only — not primary aesthetic |

### Fold path (target)

```text
ChunkSurfaceFire + ChunkSmokeField
  → fold_sources_system
  → AtmosphereClipmapStack.levels[L0].smoke_density
  → (optional) toxic burn → ContaminationState.airborne
```

### When Layer A alone is sufficient (no Layer B)

| Condition | Player sees |
|:---|:---|
| Strategic / orbital zoom | Field-only color shift on map envelope |
| Minimap | Fire/smoke **heat** channel only — no billboards |
| Diagnostics mode | Numeric smoke_density heatmap |

---

## Layer B — GPU representation (derived)

| Property | Spec |
|:---|:---|
| **Owners** | `AtmosphereRenderClipmap` → `gpu_weather_fire_field` → atmosphere/smoke render node |
| **Persistence** | **Transient** — rebuild every upload tick |
| **Writers** | Extract/upload systems only |
| **Readers** | Tactical camera composite, volumetric pass |
| **Forbidden** | GPU → sim readback without signed contract |

### Visual stack (tactical)

| Element | Treatment | α cap |
|:---|:---|:---:|
| Ground haze | Desaturated gray-brown, height-weighted | **0.45** |
| Column billboards | Soft vertical gradient, wind-shear | **0.55** per column |
| Heat shimmer | Linked to `heat` field, not separate smoke authority | 0.25 |
| Toxic burn tint | Warm olive overlay on Layer B | 0.20 |

### Partial alpha vs field-only

| Zoom | Layer B |
|:---|:---|
| **Tactical** (play) | Partial alpha haze + columns + fire sparks (D-F09) |
| **Operational** | Haze only; column billboards fade by distance |
| **Strategic** | **Field-only** — tint envelope, **no** billboards |
| **Minimap** | **Off** — use heat channel |

---

## Bridge contract (A-V3 exit)

Replace stub with **projection-graph smoke extract node**:

```text
ChunkSmokeField (A) ──fold──► L0 smoke_density
                                │
AtmosphereRenderClipmap ◄──upload──┘
        │
        ▼
smoke_visual_extract (B) ──► RepresentationResult / gpu_weather_fire_field
```

| Checkpoint | Pass |
|:---|:---|
| Stub removed | `fire_visual_emit_smoke_stub` not called on hot path |
| Tactical smoke visible | Player sees haze at fire without stub |
| Sim unchanged when B disabled | `RUST_ENGINE_SMOKE_GPU=0` → hazard still from A |
| No double density | Layer B does not write Layer A |

---

## Layer 3 — Hanabi (non-blocking)

| Allowed | Forbidden |
|:---|:---|
| ≤8 wisps per local event | World-scale smoke authority |
| Ember lift, collapse puff | Replacing Layer A density |
| Documentary material kick-up | Minimap/strategic draws |

See migration contract § Hanabi bounds.

---

## Debug overlay names (F3 / diagnostics)

Align with [`fire_streaming_debug_overlay_names_v1.md`](fire_streaming_debug_overlay_names_v1.md) style:

| Line | Template |
|:---|:---|
| Layer A | `SMK-A chunk=({cx},{cy}) dens_max={d:.2} fold_L0={l0:.2}` |
| Layer B | `SMK-B upload_tick={t} R0_smoke_max={r0:.2} stub={0|1}` |
| Bridge | `SMK-BRG extract_rows={n} gpu_field_wired={0|1}` |

**stub=1** during migration only — witness targets **stub=0** at A-V3 green.

---

## Coupling to contamination

| Event | Layer A | Layer B |
|:---|:---|:---|
| Normal fire | smoke_density ↑ | haze + column |
| Toxic fuel burn | smoke_density + **airborne** | warm tint overlay |
| Washout rain | smoke_density ↓ over N ticks | faster visual fade (filtered) |

---

## Failure modes (player-visible)

| Failure | Symptom | Designer verdict |
|:---|:---|:---|
| B without A | Pretty smoke, wrong hazard | **Fail** |
| A without B (tactical) | Hazard OK, flat map | **Accept** in diagnostics |
| Stub + B both on | Double brightness | **Fail** |
| 128² seam | Column pop at chunk edge | **Fail** — clipmap migration goal |

---

## Witness alignment

| JSON / flag | Criterion |
|:---|:---|
| `wss_atmos_clipmap_001.smoke_fold_wired` | L0 receives chunk fold |
| `wss_atmos_clipmap_001.smoke_stub_removed` | Stub path off |
| `fire_ecology_live.json` | No regression on fire rows |
| `stage5` tactical_vfx | Smoke visible in harness |

---

## Acceptance (designer)

1. Layer A/B table implemented in coder docs without merged authority.
2. Tactical α caps respected; strategic is field-only.
3. Bridge diagram path is the only approved stub replacement.
4. F3 lines use `SMK-A` / `SMK-B` / `SMK-BRG` prefixes.
5. Minimap never draws Layer B billboards.

---

## Coder mapping

| Lane | Deliverable |
|:---|:---|
| **A-V3** | `smoke_visual_extract.rs` + stub removal |
| **A-W2** | fold_sources → L0 |
| **A-W4** | complete bridge + witness |

---

## Sign-off

| Role | Status | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-26 |
| `@coder` | **Unblocked** for A-V3 / A-W4 smoke bridge | — |
