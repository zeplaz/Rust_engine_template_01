# Water surface VFX closure — `FX-WATER` track `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `FX-WATER` |
| **Version** | `1.1.0` |
| **Planner rollup** | [`../water_vfx_track_closure_plan_v1.md`](../water_vfx_track_closure_plan_v1.md) (**PLAN-WATER-TRACK-001**) |
| **Steward gate** | [`../steward_water_witness_gate_v1.md`](../steward_water_witness_gate_v1.md) |
| **Status** | **CLOSED** (2026-05-25) — sign-off [`../water_vfx_track_closure_plan_v1.md`](../water_vfx_track_closure_plan_v1.md) v2 — **do not re-queue W1/W2 foam** |
| **Audit** | [`../stage_tracks_audit_signoff_20260525.md`](../stage_tracks_audit_signoff_20260525.md) |
| **Sign-off** | [`../stage_tracks_signoff_ledger_v1.md`](../stage_tracks_signoff_ledger_v1.md) · [`../water_vfx_review_record_v1.md`](../water_vfx_review_record_v1.md) |
| **Exit milestone** | Optional designer **ACCEPTED** vs mock (tactical witness green) |
| **Index** | [`../stage_tracks_execution_index_v1.md`](../stage_tracks_execution_index_v1.md) |
| **Coder queue (detail)** | [`../../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/water_surface_vfx_coder_queue_v1.md`](../../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/water_surface_vfx_coder_queue_v1.md) |
| **Design (SIGNED)** | [`../water_surface_vfx_design_plan_v1.md`](../water_surface_vfx_design_plan_v1.md) |
| **Worksheet** | [`../../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/water_surface_vfx_decision_worksheet_v1.md`](../../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/water_surface_vfx_decision_worksheet_v1.md) |
| **Mock** | [`../../assets/vfx/reference/water/water_surface_target_v1.png`](../../assets/vfx/reference/water/water_surface_target_v1.png) |
| **Designer review** | [`../water_vfx_review_record_v1.md`](../water_vfx_review_record_v1.md) — **WATER-DESIGN-001 SIGNED (TUNE)** |
| **Related** | [`vfx_phase2_closure_plan_v1.md`](vfx_phase2_closure_plan_v1.md) (fire + shared tactical proof) |

**Snapshot (2026-05-25):** Tactical witness **green** in `stage5_full_app_live.json`. Strategic band rows **culled by design** (D-F09 parity). Designer **SIGNED TUNE** — optional **ACCEPTED** vs mock remains.

---

## Witness snapshot (2026-05-25 — `stage5_full_app_live.json`)

| Field | Current | Design intent (D-W) |
|:---|:---|:---|
| `water_w1_green` | ✅ `true` | W1 spine OK |
| `water_ocean_tiles` | ✅ `1715` | D-W04 ocean |
| `water_particle_rows` | ✅ `218` (tactical) | W2 draws |
| `water_particle_coast_foam` | ✅ `128` | D-W08 coast |
| `water_particle_river_foam` | ✅ `2` | D-W07 bend |
| `water_w1_river_read_green` | ✅ `true` | strategic ribbon read |
| `water_strategic_001_green` | ✅ `true` | strategic cull policy |
| `all_green` (tactical vfx) | ✅ `true` | harness exit |

**Remaining (optional):** in-sim PNG captures; designer **ACCEPTED** (not blocking witness).

---

## Signed decisions → remaining work

| ID | Choice | Status | Remaining slice |
|:---|:---:|:---|:---|
| D-W01 | A River polyline overlay | ☑ | **WATER-W1-RIVER-001** — strategic zoom river read + witness |
| D-W02 | A Lake ripple | ☑ landed | verify in designer review |
| D-W03 | A River UV scroll | ☑ landed | **WATER-W1-RIVER-001** |
| D-W04 | A Ocean swell + haze | ☑ code | **WATER-W1-OCEAN-001** — border-touching hydro lakes → `ocean_tiles` |
| D-W05 | A Pinpoint ≤2px | ☑ shaders | tune in **WATER-W2-TUNE-001** |
| D-W06 | B Lake glints optional | ☑ emitting | density/twinkle tune |
| D-W07 | A River streaks + bend foam | ☑ | **WATER-W2-FOAM-001** — waived when no bend in fixture; lib test proves path |
| D-W08 | B Coast foam only | ☑ | **WATER-W2-FOAM-001** — `coast_foam: 128` |
| D-W09 | A Particles fade; shaders on | ☑ caps | verify strategic `rows == 0` + motion on |
| D-W10 | A Custom WGSL spine | ☑ landed | maintain only |

