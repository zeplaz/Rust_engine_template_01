# Coder execution plan `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.5.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@coder` |
| **Sign-off ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |
| **Active queue** | [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md) |
| **Stage tracks (master)** | [`stage_tracks_execution_index_v1.md`](stage_tracks_execution_index_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/continuation_queue.json`](../../tools/orchestrator/queues/continuation_queue.json) |
| **UI index** | [`prompts/guides/ui/README.md`](../prompts/guides/ui/README.md) |
| **Product board** | [`post_stage6_active_todos.md`](post_stage6_active_todos.md) |

**Rule:** One **primary** slice per session (≤3 files per step). Parallel lanes OK when file sets are disjoint.

### Design gates (2026-05-24)

**No blocking design gates remain** for coder implementation. All §11 sign-offs are **SIGNED** (UI shell, World Preview layout, fire sparks, water VFX, Phase 2C **2C-B**).

| Optional (non-blocking) | Owner | When |
|:---|:---|:---|
| **P4-ART-01** — traced `icon_atlas_phase4_v1.png` | `@designer` | Phase 4 polish; placeholder atlas is shippable |
| **D-VFX** post-implementation review | `@designer` | **done** (TUNE) — [`vfx_design_review_record_v1.md`](vfx_design_review_record_v1.md) |

---

## Pick your lane (30 sec)

**Audited 2026-05-24:** [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) · active slices: [`stage_coder_workboard_v1.md`](stage_coder_workboard_v1.md)

**Seven tracks** — full plans: [`stage_tracks_execution_index_v1.md`](stage_tracks_execution_index_v1.md)

| Track | Plan | Primary slice |
|:---|:---|:---|
| **Stage 7 Play** | [`stages/stage7_play_plan_v1.md`](stages/stage7_play_plan_v1.md) | **S7P-IND-001** |
| **VFX Phase 2** (fire) | [`stages/vfx_phase2_closure_plan_v1.md`](stages/vfx_phase2_closure_plan_v1.md) | **P2-FIRE-SPARK-010** |
| **Water VFX** | [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) | **WATER-W2-FOAM-001** |
| **UI Phase 4** | [`stages/ui_phase4_execution_plan_v1.md`](stages/ui_phase4_execution_plan_v1.md) | **UI-WP-LAYOUT-002** |
| **Infra 5.5+** | [`stages/infra_55_execution_plan_v1.md`](stages/infra_55_execution_plan_v1.md) | **INFRA-VM09-001** |
| **Wave C** | [`stages/wave_c_depth_plan_v1.md`](stages/wave_c_depth_plan_v1.md) | **WC-DEPTH-001** |
| **Fire Phase 7** | [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) | planning only |
| **Behavioral** | [`stages/stage7_behavioral_plan_v1.md`](stages/stage7_behavioral_plan_v1.md) | gated — designer first |

**Phase 2 VFX** — detail: [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) · **Starters:** [`vfx_coder_phase2_starters_v1.md`](../prompts/guides/ui/vfx_coder_phase2_starters_v1.md)

| Agent | Primary (Phase 2) | Status |
|:---|:---|:---|
| **Coder A** | **P2-VFX-VISUAL-001** tactical visual proof | **queued** |
| **Coder A** | **P2-FIRE-SPARK-010** sparks above smoke | queued |
| **Coder A** | **P2-WATER-POLISH-001** river/ocean read | queued |
| **Coder B** | **P2-VFX-WITNESS-001** tactical unit tests | **queued** |
| **Coder B** | **P2-WATER-WITNESS-002** water particle JSON gates | queued |
| **Coder B** | **UI-WP-LAYOUT-001** / **IND-E01** | queued (disjoint) |

| Priority | Slice | Status | When |
|:---:|:---|:---|:---|
| **P1** | **P2-VFX-VISUAL-001** + **P2-VFX-WITNESS-001** | **active** | Particle rows > 0 at **tactical** zoom |
| **P1** | **UI-WP-LAYOUT-001** | queued | World preview shell (signed) |
| **P2** | **P2-FIRE-SPARK-010/011**, **P2-WATER-POLISH-001** | queued | VFX polish |
| **P2** | **IND-E01** | queued | Product parallel |
| **—** | FX-FIRE-SPARK-001…006, FX-WATER-* | **done** | See [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) § Done |

---

## Global regression (every slice)

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib minimap_compositor simulation_shell_phase2
```

Visual / witness refresh when touching render or shell:

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Slice 1 — UI-P3-M2-001 · Minimap logistics heat (PRIMARY)

**Goal:** `debug_runs/minimap_compositor_live.json` → `logistics_rows > 0` with toggle on.

**Docs:** [`ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) §3.4 · [`ui_phase3_minimap_compositor_plan.md`](ui_phase3_minimap_compositor_plan.md) · [`logistics_visual_lane_spec_v1.md`](logistics_visual_lane_spec_v1.md)

**Authority:** Read `LogisticsVisualSnapshot` only — **no** new logistics extract.

### Copy-paste

```
Lane: UI-P3-M2-001 — minimap M2 logistics heat
Read: src/dev/coder_execution_plan_v1.md § Slice 1
First: confirm logistics_heat toggle reaches compositor; seed visual run has log_rows
Do NOT: duplicate extract; touch world_preview chrome
```

### Steps

| Step | Task | Files (≤3) | Verify |
|:---:|:---|:---|:---|
| **M2-1** | Default `logistics_heat: true` in sim or expose toggle in minimap chrome | `minimap_shell.rs`, `map_view/presentation/mod.rs` | toggle visible in sim |
| **M2-2** | Confirm `upload_minimap_heat_textures` maps `LogisticsVisualSnapshot.edge_rows` | `minimap_compositor/composite.rs` | unit or harness |
| **M2-3** | Witness fields: `logistics_rows`, `logistics_heat_enabled` in live JSON | `minimap_compositor/live_proof.rs` | JSON > 0 after visual |
| **M2-4** | Operator visual run with transport seed | — | `--test visual` |

**Likely touch (already partial):**

| File | Role |
|:---|:---|
| `src/render/minimap_compositor/composite.rs` | Heat upload from snapshot |
| `src/render/minimap_compositor/pass.rs` | Uniforms + `logistics_rows` stamp |
| `src/render/minimap_compositor/gpu_compute.rs` | Compute dispatch |
| `assets/shaders/minimap/minimap_composite.wgsl` | Logistics channel blend |
| `src/gui/minimap_shell.rs` | `MinimapOverlayMask.logistics_heat` |

### Accept

- [ ] `logistics_rows > 0` in `minimap_compositor_live.json`
- [ ] Fire + logistics toggles independent
- [ ] `cargo test -p proc_A_dine01 --lib minimap_compositor stage5` green
- [ ] `dual_minimap_present: false` unchanged

---

## Slice 2 — IND-E01 · Industrial activation E2E

**Goal:** Place concrete chain in sim → green `debug_runs/industrial_activation_live.json`.

**Docs:** [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) · [`post_stage6_active_todos.md`](post_stage6_active_todos.md)

**Disjoint from UI-P3** — safe parallel session.

### Copy-paste

```
Lane: IND-E01 — concrete chain E2E
Read: src/dev/industrial_activation_pipeline.md
First: mine → kiln → mixer placement + activation bridge witness
Do NOT: simulation_shell_phase2, minimap_compositor, world_preview/window.rs
```

### Steps

| Step | Task | Entry | Witness |
|:---:|:---|:---|:---|
| **E-1** | Place aggregate mine → kiln → mixer via construction | `economy/activation/` | construction green |
| **E-2** | Verify production ECS + flow edges | building JSON + activation bridge | `industrial_activation_live.json` |
| **E-3** | Grid/substation stress optional (IND-E03) | utilities configs | overload flag if scoped |

### Accept

- [x] `concrete_chain_e2e` block in proof JSON (unit test `simulation_writes_industrial_activation_live_json_concrete_chain_e2e`)
- [x] `industrial_activation_live.json` `production_green: true` (lib proof + live witness)
- [x] `cargo test -p proc_A_dine01 --lib industrial_activation` green (2026-05-24, after FX-FIRE Phase A)

---

## Slice 3 — UI-P2A witness tail (OPTIONAL · ~15 min)

Non-blocking — Phase 2 already **SIGNED**; fixes passive proof gaps.

| Slice | Action | Command | Witness field |
|:---|:---|:---|:---|
| **UI-P2A-F03** | Hover ops strip zone until accent visible | `cargo run -p proc_A_dine01 -- --test frame` | `ops_zone_hover_token: true` |
| **UI-P2A-P4-AUTH** | Click build rail tool (writes `BuildStripState`) | `cargo run -p proc_A_dine01` | `build_rail_authoritative: true` |

Refresh: re-run sim until `ui_shell_migration_live.json` updates (or capture pass).

**Files (if hover broken):** `simulation_shell_phase2.rs` (`sync_ops_strip_zone_hover_system`).

---

## Slice 4 — UI-WP-LAYOUT-001 · World Preview chrome (**queued**)

**Design:** [`world_map_preview_layout_decision_v1.md`](../prompts/guides/ui/world_map_preview_layout_decision_v1.md) §11 **SIGNED** · `layout_mock_v1.png` committed.

### First coder slice

```
Lane: UI-WP-LAYOUT-001
Read: world_map_preview_layout_decision_v1.md signed §5
First: D-01 shell choice ONLY (max 3 files)
Verify: F8 WorldGen layout matches mock; camera stable on resize
Do NOT: raster invalidation graph, GenerateWorldEvent, motion (WP-L3)
```

| File | Change |
|:---|:---|
| `src/gui/editor/world_preview/window.rs` | Panel layout per signed D-03, D-07 |
| `src/gui/editor/world_gen_ui.rs` | D-04 generator sheet vs window |
| `src/gui/std_floating.rs` or new `archive_frame.rs` | Paper frame helpers (D-08) |

**Phased after D-01:** WP-L2 shell → WP-L3 motion §6 → WP-L5 map presentation §8.

---

## Slice 5 — UI-WP-PIPELINE · World preview raster (ALLOWED NOW)

**Allowed without layout sign-off** — pipeline bugs only.

**Docs:** [`world_preview_runbook_v1.md`](../prompts/guides/world_preview_runbook_v1.md)

| OK | Blocked |
|:---|:---|
| `render_raster.rs`, viewport contract, GPU preview path, chunk-dirty invalidation | `window.rs` panel shells, asymmetry offsets, motion, merged workspace |

### Copy-paste

```
Lane: UI-WP-PIPELINE (bugfix)
Read: prompts/guides/world_preview_runbook_v1.md
Scope: raster/GPU/viewport only — NOT window.rs layout
Verify: cargo test -p proc_A_dine01 --lib stage5 + wave_p witness if touched
```

---

## Slice 6 — UI-P2-2C · Left chrome layout (**done** — 2C-B)

**Closed 2026-05-24:** Designer picked **2C-B** — canonical dual column documented in [`ui_phase0_panel_mocks_v1.md`](../prompts/guides/ui/ui_phase0_panel_mocks_v1.md) § P4. Witness `phase2c` in `ui_shell_migration_live.json`. **No coder layout change required** (live code already matches mock).

| Option | Outcome |
|:---|:---|
| **2C-B** | **Selected** — 48px context + 52px build + 6px gap; overlay does not inset map hole |

---

## Slice 8 — FX-FIRE-SPARK · Fire pinpoint sparks (dual @coder)

**Design:** **SIGNED** · Phase A **landed** · Phase B + witness **active**.

**Dual queue:** [`fire_particle_spark_coder_queue_v1.md`](../prompts/guides/ui/fire_particle_spark_coder_queue_v1.md)

### Coder A — render / GPU

```
Lane: FX-FIRE-SPARK-002 — compute advection (Phase B)
Read: fire_particle_spark_coder_queue_v1.md § A1
      elemental/compute_expanse_BASE_A.glsl (local ref machine)
First: assets/shaders/fire/fire_spark_compute.wgsl
Do NOT: gpu_particles scatter; second fire extract
Verify: cargo test -p proc_A_dine01 --lib gpu_particles stage5; --test visual
```

Then **FX-FIRE-SPARK-004** (smoke draw order, D-F10 A).

### Coder B — policy / witness

```
Lane: FX-FIRE-SPARK-003 — witness + scatter caps
Read: fire_particle_spark_coder_queue_v1.md § B0
First: fire_spark_rows in stage5_full_app_live.json
Do NOT: fire_particle_draw.wgsl
Verify: cargo test -p proc_A_dine01 --lib stage5
```

Then **FX-FIRE-SPARK-005** (Spark/Ember class) → **006** (per-view cull).

**Parallel product (Coder B only):** **IND-E01** — disjoint files.

### Phase A accept (done)

- [x] Sparks read as pinpoint field (`fire_spark_target_v1.png`)
- [x] `cargo test -p proc_A_dine01 --lib gpu_particles` green (8 tests)
- [x] Single fire extract unchanged
- [ ] `--test visual` operator refresh (A0 verify)

### Phase B+ accept (open)

- [ ] `fire_spark_compute.wgsl` advection + respawn
- [ ] Witness: `fire_spark_rows`, `fire_spark_zoom_alpha` in JSON
- [ ] Sparks render above smoke field
- [ ] Spark vs Ember class sizing in tests

---

## Slice 9 — FX-WATER · Lake / river / ocean VFX (**dual @coder**)

**Design:** **SIGNED** · W1 **partial** (catalog + WGSL landed; GPU hook missing) · W2 **blocked**.

**Dual queue:** [`water_surface_vfx_coder_queue_v1.md`](../prompts/guides/ui/water_surface_vfx_coder_queue_v1.md)

### Signed decisions in scope

| ID | Work |
|:---|:---|
| D-W02 A | Lake slow omnidirectional ripple |
| D-W03 A | River UV scroll along flow |
| D-W04 A | Ocean swell + horizon haze |
| D-W05 A | Pinpoint particles (fire spark family) |
| D-W06 B | Lake glints optional (W2, low priority) |
| D-W07 A | River downstream streaks + bend foam |
| D-W08 B | Ocean **coast foam only** |
| D-W09 A | **Particles** fade at strategic zoom; **shader motion always on** |
| D-W10 A | Custom WGSL spine — no Hanabi |

### Coder A — W1 GPU + W2 shaders

```
Lane: FX-WATER-SHADER-001 finish
Read: water_surface_vfx_coder_queue_v1.md § W1-A
First: register_water_surface_draw in engine_with_worldgen.rs
Then: FX-WATER-PARTICLE-001 — water_particle.wgsl + water_particle_draw.wgsl (D-W05, D-W10)
Do NOT: gpu_water_particles.rs emission; hydrology sim
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual stage5
```

### Coder B — W1 witness + W2 emission

```
Lane: FX-WATER-SHADER-002 witness
Read: water_surface_vfx_design_plan_v1.md §7
First: water_w1_green in stage5_full_app_live.json
Then: FX-WATER-PARTICLE-002 — gpu_water_particles.rs (D-W06–D-W09)
Do NOT: water_overlay.wgsl
Verify: cargo test -p proc_A_dine01 --lib water_surface_visual
```

**Blocks:** W2 until `water_w1_green: true`.

---

## Slice 7 — CON-E01 P9 verify (LOW priority) — **DONE**

Runtime P9 board syncs from [`ConstructionPhase2Witness`](construction_phase2_todos.rs) tail; live proof exposes `p9_build` + `con_e01_p9_green`.

```powershell
cargo test -p proc_A_dine01 construction:: --lib
# sim session → refresh debug_runs/construction_stage_live.json
```

---

## Do-not-touch matrix

| While working on… | Do not edit |
|:---|:---|
| **UI-P3-M2** | `world_preview/window.rs`, `in_game_hud.rs` left chrome (2C), industrial ECS unless coordinated |
| **IND-E01** | minimap compositor, shell phase2, world preview chrome |
| **UI-WP-PIPELINE** | Panel layout / motion in `window.rs` |
| **Any slice** | Stage 5 spine contracts without `stage5` regression |

---

## Witness targets (quick ref)

| File | Slice | Green when |
|:---|:---|:---|
| `minimap_compositor_live.json` | UI-P3-M2-001 | `logistics_rows > 0` |
| `industrial_activation_live.json` | IND-E01 | chain operational |
| `ui_shell_migration_live.json` | UI-P2A tail | hover + rail flags true |
| `layout_mock_v1.png` | UI-WP-LAYOUT-001 | designer committed (gate) |
| `stage5_full_app_live.json` | FX-FIRE-SPARK-001 / render | vt4/vt5 ok |
| `fire_particle_spark_design_plan_v1.md` §11 | FX-FIRE-SPARK-DESIGN | **SIGNED** |
| `water_surface_vfx_design_plan_v1.md` §11 | FX-WATER-DESIGN | **SIGNED** |

---

## Document index

| Lane | Deep queue |
|:---|:---|
| UI Phase 3 minimap | [`ui_phase3_coder_queue_v1.md`](../prompts/guides/ui/ui_phase3_coder_queue_v1.md) |
| UI overhaul status | [`ui_overhaul_plan.md`](ui_overhaul_plan.md) |
| UI Phase 2 archive | [`ui_phase2_coder_queue_v1.md`](../prompts/guides/ui/ui_phase2_coder_queue_v1.md) |
| World preview layout | [`world_map_preview_layout_decision_v1.md`](../prompts/guides/ui/world_map_preview_layout_decision_v1.md) §12 — **SIGNED** |
| VFX Phase 2 (active) | [`vfx_coder_phase2_queue_v1.md`](vfx_coder_phase2_queue_v1.md) |
| Fire sparks (archive) | [`fire_particle_spark_coder_queue_v1.md`](../prompts/guides/ui/fire_particle_spark_coder_queue_v1.md) |
| Fire sparks design | [`fire_particle_spark_design_plan_v1.md`](fire_particle_spark_design_plan_v1.md) |
| Water VFX (dual coder) | [`water_surface_vfx_coder_queue_v1.md`](../prompts/guides/ui/water_surface_vfx_coder_queue_v1.md) |
| Water VFX design | [`water_surface_vfx_design_plan_v1.md`](water_surface_vfx_design_plan_v1.md) |
| Playbook | [`tools/orchestrator/agents/ui_layout_agent.md`](../../tools/orchestrator/agents/ui_layout_agent.md) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.3.1 | 2026-05-24 | Policy: no blocking design gates; optional P4-ART + VFX mock review |
| v1.3.0 | 2026-05-24 | Phase 2 queue — proof/tuning after VFX code landed |
| v1.2.1 | 2026-05-24 | Dual-coder FX-WATER + FX-FIRE first pass |
| v1.1.0 | 2026-05-23 | Slice 8 FX-FIRE-SPARK-001 (blocked on design §11) |
| v1.0.0 | 2026-05-24 | Consolidated coder slices P1–P7 |
