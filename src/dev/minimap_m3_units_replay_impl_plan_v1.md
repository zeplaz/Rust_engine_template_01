# PLAN-M3-MINMAP-001 — minimap M3 units + replay impl plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-M3-MINMAP-001** |
| **Coder lanes** | **UI-P3-M3-UNITS-001** · **UI-P3-M3-REPLAY-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — witness **CLOSED** · product depth **PARTIAL** |
| **Design M3 (FoW/EW)** | [`ui_p3_m4_minimap_coder_queue_v1.md`](ui_p3_m4_minimap_coder_queue_v1.md) — separate |
| **OH rollup** | [`ui_oh_m3_001_plan_v1.md`](ui_oh_m3_001_plan_v1.md) |
| **Witness** | `debug_runs/minimap_compositor_live.json` |

**No Rust in this deliverable.**

---

## Naming guard

| ID | Meaning |
|:---|:---|
| **UI-P3-M4-001** | Design **M3** fog + EW (`ui_p3_m4_green`) |
| **UI-P3-M3-UNITS-001** | M3-03 unit ticks (misleading “M3” in queue id) |
| **UI-P3-M3-REPLAY-001** | M3-04 replay scrub line |

---

## Executive summary

| Slice | Witness (2026-05-26) | Product depth |
|:---|:---:|:---|
| **UNITS-001** | **PASS** — `unit_marker_rows: 6`, `ui_p3_m3_units_001_green: true` | **PARTIAL** — seed coords; real strategic aggregation reader optional |
| **REPLAY-001** | **PASS** — `replay_scrub_enabled: true`, `ui_p3_m3_replay_001_green: true` | **PARTIAL** — lib ring seed; live sim ring growth optional |

**Do not** reopen **UI-P3-M4-001** or **ui_p3_m3_green** (M2 ecology) when tuning units/replay.

---

## Architecture

```text
Sim / test seed
  → MinimapOperationalSnapshot (fow, ew, unit_markers, replay ring)
  → composite.rs paint_* (CPU bridge path)
  → MinimapCompositorState (rows + flags)
  → live_proof.rs predicates
  → minimap_compositor_live.json
```

**Forbidden:** Second minimap extract; ECS fire/unit queries in compositor; hand-edited JSON greens.

---

## UI-P3-M3-UNITS-001

### Design authority

[`minimap_unit_marker_visual_spec_v1.md`](minimap_unit_marker_visual_spec_v1.md) (**DESIGN-M3-UNITS-001** **SIGNED**)

### Code anchors

| File | Symbol |
|:---|:---|
| [`composite.rs`](../render/minimap_compositor/composite.rs) | `paint_unit_markers`, `M3_UNIT_MARKER_CAP = 8` |
| [`visual_domain_snapshots.rs`](../render/visual_domain_snapshots.rs) | `seed_minimap_m3_units_replay_witness` |
| [`live_proof.rs`](../render/minimap_compositor/live_proof.rs) | `ui_p3_m3_units_001_green` |
| [`minimap_shell.rs`](../gui/minimap_shell.rs) | `MinimapOverlayMask.units` default on |

### PASS gate (witness)

```text
ui_p3_m3_units_001_green :=
  units_heat_enabled
  AND unit_marker_rows > 0
```

| # | Criterion | 2026-05-26 |
|:---:|:---|:---:|
| U1 | Toggle on | `units_heat_enabled: true` |
| U2 | Rows painted | `unit_marker_rows: 6` |
| U3 | Witness rollup | `ui_p3_m3_units_001_green: true` |
| U4 | Lib / compositor tests | `minimap_compositor` module tests green |

### Product forward (optional P2)

| # | Task | Owner |
|:---:|:---|:---|
| U-P1 | Read unit clusters from logistics / strategic snapshot | @coder |
| U-P2 | Filter unexplored FoW chunks | @coder |
| U-P3 | Respect cap 8 + designer density table | @coder |

**Witness seed alone satisfies U1–U4** — U-P1…P3 are polish, not blockers for M3 tails.

---

## UI-P3-M3-REPLAY-001

### Design authority

[`minimap_replay_scrub_visual_spec_v1.md`](minimap_replay_scrub_visual_spec_v1.md) (**DESIGN-M3-REPLAY-001** **SIGNED**)

### Code anchors

| File | Symbol |
|:---|:---|
| [`composite.rs`](../render/minimap_compositor/composite.rs) | `paint_replay_scrub` |
| [`systems/sim_frame_delta.rs`](../systems/sim_frame_delta.rs) | `CommittedSimReplayRing` |
| [`live_proof.rs`](../render/minimap_compositor/live_proof.rs) | `ui_p3_m3_replay_001_green` |

### PASS gate (witness)

```text
ui_p3_m3_replay_001_green :=
  replay_scrub_enabled
  (implies ring depth >= 2 and mask on at paint time)
```

| # | Criterion | 2026-05-26 |
|:---:|:---|:---:|
| R1 | Ring depth | `replay_scrub_enabled: true` |
| R2 | Witness rollup | `ui_p3_m3_replay_001_green: true` |
| R3 | Absent when inactive | Toggle off → no line (playtest) |
| R4 | Orthogonal to **REPLAY-PARITY-001** | Minimap tick ≠ editor parity |

### Product forward (optional P2)

| # | Task | Owner |
|:---:|:---|:---|
| R-P1 | Wire live `CommittedSimReplayRing` commits in **Simulation** (not seed-only) | @coder |
| R-P2 | Hide scrub when `stamps.len() < 2` after real session | @coder |
| R-P3 | No minimap click-to-scrub (v1 forbidden) | — |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor
cargo test -p proc_A_dine01 --lib coder_b_s7p_construction_mv_proof
```

**Refresh disk:**

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor::tests::minimap_compositor_live_witness_refresh
```

**Operator:** Simulation → minimap overlay tray → toggle **Units** / **Replay scrub**.

---

## Gate chain

```text
UI-P3-M4-001 (FoW/EW)           ☑ CLOSED
UI-P3-M3-001 (M2 ecology)        ☑ CLOSED (naming)
        │
        ├─► UI-P3-M3-UNITS-001   ☑ witness CLOSED
        └─► UI-P3-M3-REPLAY-001  ☑ witness CLOSED
```

---

## Copy-paste — @coder (product polish only)

```
Lane: UI-P3-M3-UNITS-001 / UI-P3-M3-REPLAY-001
Read: src/dev/minimap_m3_units_replay_impl_plan_v1.md
      minimap_unit_marker_visual_spec_v1.md
      minimap_replay_scrub_visual_spec_v1.md
Status: witness GREEN — do not hand-edit JSON
Optional: real unit reader + live replay ring (≤3 files each)
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor
Do NOT: reopen ui_p3_m4_green; second minimap extract
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-M3-MINMAP-001** signed — tails witness closed |
