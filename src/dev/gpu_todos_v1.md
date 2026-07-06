# GPU todos `v1` — authoritative pick board

**Date:** 2026-07-03 · **Branch:** `master` · **Owner:** `@coder` (render + gui + terrain)

**Programs:** [`plan_gpu_terrain_production_exec_001_v1.md`](plan_gpu_terrain_production_exec_001_v1.md) · [`plan_gpu_particle_backend_split_v1.md`](plan_gpu_particle_backend_split_v1.md) · parent [`plan_visual_perf_production_exec_001_v1.md`](plan_visual_perf_production_exec_001_v1.md)

**Rule:** Plan §13 tracking rows marked DONE = **scaffolding shipped** — **§10 program exit gate is NOT closed** until witnesses match (see audit below).

```powershell
cargo test -p proc_A_dine01 --lib terrain_render terrain_instanced terrain_material tile_world_fallback gpu_particle -q
cargo test -p proc_A_dine01 --lib stage5 -q
# After display slice:
cargo run -p proc_A_dine01 --release -- --test demo --stay-open
```

Read first: [`.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md`](../.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md) · **validation-first**

---

## Honest audit (2026-07-03)

| Claim | Reality |
|:---|:---|
| Plan §13 **PERF-GPU-TERRAIN-001..004 DONE** | **Partial** — P0-C′-PRIME shipped; §10 operator gate still open |
| `sim_spectrum_analytics_live.json` | Debug builds still `CpuFallback`; **release** defaults `GpuInstancedAtlas` |
| P0-C tilemap | **BLOCKED** — `DR-MIG-TILEMAP` / `DR-GPU-TERRAIN-P0C` |
| P0-C′ instanced + atlas | **Default in release** — debug keeps CPU unless `TERRAIN_GPU_INSTANCED=1` |
| P1 zero-dispatch particles | **Wired** — particle + spark + fire raster skip tests |
| §10 exit gate (p95 ≤33ms, tile_raster_ms==0, …) | **Open** — needs operator `--test demo/visual` + witness refresh |

**Yes — we can do GPU-P0C-PRIME:** use **P0-C′** (instanced atlas + dirty-gated GPU sprite display) as release Simulation default until tilemap unblocks.

---

## Witness truth (target after program close)

| Field / witness | Target |
|:---|:---|
| `sim_spectrum_analytics_live.json` → `last_frame.spine.terrain_authority` | `GpuInstancedAtlas` (or `GpuTilemap` post-unblock) |
| `last_frame.spine.tile_raster_ms` | **== 0** steady sim (CPU per-frame paint off) |
| `render_schedule.render_and_present_ms` p95 | **≤ 16 ms** |
| `stage5_full_app_live.json` | `perf.terrain_gpu_authoritative: true` |
| `minimap_compositor_live.json` | `terrain_source: gpu_atlas` |

---

## Pick order — Phase P0 (BLOCKER)

### P0-C′-PRIME — Simulation default GPU terrain (@coder)

| ☑ | ID | Scope | Exit |
|:---:|:---|:---|:---|
| ☑ | **GPU-P0C-PRIME-001** | `terrain_render_authority.rs` — `resolve_sim_default_authority()` → `GpuInstancedAtlas` in **release** sim (keep `TERRAIN_CPU_FALLBACK=1` rollback) | `sim_default_gpu_in_release_cpu_in_debug` + witness |
| ☑ | **GPU-P0C-PRIME-002** | `tile_world_fallback.rs` — dirty-gated bake under `uses_gpu_sprite_display()`; CPU raster metric gated | `gpu_authority_skips_cpu_fallback_raster_metric` |
| ☑ | **GPU-P0C-PRIME-003** | `terrain_instanced_draw.rs` — sprite-bake interim documented (plan Q3) | module header + witness `instanced_pass: wired_deferred` |
| ☑ | **GPU-P0C-PRIME-004** | Lib witness `debug_runs/gpu_terrain_p0c_prime_001_live.json` | `green: true` ✓ |

### P0-A/B — authority + atlas (mostly done — verify)

| ☑ | ID | Notes |
|:---:|:---|:---|
| ☑ | **P0-A1** | `terrain_render_authority.rs` + plugin wired |
| ☑ | **P0-A2** | `OnEnter(Simulation)` hook — **needs PRIME-001 flip** |
| ☑ | **P0-A3** | `tile_world_fallback` gates on authority |
| ☑ | **P0-B1/B2** | `terrain_material_atlas.rs` + plugin |
| ☑ | **P0-A4** | Extend tests: sim enter ⇒ non-CPU in release **without env** | `simulation_enter_applies_default_authority` |

### P0-D — building stamps off CPU blit

| ☐ | ID | Files | Exit |
|:---:|:---|:---|:---|
| ☑ | **GPU-P0-D-001** | `map_tile_atlas_stamp.rs` | `stamp_cpu_rgba_blit_enabled` + GPU index queue (dual path) |
| ☑ | **GPU-P0-D-002** | `tile_atlas_index.rs` | `variant_key_resolves_archive_uv_and_lookup_frame` |
| ☑ | **GPU-P0-D-003** | tests | `gpu_stamp_updates_instance_index_not_cpu_buffer` |

### P0-E — minimap terrain decouple

| ☐ | ID | Files | Exit |
|:---:|:---|:---|:---|
| ☑ | **GPU-P0-E-001** | `minimap_compositor/pass.rs` | `minimap_terrain_source_label` + GPU authority path |
| ☐ | **GPU-P0-E-002** | `minimap_compositor/composite.rs` | Cadence + dirty revision only (operator verify) |
| ☑ | **GPU-P0-E-003** | witness | `minimap_terrain_source_label` lib test + gpu witness field |

