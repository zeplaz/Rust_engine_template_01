# Hanabi validation spike — report v1

| Field | Value |
|:---|:---|
| **Slice** | H-A-SPIKE-001 / PLAN-HANABI-ADOPTION-001 |
| **Designer bounds** | DESIGN-HANABI-BOUNDS-001 PASS (qualified) |
| **Designer review** | DESIGN-HANABI-SPIKE-REVIEW-001 → [`hanabi_spike_review_h_a2_gate_v1.md`](../../docs/archive/2026-06-src-dev/plans/hanabi_spike_review_h_a2_gate_v1.md) — H-A2 **CLEARED (qualified)** |
| **Date** | 2026-05-27 |
| **Scope** | `experiments/hanabi_validation/` only — main `EnginePlugin` unchanged |
| **Bevy gate** | PASS — `cargo check -p hanabi_validation` |
| **Hanabi crate** | bevy_hanabi 0.18 |

## Executive summary

Bevy **0.18** + **bevy_hanabi 0.18** compile in the isolated experiment crate. Layer-3 presets for fire ember, water splash, and construction micro-spark are **PASS** against designer numeric bounds. Arcade anti-pattern sample is documented **REJECT**.

**Spike verdict:** **PASS (qualified)** — proceed to designer re-review only if a future preset hits **TUNE** in production wiring.

## Aggregate metrics (PASS/TUNE presets only)

| Metric | Measured | Bound | Verdict |
|:---|:---|:---|:---|
| Peak instances / frame | 24 | ≤ 32 | PASS |
| Worst peak α | 0.35 | ≤ 0.45 | PASS |

| Lifetime histogram | | 0.2–1.2 s window | |

| — 0.2-0.4s | 1 preset(s) | | |
| — 0.4-0.8s | 2 preset(s) | | |
| — 0.8-1.2s | 0 preset(s) | | |

## Per-preset bounds table

| Preset | Domain | Peak instances | Lifetime (s) | Peak α | Verdict |
|:---|:---|---:|:---|---:|:---|

| `fire_ember_burst` | fire_edge | 24 | 0.35–0.85 | 0.35 | **PASS** |
| `water_splash_mist` | water_surface | 16 | 0.25–0.55 | 0.28 | **PASS** |
| `construction_micro_spark` | construction_commit | 8 | 0.20–0.35 | 0.22 | **PASS** |
| `reject_arcade_muzzle_stack` | anti_pattern | 96 | 0.10–2.50 | 0.92 | **REJECT** |

## Designer rubric mapping

| Signal | Spike result |
|:---|:---|
| Material kick-up at fire/water edge | PASS presets (`fire_ember_burst`, `water_splash_mist`) |
| Construction micro-spark on commit | PASS (`construction_micro_spark`) |
| Muzzle-flash / neon / screen-fill | REJECT (`reject_arcade_muzzle_stack` — reference only) |
| Particles write L1 sim / weather | **Not attempted** — read-only L3 charter |
| Minimap / strategic zoom | **Not attempted** — tactical L3 only |

## Bevy 0.18 gate

- Root [`Cargo.toml`](../../Cargo.toml): `bevy = "0.18"` (main crate unchanged)
- Experiment: `bevy = "0.18"`, `bevy_hanabi = "0.18"`
- CI: `cargo check -p hanabi_validation`

## Regression (main app)

```powershell
cargo test -p proc_A_dine01 --lib stage7
```

Main app does **not** link `bevy_hanabi` until H-A2 feature gate.

## Optional captures

Operator may add PNGs under `assets/vfx/reference/review_captures/hanabi_spike/` (not required for spike exit).

