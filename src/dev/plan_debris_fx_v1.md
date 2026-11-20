# DEBRIS-FX-v1 — collapse particle burst (presentation)

| Field | Value |
|:---|:---|
| **Id** | **DEBRIS-FX-v1** |
| **Program** | [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) |
| **Status** | **SHIPPED (code, unit-tested; not seen live)** — collapse dust persists ~6 s through the shared dust book; was a one-frame, wrong-tile push |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` |

## Design

On demolish footprint mark, push a short burst of [`GpuParticleInstance`](../render/fire_vfx/pack.rs) rows with earth/concrete tint (low heat, dust class) into [`WorldFireParticleFrame`](../render/fire_vfx/frame.rs) — same GPU draw path, not rigid bodies.

## Production (corrected 2026-11-19)

The original push inside `execute_demolish_at_tile` was wrong twice: rows were written `(x, 1.2, -y)` (PixelWorld axes) while every consumer reads `xy` = tile position with the lift in `ember_class_radius_smoke.w`, so the burst landed on the wrong tile; and `WorldFireParticleFrame` is rebuilt every frame, so it lived one frame. `construction/debris_fx.rs` is deleted.

Now: [`blast_dust.rs`](../render/fire_vfx/blast_dust.rs) `detect_collapse_dust` reads sim components (footprint removed = site collapsed by **any** demolish path; `DamBody.column_integrity` at the collapse bar = failed dam column, once per column) into `BlastDustBook`; `append_blast_dust_particles` re-appends `smoke_puff` rows every frame for `COLLAPSE_DUST_S` (6 s) after the fire build. One path shared with blast dust (D-3).

Tests: `cargo test -p proc_A_dine01 --lib blast_dust::` (tile position + lift field, persistence over several frames, expiry, dam column fires once).

Open: `execute_demolish_at_tile` keeps an unused `_debris_frame` parameter (callers in `disaster_proxy.rs` etc.); drop it with its callers. A removed footprint also dusts for non-collapse removals (undo of a placement, scenario teardown). Dust is smoke class, not yet material-tinted.