### P0 gate (ALL required before P1 sign-off)

```powershell
cargo test -p proc_A_dine01 --lib terrain_render terrain_instanced minimap_shell tile_world_fallback --release
cargo run -p proc_A_dine01 --release -- --test demo --stay-open
# 60s: tile_raster_ms == 0 · terrain_authority == GpuInstancedAtlas
```

---

## Phase P1 — strip dead render work (verify + gap-fill)

| ☐ | ID | Scope | Status |
|:---:|:---|:---|:---|
| ☑ | **P1-A** | Fire particle zero skip | `gpu_particle_draw.rs` + test |
| ☑ | **P1-A-verify** | Spark + fire raster same contract | `gpu_spark_compute.rs`, `gpu_fire_particle_raster.rs` zero-skip tests |
| ☑ | **P1-B** | Water/weather particle zero skip | `gpu_water_particle_draw.rs`, `gpu_weather_fire_field.rs` |
| ☑ | **P1-C** | Projection graph overlay skip | `overlay_projection_idle` + test |
| ☑ | **P1-D** | Fire extract cadence (not every frame) | `fire_extract_cadence_due` + tests |
| ☑ | **P1-E** | Minimap overlay defaults | `witness_harness_overlays_richer_than_simulation_defaults` |

**Exit:** `fire_extract.extract_ms` p95 < 1ms when no fire; witness dispatch_count == 0 idle.

---

## Phase P2 — dirty-gate representation (verify + gap-fill)

| ☐ | ID | Scope | Status |
|:---:|:---|:---|:---|
| ☑ | **P2-A** | `WorldRepresentationFrame` fingerprint skip | `world_repr_fingerprint_same_except_stamp` |
| ☑ | **P2-B** | Overlay buffer push gate | `chunk_fire_heat_maps_differ` tests + sync gate in extract |
| ☑ | **P2-C** | Streaming reconstruct idle gate | `warm_gate_skips_when_pending_all_cached_and_idle` |
| ☑ | **P2-D** | Authority map doc paste | plan §6 + `07-repo-authority-map.md` |

---

## Phase P3 — profiling consolidation

| ☐ | ID | Scope |
|:---:|:---|:---|
| ☑ | **P3-A** | Optional `tracy` feature + doc |
| ☑ | **P3-B** | Witness contract (`sim_spectrum` + `render_schedule`) |
| ☑ | **P3-C** | Trim STALL spam default demo |
| ☑ | **P3-D** | `visual_test_runbook_v1.md` perf truth section |

---

## Blocked — do not pick until predicate green

| ID | Block | Unblock |
|:---|:---|:---|
| **DR-MIG-TILEMAP** | `bevy_ecs_tilemap` 0.19 on crates.io | Steward monitor only |
| **DR-GPU-TERRAIN-P0C** | P0-C1..C5 tilemap default path | DR-MIG-TILEMAP **OR** P0-C′ signed + perf gates |
| **P0-C1..C5** | Full GPU tilemap render | After DR-MIG-TILEMAP |

When unblocked: **one day** timebox P0-C; else stay on P0-C′ per plan.

---

## POST-MIG perf (related, not P0-C′)

| ☐ | ID | Plan | Note |
|:---:|:---|:---|:---|
| ☐ | **MIG-A11-DEEP** | depth prepass merge | POST-MIG — `plan_gpu_terrain` / render |
| ☐ | **MIG-A13-DEEP** | GPU light clustering vs CPU fire extract | Fire perf lane |
| ☐ | **R7** | Generic particle spine (water + fire) | `plan_cleanup_v1` — after D3 gating |
| ☐ | **DR-RTT-VR16** | Operator `--test vfx` | Display required · PERF acceptance |

---

## Program exit gate (§10 — ALL must pass)

| # | Check |
|:---:|:---|
| 1 | `cargo test -p proc_A_dine01 --lib --release` green |
| 2 | `cargo test -p proc_A_dine01 --lib stage5 --release` green |
| 3 | Demo 60s p95 frame ≤ 33ms |
| 4 | Visual 60s p95 frame ≤ 33ms |
| 5 | `tile_raster_ms == 0` steady sim |
| 6 | `render_and_present_ms` p95 ≤ 16ms |
| 7 | No stall mismatch idle 60s |
| 8 | `minimap_compositor_live.json` green |
| 9 | `stage5_full_app_live.json` green |
| 10 | Runbook updated |

**Sign-off:** append row to `post_stage6_active_todos.md` with witness paths + date.

---

## Session pick order (recommended)

1. **GPU-P0C-PRIME-001..004** — flip default + witness (single @coder session)
2. **GPU-P0-D-001..003** — stamps off CPU blit
3. **GPU-P0-E-001..003** — minimap GPU terrain source
4. **P0 gate** — operator `--test demo` + refresh sim_spectrum
5. **P1 verify** — gap-fill only where tests fail
6. **P2 → P3** — after P0 gate green
7. **P0-C tilemap** — when DR-MIG-TILEMAP clears

---

## File ownership (quick ref)

| Area | Key paths |
|:---|:---|
| Authority | `terrain_render_authority.rs`, `engine_with_worldgen.rs` |
| Atlas + instanced | `terrain_material_atlas.rs`, `terrain_instanced_draw.rs`, `assets/shaders/terrain/` |
| CPU fallback gate | `tile_world_fallback.rs` |
| Stamps | `gui/map_tile_atlas_stamp.rs`, `construction/procedural/tile_atlas_index.rs` |
| Minimap | `render/minimap_compositor/*` |
| Particles | `gpu_particle_draw.rs`, `gpu_spark_compute.rs`, `fire_vfx/` |
| Witness | `sim_spectrum_analytics.rs`, `stage5_full_app_harness.rs` |
