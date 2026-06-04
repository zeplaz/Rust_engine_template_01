# PLAN-HANABI-ADOPTION-001 — Hanabi spike adoption charter `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-HANABI-ADOPTION-001** |
| **Designer bounds** | [`hanabi_event_vfx_style_bounds_v1.md`](hanabi_event_vfx_style_bounds_v1.md) (**DESIGN-HANABI-BOUNDS-001** PASS qualified) |
| **Visual contract** | [`wssr_migration_visual_contract_v1.md`](wssr_migration_visual_contract_v1.md) § H-A |
| **Identity** | [`project_identity_guard_rail_v1.md`](project_identity_guard_rail_v1.md) · [`wssr_identity_alignment_record_v1.md`](wssr_identity_alignment_record_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). **Spike only** — no main `EnginePlugin` merge in this plan.

---

## Summary

Charter for **`experiments/hanabi_validation/`** — validate Bevy **0.18** + Hanabi-compatible particle stack as **Layer 3 embellishment** (embers, micro-sparks, local wisps). Main app adoption is a **separate** gate after spike report + designer re-review.

---

## Bevy 0.18 gate (hard)

| Check | Requirement |
|:---|:---|
| Engine Bevy | `0.18` per root [`Cargo.toml`](../../Cargo.toml) |
| Spike crate | `bevy = "0.18"` workspace-aligned; pin Hanabi fork/crate version in experiment `Cargo.toml` |
| CI | `cargo check -p hanabi_validation` (experiment package) without upgrading main crate Bevy |
| Failure | **STOP** — do not patch main app Bevy to “make Hanabi work” |

---

## Scope boundaries

| In scope (L3) | Out of scope |
|:---|:---|
| Local burst at fire edge / water splash / construction micro-spark | Weather, smoke Layer A, atmosphere clipmap |
| Read-only sampling from extract / domain snapshots | Writing `ChunkWeather`, slab, hydrology |
| Tactical zoom band only (per migration contract) | Minimap, strategic map, World Preview |
| Instance caps per designer bounds (≤32/event) | Screen-fill VFX, neon trails |

---

## Spike deliverables

| # | Artifact | Owner |
|:---:|:---|:---|
| 1 | `experiments/hanabi_validation/Cargo.toml` | @coder spike |
| 2 | `experiments/hanabi_validation/README.md` — how to run | @coder |
| 3 | `experiments/hanabi_validation/report_v1.md` — counts, α, lifetime vs bounds table | @coder |
| 4 | Screenshots or capture paths under `assets/vfx/reference/review_captures/hanabi_spike/` | @operator optional |
| 5 | Witness row `hanabi_spike_report_present: true` (when wired in dev proof) | @coder |

**Report must include:** peak instances/frame, worst α, lifetime histogram, PASS/TUNE/REJECT per bounds rubric.

---

## Adoption phases (post-spike only)

| Phase | Action | Gate |
|:---|:---|:---|
| **H-A0** | Spike in `experiments/` only | This plan READY |
| **H-A1** | Designer re-review if any **TUNE** overrun | DESIGN-HANABI-BOUNDS signoff row |
| **H-A2** | Optional `HanabiEmbellishmentPlugin` behind feature flag `hanabi_l3` | `hanabi_spike_report_present` + stage5 tactical VFX green |
| **H-A3** | Wire to projection graph consumer slot (fire/water extract) | W4-C event embellishment plan when product opens |

**Forbidden until H-A2:** `app.add_plugins(HanabiPlugin)` on default engine binary.

---

## Witness / regression

| File | Required |
|:---|:---|
| `stage5_full_app_live.json` | `readiness.passes: true`; `tactical_vfx_witness` unchanged or improved |
| `fire_ecology_live.json` | no F1 regression when spike touches fire read path |
| Future | `debug_runs/hanabi_spike_live.json` optional — not required for charter exit |

```powershell
cargo check -p hanabi_validation
cargo test -p proc_A_dine01 --lib stage5
# After H-A2 only:
cargo test -p proc_A_dine01 --lib stage5 --features hanabi_l3
```

---

## Anti-patterns

- Main-plugin merge before spike report
- Particles writing L1 sim or slab fields
- Minimap/strategic particle draws
- Arcade VFX (muzzle stacks, neon fill) per designer **REJECT** row
- Reopening WSS weather/smoke authority plans

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | `H-A` spike lane (experiments); future W4-C embellishment |
| **Depends on** | DESIGN-HANABI-BOUNDS-001 PASS |
| **Acceptance** | `experiments/hanabi_validation/report_v1.md` exists; bounds table PASS/TUNE documented; main app unchanged |
