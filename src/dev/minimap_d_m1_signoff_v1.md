# GPU minimap compositor M1 — `D-MINIMAP-M1` sign-off `v1`

| Field | Value |
|:---|:---|
| **Review ID** | **D-MINIMAP-M1** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` → `@coder` **UI-P3-M1** / **UX-E01** |
| **Status** | **SIGNED** — **M1 done** |
| **Design** | [`ux_gpu_minimap_design_v1.md`](ux_gpu_minimap_design_v1.md) |
| **Architecture** | [`ux_gpu_minimap_m1_architecture_v1.md`](ux_gpu_minimap_m1_architecture_v1.md) |
| **Planner** | [`../prompts/guides/ui/ui_phase3_gpu_minimap_m1_planner_v1.md`](../prompts/guides/ui/ui_phase3_gpu_minimap_m1_planner_v1.md) |
| **Coder queue** | [`../prompts/guides/ui/ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) |
| **Witness** | [`debug_runs/minimap_compositor_live.json`](../debug_runs/minimap_compositor_live.json) |
| **Steward gate** | [`minimap_m1_gate_v1.md`](minimap_m1_gate_v1.md) — **S-M1 GO** 2026-05-24 |
| **Successor** | [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) (**D-MINIMAP-M2**) |

---

## Executive summary

**M1** moves simulation minimap pixels off egui CPU raster to a **GPU compositor** (`MinimapCompositorPass` → `SharedRenderTargetImage`). **No duplicate ECS extract.** egui/Bevy shell **displays** only.

**Verdict:** ☑ **SIGNED — M1 COMPLETE** (2026-05-24). **M2/M3** remain open per design §7.

---

## Signed decisions (M1 scope)

| ID | Decision | Choice | Evidence |
|:---|:---|:---:|:---|
| **M1-01** | Pixel authority | GPU compositor, not egui raster | `presentation_source: SharedRenderTargetImage` |
| **M1-02** | Extract rule | Read published frames only | No `MinimapOnlyExtract` |
| **M1-03** | RT ownership | Dedicated `MinimapRenderTargetRegistry` | Not aliased to World Preview RT |
| **M1-04** | Default path | GPU on; CPU opt-out | `MINIMAP_GPU_COMPOSITOR=0` fallback |
| **M1-05** | Fire overlay | Sample shared overlay field | `fire_heat_enabled: true`, `composite_ok: true` |
| **M1-06** | View isolation | Minimap never writes WorldMain camera | `infrastructure_view_isolation` green (related proof) |

**M2:** see [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) — overlays **SIGNED done**; tray bridge optional.

---

## Acceptance criteria (§9 design doc)

| # | Criterion | Witness / test | Met |
|:---:|:---|:---|:---:|
| 1 | GPU texture in sim; fire toggle | `composite_path: GpuCompute`, `fire_heat_enabled: true` | ☑ |
| 2 | No new ECS extract; lib green | `cargo test` minimap_compositor + stage5 | ☑ |
| 3 | View isolation | `infrastructure_view_isolation_live.json` (related) | ☑ |
| 4 | RT bound + revision | `rt_bound: true`, `compositor_revision: 1` | ☑ |
| 5 | Perf &lt; 0.5 ms median | Track in `perf_attribution_60s.md` | ◐ optional measure |

**Witness excerpt (2026-05-24):**

| Field | Value |
|:---|:---|
| `composite_ok` | `true` |
| `composite_path` | `GpuCompute` |
| `presentation_source` | `SharedRenderTargetImage` |
| `extent` | 260×220 |
| `ui_p3_001_green` | `true` |
| `logistics_rows` | `2` |
| `ecology_rows` | `100` |
| `construction_rows` | `18` |

---

## §11 Designer sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | North star: compositor spine, not egui raster | ☑ |
| 2 | Authority table (shell vs pixels) | ☑ |
| 3 | M1 scope bounded (M2/M3 explicit defer) | ☑ |
| 4 | Witness `minimap_compositor_live.json` green | ☑ |
| 5 | Default GPU flip documented | ☑ |
| 6 | Stage 5/6 gates not reopened | ☑ |

**Verdict:** ☑ **SIGNED — M1 COMPLETE**

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** |
| Coder | 2026-05-24 | **Done** — 3.1 landed per planner |

---

## Unblocks / still open

| Slice | Status |
|:---|:---|
| **UX-E01 / UI-P3-M1** | **done** |
| **D-MINIMAP-M2** | **done** — [`minimap_d_m2_signoff_v1.md`](minimap_d_m2_signoff_v1.md) |
| **UI-P3-M2-TRAY-OPT** | overlay tray → mask sync — optional |
| **D-MINIMAP-M3** | fog/EW/units — design M3, not started |

---

## Verification commands

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor stage5
$env:MINIMAP_GPU_COMPOSITOR = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | **D-MINIMAP-M1** sign-off; M1 witness green |
