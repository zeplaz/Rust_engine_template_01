# Water VFX track — closure sign-off `v2` (PLAN-WATER-TRACK-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-WATER-TRACK-001** |
| **Version** | `2.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` (sign-off only) |
| **Track** | **FX-WATER** |
| **Status** | **CLOSED** — **do not re-queue W1 / W2 / foam slices** |
| **Doc type** | **Closure sign-off only** — not an implementation queue |
| **Coder detail (archive)** | [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) |
| **Steward gate** | [`steward_water_witness_gate_v1.md`](steward_water_witness_gate_v1.md) — **PASS** |
| **Designer** | [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) v1.1 **PASS** |
| **Ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) · **G-FX-WATER** |

**No new Rust.** This document **records** track closure. **W1**, **W2** (incl. coast foam), strategic cull, witness rollup, and **STEWARD-WATER-WITNESS-001** are **already done** — maintain regression only.

---

## Do not re-queue (hard rule)

| ID | Layer | Status | Policy |
|:---|:---|:---:|:---|
| **WATER-W1-OCEAN-001** | W1 ocean | **DONE** | No rework — `water_w1_green`, `water_ocean_tiles > 0` |
| **WATER-W1-RIVER-001** | W1 rivers | **DONE** | No rework — river streaks in witness |
| **WATER-W2-FOAM-001** | W2 coast + bend foam | **DONE** | **Do not re-queue foam** — `water_w2_foam_001_green: true`; river foam `0` **waived** on fixture (lib proves bend) |
| **WATER-STRATEGIC-001** | D-W09 cull | **DONE** | No rework |
| **WATER-WITNESS-001** | Harness rollup | **DONE** | No rework |
| **STEWARD-WATER-WITNESS-001** | Sim-steward | **PASS** | Witness refresh only if water **render** code changes |
| **FX-WATER-SHADER/PARTICLE-001/002** | Spine | **DONE** | Archive |
| **WATER-DESIGN-001** | Designer | **PASS** | Optional **TUNE** — not track blocker |

**Queued items with stale titles** (`river_foam: 0 — NOT CLOSED`, `ocean_tiles: 0`) are **superseded** by this sign-off — mark **done** in [`continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json), do **not** spawn new coder lanes.

**Optional polish (not closure):** `WATER-W2-TUNE-001`, `P2-WATER-POLISH-001`, `VFX-CAPTURE-INSIM-001` — designer/operator only; **no** ledger reopen.

---

## Closure record (completed)

```text
DONE  FX-WATER-SHADER-001/002 · FX-WATER-PARTICLE-001/002
DONE  WATER-W1-OCEAN-001 · WATER-W1-RIVER-001
DONE  WATER-W2-FOAM-001 (coast foam 128; river bend waived on catalog)
DONE  WATER-STRATEGIC-001 · WATER-WITNESS-001
PASS  STEWARD-WATER-WITNESS-001
PASS  WATER-DESIGN-001 (tactical channel)
      │
      ▼
CLOSED  FX-WATER / PLAN-WATER-TRACK-001
```

---

## Witness snapshot (fleet truth — use for audits)

**File:** [`debug_runs/stage5_full_app_live.json`](../../debug_runs/stage5_full_app_live.json)

| Field | Value | Meaning |
|:---|:---|:---|
| `water_w1_green` | `true` | W1 spine |
| `water_ocean_tiles` | `1303` | D-W04 ocean |
| `water_particle_rows` | `216` | tactical draws |
| `water_particle_coast_foam` | `128` | W2 coast — **closure met** |
| `water_particle_river_foam` | `0` | **Waived** — `catalog_has_river_bend: false` on visual seed |
| `water_w2_foam_001_green` | `true` | W2 rollup — **do not re-open for foam** |
| `water_strategic_001_green` | `true` | D-W09 |
| `water_witness_001_green` | `true` | WATER-WITNESS-001 |
| `tactical_vfx_witness.all_green` | `true` | Stage 5 spine |

**Steward:** [`steward_water_witness_gate_v1.md`](steward_water_witness_gate_v1.md) — **PASS** (2026-05-25).

**River bend proof (off-fixture):** `cargo test -p proc_A_dine01 --lib water_w2_foam_001_river_bend_emits_foam`

---

## W2 foam — closure note (why `river_foam: 0` is OK)

| Channel | Witness | Verdict |
|:---|:---|:---:|
| Coast (D-W08) | `water_particle_coast_foam: 128` | **CLOSED** |
| River bend (D-W07) | `river_foam: 0` on visual catalog | **WAIVED** — bend path covered by lib test |
| Parity | `river_streaks: 24`, `water_w2_foam_001_green: true` | **CLOSED** |

**Agents must not** interpret `river_foam: 0` as “re-queue WATER-W2-FOAM-001” when steward gate is **PASS** and `water_w2_foam_001_green: true`.

---

## Maintenance only (after water render edits)

```powershell
cargo test -p proc_A_dine01 --lib gpu_water_particles water_surface_visual stage5
cargo test -p proc_A_dine01 --lib water_witness_001
cargo run -p proc_A_dine01 --release -- --test visual
```

Re-run **STEWARD-WATER-WITNESS-001** checklist if `gpu_water_particles.rs`, `water_overlay.wgsl`, or harness rollup changes.

**Fire track** closed separately: [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) (**PLAN-FIRE-VFX-CLOSURE-001**).

---

## Authority map (archive)

| Writer | Files | Closed rule |
|:---|:---|:---|
| Coder A (W1) | `water_overlay.wgsl`, `water_surface_visual.rs` | No second hydrology extract |
| Coder B (W2) | `gpu_water_particles.rs`, harness | No `water_overlay.wgsl` in same slice |
| Steward | witness refresh | No feature work in closure pass |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-WATER-TRACK-001 closure only |
| Sim-steward | 2026-05-25 | **STEWARD-WATER-WITNESS-001 PASS** |
| Designer | 2026-05-25 | **WATER-DESIGN-001 PASS** (optional TUNE remains) |
| Coder W1/W2 | 2026-05-24–25 | **DONE** — **no re-queue** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v2.0.0 | 2026-05-25 | Closure sign-off only — do not re-queue foam/W1/W2 |
| v1.0.0 | 2026-05-25 | Rollup narrative (superseded by v2 for queue policy) |
