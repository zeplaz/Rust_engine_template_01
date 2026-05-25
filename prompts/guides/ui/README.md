# UI guides index

**Master lane plan:** [`src/dev/ui_overhaul_plan.md`](../../../src/dev/ui_overhaul_plan.md) v1.1.0  
**Sign-off ledger:** [`src/dev/stage_tracks_signoff_ledger_v1.md`](../../../src/dev/stage_tracks_signoff_ledger_v1.md) · **7 tracks:** [`stage_tracks_execution_index_v1.md`](../../../src/dev/stage_tracks_execution_index_v1.md)  
**Machine queue:** [`tools/orchestrator/queues/continuation_queue.json`](../../../tools/orchestrator/queues/continuation_queue.json)  
**Playbook:** [`tools/orchestrator/agents/ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md)

### Optional follow-ups (not blocking coders)

| Item | Notes |
|:---|:---|
| **Phase 4 icon atlas PNG** | Replace placeholder per [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) — code path already green |
| **VFX mock review** | [`vfx_post_implementation_review_v1.md`](vfx_post_implementation_review_v1.md) · captures in `assets/vfx/reference/review_captures/` |

---

## Phase map

| Phase | Status | Entry doc |
|:---|:---|:---|
| **Phase 0** — panel mocks | CLOSED | [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) |
| **Phase 1** — theme / tokens | CLOSED | [`design_theme.md`](design_theme.md), [`palette_v2_tokens.md`](palette_v2_tokens.md) |
| **Phase 2** — Bevy shell + egui dedupe | **CLOSED** | [`ui_overhaul_plan.md`](../../../src/dev/ui_overhaul_plan.md) |
| **Phase 2 sign-off** | **SIGNED** | [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) v2.1.1 |
| **Phase 3 M1/M1.5** — GPU minimap | **CLOSED** · plan **APPROVED** | [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) v1.0.1 · archive [`ui_phase3_minimap_compositor_plan.md`](../../../src/dev/ui_phase3_minimap_compositor_plan.md) |
| **Phase 3 M2** — strategic overlays | **SIGNED** | [`minimap_d_m2_signoff_v1.md`](../../../src/dev/minimap_d_m2_signoff_v1.md) · [`ui_phase3_coder_queue_v1.md`](ui_phase3_coder_queue_v1.md) |
| **Phase 4** — icon atlas | PARTIAL (code done) | [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) — **optional** traced PNG |
| **World Map Preview layout** | **SIGNED** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) · [**D-WP**](world_preview_d_wp_track_signoff_v1.md) · [D-01](world_preview_d01_shell_signoff_v1.md) · [D-02 opt](world_preview_d02_map_dominance_signoff_v1.md) |
| **GPU minimap M1** | **SIGNED done** | [`minimap_d_m1_signoff_v1.md`](../../../src/dev/minimap_d_m1_signoff_v1.md) (**D-MINIMAP-M1**) |
| **GPU minimap M2** | **SIGNED done** | [`minimap_d_m2_signoff_v1.md`](../../../src/dev/minimap_d_m2_signoff_v1.md) (**D-MINIMAP-M2**) |
| **D-VFX** post-implementation review | **SIGNED TUNE** | [`vfx_design_review_record_v1.md`](../../../src/dev/vfx_design_review_record_v1.md) · brief [`vfx_post_implementation_review_v1.md`](vfx_post_implementation_review_v1.md) |
| **Fire pinpoint sparks (VFX)** | **ACTIVE** | [`fire_particle_spark_coder_queue_v1.md`](fire_particle_spark_coder_queue_v1.md) · [`vfx_coder_phase2_queue_v1.md`](../../../src/dev/vfx_coder_phase2_queue_v1.md) |
| **Water surface VFX** | **ACTIVE · NOT CLOSED** | [`water_vfx_closure_plan_v1.md`](../../../src/dev/stages/water_vfx_closure_plan_v1.md) · [`water_surface_vfx_coder_queue_v1.md`](water_surface_vfx_coder_queue_v1.md) |

---

## Active slices (continuation queue) — **Phase 2**

| ID | Agent | Status |
|:---|:---|:---|
| **P2-VFX-VISUAL-001** | `@coder` A | **queued** — tactical zoom; `fire_spark_rows` / `water_particle_*` > 0 |
| **P2-VFX-WITNESS-001** | `@coder` B | **queued** — unit tests at `zoom_alpha ≥ 0.65` |
| **P2-FIRE-SPARK-010** | `@coder` A | **queued** — sparks above smoke overlay |
| **P2-FIRE-SPARK-011** | `@coder` A | queued — compute motion tuning |
| **P2-WATER-POLISH-001** | `@coder` A | queued — river read / ocean tiles |
| **P2-WATER-WITNESS-002** | `@coder` B | queued — refresh water particle JSON |
| **UI-WP-LAYOUT-001** | `@coder` | **done** — D-01 unified workspace shell |
| **IND-E01** | `@coder` B | queued — industrial chain |
| **D-VFX** / VFX-POST-REVIEW | `@designer` | **done** — [`vfx_design_review_record_v1.md`](../../../src/dev/vfx_design_review_record_v1.md) **TUNE** |

**Done (no rework):** FX-FIRE-SPARK-001…006 · FX-WATER-SHADER-001/002 · FX-WATER-PARTICLE-001/002

**Next primary:** **P2-VFX-VISUAL-001** + **P2-VFX-WITNESS-001** in parallel.

**Phase 2 doc:** [`vfx_coder_phase2_queue_v1.md`](../../../src/dev/vfx_coder_phase2_queue_v1.md)

---

## Documents by role

| Role | Read first |
|:---|:---|
| **@coder** | [`vfx_coder_phase2_queue_v1.md`](../../../src/dev/vfx_coder_phase2_queue_v1.md) · [`coder_execution_plan_v1.md`](../../../src/dev/coder_execution_plan_v1.md) |
| **@coder** A (render) | [`water_vfx_closure_plan_v1.md`](../../../src/dev/stages/water_vfx_closure_plan_v1.md) · WATER-W1-OCEAN-001, WATER-W1-RIVER-001 |
| **@coder** B (policy) | WATER-W2-FOAM-001, WATER-WITNESS-001 · fire queue § B |
| **@designer** (water) | **WATER-DESIGN-001** **done** — [`water_vfx_review_record_v1.md`](../../../src/dev/water_vfx_review_record_v1.md) **TUNE** |
| **@designer** | [`fire_particle_spark_decision_worksheet_v1.md`](fire_particle_spark_decision_worksheet_v1.md) · WP-L1 paper assets (optional Figma spec § below) |
| **@sim-steward** | **`UI-P3-PREFLIGHT`** before [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) → `@coder` |
| **@planner** | Phase 3 plan **APPROVED** (2026-05-24) — M3 slices when queued |
| **@coder** | **`UI-P3-001`** per compositor plan v1 (after preflight GO) |

---

## Witness JSON

| File | Status |
|:---|:---|
| `debug_runs/ui_shell_migration_live.json` | ✅ Phase 2 closed (2 tail flags optional) |
| `debug_runs/minimap_compositor_live.json` | ✅ M2 green · `logistics_rows: 2` |
| `debug_runs/stage5_full_app_live.json` | ✅ refreshed 2026-05-24 |

---

## Related

| Doc | Purpose |
|:---|:---|
| [`ui_phase2_sprint_queue.md`](../../../src/dev/ui_phase2_sprint_queue.md) | Phase 2 archive |
| [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) | UX-A design |
| [`post_stage6_active_todos.md`](../../../src/dev/post_stage6_active_todos.md) | Product board |
