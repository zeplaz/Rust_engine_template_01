# PLAN-WSS-SMOKE-BRIDGE-EXEC-001 — Smoke Layer A → Layer B bridge `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WSS-SMOKE-BRIDGE-001** |
| **Slice ID** | **WSS-SMOKE-BRIDGE-001** |
| **Coder lane** | **A-V3** (primary) · fallback **A-W4** · blocked if **A-V2** F2 incomplete |
| **Prior** | [`wssr_plan_004_atmosphere_unification_v1.md`](wssr_plan_004_atmosphere_unification_v1.md) (reference only — **do not re-sign**) |
| **Related exec** | [`plan_wss_atmos_clipmap_exec_001_v1.md`](plan_wss_atmos_clipmap_exec_001_v1.md) W4-D |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **READY** — prefer after `fire_instance_buffer_rows > 0` |

**No Rust in this deliverable.**

---

## Summary

Replace **`fire_visual_emit_smoke_stub`** with a real **Layer B** extraction path: **`ChunkSmokeField` + atmosphere fold → `SimChunkSmokeVisualExtract` → projection graph smoke node → GPU field / compositor** — without making GPU ping-pong authoritative for gameplay. Hanabi wisps remain optional Layer 3 embellishment **after** bridge green.

---

## Current state

| Piece | Location | Status |
|:---|:---|:---|
| Sim smoke gen | `ChunkSmokeField`, `chunk_smoke_field_tick` | **L1 exists** |
| Atmosphere fold | `atmosphere_field_blend_fire_overlay_sources` | partial |
| Extract stub | `fire_visual_emit_smoke_stub` in `fire_visual_extract.rs` | **no-op** |
| GPU field | `gpu_weather_fire_field`, smoke channel in WGSL | visual diffusion |
| Designer | **DESIGN-SMOKE-AB-001** on designer parallel board | Layer A/B contract |

---

## Authority map

| Layer | Owner | Storage / path |
|:---|:---|:---|
| **L1** | `ChunkSmokeField` + `AtmosphereField` advect | ECS + grid resource |
| **L2** | `build_smoke_visual_extract` (new) | `SimChunkSmokeVisualExtract` or extend existing |
| **L2** | `RenderProjectionGraph` smoke node | reads committed stamp + atmosphere sample |
| **L3** | `AtmosphereGpuFieldBridge` | uploads render clipmap / partial writes |
| **L3** | Hanabi (future) | local wisps only |

```text
ChunkSmokeField  →  atmosphere fold  →  smoke visual extract  →  projection node
       →  AtmosphereRenderClipmap (when W4-C ready)  →  gpu_weather_fire_field
```

**Forbidden:** GPU field writeback to `ChunkSmokeField`; second smoke ECS scan in render world; deleting stub before extract node wired.

---

## PR plan (≤3 files per PR)

### SM-PR-1 — Smoke visual extract resource

| File | Change |
|:---|:---|
| `src/render/extraction/smoke_visual_extract.rs` | **new** — build `SmokeVisualExtract` from `ChunkSmokeField` + `AtmosphereField` sample |
| `src/render/extraction/mod.rs` | register module + plugin hook |
| `src/render/extraction/fire_visual_extract.rs` | schedule `build_smoke_visual_extract` before `ProjectGpu` |

**Exit:** lib test — non-zero `smoke_density_sum` in extract when field ticked

---

### SM-PR-2 — Projection graph node

| File | Change |
|:---|:---|
| `src/render/extraction/render_projection_graph.rs` | `SmokeProjectionNode` or extend fire node with smoke hints channel |
| `src/systems/atmosphere/gpu_field_bridge.rs` | consume smoke extract rows for upload params |
| `src/render/gpu_weather_fire_field.rs` | read smoke channel from bridge only |

**Exit:** `smoke_extract_wired: true` in witness

---

### SM-PR-3 — Remove stub + witness

| File | Change |
|:---|:---|
| `src/render/extraction/fire_visual_extract.rs` | remove `fire_visual_emit_smoke_stub`; wire `FireVisualFrameSet::EmitSmoke` to real path |
| `src/substrate/live_proof.rs` or `src/render/view_runtime/live_proof.rs` | extend `wss_substrate_live.json` or `fire_ecology_live.json` |
| `src/dev/visual_run_blockers.md` | note stub removed |

**Exit:** `smoke_stub_removed: true`

---

## Witness schema

**Files:** `debug_runs/wss_substrate_live.json` and/or `debug_runs/fire_ecology_live.json`

| Pointer | Type | Green |
|:---|:---|:---|
| `/smoke_stub_removed` | bool | `true` |
| `/smoke_extract_wired` | bool | `true` |
| `/smoke_density_sum` | number | `> 0` in tactical fixture |
| `/smoke_gpu_field_dispatch` | bool | `true` when atmosphere bridge runs |
| `/hybrid_ecs_smoke_authoritative` | bool | **`true` until WSS-SLAB-PR-5** — `ChunkSmokeField` still sim writer |

**Regression:** `tactical_vfx_witness.all_green` — do not regress spark rows

---

## Hybrid matrix (ECS remains)

| Component | Authority until | Notes |
|:---|:---|:---|
| `ChunkSmokeField` | **WSS-SLAB-PR-5** | sim tick writer |
| `AtmosphereField` | atmosphere advect set | fold source |
| Slab `thermal` / future smoke slice | PR-5+ | not in SM-PR-1 |

---

## Lib tests

```powershell
cargo test -p proc_A_dine01 --lib smoke_visual_extract
cargo test -p proc_A_dine01 --lib atmosphere
cargo test -p proc_A_dine01 --lib fire_visual_extract
```

| Test | Predicate |
|:---|:---|
| `smoke_extract_nonzero_after_fire_tick` | extract density > 0 |
| `smoke_stub_not_registered` | schedule has no stub system name |
| `gpu_bridge_reads_extract_not_ecs` | no `ChunkSmokeField` query in render extract |

---

## Dependencies

| Dependency | Required? |
|:---|:---:|
| `wss_chunk_slab_001.green` | optional for PR-1 |
| `wss_atmos_clipmap_001` L1 bridge | partial OK — may use legacy `AtmosphereField` alias |
| **FIRE-F2-EXTRACT-001** | **recommended** — avoid extract schedule fights in same PR as F2-PR-2 |

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| Hanabi as smoke authority | WSS-PLAN-004 |
| Re-open WSS-PLAN-004 sign-off text | planner policy |
| Delete `ChunkSmokeField` | L1 sim |
| Full 128² removal in same PR | atmos clipmap migration |

---

## Designer

**DESIGN-SMOKE-AB-001** — player read: column vs haze vs heat shimmer when stub removed.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | A-V3 / A-W4 exec plan |
