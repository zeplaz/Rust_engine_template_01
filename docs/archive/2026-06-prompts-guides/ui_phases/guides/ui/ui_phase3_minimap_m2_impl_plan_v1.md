# UI Phase 3 M2 — minimap overlay implementation plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-P3-M2-IMPL-001** |
| **Version** | `1.1.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` (impl rollup) · **execute:** `@coder` |
| **Status** | **CLOSED** — all M2 implementation slices landed |
| **Full plan** | [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) — **unblocks UI-P3-M2-CODER-A** |
| **Overlay plan** | [`ui_phase3_m2_minimap_overlay_plan_v1.md`](ui_phase3_m2_minimap_overlay_plan_v1.md) (**UI-P3-M2-PLAN**) |
| **Compositor spine** | [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) (M1) |
| **Compositor rollup** | [`ui_phase3_minimap_compositor_full_plan_v1.md`](ui_phase3_minimap_compositor_full_plan_v1.md) (**PLAN-UI-P3-COMPOSITOR-001**) |
| **Design sign-off** | [`minimap_d_m2_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/minimap_d_m2_signoff_v1.md) (**D-MINIMAP-M2**) |
| **Witness** | [`debug_runs/minimap_compositor_live.json`](../../../debug_runs/minimap_compositor_live.json) |
| **Ledger** | [`stage_tracks_signoff_ledger_v1.md`](../../../docs/archive/2026-06-src-dev/plans/stage_tracks_signoff_ledger_v1.md) |

**No new Rust in this doc.** Coder execution map for **M2 strategic overlays** on the GPU minimap compositor — post **M1** spine.

**Coder A entry:** [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) (gate chain + M2-02/03 contract).

---

## Executive summary

| Track slice | Verdict |
|:---|:---|
| **M2-01** logistics heat | **DONE** — `logistics_rows: 2` |
| **M2-02** construction heat | **DONE** — `construction_rows: 18` |
| **M2-03** ecology heat | **DONE** — `ecology_rows: 100` |
| **M2-04** no duplicate extract | **DONE** — snapshot readers only |
| **M2-05** overlay mask bits | **DONE** — `MinimapOverlayMask` |
| **M2-06** tray → compositor | **DONE** — `ui_p3_m2_tray_opt_green` |

**Naming:** [`ui_phase3_minimap_track_naming_v1.md`](ui_phase3_minimap_track_naming_v1.md) — **UI-P3-M3-001** = M2 construction + ecology; design M3 = **UI-P3-M4-001** / **D-MINIMAP-M3**.

---

## Architecture (implementation)

```text
Published snapshots (sim systems)
  LogisticsVisualSnapshot
  CorridorConstructionBook  ──► seed_minimap_m2_overlay_witness (test/visual)
  EcologyVisualSnapshot
  SharedOverlayFieldBuffers (fire — M1)
        │
        ▼
MinimapCompositorPlugin (GpuCompute path)
  composite.rs / pass.rs — sample fields into overlay uniforms
  minimap_composite.wgsl — fire + logistics + construction + ecology channels
        │
        ▼
MinimapShellState.overlays (MinimapOverlayMask)
  ◄── simulation_minimap_overlay_defaults()
  ◄── dock_shell overlay tray (UI-P3-M2-TRAY-OPT)
        │
        ▼
Bevy minimap chrome (MinimapGpuImageNode) — single presentation surface
```

**Forbidden:** second `MinimapOnlyExtract`; gameplay mutation from preview chrome; parallel LOD extract for M2 channels.

---

## Implementation sequence (landed)

| Phase | ID | Coder slice | Primary files | Witness |
|:---|:---|:---|:---|:---|
| 1 | **M2-01** | **UI-P3-M2-001** | `minimap_compositor/composite.rs`, `pass.rs`, WGSL | `logistics_rows > 0` |
| 2 | **M2-02/03** | **UI-P3-M2-CODER-A** / **UI-P3-M3-001** | `visual_domain_snapshots.rs`, compositor diagnostics | `construction_rows`, `ecology_rows`, `ui_p3_m3_green` |
| 3 | **M2-05** | (with M1/M2) | `minimap_shell.rs` — `MinimapOverlayMask` | mask bits in proof JSON |
| 4 | **M2-06** | **UI-P3-M2-TRAY-OPT** | `dock_shell.rs`, tray sync | `ui_p3_m2_tray_opt_green` |
| 5 | Rollup | **live_proof.rs** | `ui_p3_m2_minimap_acceptance_green` | `ui_p3_m2_green: true` |

### Data contract — `MinimapOverlayMask`

| Bit | Source snapshot | WGSL channel |
|:---|:---|:---|
| `fire_heat` | `SharedOverlayFieldBuffers` | M1 (existing) |
| `logistics_heat` | `LogisticsVisualSnapshot` | M2-01 |
| `construction_heat` | `CorridorConstructionBook` + site rows | M2-02 |
| `ecology_heat` | `EcologyVisualSnapshot` | M2-03 |

**Defaults:** [`simulation_minimap_overlay_defaults`](../../../src/gui/minimap_shell.rs) — all four **on** in sim (ecology default fixed 2026-05-25 for witness).

---

## Witness gates (code)

| Function | File | Pass when |
|:---|:---|:---|
| `ui_p3_m2_minimap_acceptance_green` | `live_proof.rs` | M1 ok + logistics rows + M2 channels + optional tray parity |
| `ui_p3_m3_minimap_acceptance_green` | `live_proof.rs` | construction + ecology enabled with rows |
| `ui_p3_m2_tray_opt_green` | `live_proof.rs` | tray mask == compositor uniform |
| `seed_minimap_m2_overlay_witness` | `visual_domain_snapshots.rs` | test harness populates construction + ecology |

**Rollup JSON:** `write_minimap_compositor_live_proof` → `debug_runs/minimap_compositor_live.json`

### Fleet snapshot (2026-05-25)

| Field | Value |
|:---|:---|
| `composite_ok` | `true` |
| `composite_path` | `GpuCompute` |
| `logistics_rows` | `2` |
| `construction_rows` | `18` |
| `ecology_rows` | `100` |
| `ui_p3_m2_green` | `true` |
| `ui_p3_m3_green` | `true` |
| `ui_p3_m2_tray_opt_green` | `true` |
| `ui_oh_m2_001.green` | `true` (logistics + construction channels) |
| `dual_minimap_present` | `false` |

---

## Slice copy-paste (archive — do not redo)

### UI-P3-M2-001 (logistics)

```
Lane: UI-P3-M2-001
Read: ui_phase3_minimap_m2_impl_plan_v1.md
      ui_phase3_minimap_compositor_plan_v1.md § M2
First: wire LogisticsVisualSnapshot into compositor row count + WGSL logistics channel
Do NOT: new extract; Hanabi
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor stage5
Witness: minimap_compositor_live.json logistics_rows > 0
```

### UI-P3-M2-CODER-A (construction + ecology)

```
Lane: UI-P3-M2-CODER-A (aka UI-P3-M3-001 in queue)
Read: docs/archive/2026-06-src-dev/plans/minimap_d_m2_signoff_v1.md M2-02/M2-03
First: seed_minimap_m2_overlay_witness + compositor uniform bits for construction/ecology
Max files: 3 — visual_domain_snapshots.rs, composite/pass, live_proof
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor
Witness: construction_rows > 0, ecology_rows > 0, ui_p3_m3_green: true
```

### UI-P3-M2-TRAY-OPT

```
Lane: UI-P3-M2-TRAY-OPT
Read: src/gui/hud/dock_shell.rs § UI-P3-M2-TRAY-OPT
First: overlay tray checkboxes → MapViewInstances minimap compositor mask
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor
Witness: ui_p3_m2_tray_opt_green: true
```

---

## Acceptance — M2 implementation CLOSED

| # | Criterion | Met |
|:---:|:---|:---:|
| I1 | M1 spine still green (`ui_p3_001_green`, `composite_ok`) | ☑ |
| I2 | All three strategic channels emit rows when seeded | ☑ |
| I3 | `ui_p3_m2_green` + `ui_p3_m3_green` in live JSON | ☑ |
| I4 | Tray bridge bidirectional | ☑ |
| I5 | `cargo test -p proc_A_dine01 --lib minimap_compositor stage5` | ☑ |
| I6 | `--test visual` with `MINIMAP_GPU_COMPOSITOR=1` refreshes proof | ☑ |

---

## Out of scope (M2 impl — forward)

| Item | Track | Plan |
|:---|:---|:---|
| Fog-of-war, EW, unit markers | Design **M3** | [`minimap_m3_operational_overlay_spec_v1.md`](minimap_m3_operational_overlay_spec_v1.md) · **UI-P3-M4-001** |
| Overlay legend PNG | Designer optional | **D-MINIMAP-M2-LEGEND** in overlay plan |
| CPU duplicate upload removal | Polish | `ux_gpu_minimap_design_v1.md` §8 |

---

## Regression (maintain M2)

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

**Stage 5 rule:** M2 edits must not break FULL_APP spine — fix witness only, not readiness predicates.

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-UI-P3-M2-IMPL-001 |
| Designer | 2026-05-24 | **D-MINIMAP-M2 SIGNED** |
| Coder | 2026-05-24–25 | M2 slices **DONE** |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | Link full plan — unblocks UI-P3-M2-CODER-A |
| v1.0.0 | 2026-05-25 | PLAN-UI-P3-M2-IMPL-001 — M2 implementation closure rollup |
