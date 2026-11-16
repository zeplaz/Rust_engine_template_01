# PLAN-DENSE-HEAT-WGSL-v1 — TAC-03 dense heat (GPU-primary)

| Field | Value |
|:---|:---|
| **Id** | **PLAN-DENSE-HEAT-WGSL-v1** |
| **Status** | **PROGRAM CLOSE** — DH-01…03b shipped; **TAC-03 ADOPTED** |
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
| **Status** | **DONE** 2026-11-16 (WGSL + dense play; full RenderApp dispatch readback = follow-on) |

### DH-03a — fire lead + weather RO (**partial**)
| Field | Value |
|:---|:---|
| **Needs** | DH-02 |
| **Files** | `heat_weather.rs` · `heat_diffusion_gpu` · `fire_vfx/emit.rs` |
| **Do** | Climate RO → wind advection + rain damp on dense heat; fire VFX RO samples dense heat |
| **Accept** | `--lib tac03_dh03a` · `--lib tac03_dense_gpu` |
| **Status** | **DONE** 2026-11-16 |

### DH-03b — nav + attention · retire sparse · ADOPT
| Field | Value |
|:---|:---|
| **Needs** | DH-03a |
| **Files** | `NavFieldFrame` heat costs · `sim_attention` dense RO · `heat_consumers.rs` |
| **Do** | Wire nav + attention RO; keep sparse gated; ADOPT TAC-03 |
| **Accept** | `--lib tac03_dh03b` · `adoption.tsv` **TAC-03** ADOPTED |
| **Unlocks** | **TAC-03** |
| **Status** | **DONE** 2026-11-16 |

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
