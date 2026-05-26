# UI Phase 3 — GPU minimap compositor full plan `v1` (PLAN-UI-P3-COMPOSITOR-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-UI-P3-COMPOSITOR-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — M1 **CLOSED** · M2 **CLOSED** · M3 **PARTIAL** (FoW/EW done) |
| **Spine (M1)** | [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) v2 |
| **M2 impl** | [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) |
| **M3 spec** | [`minimap_m3_operational_overlay_spec_v1.md`](minimap_m3_operational_overlay_spec_v1.md) |
| **Naming** | [`ui_phase3_minimap_track_naming_v1.md`](ui_phase3_minimap_track_naming_v1.md) |
| **Witness** | [`debug_runs/minimap_compositor_live.json`](../../../debug_runs/minimap_compositor_live.json) |

**No Rust.** Three-track rollup for the GPU minimap compositor — **M1 spine**, **M2 strategic heat**, **M3 operational overlays**. M2 has a dedicated impl plan; this doc fills the **M1 + M3** planner gap.

---

## Track map (design §7)

| Design phase | Queue / review | Impl plan | Compositor status |
|:---|:---|:---|:---|
| **M1** — GPU spine | **UI-P3-001** · **D-MINIMAP-M1** | [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) § M1 | **CLOSED** |
| **M2** — strategic heat | **UI-P3-M2-001**, **UI-P3-M3-001** (code id) | [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) | **CLOSED** |
| **M3** — operational | **UI-P3-M4-001** · **D-MINIMAP-M3** | **this doc § M3** | **PARTIAL** — FoW/EW **DONE**; units/replay **OPEN** |

**Critical naming:** `UI-P3-M3-001` = **M2** construction/ecology — **not** design M3. Design M3 = **`UI-P3-M4-001`**.

---

## Master gate chain

```text
UI-P2B-GATE (egui retired in sim)              ☑
        │
        ▼
UI-P3-PREFLIGHT + S-M1                         ☑ GO 2026-05-24
        │
        ▼
M1  UI-P3-001 — GpuCompute spine               ☑ CLOSED
        │
        ▼
M2  UI-P3-M2-001 / UI-P3-M3-001 / TRAY-OPT     ☑ CLOSED (see M2 impl plan)
        │
        ▼
M3  UI-P3-M4-001 — FoW + EW                     ☑ CLOSED (ui_p3_m4_green)
M3  UI-P3-M3-UNITS-001 / REPLAY-001            ☐ OPEN (optional tails)
```

---

## M1 — GPU compositor spine (**CLOSED**)

**Authority:** [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) — architecture, forbidden extract, schedule, **UI-P3-001** handoff.

### M1 deliverables (landed)

| ID | Goal | Status |
|:---|:---|:---:|
| **UI-P3-PREFLIGHT** | Steward GO — no duplicate extract | **DONE** |
| **S-M1** / **minimap_m1_gate_v1** | M1 gate | **GO** |
| **UI-P3-001** | `MinimapCompositorPlugin` operational default | **DONE** |
| **D-MINIMAP-M1** | Designer sign-off | **SIGNED** |

### M1 witness (fleet truth)

| Field | Value |
|:---|:---|
| `composite_ok` | `true` |
| `composite_path` | `GpuCompute` |
| `ui_p3_001_green` | `true` |
| `dual_minimap_present` | `false` |
| `fire_heat_enabled` | per mask |
| `rt_bound` | `true` |

### M1 regression

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5 simulation_shell_phase2
```

**Do not reopen M1** for M2/M3 channel work — extend compositor pass only.

---

## M2 — strategic overlays (**CLOSED** — separate impl plan)

**Do not duplicate M2 here.** Full gate chain, coder A lanes, and witness fields:

→ [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) (**PLAN-UI-P3-M2-IMPL-001**)

| Channel | Witness |
|:---|:---|
| Logistics | `logistics_rows > 0`, `ui_p3_m2_green` |
| Construction + ecology | `construction_rows`, `ecology_rows`, `ui_p3_m3_green` |
| Tray | `ui_p3_m2_tray_opt_green` |

---

## M3 — operational overlays (**PARTIAL**)

**Design:** [`minimap_d_m3_signoff_v1.md`](../../../src/dev/minimap_d_m3_signoff_v1.md) · [`minimap_m3_operational_overlay_spec_v1.md`](minimap_m3_operational_overlay_spec_v1.md) **SIGNED**.

**North star:** Fog-of-war veil, EW stress, unit aggregation glyphs, replay scrub ticks — **on top of** M1+M2 composite — still **no** `MinimapOnlyExtract`.

### M3 layer stack (WGSL order)

```text
terrain / fallback
  → fire (M1)
  → logistics / construction / ecology (M2)
  → fog-of-war veil (M3-01)      ☑ UI-P3-M4-001
  → EW stress (M3-02)             ☑ UI-P3-M4-001
  → unit aggregation (M3-03)    ☐ UI-P3-M3-UNITS-001
  → replay scrub ticks (M3-04)    ☐ UI-P3-M3-REPLAY-001
