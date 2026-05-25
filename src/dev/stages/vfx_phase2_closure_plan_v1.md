# VFX Phase 2 closure — tactical proof + tuning `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `VFX-P2` |
| **Version** | `1.0.0` |
| **Status** | **ACTIVE** — core **DONE** · tune **OPEN** |
| **Exit milestone** | **VFX Phase 2 CLOSED** — fire tactical proof + compositing + designer ACCEPTED |
| **Sign-off** | [`../stage_tracks_signoff_ledger_v1.md`](../stage_tracks_signoff_ledger_v1.md) · [`../vfx_design_review_record_v1.md`](../vfx_design_review_record_v1.md) **SIGNED TUNE** |
| **Triage board** | [`../vfx_triage_v1.md`](../vfx_triage_v1.md) — **S-VFX** harness vs operator split |
| **Water (separate track)** | [`water_vfx_closure_plan_v1.md`](water_vfx_closure_plan_v1.md) — **NOT CLOSED** |
| **Queue (detail)** | [`../vfx_coder_phase2_queue_v1.md`](../vfx_coder_phase2_queue_v1.md) |
| **Starters** | [`../../prompts/guides/ui/vfx_coder_phase2_starters_v1.md`](../../prompts/guides/ui/vfx_coder_phase2_starters_v1.md) |

**Context:** Fire shaders/compute **landed**. `fire_spark_rows: 0` at **strategic** zoom is **correct** (D-F09). Closure = **tactical fire proof** + compositing + tuning.

**Water is not part of this exit.** Use [`water_vfx_closure_plan_v1.md`](water_vfx_closure_plan_v1.md) — ocean, river read, bend/coast foam, designer mock PASS still open.

---

## Witness exit

| File | Fields |
|:---|:---|
| `debug_runs/stage5_full_app_live.json` | `fire_spark_rows > 0`, `water_particle_rows > 0` at `zoom_alpha ≥ 0.65`; `fire_sparks_above_smoke: true` |
| Unit tests | Tactical zoom in `gpu_particles` / `gpu_water_particles` / `stage5` harness |

---

## @designer instructions

### VFX2-DESIGN-001 — Post-implementation review (after P2-VFX-VISUAL-001)

**Read:** [`../../prompts/guides/ui/vfx_post_implementation_review_v1.md`](../../prompts/guides/ui/vfx_post_implementation_review_v1.md)

| Task | Deliverable |
|:---|:---|
| Capture tactical stills | `assets/vfx/reference/review_captures/` |
| Compare fire | vs `fire_spark_target_v1.png` |
| Compare water | vs `water_surface_target_v1.png` |
| Record | [`../vfx_design_review_record_v1.md`](../vfx_design_review_record_v1.md) — **D-VFX** |

**Status (2026-05-24):** ☑ **SIGNED — TUNE ROUND** — prerequisites met (`fire_spark_rows: 308`, tactical zoom 0.85); PNG captures pending; fire **F-T*** / water **W-T*** tickets open.

**Does not block** coder slices; **VFX-P2 CLOSED** needs tune completion or designer **ACCEPTED**.

### VFX2-DESIGN-002 — Spark read at tactical zoom (optional)

One mock annotation: acceptable spark density over burning cells (not blob). Attach to fire worksheet if TUNE needed.

---

## @coder instructions — dual lane

### Coder A (render / visual)

| ID | Title | Status | Evidence |
|:---|:---|:---|:---|
| **P2-VFX-VISUAL-001** | Tactical visual harness | **DONE** | `fire_spark_rows: 308`, tactical gates green |
| **P2-FIRE-SPARK-010** | Sparks above smoke | **DONE** | `fire_sparks_above_smoke: true` |
| **P2-FIRE-SPARK-011** | Spark motion tuning | **OPEN** | F-T01/F-T03 in vfx_design_review_record |

### Coder B (witness / CI)

| ID | Title | Status | Evidence |
|:---|:---|:---|:---|
| **P2-VFX-WITNESS-001** | Tactical unit tests | **PARTIAL→DONE** | lib tests green; close administratively |

Water slices → [`water_vfx_closure_plan_v1.md`](water_vfx_closure_plan_v1.md) only.

**Rule:** Do **not** remove strategic cull rules to make witness green.

### Copy-paste — Coder A (P2-VFX-VISUAL-001)

```
Track: VFX-P2 — P2-VFX-VISUAL-001
Read: src/dev/stages/vfx_phase2_closure_plan_v1.md
      src/dev/vfx_coder_phase2_queue_v1.md § P2-VFX-VISUAL-001
First: set MapCameraDesired / zoom_alpha >= 0.65 before witness stamp in harness
Do NOT: disable D-F09/D-W09 strategic cull globally
Verify: cargo run -p proc_A_dine01 --release -- --test visual
Witness: stage5_full_app_live.json fire_spark_rows > 0, water_particle_rows > 0
```

### Copy-paste — Coder B (P2-VFX-WITNESS-001)

```
Track: VFX-P2 — P2-VFX-WITNESS-001
Read: src/dev/stages/vfx_phase2_closure_plan_v1.md
First: add #[test] with tactical zoom_alpha fixture; assert rows > 0
Do NOT: touch WGSL unless test proves shader bug
Verify: cargo test -p proc_A_dine01 --lib gpu_particles stage5
```

### Done — do not re-implement

FX-FIRE-SPARK-001…006, FX-WATER-SHADER/PARTICLE-001/002 — see queue § Done.

### Acceptance — VFX Phase 2 CLOSED

| # | Criterion |
|:---:|:---|
| V1 | P2-VFX-VISUAL-001 + P2-VFX-WITNESS-001 complete |
| V2 | `fire_sparks_above_smoke: true` stable |
| V3 | `cargo test -p proc_A_dine01 --lib stage5 gpu_particles` green |
| V4 | Designer VFX2-DESIGN-001 recorded (PASS or TUNE list) | ☑ **D-VFX** TUNE 2026-05-24 |
| V5 | P2-FIRE-SPARK-010/011 done or deferred to VFX-P3 |
| V6 | **Water VFX CLOSED** per [`water_vfx_closure_plan_v1.md`](water_vfx_closure_plan_v1.md) (separate track) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Closure plan aligned to landed code |
