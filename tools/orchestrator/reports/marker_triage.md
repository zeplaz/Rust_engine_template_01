# Marker triage (A-06)

| Kind | File | Line | Notes |
|------|------|-----:|-------|
| DEPRECATED | `src/bevysubengines/world_generator_plugin.rs` | 170 | #[deprecated(note = "Use terrain::family::TerrainFamilyId")] |
| VIEWPORT_AUTHORITY | `src/dev/debug_run_envelope.rs` | 71 | "VIEWPORT_AUTHORITY_DEBUG": env_flag("VIEWPORT_AUTHORITY_DEBUG"), |
| TODO | `src/dev/stage5_live_todos.rs` | 13 | //! 1. **TODO-01** — runtime readiness fence (`evaluate_app_stage5_readiness`).  |
| TODO | `src/dev/stage5_live_todos.rs` | 14 | //! 2. **TODO-04** — view authority (ViewManager vs `MapCameraDesired`). Real en |
| TODO | `src/dev/stage5_live_todos.rs` | 15 | //! 3. **TODO-06** — `CommittedVisualSnapshotFence` / frame sync. Unlocks GPU +  |
| TODO | `src/dev/stage5_live_todos.rs` | 17 | //! Then: TODO-02, TODO-03, TODO-05, GPU layer (07–09), fire (10–11), preview/LO |
| TODO | `src/dev/stage5_live_todos.rs` | 42 | /// Post-mirror pose delta threshold (translation + zoom) for TODO-04 / TODO-05  |
| TODO | `src/dev/stage5_live_todos.rs` | 44 | /// Consecutive frames under [`MAP_BRIDGE_DRIFT_OK`] with a live WorldMain view  |
| TODO | `src/dev/stage5_live_todos.rs` | 83 | id: "TODO-01", |
| TODO | `src/dev/stage5_live_todos.rs` | 92 | id: "TODO-02", |
| TODO | `src/dev/stage5_live_todos.rs` | 101 | id: "TODO-03", |
| TODO | `src/dev/stage5_live_todos.rs` | 110 | id: "TODO-04", |
| TODO | `src/dev/stage5_live_todos.rs` | 119 | id: "TODO-05", |
| TODO | `src/dev/stage5_live_todos.rs` | 128 | id: "TODO-06", |
| TODO | `src/dev/stage5_live_todos.rs` | 137 | id: "TODO-07", |
| TODO | `src/dev/stage5_live_todos.rs` | 146 | id: "TODO-08", |
| TODO | `src/dev/stage5_live_todos.rs` | 155 | id: "TODO-09", |
| TODO | `src/dev/stage5_live_todos.rs` | 164 | id: "TODO-10", |
| TODO | `src/dev/stage5_live_todos.rs` | 173 | id: "TODO-11", |
| TODO | `src/dev/stage5_live_todos.rs` | 182 | id: "TODO-12", |
| TODO | `src/dev/stage5_live_todos.rs` | 191 | id: "TODO-13", |
| TODO | `src/dev/stage5_live_todos.rs` | 201 | /// Root gates: **TODO-01 → TODO-04 → TODO-06** before treating GPU / fire / LOD |
| TODO | `src/dev/stage5_live_todos.rs` | 202 | pub const STAGE5_ROOT_GATE_SEQUENCE: &[&str] = &["TODO-01", "TODO-04", "TODO-06" |
| TODO | `src/dev/stage5_live_todos.rs` | 356 | "TODO-01" => ctx.inv >= 1, |
| TODO | `src/dev/stage5_live_todos.rs` | 357 | "TODO-02" => ctx.report.violations.is_empty(), |
| TODO | `src/dev/stage5_live_todos.rs` | 358 | "TODO-03" => ctx.report.projection_domains >= 3, |
| TODO | `src/dev/stage5_live_todos.rs` | 359 | "TODO-04" => ctx.vm_wm && ctx.bridge.last_post_mirror_drift <= MAP_BRIDGE_DRIFT_ |
| TODO | `src/dev/stage5_live_todos.rs` | 360 | "TODO-05" => { |
| TODO | `src/dev/stage5_live_todos.rs` | 363 | "TODO-06" => ctx.fence_ok, |
| TODO | `src/dev/stage5_live_todos.rs` | 364 | "TODO-07" => ctx.report.instanced_dispatch_ok, |
| TODO | `src/dev/stage5_live_todos.rs` | 365 | "TODO-08" => ctx.report.gpu_field_authoritative, |
| TODO | `src/dev/stage5_live_todos.rs` | 366 | "TODO-09" => ctx.report.overlay_from_shared_buffers_only, |
| TODO | `src/dev/stage5_live_todos.rs` | 367 | "TODO-10" => ctx.report.single_fire_extract, |
| TODO | `src/dev/stage5_live_todos.rs` | 368 | "TODO-11" => ctx.fire_w.world_main_visible_orphan_chunks == 0, |
| TODO | `src/dev/stage5_live_todos.rs` | 369 | "TODO-12" => ctx.report.phase_d_ok && ctx.report.preview_render_target_active, |
| TODO | `src/dev/stage5_live_todos.rs` | 370 | "TODO-13" => ctx.lod_w.lod_band_log_emissions >= 1, |
| TODO | `src/dev/stage5_live_todos.rs` | 764 | "STAGE5_TODOS must list TODO-01 through TODO-13" |
| TODO | `src/dev/stage5_live_todos.rs` | 791 | let i4 = STAGE5_TODOS.iter().position(/t/ t.id == "TODO-04").unwrap(); |
| TODO | `src/dev/stage5_live_todos.rs` | 792 | let i5 = STAGE5_TODOS.iter().position(/t/ t.id == "TODO-05").unwrap(); |
| TODO | `src/dev/stage5_live_todos.rs` | 807 | let i11 = STAGE5_TODOS.iter().position(/t/ t.id == "TODO-11").unwrap(); |
| TODO | `src/dev/stage5_live_todos.rs` | 820 | let i13 = STAGE5_TODOS.iter().position(/t/ t.id == "TODO-13").unwrap(); |
| TODO | `src/dev/stage5_live_todos.rs` | 850 | let i13 = STAGE5_TODOS.iter().position(/t/ t.id == "TODO-13").unwrap(); |
| DEPRECATED | `src/engine/lmodels/mod.rs` | 19 | #[deprecated( |
| TODO | `src/entities/production/core/manufacturing_plugin.rs` | 39 | // TODO: drive throughput vs blueprint, decay/efficiency curves, alert events. |
| TODO | `src/gui/diagnostics_ui.rs` | 560 | // TODO: tabs — chunk streamer, production manifest summary, faction roster. |
| TODO | `src/gui/faction_tools_ui.rs` | 91 | // TODO: list + add/duplicate/retire (authority-gated). |
| TODO | `src/gui/faction_tools_ui.rs` | 95 | // TODO: bind selected FactionBlueprint fields. |
| TODO | `src/gui/faction_tools_ui.rs` | 99 | // TODO: render N×N stance grid; integrate DiplomaticRelations permission gate. |
| TODO | `src/gui/faction_tools_ui.rs` | 103 | // TODO: file dialog (crate vs native — see implementation_questions §7). |
| VIEWPORT_AUTHORITY | `src/gui/hud/mod.rs` | 155 | VIEWPORT_AUTHORITY_TARGET, |
| VIEWPORT_AUTHORITY | `src/gui/hud/viewport_authority_debug.rs` | 12 | pub const VIEWPORT_AUTHORITY_TARGET: &str = "viewport_authority"; |
| VIEWPORT_AUTHORITY | `src/gui/hud/viewport_authority_debug.rs` | 146 | target: VIEWPORT_AUTHORITY_TARGET, |
| VIEWPORT_AUTHORITY | `src/gui/hud/viewport_authority_debug.rs` | 153 | "VIEWPORT_AUTHORITY" |
| TODO | `src/gui/in_game_ui.rs` | 8 | // TODO: rewrite with: |
| TODO | `src/gui/map_camera.rs` | 116 | /// Live Stage 5 audit (TODO-04): log **mutations** to [`MapCameraDesired`] when |
| TODO | `src/gui/representation_governance.rs` | 55 | /// Fix priority when building a cycle TODO queue from FULL_APP failures (direct |
| TODO | `src/render/stage5_closure_witnesses.rs` | 2 | //! TODO-01…TODO-13 [`TodoStatus::Done`] only when that row’s predicate is satis |
| DEPRECATED | `src/strategic/behavior_plugin.rs` | 12 | #[deprecated( |
| TODO | `src/systems/damage/damage_system.rs` | 22 | // TODO: apply damage accumulation logic here |
| TODO | `src/systems/navigation/potental_feild_nav.rs` | 60 | // TODO: replace default with entity-derived owner when AgentOwnable is on the v |
| TODO | `src/systems/production/serialization.rs` | 11 | // TODO: register `ConcreteProductionConfig` load/save from RON/JSON. |
| TODO | `src/systems/production/serialization.rs` | 20 | // TODO: register `AluminumProductionConfig` persistence. |
| TODO | `src/systems/production/serialization.rs` | 29 | // TODO: persist substation graph edges + plant specs as serializable DTOs. |
| DEPRECATED | `src/terrain/bevy_terrain.rs` | 21 | #[deprecated(note = "Use terrain::family::TerrainFamilyId for dominant terrain f |
| TODO | `src/terrain/generation/passes/p5_agent_overlay.rs` | 6 | // TODO: items 71–74 (`implementation_questions_v1.md`) |
| DEPRECATED | `src/terrain/generation/world_generator_enhanced.rs` | 441 | #[deprecated(note = "Use terrain::family::TerrainFamilyId")] |

## Triage policy

- `VIEWPORT_AUTHORITY` / `MIGRATION` → do not auto-clean
- `TODO` / `FIXME` → link to STAGE5 or continuation queue
- `HACK` / `TEMP` → requires owner in knowledge JSON
