# S-VFX triage board `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `S-VFX` |
| **Date** | 2026-05-24 |
| **Owner** | `@sim-steward` triage → `@coder` ×2 / `@designer` captures |
| **Tracks** | [`stages/vfx_phase2_closure_plan_v1.md`](stages/vfx_phase2_closure_plan_v1.md) (`VFX-P2`) · [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) (`FX-WATER`) |
| **Queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **Witness** | [`debug_runs/stage5_full_app_live.json`](../../debug_runs/stage5_full_app_live.json) |
| **Reviews** | [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) (**D-VFX**) · [`water_vfx_review_record_v1.md`](water_vfx_review_record_v1.md) |

**Rule:** Witness JSON at **tactical** zoom (`--test visual` / harness) is **not** the same problem as **normal Simulation** presentation (map-wide tint, default zoom). Triage both columns below.

---

## Executive verdict

| Track | Harness / CI | Operator sim | Track **CLOSED**? |
|:---|:---|:---|:---:|
| **VFX-P2** (fire + shared proof) | ☑ **GREEN** — `tactical_vfx_witness.all_green: true`, `fire_spark_rows: 308`, tests 18+25 pass | ⚠ **OPEN** — full-map pink wash / non-pinpoint read reported; verify **not** VfxSandbox; `fire_heat` off on sim enter | ☐ |
| **FX-WATER** | ☑ W1/W2 first pass; tactical particles **76–96** rows | ⚠ River vs lake read; **ocean/foam = 0** in witness | ☐ |
| **D-VFX / WATER-DESIGN** | Prerequisites met | ☑ **SIGNED — TUNE ROUND**; PNG captures **PENDING** | ☐ |

**Primary lane after triage:** **P0 operator fire read** (presentation defaults + zoomed-out overlay) **in parallel with** **WATER-W1-OCEAN-001** + **WATER-W2-FOAM-001** (disjoint files).

---

## Witness snapshot (2026-05-24)

Source: `debug_runs/stage5_full_app_live.json` (`--test visual`, tactical harness).

| Field | Value | Triage |
|:---|:---|:---|
| `fire_spark_zoom_alpha` | **0.85** | Tactical proof band OK (D-F09) |
| `fire_spark_rows` | **308** | Sparks emit under harness — not a “rows=0” bug |
| `fire_sparks_above_smoke` | **true** | D-F10 witness green |
| `fire_instance_buffer_rows` | **0** | Projection on **overlay_bootstrap** — F-T02 follow-up |
| `fire_spark_projection_view` | `overlay_bootstrap` | Non-blocking for P2; blocks “graph-native” fire |
| `water_particle_rows` | **76** | Tactical W2 OK |
| `water_particle_river_foam` | **0** | **W-T03** → WATER-W2-FOAM-001 |
| `water_particle_coast_foam` | **0** | **W-T04** → WATER-W2-FOAM-001 |
| `water_ocean_tiles` | **0** | **W-T02** → WATER-W1-OCEAN-001 + fixture |
| `tactical_vfx_witness.all_green` | **true** | P2-VFX-VISUAL-001 met |

