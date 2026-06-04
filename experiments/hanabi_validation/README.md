# Hanabi validation spike (H-A-SPIKE-001)

Isolated experiment crate — **does not** modify main `EnginePlugin` or link into `proc_A_dine01`.

## Charter

| Doc | Role |
|:---|:---|
| [`src/dev/plan_hanabi_adoption_v1.md`](../../src/dev/plan_hanabi_adoption_v1.md) | Adoption phases H-A0…H-A3 |
| [`src/dev/hanabi_event_vfx_style_bounds_v1.md`](../../src/dev/hanabi_event_vfx_style_bounds_v1.md) | Designer numeric bounds (PASS qualified) |

## Bevy 0.18 gate

- Main crate: `bevy = "0.18"` (unchanged)
- This crate: `bevy = "0.18"`, `bevy_hanabi = "0.18"`

## Commands

```powershell
# Spike compile gate (required)
cargo check -p hanabi_validation

# Unit tests + Hanabi plugin registration
cargo test -p hanabi_validation

# Regenerate exit report
cargo run -p hanabi_validation --bin hanabi_validation

# Main app regression (unchanged binary)
cargo test -p proc_A_dine01 --lib stage7
```

## Exit artifact

[`report_v1.md`](report_v1.md) — peak instances, worst α, lifetime histogram, PASS/TUNE/REJECT table.

Optional operator captures: `assets/vfx/reference/review_captures/hanabi_spike/`

## Scope

Layer 3 embellishment only (embers, micro-sparks, local wisps). **Forbidden:** weather/smoke authority, minimap/strategic draws, L1 sim writes, main-plugin `HanabiPlugin` until H-A2.
