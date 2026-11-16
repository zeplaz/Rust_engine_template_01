# PLAN-DENSE-HEAT-WGSL-v1 — TAC-03 dense heat (GPU-primary)

| Field | Value |
|:---|:---|
| **Id** | **PLAN-DENSE-HEAT-WGSL-v1** |
| **Status** | **REOPENED 2026-11-16** — GPU-primary not delivered: WGSL never loaded or dispatched, play runs on CPU. **TAC-03 → PARTIAL** · cleanup **CQ-50** (+CQ-38, CQ-63) |
| **Date** | 2026-11-16 |
| **Parents** | [`plan_ref_books_defer_unlock_v1.md`](plan_ref_books_defer_unlock_v1.md) §1/W5 · **DR-BOOKS-TAC03-DENSE** |
| **Unlocks** | **TAC-03** |
| **Dual-cite** | aig3 separable / dense convolution (book TAC-03) |

## Product stance (operator 2026-11-16)

| Decision | Choice |
|:---|:---|
| Runtime | **GPU-primary** dense separable diffusion — not “CPU sim + texture upload” |
| CPU role | **Parity / test / replay reference only** (same stencil, ≤1e-5) — not the play authority |
| Density | Full lattice (chunk / padded ROI) — retire sparse Jacobi as play path |
| Consumers | **All three:** fire · nav · attention |
| Lead quality | **Fire simulation + VFX** are first-class; heat must serve fire truth + presentation |
| Weather | Fire↔weather coupling stays in scope (wind / moisture / grounding inputs into heat+fire, RO weather → heat/fire writers) |

**Evidence path:** product requires real GPU diffusion + fire quality (not profiler-only). Sparse path is insufficient by design for this product.

## Review 2026-11-16 — reopened