```powershell
cargo test -p proc_A_dine01 --lib gpu_particles
cargo test -p proc_A_dine01 --lib stage5
# optional refresh:
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## P0 — Operator / product (not Stage 5 gate)

| ID | Symptom | Likely cause | Owner | Fix slice |
|:---|:---|:---|:---|:---|
| **VX-P0-01** | Full-map pink/red fire wash in **Simulation** | CPU `chunk_fire_heat` overlay + tile tint when `fire_heat` on; zoomed-out `fire_boost`; VfxSandbox seeds 28 fires | `@coder` | Verify `simulation_session` defaults (`fire_heat: false`); operator zoom **in** for sparks; diagnostics toggle |
| **VX-P0-02** | “No pinpoint sparks” at default zoom | D-F09 **strategic cull** — sparks intentional at low `zoom_alpha` | `@operator` | Zoom to tactical (~40–70% map); compare vs `fire_spark_target_v1.png` |
| **VX-P0-03** | Background rain/snow missing when zoomed out | Was `zoom_t > 0.45` gate — `background_aesthetic` path | `@coder` | Confirm `weather_visual.rs` + diagnostics toggle |
| **VX-P0-04** | Tactical PNGs missing | Blocks **ACCEPTED** not coder queue | `@operator` / `@designer` | Save under `assets/vfx/reference/review_captures/` |

**Do not** disable strategic cull globally to green witness — breaks D-F09 / D-W09.

---

## P1 — Coder queue (closure work)

| Priority | ID | Status | Agent | Exit |
|:---:|:---|:---|:---|:---|
| 1 | **WATER-W1-OCEAN-001** | queued | Coder A | `water_ocean_tiles > 0` in witness |
| 1 | **WATER-W2-FOAM-001** | queued | Coder A | `river_foam` / `coast_foam` > 0 |
| 2 | **P2-FIRE-SPARK-011** | queued | Coder A | F-T01/T03 shower read vs mock |
| 2 | **P2-FIRE-SPARK-010** | queued | Coder A | Re-audit smoke pass if operator sees sparks under smoke |
| 2 | **P2-VFX-WITNESS-001** | **partial** | Coder B | Lib tests exist; mark done when harness W-3 documented |
| 3 | **P2-WATER-POLISH-001** | queued | Coder A | River ribbon read at strategic |
| 3 | **P2-WATER-WITNESS-002** | queued | Coder B | Dual-band water witness in CI |

**Done — do not reopen:** FX-FIRE-SPARK-001…006, FX-WATER-SHADER/PARTICLE-001/002, P2-VFX-VISUAL-001.

---

## P2 — Deferred / infra (non–VFX-P2 exit)

| ID | Item | Notes |
|:---|:---|:---|
| **VX-P2-01** | `fire_instance_buffer_rows: 0` | F-T02 — projection graph when visibility stable |
| **VX-P2-02** | VT-5 intermittent @ low `fire_inst` | [`visual_run_blockers.md`](visual_run_blockers.md) VR-04 — not FULL_APP gate |
| **VX-P2-03** | Fire sim Phase 7 LOD | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) — separate from spark VFX |

---

## Queue ↔ triage reconciliation

| Queue row | Triage verdict | Action |
|:---|:---|:---|
| P2-VFX-VISUAL-001 | ☑ done | — |
| FX-FIRE-SPARK-001…006 | ☑ done | — |
| P2-VFX-WITNESS-001 | ◐ **partial** | `gpu_particles` + `stage5_full_app_harness::tactical_vfx_*` green; close after doc line in `debug_runs/README.md` |
| P2-FIRE-SPARK-010 | open | Only if operator reproduces smoke-over-spark |
| P2-FIRE-SPARK-011 | open | After VX-P0-01 confirmed fixed in sim |
| WATER-W1-OCEAN-001, WATER-W2-FOAM-001 | open | **Primary FX-WATER** |
| FX-WATER track **CLOSED** | blocked | D-VFX + water witness exit + captures |

---

## Recommended dual-coder split (next session)

```text
Coder A (render)     WATER-W1-OCEAN-001 → WATER-W2-FOAM-001 → P2-FIRE-SPARK-011
Coder B (witness)  P2-VFX-WITNESS-001 close-out → P2-WATER-WITNESS-002
Operator             VX-P0-04 captures + normal Simulation fire read (not VfxSandbox)
Designer             Re-run D-VFX → ACCEPTED when PNGs + W-T* closed
```

---

## HANDOFF one-liner

**S-VFX:** Harness **green**; tracks **not CLOSED**. Fix **operator fire presentation (VX-P0-01)** and **water ocean/foam (W1/W2)** in parallel; capture tactical PNGs for designer **ACCEPTED**.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Initial S-VFX triage after S7P-IND-001 + FX Phase A |
