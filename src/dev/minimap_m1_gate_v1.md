# S-M1 gate — GPU minimap M1 `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `S-M1` |
| **Date** | 2026-05-24 |
| **Owner** | `@sim-steward` gate → **`@coder` `UI-P3-001`** verify |
| **Sign-off** | [`minimap_d_m1_signoff_v1.md`](minimap_d_m1_signoff_v1.md) (**D-MINIMAP-M1**) |
| **Plan** | [`../prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md`](../prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md) |
| **Prereq** | **`UI-P2B-GATE`** PASS · **`UI-P3-PREFLIGHT`** GO ([`HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md)) |
| **Witness** | [`debug_runs/minimap_compositor_live.json`](../../debug_runs/minimap_compositor_live.json) |

**Scope:** M1 foundation only — GPU compositor spine, no duplicate extract, default GPU path. **Not** M2 tray polish, M3 fog/EW, or `ui_p3_m3_green`.

---

## Verdict: **GO**

M1 acceptance is **met** in witness + lib tests. Safe to treat **UI-P3-M1 / UX-E01 M1** as **closed** and route forward to **UI-P3-001** hardening (if any gap) or **UI-P3-M2-TRAY-OPT** optional.

---

## Gate matrix

### Prerequisite — `UI-P2B-GATE`

| Gate | Required | Verdict |
|:---|:---|:---:|
| G-2B-01 `phase2b_closed` | `true` | ✅ |
| G-2B-02 `egui_pass_count_in_sim` | `0` | ✅ |
| G-2B-03 `minimap_chrome_aligned` | `true` | ✅ |
| G-2B-04 GPU path | compositor `GpuCompute` | ✅ *qualified* |
| G-2B-05 `P3_minimap_texture` | `bevy_ui_gpu` | ✅ |
| G-2B-06 `dual_minimap_present` | `false` | ✅ |
| G-2B-07 minimap camera isolation | shell does not write world desired | ✅ |
| G-2B-08 FULL_APP | operational | ✅ |

### M1 — `D-MINIMAP-M1` (G-M1)

| Gate ID | Criterion | Evidence | Verdict |
|:---|:---|:---|:---:|
| **G-M1-01** | Pixel authority = GPU compositor | `presentation_source: SharedRenderTargetImage`, `composite_path: GpuCompute` | ✅ |
| **G-M1-02** | No `MinimapOnlyExtract` | grep `src/` — none | ✅ |
| **G-M1-03** | Dedicated `MinimapRenderTargetRegistry` | `minimap_and_preview_handles_differ_*` test | ✅ |
| **G-M1-04** | Default GPU; `MINIMAP_GPU_COMPOSITOR=0` opt-out | `gpu_compositor_env_default_on_when_unset` | ✅ |
| **G-M1-05** | Fire overlay samples shared field | `fire_heat_enabled: true`, `composite_ok: true` | ✅ |
| **G-M1-06** | View isolation | `minimap_shell_wrote_map_camera_desired: false` | ✅ |
| **G-M1-07** | RT bound + stamp | `rt_bound: true`, `stamp > 0` | ✅ |
| **G-M1-08** | `ui_p3_001_green` | `true` in live JSON | ✅ |
| **G-M1-09** | Lib tests | minimap_compositor **14/14** (`--test-threads=1`) | ✅ |
| **G-M1-10** | Shell phase2 | simulation_shell_phase2 **13/13** | ✅ |

### Out of M1 gate (do not fail S-M1)

| Item | Status | Track |
|:---|:---|:---|
| `logistics_rows` in witness | `2` (M2-related) | OK if ≥0 at proof; M1 does not require LOG scenario |
| `ecology_rows` / `construction_rows` | `0` in latest JSON | **M3** — `ui_p3_m3_green: false` expected |
| `extent_match_px` | `0.0` in witness | A5 for UI-P3-001; M1 sign-off allows ◐ perf |
| Operator PNG review | N/A for M1 | Designer M1 checklist already **SIGNED** |

---

## Witness snapshot (2026-05-24)

`debug_runs/minimap_compositor_live.json`:

| Field | Value |
|:---|:---|
| `composite_ok` | `true` |
| `composite_path` | `GpuCompute` |
| `dual_minimap_present` | `false` |
| `gpu_compositor_env` | `true` |
| `ui_p3_001_green` | `true` |
| `logistics_rows` | `2` |
| `extent` | 128×128 |

---

## Forbidden-pattern scan

| Pattern | Result |
|:---|:---|
| `MinimapOnlyExtract` in `src/` | **none** |
| Compositor ECS fire `Query` (duplicate extract) | **none** (reads `SharedOverlayFieldBuffers`) |

---

## Qualified (non-blocking)

| ID | Issue | Mitigation |
|:---|:---|:---|
| **SM1-Q1** | `minimap_compositor` tests race on `MINIMAP_GPU_COMPOSITOR` under default parallel harness | CI/local: `--test-threads=1` for minimap_compositor module |
| **SM1-Q2** | Shell `phase2.minimap_gpu_path: false` at single proof frame | Qualified in UI-P2B-GATE; compositor witness is authoritative |
| **SM1-Q3** | `gpu_budget.justified: false` at proof frame | Cadence / no terrain in headless proof — not M1 blocker |

---

## Verification commands

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor -- --test-threads=1
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 stage5 map_view
Remove-Item Env:MINIMAP_GPU_COMPOSITOR -ErrorAction SilentlyContinue
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Route after **GO**

| Next | Agent | Notes |
|:---|:---|:---|
| **UI-P3-001** | `@coder` | Verify/harden if not already green; witness `ui_p3_001_green` already true |
| **UI-P3-M2-TRAY-OPT** | `@coder` | Optional — overlay tray → mask |
| **D-MINIMAP-M3** | design + coder | Fog/EW/units — not started |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | S-M1 gate steward pass — **GO** |