> Operator code review of the Cursor wave ([`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md)). The product stance above stands: **GPU-primary**. The review found the GPU half missing, so TAC-03 goes back to PARTIAL.

| Finding | Evidence | Fix (CQ-50) |
|:---|:---|:---|
| Play heat is CPU | `compute_dispatch_graph.rs:247,315` runs `run_dense_heat_step` | RenderApp compute node: inject → H → V → decay, ping-pong storage buffers (pattern: `render/pipelines/gpu_spark_compute.rs`, `gpu_weather_fire_field.rs`) |
| No readback | no `Readback` anywhere in `src/` | Bevy 0.20 `bevy_render::gpu_readback::Readback::buffer` + `ReadbackComplete` observer = **sole** main-world writer of `DenseHeatPlayField`, tagged with its `SimStepStamp` |
| Fake parity | `heat_diffusion_gpu.rs:206-235` | Real GPU-vs-CPU ≤1e-5 check where a GPU exists (playcheck / `--ignored`); CPU kernel = headless/lib-test reference + no-GPU fallback chosen at startup, never alongside GPU |
| Fire VFX reads main-world CPU field | `fire_vfx/emit.rs:227,388` | Bind the GPU buffer in the render world (no readback for VFX) |
| Nav consumer write-only | `heat_diffusion.rs:334-352` | Feed into the single route cost (CQ-52/CQ-53) |
| ROI bugs | `heat_diffusion_gpu.rs:67`, `sim_attention.rs:219` | Union on expand, bounded/tiled ROI, cap + throttle attention foci (CQ-63) |

**Guard:** fire spread (`ChunkFireOverlay`) must not read dense heat, because async readback would make seeded runs frame-timing dependent. **Follow-on:** once GPU-primary works, consider per-cell instead of per-chunk resolution for fire VFX quality.

**Close again only when:** the GPU node dispatches in play, the readback observer is the only play writer, VFX reads the GPU buffer, nav reads the cost, and the real parity check passes. Every ADOPTED claim cites a production caller `file:line` (a test alone does not count).

## Why current path is not TAC-03

`compute/heat_diffusion.rs` = sparse active-set Jacobi + variable neighbor mean + decay/source. GPU path today **uploads packed CPU rows** — **no WGSL diffusion kernel**. That is the opposite of GPU-primary. `combat/relax.rs` / `strategic/hybrid_fields.rs` stay separate operators unless a later charter says otherwise.

## Authority (frozen)

```text
⊚Weather / fuel / fire sources ═▶ ⊨ inputs (RO into heat step)
⊚HeatDiffusion GPU dense step ═▶ ⊨ play Heat field (sole play writer)
⊚HeatDiffusion CPU dense step ═▶ ⊨ parity fixture / replay check only
⊚Fire sim + fire VFX ◂⊳[RO] dense heat (+ weather RO)
⊚Nav cost / Attention gates ◂⊳[RO] dense heat
⛔ Sparse Jacobi as play authority after DH-03
⛔ Dual play writers (CPU+GPU both mutating play field)
```

## Phases

### DH-00 — sign dense domain ✅
| Field | Value |
|:---|:---|
| **Do** | Freeze domain + stencil + parity ≤1e-5 + consumer list + GPU-primary |
| **Accept** | This charter **SIGNED** |

### DH-01 — dense CPU reference (parity harness)
| Field | Value |
|:---|:---|
| **Needs** | DH-00 |
| **Files** | `src/compute/heat_diffusion_dense.rs` · `HeatDiffusionConfig.use_sparse_play` |
| **Do** | Fixed-stencil separable dense CPU for **tests/replay only**; flag-gate sparse |
| **Accept** | `--lib tac03_dense_cpu` |
| **Don't** | Ship CPU as play default |
| **Status** | **DONE** 2026-11-16 |

### DH-02 — WGSL dense diffusion (play authority)
| Field | Value |
|:---|:---|
| **Needs** | DH-01 |
| **Files** | `assets/shaders/compute/heat_diffusion_dense.wgsl` · `heat_diffusion_gpu.rs` |
| **Do** | Real H/V diffusion kernel; parity vs CPU ≤1e-5; sparse gated off; dense play packs field buffers |
| **Accept** | `--lib tac03_dense_gpu` · parity fixture |
| **Don't** | CPU-only rename as GPU |
| **Status** | **REOPENED 2026-11-16** — the WGSL text exists, but there is no pipeline, bind group, dispatch or readback. The `tac03_dense_gpu` "parity" test compares the CPU step with a clone of itself (`heat_diffusion_gpu.rs:226-231`), and the WGSL check substring-matches constants. This is the "CPU-only rename as GPU" this phase's **Don't** row forbids. → CQ-50 |

### DH-03a — fire lead + weather RO (**partial**)
| Field | Value |
|:---|:---|
| **Needs** | DH-02 |
| **Files** | `heat_weather.rs` · `heat_diffusion_gpu` · `fire_vfx/emit.rs` |
| **Do** | Climate RO → wind advection + rain damp on dense heat; fire VFX RO samples dense heat |
| **Accept** | `--lib tac03_dh03a` · `--lib tac03_dense_gpu` |
| **Status** | **PARTIAL 2026-11-16** — weather RO into heat is real. Fire VFX reads the CPU `DenseHeatPlayField` from the main world with a magic ×0.85 blend (`fire_vfx/emit.rs:227,388`); it must bind the GPU buffer in the render world. `heat_weather.rs:48` reads render `ClimateVisualAggregate` (layer inversion, CQ-37). → CQ-50, CQ-38 |

### DH-03b — nav + attention · retire sparse · ADOPT
| Field | Value |
|:---|:---|
| **Needs** | DH-03a |
| **Files** | `NavFieldFrame` heat costs · `sim_attention` dense RO · `heat_consumers.rs` |
| **Do** | Wire nav + attention RO; keep sparse gated; ADOPT TAC-03 |
| **Accept** | `--lib tac03_dh03b` · `adoption.tsv` **TAC-03** ADOPTED |
| **Unlocks** | **TAC-03** |
| **Status** | **PARTIAL 2026-11-16** — the attention RO is real (`sim_attention.rs:143`), but it scans the full grid every frame, one focus per hot cell (CQ-63). Nav `heat_cost_samples` / `mean_heat_cost` are written and never read by any pathfinder (CQ-52 → CQ-53 route cost). The ROI never shrinks and drops heat on expand (CQ-63). |

## Rejects

- “Finish TAC-03” on sparse path  
- WGSL that only uploads CPU rows  
- Dual **play** heat authorities  
- Dropping fire↔weather to “later polish”  
- Skipping nav/attention after promising all three  

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | Conditional outline; DH-00…DH-03. |
| 2.0.0 | 2026-11-16 | **SIGNED** — GPU-primary; CPU=parity only; consumers fire+nav+attention; fire lead + weather in scope. |
| 2.1.0 | 2026-11-16 | **DH-01** — `heat_diffusion_dense` separable H∥V + `HeatDiffusionConfig` sparse gate; Accept `--lib tac03_dense_cpu`. |
| 2.2.0 | 2026-11-16 | **DH-02** — WGSL dense H/V kernels + dense play path; Accept `--lib tac03_dense_gpu`. |
| 2.3.0 | 2026-11-16 | **DH-03a** — weather RO → dense heat; fire VFX RO blend; Accept `--lib tac03_dh03a`. |
| 2.4.0 | 2026-11-16 | **DH-03b** — nav heat costs + attention RO; sparse retired; Accept `--lib tac03_dh03b`; **TAC-03 ADOPTED**. **PROGRAM CLOSE**. |
| 2.5.0 | 2026-11-16 | **REOPENED** — review: GPU dispatch/readback missing, parity test tautological, nav consumer write-only; TAC-03 → PARTIAL; finish via cleanup CQ-50. |
