# UI Phase 3 — coder execution queue `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.1.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@coder` / `ui_layout_agent` |
| **Planner (M1)** | [`ui_phase3_gpu_minimap_m1_planner_v1.md`](ui_phase3_gpu_minimap_m1_planner_v1.md) |
| **Design** | [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) |
| **Phase 2 queue** | [`ui_phase2_coder_queue_v1.md`](ui_phase2_coder_queue_v1.md) *(close first)* |
| **Product board** | [`post_stage6_active_todos.md`](../../../src/dev/post_stage6_active_todos.md) |

**Rule:** One primary lane per session. Phase 2 is **CLOSED** — active work is M2+ and product lanes.

---

## Status snapshot (2026-05-24)

### Phase 2 UI shell — **CLOSED · SIGNED**

| Area | Status |
|:---|:---|
| 2A / 2B witnesses | ✅ `phase2a_closed` + `phase2b_closed` |
| Designer sign-off | ✅ v2.1.1 **SIGNED** |
| `--test visual` | ✅ 2026-05-24 |
| Witness tail | ☐ optional — `ops_zone_hover_token`, `build_rail_authoritative` |

**Archive:** [`ui_phase2_sprint_queue.md`](../../../src/dev/ui_phase2_sprint_queue.md)

### Phase 3 UX-E01 — **M1/M1.5/M2 CLOSED**

| Task | Status | Evidence |
|:---|:---|:---|
| 3.1 M1 foundation | ✅ | `minimap_compositor_live.json` · `composite_ok` |
| 3.2 witness closure | ✅ | cross-witnesses refreshed 2026-05-24 |
| 3.3 GpuCompute compositor | ✅ | `composite_path: GpuCompute` in live JSON |
| 3.5 default flip | ✅ | `MINIMAP_GPU_COMPOSITOR` unset → GPU on; `=0` CPU opt-out |
| 3.4 M2 logistics | ✅ **done** | `logistics_rows: 2` in witness |
| Lib tests | ✅ | `minimap_compositor` + `stage5` |

### Parallel product lanes

| Lane | Status | Coder entry |
|:---|:---|:---|
| **IND-E01** industrial chain | ☐ | [`industrial_activation_pipeline.md`](../../../src/dev/industrial_activation_pipeline.md) |
| **LOG-E01** | ✅ code | Operator `--test visual` |
| **CON-E01 P9** | ☐ verify | `construction_p9_todos.rs` |

---

## Priority order (pick one primary)

```text
P1  Phase 3 task 3.4 (M2 logistics heat)     → UI-P3-M2-001
P2  IND-E01 industrial chain                 → parallel, disjoint files
P3  Phase 2 witness tail (optional)          → UI-P2A-F03 / UI-P2A-P4-AUTH
P4  Phase 3 M3 / Phase 4 art                 → after M2 witness
```

---

## @coder — copy-paste starters

### Lane A — M2 logistics (primary)

```
Lane: UI Phase 3 — UI-P3-M2-001 (M2 logistics heat)
Read: ui_phase3_coder_queue_v1.md §3.4 + ux_gpu_minimap_design_v1.md §4 M2
First: extend MinimapOverlayMask + compositor uniforms for logistics_heat
Exit: minimap_compositor_live.json logistics_rows > 0
```

### Lane B — IND-E01 (parallel)

```
Lane: Industrial activation E2E
Read: src/dev/industrial_activation_pipeline.md
First: place concrete chain in sim → industrial_activation_live.json
Do NOT: touch minimap_compositor unless logistics slice needs same session
```

---

## Phase 3 task 3.4 — M2 logistics layer (**DONE** — see [`minimap_d_m2_signoff_v1.md`](../../../src/dev/minimap_d_m2_signoff_v1.md))

**Goal:** First M2 overlay — logistics heat strip (see [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) §4 M2).

**Prerequisite:** LOG-E01 `log_rows≥1` in visual run ([`logistics_visual_lane_spec_v1.md`](../../../src/dev/logistics_visual_lane_spec_v1.md)).

**Witness gap:** `minimap_compositor_live.json` → `logistics_rows: 0`.

| Step | Task | Files |
|:---:|:---|:---|
| 3.4.1 | Extend `MinimapOverlayMask` with `logistics_heat: bool` | `minimap_shell.rs`, compositor uniforms |
| 3.4.2 | Sample `LogisticsVisualSnapshot` / projection `log_rows` | `gpu_compute.rs`, `composite.rs` |
| 3.4.3 | Witness `logistics_rows` + `logistics_layer_enabled` | `live_proof.rs` |

**Accept:** `logistics_rows > 0` in live JSON when transport scenario seeded; `cargo test -p proc_A_dine01 --lib minimap_compositor stage5` green.

**Defer:** construction phase channel, ecology macro band, fog-of-war (M2/M3).

---

## Phase 3 tasks 3.2 / 3.3 / 3.5 — **CLOSED** (2026-05-24)

<details>
<summary>Archive — M1 witness + GpuCompute + default flip</summary>

- 3.2 witness bundle — `minimap_compositor_live.json` + `stage5_full_app_live.json` refreshed
- 3.3 GpuCompute — `composite_path: GpuCompute` (not CPU bridge)
- 3.5 default flip — `minimap_gpu_compositor_env_enabled()` defaults **on**; `MINIMAP_GPU_COMPOSITOR=0` opt-out

</details>

---

## Phase 2 — **CLOSED**

See [`ui_overhaul_plan.md`](../../../src/dev/ui_overhaul_plan.md) · optional tail: **UI-P2A-F03**, **UI-P2A-P4-AUTH**.

---

## Regression commands (every slice)

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib minimap_compositor
cargo test -p proc_A_dine01 --lib simulation_shell_phase2
cargo run -p proc_A_dine01 -- --test demo
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Agent routing

| Situation | Delegate |
|:---|:---|
| M2 logistics heat | `@coder` **UI-P3-M2-001** |
| Industrial chain | `@coder` **IND-E01** (parallel) |
| Witness tail (hover / rail) | `@coder` UI-P2A-F03 / P4-AUTH (optional) |
| VT-4/VT-5 regression | `@sim-steward` before shell edits |
| M3 overlay UX | `@designer` then `@coder` |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.1.0 | 2026-05-24 | Phase 2 closed; M1/M1.5 closed; M2 active; queue realigned |
| 1.0.0 | 2026-05-23 | Initial queue |
