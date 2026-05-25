# GPU minimap compositor M2 — `D-MINIMAP-M2` sign-off `v1`

| Field | Value |
|:---|:---|
| **Review ID** | **D-MINIMAP-M2** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **UI-P3-M2-001** / **UI-P3-M3-001** |
| **Status** | **SIGNED — M2 COMPLETE** (strategic overlays) |
| **Prerequisite** | [`minimap_d_m1_signoff_v1.md`](minimap_d_m1_signoff_v1.md) (**D-MINIMAP-M1**) |
| **Design** | [`ux_gpu_minimap_design_v1.md`](ux_gpu_minimap_design_v1.md) §7 M2 |
| **Plan** | [`ui_phase3_minimap_compositor_plan.md`](ui_phase3_minimap_compositor_plan.md) |
| **Coder queue** | [`../prompts/guides/ui/ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) §3.4 |
| **Witness** | [`debug_runs/minimap_compositor_live.json`](../debug_runs/minimap_compositor_live.json) |

---

## Executive summary

**M2** adds **strategic shell overlays** on the GPU minimap compositor: **logistics**, **construction**, and **ecology** heat channels — sampled from published snapshots only (no new extract).

**Verdict:** ☑ **SIGNED — M2 COMPLETE** (2026-05-24). One **deferred** polish item: HUD **overlay tray** → `MinimapOverlayMask` live sync (**UI-P3-M2-TRAY-OPT**).

**Naming note:** queue id **UI-P3-M3-001** in code = construction + ecology **M2** channels (`ui_p3_m3_green`); **M3** in design doc = fog/EW/units (still open).

---

## Signed decisions (M2 scope)

| ID | Decision | Choice | Evidence |
|:---|:---|:---:|:---|
| **M2-01** | Logistics heat | `LogisticsVisualSnapshot` → compositor | `logistics_rows: 2`, `logistics_heat_enabled: true` |
| **M2-02** | Construction phase | `CorridorConstructionBook` → `construction_heat` | `construction_rows: 18`, `construction_heat_enabled: true` |
| **M2-03** | Ecology macro band | `EcologyVisualSnapshot` → `ecology_heat` | `ecology_rows: 100`, `ecology_heat_enabled: true` |
| **M2-04** | No duplicate extract | Read projection / snapshots only | No `MinimapOnlyExtract` |
| **M2-05** | Overlay mask bits | `MinimapOverlayMask` in `minimap_shell.rs` | `logistics_heat`, `construction_heat`, `ecology_heat` |
| **M2-06** | Tray → mask sync | HUD overlay tray drives toggles live | **deferred** — defaults only today |

---

## Acceptance criteria

| # | Criterion | Witness / code | Met |
|:---:|:---|:---|:---:|
| 1 | Logistics rows when LOG seeded | `logistics_rows > 0` | ☑ (`2`) |
| 2 | Construction heat channel | `construction_heat_enabled` + rows | ☑ (`18`) |
| 3 | Ecology heat channel | `ecology_heat_enabled` + rows | ☑ (`100`) |
| 4 | M1 spine still green | `composite_ok`, `GpuCompute`, `ui_p3_001_green` | ☑ |
| 5 | M3 witness rollup (code id) | `ui_p3_m3_green: true` | ☑ |
| 6 | Lib tests green | `minimap_compositor` + `stage5` | ☑ |
| 7 | Overlay tray live bridge | tray toggles ↔ compositor uniforms | ☐ **deferred** |

**Witness excerpt (2026-05-24):**

| Field | Value |
|:---|:---|
| `composite_ok` | `true` |
| `logistics_rows` | `2` |
| `construction_rows` | `18` |
| `ecology_rows` | `100` |
| `ui_p3_001_green` | `true` |
| `ui_p3_m3_green` | `true` |

**Code anchors:** [`composite.rs`](../render/minimap_compositor/composite.rs) · [`pass.rs`](../render/minimap_compositor/pass.rs) · [`minimap_composite.wgsl`](../assets/shaders/minimap/minimap_composite.wgsl)

---

## Coder slices

| Slice | M2 item | Status |
|:---|:---|:---|
| **UI-P3-M2-001** | M2-01 logistics heat | **done** |
| **UI-P3-M3-001** | M2-02 / M2-03 construction + ecology | **done** |
| **UI-P3-M2-TRAY-OPT** | M2-06 overlay tray bridge | **optional** — queued |

---

## §11 Designer sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | M1 prerequisite **SIGNED** | ☑ |
| 2 | §4 layer stack M2 rows match design | ☑ |
| 3 | Witness `minimap_compositor_live.json` green | ☑ |
| 4 | No second fire/logistics extract | ☑ |
| 5 | Tray bridge gap documented (not silent) | ☑ |
| 6 | M3 fog/EW explicitly out of M2 | ☑ |

**Verdict:** ☑ **SIGNED — M2 COMPLETE**

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** |
| Coder | 2026-05-24 | **Done** — overlays in compositor + witness |

---

## Still open (M3 / polish)

| Item | Track |
|:---|:---|
| Fog-of-war, EW, unit markers | Design **M3** — Stage 7 alignment |
| Overlay tray → mask sync | **UI-P3-M2-TRAY-OPT** |
| Remove duplicate CPU upload hot path | `ux_gpu_minimap_design_v1.md` §8 step 3 |
| **D-MINIMAP-M3** sign-off | After M3 scope lands |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | **D-MINIMAP-M2** — strategic overlays SIGNED; tray bridge deferred |