```

### M3 data contract

| Channel | Snapshot / source | Mask bit | Witness |
|:---|:---|:---|:---|
| FoW | `MinimapOperationalSnapshot` | `fow` | `fow_enabled`, rows in compositor state |
| EW | same + `ew_tex` upload | `ew` | `ew_overlay_enabled` |
| Units | unit aggregation snapshot (Stage 7 LOD) | `units` | `unit_marker_rows` |
| Replay | replay timeline resource | `replay_scrub` | `replay_scrub_enabled` |

**Seed (tests):** `seed_minimap_m3_fow_ew_witness` — **not** design M2 `seed_minimap_m2_overlay_witness`.

### M3 slices

| ID | Scope | Status | Witness |
|:---|:---|:---:|:---|
| **MINIMAP-DESIGN-M3-001** | Spec **SIGNED** | **DONE** | spec v0.1.1 |
| **UI-P3-M4-001** | M3-01 FoW + M3-02 EW | **DONE** | `ui_p3_m4_green: true` |
| **UI-P3-M3-UNITS-001** | M3-03 unit glyphs | **OPEN** | `unit_marker_rows` |
| **UI-P3-M3-REPLAY-001** | M3-04 replay scrub | **OPEN** | `replay_scrub_enabled` |
| **UI-P3-M2-TRAY-OPT** | Extend tray for M3 bits | **OPTIONAL** | inherits M2 tray |

### M3 witness (fleet truth 2026-05-25)

| Field | Value |
|:---|:---|
| `ui_p3_m4_green` | `true` |
| `ui_p3_m2_green` | `true` (M2 rollup still required) |
| `ui_p3_001_green` | `true` |

### M3 acceptance — track **COMPLETE**

| # | Criterion |
|:---:|:---|
| M3-1 | FoW + EW green (**UI-P3-M4-001**) — **met** |
| M3-2 | Units or documented empty fixture — **open** |
| M3-3 | Replay scrub when timeline active — **open** |
| M3-4 | M1+M2 regression — **met** |
| M3-5 | No new ECS extract — **policy** |

### Copy-paste — UI-P3-M4-001 (archive — done)

```
Lane: UI-P3-M4-001 — design M3 FoW + EW (NOT UI-P3-M3-001)
Read: prompts/guides/ui/ui_phase3_minimap_compositor_full_plan_v1.md § M3
      prompts/guides/ui/minimap_m3_operational_overlay_spec_v1.md
First: MinimapOperationalSnapshot + fill_operational_heat_layers in composite.rs
Do NOT: MinimapOnlyExtract; confuse with UI-P3-M3-001 (M2 ecology)
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor
Witness: ui_p3_m4_green: true
```

### Copy-paste — UI-P3-M3-UNITS-001 (@coder)

```
Lane: UI-P3-M3-UNITS-001 — M3-03 unit aggregation markers
Read: ui_phase3_minimap_compositor_full_plan_v1.md § M3
      src/dev/minimap_d_m3_signoff_v1.md M3-03
Prereq: ui_p3_m4_green true (FoW/EW landed)
First: extend MinimapOverlayMask + compositor uniforms; cap 8 markers / extent
Max files: 3 — composite.rs, live_proof.rs, snapshot reader
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor stage5
Witness: minimap_compositor_live.json → unit_marker_rows
```

---

## Unified witness bundle

| File | Tracks |
|:---|:---|
| `minimap_compositor_live.json` | M1 + M2 + M3 fields |
| `ui_shell_migration_live.json` | `minimap_gpu_path`, chrome alignment |
| `infrastructure_view_isolation_live.json` | minimap camera isolation |
| `stage5_full_app_live.json` | FULL_APP rollup |

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Forbidden (all tracks)

| Pattern | Applies |
|:---|:---|
| `MinimapOnlyExtract` | M1–M3 |
| Compositor `Query<` fire entities | M1–M3 |
| `RenderProjectionGraph::evaluate` inside compositor | M1–M3 |
| Shell writes `SharedOverlayFieldBuffers` | M1–M3 |
| egui minimap + GPU `ImageNode` same frame | M1 (`dual_minimap_present`) |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-UI-P3-COMPOSITOR-001 |
| Designer M1 | 2026-05-24 | **D-MINIMAP-M1 SIGNED** |
| Designer M2 | 2026-05-24 | **D-MINIMAP-M2 SIGNED** |
| Designer M3 | 2026-05-25 | **D-MINIMAP-M3 SIGNED** (impl partial) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | M1/M3 compositor plan; M2 points to impl full plan |
