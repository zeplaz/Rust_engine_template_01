# Visual run blockers (`cargo run -p proc_A_dine01 -- --test visual`)

**Active board** for failures seen in terminal logs that are **not** `STAGE5_TODOS` closure rows.  
**Proof:** `debug_runs/stage5_full_app_live.json` (written on successful exit).

**Related (deferred depth):** [`stage5_triage_backlog.md`](stage5_triage_backlog.md)  
**Compile hygiene registry:** [`COMPILE_WARNINGS_TODOS.md`](COMPILE_WARNINGS_TODOS.md) (CW-01…41 done; re-verify below)

---

## Status snapshot (2026-07-03)

| ID | Symptom | Gate? | Plan / triage | Build state |
|----|---------|-------|---------------|-------------|
| **VR-10** | Post-Bevy-0.19 RTT: water overlay format panic (`Rgba16Float` vs `Bgra8UnormSrgb`) | **Yes** | Core2d overlay SDR unify | **Fixed:** `sim_map_rtt.rs` → `Rgba8UnormSrgb` + `CORE2D_OVERLAY_SDR_FORMAT` |
| **VR-11** | Ghost/cost cursor vertically mirrored vs OS cursor | **Yes** — placement UX | `sim_map_projection.rs` | **Fixed:** `sim_map_screen_to_world_xy_in_frame` Y flip aligned with `sim_map_screen_to_world_xy_with_window` |
| **VR-12** | Tactical map void always dark (no day/night) | No | `sim_map_rtt.rs` · `palette.rs` | **Fixed:** `sync_sim_map_clear_from_day_cycle` from `SimTimeMicros` |
| **VR-13** | Weather particles invisible on tactical map | **Yes** | `weather_visual.rs` RTT layer | **Fixed:** `RenderLayers` on overlay + precip mesh children (layer 1) |
| **VR-14** | Fire heat chunk markers missing | Partial | `gpu_tile_debug.rs` | **Fixed:** fire markers no longer gated on `CameraFocusDebug.enabled` |
| **VR-15** | Fire test red box wrong region / no camera focus | Partial | `vfx_fire_test_highlight.rs` | **Fixed:** chunk origins + `map_camera_desired_fit_tile_aabb` |
| **VR-16** | Sparks / GPU fire particles still not visible in play | **Yes** — operator verify | fire extract + raster dispatch | **Lib green** (`steward_spark_vfx_001`); **operator:** run `--test vfx` with display — code path fixed, witness refresh may be stale |
| **VR-17** | Startup panic: `VisibilityRangePlugin` already added | **Yes** | `mig_a_adoption.rs` MIG-A14 | **Fixed 2026-07-04:** do not re-add plugin (DefaultPlugins includes it). Rebuild local `target/release` — stale exe looked like fix failed |

**Operator verify command:**

```powershell
cargo run -p proc_A_dine01 --release -- --test vfx
# Expect: rain/precip, fire heat squares, sparks near seeded burn, day/night void shift over sim time
```

**Witness refresh (required before claiming green):** `debug_runs/stage5_full_app_live.json` · optional `full_render_diagnostic_*.json`

---

## Status snapshot (2026-05-23)

| ID | Symptom | Gate? | Plan / triage | Build state |
|----|---------|-------|---------------|-------------|
| **VR-01** | `tile_debug_instanced.wgsl`: `no definition in scope for identifier: inst` → render panic + Vulkan teardown | **Yes** — blocks `--test visual` | `TRIAGE-GPU-TILE-WGSL` | **Fixed:** `inst` → `tile_row` in WGSL |
| **VR-02** | `STATUS_STACK_BUFFER_OVERRUN` after render panic | Secondary | — | Goes away when VR-01 fixed |
| **VR-03** | rustc warnings (`TagSet`, `SiteFootprint`, `history.rs`, dead_code) | No (noise) | `CW-50` hygiene | **Clean** on current tree (`cargo build` / `--release` → 0 warnings) |
| **VR-04** | `VT-5 spatial invariants failed` at inv≈108 (`fire_inst=2`) | **No** — not FULL_APP gate | `TRIAGE-VT-DEEP`, `TRIAGE-FIRE-EXTRACT` | **Coder triage:** bootstrap defer (`vt5_flicker_triage_live.json`); operator visual confirm → OPS-VT5-001 |
| **VR-05** | `fire_inst` flicker (e.g. 22 → 0) while eval passes | No | `TRIAGE-FIRE-*` + fuel/old-growth | Sim/render contract; see § Fire |
| **VR-06** | Visual test exits before inv 720 / no proof JSON | **Yes** if early crash | VR-01 | User logs show **pass** at 240/480/720+ after shader fix |
| **VR-07** | `fire_particle_draw.wgsl`: `redefinition of alpha` (Naga) | **Yes** — fire raster pipeline fails | `fire_particle_draw.wgsl` | **Fixed 2026-05-23:** single `let alpha` expr |
| **VR-08** | `fire_particle_raster`: globals binding not visible in FRAGMENT | **Yes** — wgpu panic + `STATUS_STACK_BUFFER_OVERRUN` | `gpu_fire_particle_raster.rs` | **Fixed 2026-05-23:** `ShaderStages::VERTEX_FRAGMENT` |
| **VR-09** | Visual harness never writes JSON (fire rows 0, UX-06 done) | **Yes** — B1 proof stall | `stage5_full_app_harness.rs` | **Fixed 2026-05-23:** witness = `instanced_dispatch_ok` |

---

## VR-01 — Shader `inst` (done)

