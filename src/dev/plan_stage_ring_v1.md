# STAGE-RING-v1 — tiered presentation upload ring

| Field | Value |
|:---|:---|
| **Id** | **STAGE-RING-v1** |
| **Program** | [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) |
| **Status** | **ADOPTED** (Tier 0 only) — mask always uploads; Tier 1+ budgets PLANNED |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` (render) |
| **Parents** | DESTROY-T2 · PLAN-DESTROY-PRESENTATION-v1 |

## Objective

Battle-tick dirty storms must not stall the **destruction mask** (or other Tier-0 presence). Time-slice heavier field uploads behind a per-frame byte/chunk budget.

## Tiers (locked)

| Tier | Content | Budget |
|:---|:---|:---|
| **0** | Destruction mask GPU upload | **Always** — no skip |
| **1** | On-frustum terrain remesh priority (DESTROY-T2b) | `CHUNKS_PER_FRAME` |
| **2** | Scorch / fire overlay / off-frustum remesh | Future byte budget |
| **3** | Bake / atlas / cold staging | Deferred frames |

## Accept (v1 Tier 0)

| Check | Pass |
|:---|:---|
| Mask upload | [`upload_destruction_mask_gpu`](../render/pixel_world/destruction_mask.rs) runs whenever `dirty` — never behind a shared skip flag |
| Gate resource | [`PresentationUploadRing`](../render/pixel_world/destruction_mask.rs) documents tiers; Tier 0 path ignores `tier2_budget_exhausted` |
| Lib | `destroy_t2` / stage_ring honesty test |

## Non-goals (v1)

- Full byte accounting for scorch/water
- Multi-frame off-screen queue scheduler