---

## Witness exit (Water VFX CLOSED)

| File | Required |
|:---|:---|
| `debug_runs/stage5_full_app_live.json` | `water_w1_green: true` |
| | `water_shader_motion_always_on: true` at strategic + tactical |
| | `water_particle_rows > 0` at tactical (`zoom_alpha ≥ 0.65`) |
| | `water_particle_rows == 0` at strategic (D-W09) |
| | `water_particle_river_foam > 0` OR documented “no bends in fixture” |
| | `water_particle_coast_foam > 0` OR `water_ocean_tiles > 0` with ocean fixture |
| | `water_ocean_tiles > 0` when world has deep-water band |
| Designer record | PASS on [`water_surface_target_v1.png`](../../assets/vfx/reference/water/water_surface_target_v1.png) |

---

## @designer instructions

### WATER-DESIGN-001 — Visual review — **SIGNED — TUNE ROUND** (2026-05-24)

**Record:** [`../water_vfx_review_record_v1.md`](../water_vfx_review_record_v1.md) — **does not close track** until W-T01…W-T07 + witness exit.

**Read:** [`../../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/vfx_post_implementation_review_v1.md`](../../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/vfx_post_implementation_review_v1.md) § water

| Capture | Compare |
|:---|:---|
| Lake + ripple | mock lake panel |
| River channel (directional) | mock river strip — **must differ from lake** |
| Ocean coast (if seeded) | mock ocean + haze |
| Tactical particles | glints / streaks / foam per §6 tokens |

**Deliver:** [`../water_vfx_review_record_v1.md`](../water_vfx_review_record_v1.md) with PASS / TUNE per channel.

**Status (2026-05-24):** ☑ **SIGNED — TUNE ROUND** (W-T01…W-T07 filed). **Does not** close track — witness exit + coder slices remain.

**Blocks track CLOSED** until witness exit green **and** overall water verdict **PASS** (or designer accepts residual deferrals).

### WATER-DESIGN-002 — Fixture request (optional)

**Deliver:** [`../water_ocean_fixture_request_v1.md`](../water_ocean_fixture_request_v1.md) — queue **WATER-DESIGN-002**.

If review needs ocean/coast: specify one world-gen seed or unittest fixture name for coders (**WATER-W1-OCEAN-001**).

**Done when:** fixture doc **SIGNED** with seed or test name.

---

## @coder instructions — dual lane

**Rule:** W1 shaders = **Coder A** · emission/witness = **Coder B** · ≤3 files per step · **no second hydrology extract** · **no Hanabi**.

### Remaining slices (priority order)

| ID | Owner | Goal | First files |
|:---|:---|:---|:---|
| **WATER-W1-OCEAN-001** | A | Ocean branch visible; `water_ocean_tiles > 0` | `water_overlay.wgsl`, world gen fixture or `water_surface_visual.rs` |
| **WATER-W1-RIVER-001** | A | River ribbon distinct from lake at **strategic** zoom — **done** | `water_overlay.wgsl`, `water_surface_visual.rs` |
| **WATER-W2-FOAM-001** | B | Bend foam + coast foam emitters | `gpu_water_particles.rs` |
| **WATER-W2-TUNE-001** | A+B | Pinpoint read; streak elongation; blend | `water_particle_draw.wgsl`, `gpu_water_particle_raster.rs` |
| **WATER-WITNESS-001** | B | Harness fields + unit tests for foam/ocean — **done** | `water_vfx_witness` JSON + rollup gates |
| **WATER-STRATEGIC-001** | B | Assert D-W09: particles 0, shader on at strategic — **done** | `evaluate_water_vfx_witness_bands` |