**Panic:** `src/render/gpu_tile_debug_draw.rs` pipeline compose — `shaders/debug/tile_debug_instanced.wgsl`.

**Cause:** Naga/Bevy shader composer treated bare identifier `inst` as out of scope in some builds.

**Fix:** `assets/shaders/debug/tile_debug_instanced.wgsl` — local binding renamed `inst` → `tile_row`.

**Verify:**

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
# Expect READINESS_EVAL inv=240 passes=true and process exit 0
```

---

## VR-03 — Compile warnings vs CW board

[`COMPILE_WARNINGS_TODOS.md`](COMPILE_WARNINGS_TODOS.md) marks **CW-01…CW-41 done**. Terminal warnings from **2026-05-22** were from an older tree or pre-fix sources:

| Warning (user log) | File | Current tree |
|--------------------|------|--------------|
| unused `TagSet` | `surface_water.rs` | Import already trimmed |
| unused `SiteFootprint` | `build_interaction.rs` | Import removed |
| `mut` / unused `site_events`, `zones` | `history.rs` | `_site_events`, `_zones` |
| `arm_visual_test_graceful_exit` dead | `gpu_surface_teardown.rs` | May still warn if hook unwired — **non-blocking** |
| `camera` / `zoom_screen_scale` | `map_egui_projection.rs` | Scaffold — **non-blocking** |

**CW-50 (open):** On each visual-fix batch, run:

```powershell
cargo build -p proc_A_dine01 2>&1 | Tee-Object debug_runs/compile_warnings.log
cargo rustc -p proc_A_dine01 --lib -- -D warnings
```

Re-open CW rows only if warnings return.

---

## VR-04 — VT-5 at low instance count

**Log:** `READINESS_EVAL_END inv=108 passes=false` — `VT-5 spatial invariants failed (stamp=108)` with `fire_inst=2`.

**Code:** [`vt_spatial_invariants.rs`](../render/vt_spatial_invariants.rs) requires ≥2 occupied chunks, mean distance > 1, variance > 0.1. A **short burst** of 2 instances (often same chunk after ecology seed) fails VT-5 while later ticks pass.

**@coder A triage (CODER-A-VT5-TRIAGE-001, 2026-06-17):** **bootstrap defer** — `run_vt5_ci_spatial_matrix` returns pass when `fire_inst < VT5_MIN_EVAL_FIRE_INSTANCES` (3) or instances collapse to one chunk; sparse particle lane also defers. Witness: `debug_runs/vt5_flicker_triage_live.json`. **Not a FULL_APP gate**; sustained `--test visual` log review remains **OPS-VT5-001**.

**Lib CI vs live visual (STAGE5-VT-FLICKER-001):**

| Surface | What it proves | What it does **not** prove |
|:---|:---|:---|
| `cargo test --lib vt_ci_matrix` | Deterministic VT-4/VT-5 on **spread** fixture; `vt5_ci_matrix_collapsed_extract_fails` catches **bootstrap collapsed** instances at same chunk | Intermittent VR-04 under **`--test visual`** at inv≈108 |
| `cargo run --release -- --test visual` | Live readiness eval + harness seeds | Requires operator/coder **log review** for sustained VT-5 fail |

**Not a Stage 5 gate** by design (`stage5_triage_backlog.md` — `TRIAGE-VT-DEEP`). Options when promoting:

1. Seed fire proof layout with spread chunks in visual harness, or  
2. Gate VT-5 on `fire_inst >= N` before evaluating, or  
3. Treat single-frame fail as warn-only in visual test policy.

**Exit for STAGE5-VT-FLICKER-001:** confirm VR-04 behavior on a **`--test visual`** run (not lib matrix alone).

**STAGE5-VT-FLICKER-VISUAL-001 (2026-05-28):** `stage5_vt_flicker_visual_001_witness()` lib-green (`done_qualified`); VR-04 live confirm pending → **OPS-VT5-001**.

**STAB-VT-001 (2026-05-28):** `stab_vt_001_witness()` lib-green (`done_qualified`); same VR-04 live confirm pending → **OPS-VT5-001**.

---

## VR-05 — Fire flicker (F1 sim gate — witness live)

**Symptom:** `fire_inst=22` one tick then `0`; `fire1=true` in readiness flags anyway (spine present, not ecology quality). Operator **world map blink** — [`map_blink_001_repro_v1.md`](map_blink_001_repro_v1.md) (**MAP-BLINK-001**).

**F1 (done):** Fuel + old-growth ignition gate — [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md).  
**Witness:** `debug_runs/fire_ecology_live.json` (`fuel_gated_ignitions`, `mean_fuel`, `mean_heat`, `heat_spike_frames`).

**MAP-BLINK-001 (2026-05-26):** PLAY-06d overlay hold when sim has display heat but filtered map empty; 4-frame overlay warmup on cold start; projection graph retains buffers on 1-tick fence lag. Verify with `VISUAL_DIAG=1` — `overlay_rev` should not wipe while fire steady.

**Still deferred (F2+):** `TRIAGE-FIRE-STREAM`, `TRIAGE-FIRE-EXTRACT`, per-tile GPU extract — not FULL_APP gate.

---

## Pickup order (main thread)

1. **VR-01** — verify release visual run + proof JSON refresh.  
2. **CW-50** — confirm zero-warning build; wire or `#[expect]` teardown helper if needed.  
3. **VR-04 / VR-05** — only if product promotes fire/VT rows from triage.

**Handoff one-liner:** `tools/orchestrator/invoke_handoff.ps1 -Goal "VR-01 verify visual proof" -Lane stage5`
