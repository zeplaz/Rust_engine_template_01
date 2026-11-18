# DEBRIS-FX-v1 — collapse particle burst (presentation)

| Field | Value |
|:---|:---|
| **Id** | **DEBRIS-FX-v1** |
| **Program** | [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) |
| **Status** | **ADOPTED** — enqueue material-tint burst into fire_vfx particle spine |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` |

## Design

On demolish footprint mark, push a short burst of [`GpuParticleInstance`](../render/fire_vfx/pack.rs) rows with earth/concrete tint (low heat, dust class) into [`WorldFireParticleFrame`](../render/fire_vfx/frame.rs) — same GPU draw path, not rigid bodies.

## Production

[`debris_fx.rs`](../construction/debris_fx.rs) called from `execute_demolish_at_tile` when optional frame present.
