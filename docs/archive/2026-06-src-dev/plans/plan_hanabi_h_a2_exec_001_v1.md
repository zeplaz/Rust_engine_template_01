# PLAN-HANABI-H-A2-EXEC-001 — Hanabi L3 feature-flag plugin `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-HANABI-H-A2-EXEC-001** |
| **Slice ID** | **H-A2-001** |
| **Parent** | [`plan_hanabi_adoption_v1.md`](plan_hanabi_adoption_v1.md) phase **H-A2** |
| **Designer gate** | [`hanabi_spike_review_h_a2_gate_v1.md`](hanabi_spike_review_h_a2_gate_v1.md) — **CLEARED (qualified)** |
| **Bounds** | [`hanabi_event_vfx_style_bounds_v1.md`](hanabi_event_vfx_style_bounds_v1.md) |
| **Spike** | [`experiments/hanabi_validation/report_v1.md`](../../experiments/hanabi_validation/report_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). **Default engine binary must not** register Hanabi until witness row green **and** `hanabi_l3` feature enabled at build time.

---

## Summary

Wire optional **`HanabiEmbellishmentPlugin`** behind Cargo feature **`hanabi_l3`**. L3 reads tactical extract / domain snapshots only; **no** L1 slab or weather writes. Witness `hanabi_spike_report_present` proves spike path without implying default-binary merge.

---

## Entry gates (all required)

| Gate | Evidence |
|:---|:---|
| Spike report | `experiments/hanabi_validation/report_v1.md` PASS (qualified) |
| Designer H-A2 | `hanabi_spike_review_h_a2_gate_v1.md` **CLEARED (qualified)** |
| Bevy 0.18 experiment | `cargo check -p hanabi_validation` |
| Stage 5 tactical VFX | `stage5_full_app_live.json` → `tactical_vfx_witness.all_green: true` |
| Main crate unchanged default | no `bevy_hanabi` in default `EnginePlugin` dependency graph |

---

## Authority map

| Resource | Single writer | Allowed | Must NOT |
|:---|:---|:---|:---|
| Hanabi particle spawn | `HanabiEmbellishmentPlugin` (L3) | burst on extract events | `ChunkWeather`, slab, hydrology |
| Tactical extract inputs | existing fire/smoke extract | read-only sample | duplicate smoke Layer A |
| Witness `hanabi_spike_report_present` | dev proof / stage5 harness | bool from report path check | hand-edited JSON |
| Default `EnginePlugin` | `engine_with_worldgen` | **no** Hanabi without feature | unconditional `HanabiPlugin` |

---

## PR plan (≤3 files each)

### H-A2-1 — Cargo feature + optional dependency

| File | Change |
|:---|:---|
| `Cargo.toml` (root) | `[features] hanabi_l3 = []`; optional `bevy_hanabi` path aligned to 0.18 |
| `Cargo.toml` (`proc_A_dine01`) | `bevy_hanabi` optional dep behind `hanabi_l3` |
| `experiments/hanabi_validation/Cargo.toml` | reference only — do not fork versions |

**Rule:** `cargo check` without features must succeed unchanged.

### H-A2-2 — Plugin module (feature-gated)

| File | Change |
|:---|:---|
| `src/render/hanabi_embellishment.rs` | **new** — `HanabiEmbellishmentPlugin`, preset table from bounds tokens |
| `src/render/mod.rs` | `#[cfg(feature = "hanabi_l3")]` plugin registration hook |
| `src/engine/engine_with_worldgen.rs` | `add_plugins` only when `cfg!(feature = "hanabi_l3")` **and** env `RUST_ENGINE_HANABI_L3=1` (double gate) |

**Presets (production caps):**

| Token | Max instances | Notes |
|:---|---:|:---|
| `fire_ember_burst` | ≤ 20 | designer TUNE headroom vs spike 24 |
| `water_splash_mist` | ≤ 16 | per report |
| `construction_micro_spark` | ≤ 8 | one-shot on commit event |
| `reject_arcade_muzzle_stack` | **forbidden** | anti-pattern reference only |

### H-A2-3 — Witness + lib tests

| File | Change |
|:---|:---|
| `src/render/stage5_closure_witnesses.rs` or `src/dev/stage5_live_todos.rs` | `hanabi_spike_report_present: true` when report exists |
| `src/render/hanabi_embellishment.rs` | lib test: plugin builds under `hanabi_l3`; no systems without feature |
| — | `cargo test -p proc_A_dine01 --lib stage5 --features hanabi_l3` |

---

## Witness schema

| File | Pointer | Meaning |
|:---|:---|:---|
| `stage5_full_app_live.json` | `/hanabi_spike_report_present` | report on disk |
| `stage5_full_app_live.json` | `/hanabi_l3_plugin_wired` | optional — true only with feature + env |
| `experiments/hanabi_validation/report_v1.md` | — | source of truth for spike |

**H-A2 green rollup:**

```text
hanabi_spike_report_present == true
AND tactical_vfx_witness.all_green == true
AND default binary: hanabi_l3_plugin_wired == false OR absent
AND (with --features hanabi_l3 + env): plugin registers without stage5 regression
```

---

## Verification

```powershell
cargo check -p hanabi_validation
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib stage5
# Optional implementation proof:
cargo test -p proc_A_dine01 --lib stage5 --features hanabi_l3
```

---

## Anti-patterns

- `app.add_plugins(HanabiPlugin)` on default binary
- Particles writing L1 / slab / `ChunkSmokeField`
- Minimap / strategic / World Preview draws
- Presets derived from `reject_arcade_muzzle_stack`
- Reopening **PLAN-HANABI-ADOPTION-001** charter (reference only)
- Upgrading main Bevy to satisfy Hanabi

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `H-A2-001` |
| **Witness** | `stage5_full_app_live.json` + spike report path |
| **Mutex** | disjoint from `src/substrate/ecs_retire.rs` unless coordinated |
| **Acceptance** | H-A2 green rollup above |
