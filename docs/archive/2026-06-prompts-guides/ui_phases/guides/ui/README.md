# UI guides index

**Master lane plan:** [`docs/archive/2026-06-src-dev/plans/ui_overhaul_plan.md`](../../../docs/archive/2026-06-src-dev/plans/ui_overhaul_plan.md) v1.1.0  
**Sign-off ledger:** [`docs/archive/2026-06-src-dev/plans/stage_tracks_signoff_ledger_v1.md`](../../../docs/archive/2026-06-src-dev/plans/stage_tracks_signoff_ledger_v1.md) · **7 tracks:** [`stage_tracks_execution_index_v1.md`](../../../src/dev/stage_tracks_execution_index_v1.md)  
**Machine queue:** [`tools/orchestrator/queues/continuation_queue.json`](../../../tools/orchestrator/queues/continuation_queue.json)  
**Playbook:** [`tools/orchestrator/agents/ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md)

### Optional follow-ups (not blocking coders)

| Item | Notes |
|:---|:---|
| **Phase 4 icon atlas PNG** | Optional **UI-OH-P4-ART-001** — [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) · **SIGNED** 2026-05-25 |
| **VFX mock review** | [`vfx_post_implementation_review_v1.md`](vfx_post_implementation_review_v1.md) · captures in `assets/vfx/reference/review_captures/` |

---

## Phase map

| Phase | Status | Entry doc |
|:---|:---|:---|
| **Phase 0** — panel mocks | CLOSED | [`ui_phase0_panel_mocks_v1.md`](ui_phase0_panel_mocks_v1.md) |
| **Phase 1** — theme / tokens | CLOSED | [`design_theme.md`](design_theme.md), [`palette_v2_tokens.md`](palette_v2_tokens.md) |
| **Phase 2** — Bevy shell + egui dedupe | **CLOSED** | [`ui_overhaul_plan.md`](../../../docs/archive/2026-06-src-dev/plans/ui_overhaul_plan.md) |
| **Phase 2 sign-off** | **SIGNED** | [`ui_phase2_designer_signoff_v1.md`](ui_phase2_designer_signoff_v1.md) v2.1.1 |
| **Phase 3 GPU minimap** | M1/M2/M3 | [`ui_phase3_minimap_compositor_full_plan_v1.md`](ui_phase3_minimap_compositor_full_plan_v1.md) · M1 [`ui_phase3_minimap_compositor_plan_v1.md`](ui_phase3_minimap_compositor_plan_v1.md) · M2 [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) |
| **Phase 3 M2** — strategic overlays | **CLOSED** | [`minimap_d_m2_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/minimap_d_m2_signoff_v1.md) · full plan [`ui_phase3_minimap_m2_impl_full_plan_v1.md`](ui_phase3_minimap_m2_impl_full_plan_v1.md) |
| **Phase 4** — World Preview handoff | **CODER CLOSED** | D-04 + LAYOUT-002 done · optional WP-L3/L4 |
| **Phase 4** — icon atlas / petroleum | P4.1+P5 **DONE** · art/vehicles open | [`ui_phase4_icon_atlas_plan_v1.md`](ui_phase4_icon_atlas_plan_v1.md) · brief [`ui_phase4_icon_atlas_brief_v1.md`](ui_phase4_icon_atlas_brief_v1.md) |
| **UI Phase 2B egui gate** | **CLOSED** | [`ui_phase2b_gate_plan_v1.md`](ui_phase2b_gate_plan_v1.md) · tasks [`ui_p2b_coder_b_numbered_tasks_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ui_p2b_coder_b_numbered_tasks_v1.md) |
| **Phase 5** — pause menu | **OPEN (P2)** | [`ui_phase5_pause_menu_plan_v1.md`](ui_phase5_pause_menu_plan_v1.md) — scaffold egui partial |

| **World Map Preview product** | **SIGNED** | [`world_preview_product_decision_v1.md`](world_preview_product_decision_v1.md) · full plan [`world_preview_product_full_plan_v1.md`](world_preview_product_full_plan_v1.md) |
| **World Map Preview layout** | **SIGNED** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) · [**D-WP**](world_preview_d_wp_track_signoff_v1.md) · [D-01](world_preview_d01_shell_signoff_v1.md) · [D-02 opt](world_preview_d02_map_dominance_signoff_v1.md) |
| **D-WP post-impl review** | **SIGNED PASS** | [`world_preview_d_wp_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/world_preview_d_wp_review_record_v1.md) · brief [`world_preview_d_wp_post_impl_review_v1.md`](world_preview_d_wp_post_impl_review_v1.md) |
| **GPU minimap M1** | **SIGNED done** | [`minimap_d_m1_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/minimap_d_m1_signoff_v1.md) (**D-MINIMAP-M1**) |
| **GPU minimap M2** | **SIGNED done** | [`minimap_d_m2_signoff_v1.md`](../../../docs/archive/2026-06-src-dev/plans/minimap_d_m2_signoff_v1.md) (**D-MINIMAP-M2**) |
| **GPU minimap M3** | **FoW+EW DONE** · units/replay optional | **UI-P3-M4-001** — not **UI-P3-M3-001** ([`ui_phase3_minimap_track_naming_v1.md`](ui_phase3_minimap_track_naming_v1.md)) |
| **DESIGN-D-VFX-POST-001** | **SIGNED PASS** | [`vfx_design_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md) · closure [`fire_spark_track_closure_plan_v1.md`](../../../docs/archive/2026-06-src-dev/plans/fire_spark_track_closure_plan_v1.md) |
| **Fire pinpoint sparks (VFX)** | **ACTIVE** | [`fire_particle_spark_coder_queue_v1.md`](fire_particle_spark_coder_queue_v1.md) · [`vfx_coder_phase2_queue_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_coder_phase2_queue_v1.md) |
| **Water surface VFX** | **CLOSED** | [`water_vfx_track_closure_plan_v1.md`](../../../docs/archive/2026-06-src-dev/plans/water_vfx_track_closure_plan_v1.md) (**PLAN-WATER-TRACK-001** — do not re-queue W1/W2 foam) |

---

## Active slices (continuation queue) — **Phase 2**

| ID | Agent | Status |
|:---|:---|:---|
| **P2-VFX-VISUAL-001** | `@coder` A | **queued** — tactical zoom; `fire_spark_rows` / `water_particle_*` > 0 |
| **P2-VFX-WITNESS-001** | `@coder` B | **queued** — unit tests at `zoom_alpha ≥ 0.65` |
| **P2-FIRE-SPARK-010** | `@coder` A | **queued** — sparks above smoke overlay |
| **P2-FIRE-SPARK-011** | `@coder` A | **done** — unblocks **DESIGN-D-VFX-POST-001** |
| **DESIGN-D-VFX-POST-001** | `@designer` | **done** — after **P2-FIRE-SPARK-011** |
| **P2-WATER-POLISH-001** | `@coder` A | queued — river read / ocean tiles |
| **P2-WATER-WITNESS-002** | `@coder` B | queued — refresh water particle JSON |
| **UI-WP-LAYOUT-001** | `@coder` | **done** — D-01 unified workspace shell |
| **IND-E01** | `@coder` B | queued — industrial chain |
| **D-VFX** / VFX-POST-REVIEW | `@designer` | **done** — [`vfx_design_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_design_review_record_v1.md) **PASS** (same as **DESIGN-VFX-CAPTURE-001**) |

