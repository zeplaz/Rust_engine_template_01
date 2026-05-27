# PLAN-VISUAL-RUN-GATE-001 — visual run acceptance matrix `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-VISUAL-RUN-GATE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** |
| **Blocker board** | [`visual_run_blockers.md`](visual_run_blockers.md) |
| **Compile hygiene** | [`COMPILE_WARNINGS_TODOS.md`](COMPILE_WARNINGS_TODOS.md) · **CW-50** |

**No Rust in this deliverable.** Defines what **`cargo run -p proc_A_dine01 --release -- --test visual`** must prove vs what **lib-only** or **STALE** witnesses satisfy.

---

## Rule (one line)

**`--test visual` is the operator gate for tactical VFX + FULL_APP JSON refresh.** Lib tests and seed witnesses are **necessary but not sufficient** for **VFX-VISUAL-SIGNOFF-001** and **UI-WP-VISUAL-001** product sign-off.

---

## Command

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

**Success:** process exit **0**, `debug_runs/stage5_full_app_live.json` written, `readiness.passes: true` (spine).

---

## Matrix — VR rows

| ID | Symptom | Blocks visual run? | Owner | Lib-only OK? | Exit when |
|:---|:---|:---:|:---|:---:|:---|
| **VR-01** | WGSL `inst` / tile debug panic | **Yes** (fixed) | @coder | No | Visual run completes inv 240+ |
| **VR-02** | Stack overrun after render panic | Secondary | — | — | VR-01 fixed |
| **VR-03** | rustc warnings | **No** | @coder hygiene | Partial | `cargo build` clean; **CW-50** |
| **VR-04** | VT-5 fail @ low `fire_inst` | **No** | @coder P2 | Warn-only policy | Documented in triage |
| **VR-05** | `fire_inst` flicker | **No** | sim + F1 | Lib F1 witness | `fire_ecology_live.json` |
| **VR-06** | Early exit / no JSON | **Yes** | @coder | No | Proof file exists |
| **VR-07** | Fire particle WGSL redef | **Yes** (fixed) | @coder | No | Fire raster loads |
| **VR-08** | Fire globals binding | **Yes** (fixed) | @coder | No | No wgpu panic |
| **VR-09** | Harness never writes JSON | **Yes** (fixed) | @coder | No | `instanced_dispatch_ok` path |
| **VR-10** | GPU surface teardown panic on exit | **P2** | **TRIAGE-VISUAL-TEARDOWN-001** | Lib teardown test | Clean exit code 0 |

**Active board:** only **VR-10** and operator discipline remain **open** for full sign-off lanes.

---

## Downstream lanes (who uses visual run)

| Lane ID | Agent | Requires `--test visual`? | Lib / qualified alternative |
|:---|:---|:---:|:---|
| **VFX-VISUAL-SIGNOFF-001** | @coder + operator | **Yes** for **product** close | Lib `tactical_vfx` witness — **STALE** for sign-off |
| **UI-WP-VISUAL-001** | @coder + operator | **Yes** for D-02/D-09 pixels | Lib `wave_p_live.json` layout greens — **qualified** |
| **LOG-E01-VISUAL-CONFIRM-001** | operator | **Yes** | [`log_e01_full_app_witness_spec_v1.md`](log_e01_full_app_witness_spec_v1.md) |
| **Stage 5 FULL_APP** | regression | **Yes** (primary) | Lib `cargo test stage5` — spine only |
| **FIRE7 / S7B** | maintain | **No** | Lib tests |

---

## Witness files touched by visual run

| File | Fields operators care about |
|:---|:---|
| `stage5_full_app_live.json` | `readiness.passes`, `fire_spark_rows`, `water_*`, `instanced_dispatch_ok`, `logistics_active_rows` |
| `minimap_compositor_live.json` | Often refreshed in same session — not VR-gated |
| `fire_ecology_live.json` | F1 — separate sim witness |
| `wave_p_live.json` | **Not** always rewritten — **UI-WP-VISUAL** needs visual or dedicated WP run |

---

## PASS gate — visual run (operator)

| # | Criterion | Evidence |
|:---:|:---|:---|
| VR-G1 | Process exit 0 | Terminal |
| VR-G2 | Proof JSON exists | `stage5_full_app_live.json` |
| VR-G3 | Readiness green | `readiness.passes: true` |
| VR-G4 | No shader panic mid-run | Log clean through inv 720+ |
| VR-G5 | Tactical VFX rows (sign-off lanes) | `fire_spark_rows > 0` at tactical zoom profile |
| VR-G6 | Teardown clean (VR-10) | No surface destroy panic on exit |

---

## PASS gate — lib-only (regression only)

| # | Criterion | Enough for |
|:---:|:---|:---|
| L1 | `cargo test -p proc_A_dine01 --lib stage5` | Spine regression |
| L2 | `refresh_tactical_vfx_stage5_live_witness` | **Not** VFX-VISUAL product sign-off |
| L3 | Strategic-zoom JSON with `fire_spark_rows: 0` | **Expected** — not failure |

---

## STALE policy

| Observation | Verdict | Action |
|:---|:---|:---|
| Disk JSON old timestamp, lib tests green | **STALE** | Re-run visual |
| `fire_spark_rows: 0` at strategic zoom only | **Not regression** if tactical run green | Document zoom in [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) |
| VT-5 single-frame fail | **WARN** | See VR-04 |

---

## Pickup order

1. **VR-10** / teardown — `TRIAGE-VISUAL-TEARDOWN-001`  
2. **VFX-VISUAL-SIGNOFF-001** — tactical visual run + optional PNG capture  
3. **UI-WP-VISUAL-001** — visual run or WP-specific harness refresh  
4. **LOG-E01-VISUAL-CONFIRM-001** — refresh `logistics_active_rows`  

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **PLAN-VISUAL-RUN-GATE-001** signed |