**Landed — do not redo from scratch:**

| Slice | Evidence |
|:---|:---|
| FX-WATER-SHADER-001/002 | `register_water_surface_draw`, `water_w1_green` |
| FX-WATER-PARTICLE-001/002 | `water_particle*.wgsl`, `gpu_water_particles.rs` |

### Copy-paste — Coder A (WATER-W1-OCEAN-001)

```
Track: FX-WATER — WATER-W1-OCEAN-001
Read: docs/archive/2026-06-src-dev/trees/stages/water_vfx_closure_plan_v1.md
      docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/water_surface_vfx_coder_queue_v1.md § D-W04
First: ensure unittest/visual world has deep_water band; verify ocean swell in water_overlay.wgsl
Do NOT: new HydrologyResult extract; Hanabi
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual stage5
Witness: stage5_full_app_live.json water_ocean_tiles > 0
```

### Copy-paste — Coder A (WATER-W1-RIVER-001)

```
Track: FX-WATER — WATER-W1-RIVER-001
Read: water_surface_vfx_design_plan_v1.md § River
First: side-by-side lake vs river at strategic zoom — directional scroll visible
Do NOT: merge river into lake tint only
Verify: cargo run -p proc_A_dine01 --release -- --test visual
```

### Copy-paste — Coder B (WATER-W2-FOAM-001)

```
Track: FX-WATER — WATER-W2-FOAM-001
Read: water_surface_vfx_coder_queue_v1.md § W2-B3/B4
First: RiverFoam at curvature peaks; OceanFoam at coast tiles only (D-W08 B)
Do NOT: water_overlay.wgsl; open-ocean spray
Verify: cargo test -p proc_A_dine01 --lib gpu_water_particles stage5
Witness: water_particle_river_foam > 0 and/or water_particle_coast_foam > 0
```

### Copy-paste — Coder B (WATER-WITNESS-001)

```
Track: FX-WATER — WATER-WITNESS-001
Read: docs/archive/2026-06-src-dev/trees/stages/water_vfx_closure_plan_v1.md § Witness exit
First: stage5 gates for foam + ocean + strategic particle cull
Verify: cargo test -p proc_A_dine01 --lib stage5 water_surface_visual
```

### Global regression

```powershell
cargo test -p proc_A_dine01 --lib water_surface_visual stage5 gpu_water_particles
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Do-not-touch matrix

| Working on… | Do not edit |
|:---|:---|
| W1 shaders (A) | `gpu_water_particles.rs` emission tables |
| W2 emission (B) | `water_overlay.wgsl`, `water_particle_draw.wgsl` |
| Any water slice | Second terrain/hydrology extract; Hanabi |
| Fire parallel | `gpu_particles.rs` same session unless coordinated |

---

## Acceptance — Water VFX CLOSED

| # | Criterion |
|:---:|:---|
| W1 | All D-W01…D-W10 rows ☑ or documented defer with designer OK |
| W2 | Witness exit table (§ above) green in one `--test visual` run |
| W3 | `cargo test -p proc_A_dine01 --lib water_surface_visual stage5 gpu_water_particles` green |
| W4 | **WATER-DESIGN-001** SIGNED (PASS or TUNE list only) | ☑ TUNE 2026-05-24 |
| W5 | No regression: `fire_spark_*` / Stage 5 spine still green |

---

## Parallel with other tracks

| Track | Parallel? |
|:---|:---|
| Stage 7 Play | ✅ disjoint |
| Fire VFX (P2-FIRE-*) | ⚠️ coordinate if touching `gpu_*_raster` order |
| UI Phase 4 | ✅ |
| Infra 5.5+ | ✅ |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | Track CLOSED; link PLAN-WATER-TRACK-001 rollup |
| v1.0.0 | 2026-05-24 | Dedicated FX-WATER closure track; honest partial status |