**Done (no rework):** FX-FIRE-SPARK-001…006 · FX-WATER-SHADER-001/002 · FX-WATER-PARTICLE-001/002

**Next primary:** **P2-VFX-VISUAL-001** + **P2-VFX-WITNESS-001** in parallel.

**Phase 2 doc:** [`vfx_coder_phase2_queue_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_coder_phase2_queue_v1.md)

---

## Documents by role

| Role | Read first |
|:---|:---|
| **@coder** | [`vfx_coder_phase2_queue_v1.md`](../../../docs/archive/2026-06-src-dev/plans/vfx_coder_phase2_queue_v1.md) · [`coder_execution_plan_v1.md`](../../../docs/archive/2026-06-src-dev/plans/coder_execution_plan_v1.md) |
| **@coder** A (render) | [`water_vfx_closure_plan_v1.md`](../../../docs/archive/2026-06-src-dev/trees/stages/water_vfx_closure_plan_v1.md) · WATER-W1-OCEAN-001, WATER-W1-RIVER-001 |
| **@coder** B (policy) | WATER-W2-FOAM-001, WATER-WITNESS-001 · fire queue § B |
| **@designer** (water) | **WATER-DESIGN-001** **done** — [`water_vfx_review_record_v1.md`](../../../docs/archive/2026-06-src-dev/plans/water_vfx_review_record_v1.md) **TUNE** |
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
| [`ui_phase2_sprint_queue.md`](../../../docs/archive/2026-06-src-dev/plans/ui_phase2_sprint_queue.md) | Phase 2 archive |
| [`ux_gpu_minimap_design_v1.md`](../../../docs/archive/2026-06-src-dev/plans/ux_gpu_minimap_design_v1.md) | UX-A design |
| [`post_stage6_active_todos.md`](../../../src/dev/post_stage6_active_todos.md) | Product board |
