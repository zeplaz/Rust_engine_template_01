# PLAN-EGUI-GAME-RETIRE-001 — retire egui from the playing game `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-EGUI-GAME-RETIRE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-09-26 |
| **Owner** | `@planner` (this file) · impl → `@coder` |
| **Status** | **PLN-1..3 SUPERSEDED** (2026-10-11). UPM-1..5 / city-bug / EGR ★done. **PLN-1** (plugin-pass) **SUPERSEDED** by [`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md) (`bevy_egui` removed). **PLN-2 / PLN-3** queue rows **SUPERSEDED** (¬re-queue from this plan). Sim mesh/pointer / one-map tile rules below still apply. |
| **Supersedes the shell-perf claim in** | [`ui_phase6_shell_perf_multiview_plan_v1.md`](ui_phase6_shell_perf_multiview_plan_v1.md) P6-1 only |
| **Witness spec extended by** | [`ui_shell_witness_spec_v1.md`](ui_shell_witness_spec_v1.md) |
| **Index codes** | GU-SHL · GU-HUD · GU-APP · CB-TOOL · RN-MMC |
| **Locked files (other agent)** | `src/construction/world_map_frame.rs` · `src/construction/mod.rs` |

Historical framing (pre Cargo drop): editor/world-gen kept egui; play was Bevy. **Cargo path SUPERSEDED** by [`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md) (`bevy_egui` removed). The playing session is `BaseState::Simulation`. The GPU minimap compositor stays the minimap. The world camera stays the world camera. No second camera, no second minimap, no new UI crate.

---

## Summary

`product_egui_shell_active` is already the editor shell (`BaseState::Editor` and not `AppState::WorldGen`). The sim product path is Bevy UI in `simulation_shell_phase2` (GU-SHL). That split is real for the dock, the side rail, the pause menu, and the GPU minimap.

It is not real for the egui schedule. `egui_pass_count_in_sim` only increments inside `hud_product_shell_egui_root`, which is already gated off Simulation. Phase 6 can print `0` while other systems still run on `EguiPrimaryContextPass` because their `run_if` is `in_simulation_or_editor`, or because they have no `run_if` at all.

World paint of roads, rails, zones, and construction footprints was just moved behind `product_egui_shell_active`. In Simulation those strokes no longer draw. Buildings and sites already have world-space `Mesh2d` quads on the terrain sprite (`sync_world_map_footprints`). Power lines and logistics hauls still paint with egui screen strokes during play. Chrome sheets that already have Bevy widgets still enter the egui pass and return. Chrome that is still the only panel (power repair, plant card, power hover, hidden-OS crosshair) still paints.

Retirement means: every `EguiPrimaryContextPass` system whose condition is true in Simulation either moves to an editor/world-gen gate, or is replaced first by the Bevy shell (chrome) or by a mesh on the existing terrain plane (world geometry), then leaves the sim schedule. The request buffers stay the authority. The mesh writers consume them. The editor painters may keep reading them.

```text
⊚BuildPlanningPlugin Update chain ═▶ ⊨ ConstructionVisualRequests
   (clear → roads → rails → zones → footprints → place-feedback → site stub → corridor)
⊚InfrastructureOverlayPlugin Update ═▶ ⊨ InfrastructureOverlayDrawRequests
   + PowerMapOverlayPresentation
⊚LogisticsIntentDrawFrameRes ═▶ ⊨ haul / site outline snapshot (PostUpdate sync already exists)
⊚sync_world_map_footprints ═▶ ⊨ sim footprint Mesh2d  (SHIPPED, locked file)
⊚WorldMapNetworkMeshPlugin ═▶ ⊨ sim path / zone / power / logistics meshes
   ⊰ those snapshots · same world camera · same terrain plane
⊚simulation_shell_phase2 Bevy Nodes ═▶ ⊨ playing-game chrome
⊚EguiPrimaryContextPass ═▶ ⊨ editor shell + world-gen chrome + map editor
   Simulation acquires no egui context
⊚MinimapCompositor ═▶ ⊨ minimap  (untouched)
```

---

## Current problems

1. **Counter vs schedule.** `ProductShellDiagnostics::record_egui_pass` is called from `hud_product_shell_egui_root` (`hud_root_tick.rs`). The 2B formula in the witness spec can be green while the pass still runs in play.
2. **`in_simulation_or_editor` is the sim leak.** `ui_gates.rs`: that predicate is Simulation or Editor. `product_egui_shell_active` is Editor and not WorldGen. Sheets left on the first predicate still run in play.
3. **World painters left the play session without a mesh.** `draw_construction_visual_requests_egui` is editor-gated. Roads, rails, zones, corridor paths, and phase words therefore have no sim pixels. `sync_*_visual_requests` still fills the buffer every play frame.
4. **Two world paints are still egui, and they are sim-only.** `draw_power_map_overlay_egui` returns unless `BaseState::Simulation`. `draw_logistics_intent_overlay_egui` paints in Simulation and Editor. Removing them before a mesh exists removes the only picture of those networks.
5. **Footprint GPU witness is not a mesh.** `footprint_tile_instances` records `egui_path_active` and sets `gpu_path_active = false`. The live mesh is `world_map_frame`, one material, cells only.
6. **Dead egui painters must not be reattached.** `draw_road_path_ghost_egui`, `draw_rail_path_ghost_egui`, and `draw_zone_paint_ghost_egui` are exported and unscheduled. Rescheduling them would put screen quads back on the play map.
7. **Dev windows share the play pass.** F-key panels use `in_simulation_or_editor` or no gate. A visible flag still means the system runs, and several call `ctx_mut` before the flag.
8. **`construction/mod.rs` and `world_map_frame.rs` are owned.** Schedule edits and footprint-mesh edits wait. New geometry registers beside `BuildPlanningPlugin` from `src/engine/engine_with_worldgen.rs`.

---

## Target architecture

Playing game, one camera (`MainWorldCamera` / `ViewProjectionAuthority`). Zoom is that camera. A cell center is `world_map_tile_center` on the terrain sprite plane. UI does not own that plane.

| Surface | Authority | Playing-game presentation | Editor presentation |
|:---|:---|:---|:---|
| Build picker, road sheet, power sheet, build tray | existing Bevy shell resources | Bevy `Node` (already) | egui may remain |
| Power-repair list, plant card, node hover, crosshair | same snapshots the egui fns read today | Bevy `Node` in GU-SHL | egui may remain |
| Building / site cells | `ConstructionVisualRequests.footprint_tiles` + `SiteFootprint` | `Mesh2d` in `world_map_frame` | egui painter |
| Roads, rails, zones, corridor | `ConstructionVisualRequests` | new world meshes | egui painter stays legal |
| Power graph + placement preview | overlay draw requests + preview segments | world meshes on the same plane | do not add an editor-only egui clone as the sim fix |
| Logistics hauls | `LogisticsIntentDrawFrameRes` | world meshes | egui painter may remain for Editor |
| Minimap | `MinimapCompositor` (RN-MMC) | GPU composite + existing Bevy chrome | egui minimap window stays editor |
| Dev F-keys, permissions, keybindings | existing state resources | absent in Simulation | egui windows |

Layer order stays Sim → View → Render → UI. Mesh spawn reads the request buffer after the Update chain that clears and refills it. It does not write sim state. Bevy chrome reads the same panel snapshots the egui sheets read. It does not commit build orders by a second path; the existing click systems stay the commit.

`ZoneTileRequest.color` is `egui::Color32` today. The mesh writer converts that RGBA to `bevy::prelude::Color` at the spawn boundary. The request struct stays until a later cleanup so the editor painter keeps compiling.

---

## Inventory — systems that run in Simulation

“Runs” means the schedule condition is true in `BaseState::Simulation`. “Paints” means the body calls `ctx_mut` on the play path a player can hit. Early return still counts as a running system.

### A. Product chrome (Bevy shell)

| Status | File · function | Class | What it does in play | Replacement |
|:---|:---|:---|:---|:---|
| **SHIPPED** widget · **PLANNED** unschedule | `src/gui/hud/sim_build_picker_sheet.rs` · `draw_sim_build_picker_sheet_egui` | chrome | Runs. Returns while `SIM_BUILD_PICKER_USE_BEVY` | Keep `sim_build_picker_bevy`. Drop the system from the sim pass. Do not delete the Bevy picker. |
| **SHIPPED** widget · **PLANNED** unschedule | `src/gui/hud/sim_road_tool_sheet.rs` · `draw_sim_road_tool_sheet_egui` | chrome | Runs. Sim returns while `SIM_ROAD_POWER_SHEETS_USE_BEVY`. Editor branch calls `draw_road_tool_popup_egui` | Keep `sim_road_power_sheets_bevy`. Editor popup stays on the editor gate. |
| **SHIPPED** widget · **PLANNED** unschedule | `src/gui/hud/sim_power_tool_sheet.rs` · `draw_sim_power_tool_sheet_egui` | chrome | Same flag | Same Bevy sheet module. |
| **SHIPPED** widget · **PLANNED** unschedule | `src/gui/hud/context_tray_build_egui.rs` · `draw_context_tray_build_body_egui` | chrome | Runs. Returns while `CONTEXT_TRAY_BUILD_USE_BEVY` | Keep `ContextTrayBodyLine` / `ContextTrayBodyRoot`. |
| **PLANNED** | `src/gui/hud/context_tray_power_repair_egui.rs` · `draw_context_tray_power_repair_egui` | chrome | **Paints** when the Logistics tray is open | Extend the existing context-tray Bevy body. Unschedule egui after that body lists `PowerRepairQueue`. |
| **PLANNED** | `src/gui/hud/plant_focus_card.rs` · `draw_plant_focus_card_egui` | chrome | **Paints** when `PlantFocusCardSnapshot.visible` and power hover is hidden | Bevy `Node` fed by the snapshot `sync_plant_focus_card_visibility` already writes. |
| **PLANNED** | `src/gui/hud/power_node_hover_egui.rs` · `draw_power_node_hover_egui` | chrome | **Paints** when `PowerNodeHoverState.card_visible` | Bevy `Node` on the sim shell, placed with the existing map projection (`world_to_sim_map` data the egui card already uses). |
| **PLANNED** | `src/gui/hud/simulation_pointer_gate.rs` · `draw_simulation_unified_cursor_egui_system` | chrome | **Paints** when the OS cursor is hidden | Bevy `Node` crosshair. Position is `SimulationMapPointerGate.cursor`. `apply_simulation_unified_cursor_system` stays in Update. |
| **PLANNED** unschedule | `src/construction/staged_ghost_panel.rs` · `draw_staged_placements_panel_egui_system` | chrome | Runs. Returns in Simulation (tray owns staging) | Editor gate only. Tray copy stays Bevy. |
| **PLANNED** unschedule | `src/construction/tool_hints.rs` · `draw_tool_hints_egui` | chrome | Runs. Returns in Simulation | Editor gate only. Sim hints stay on the tray. |

These ten are inside `BuildPlanningPlugin`’s `EguiPrimaryContextPass` tuple (`.run_if(in_simulation_or_editor)`), except the cursor, which is its own system in `simulation_shell_phase2` with the same predicate. Moving them off Simulation edits `construction/mod.rs` for the nine construction rows. **That edit waits until the world-map owner releases `construction/mod.rs`.** Until then the four flagged sheets are already invisible; the four painters in the “Paints” rows stay, because they are the live UI.

### B. World geometry that still paints with egui in play

| Status | File · function | Class | What it does in play | Replacement |
|:---|:---|:---|:---|:---|
| **PLANNED** mesh, then unschedule | `src/render/pipelines/power_map_overlay_draw.rs` · `draw_power_map_overlay_egui` | world paint | **Paints** in Simulation when power overlay or island highlight is on. Also strokes `preview_segments`. No `run_if` on `InfrastructureOverlayPlugin` | World-space line meshes on the terrain plane from `InfrastructureOverlayDrawRequests` (power edges) and `PowerMapOverlayPresentation.preview_segments`. Damage glyphs become small world meshes or `Text2d` on that plane. Then remove the system from the sim pass. |
| **PLANNED** mesh, then unschedule | `src/economy/logistics/intent_draw.rs` · `draw_logistics_intent_overlay_egui` | world paint | **Paints** in Simulation and Editor when haul polylines or AI site outlines exist. No `run_if` | World meshes from `LogisticsIntentDrawFrameRes` (already built in PostUpdate). Sim stops calling egui. Editor may keep the function behind `product_egui_shell_active`. |

`draw_power_strokes_on_minimap` is called from `draw_simulation_minimap_egui` inside the product shell root. That root is editor-only. Leave minimap power strokes there. Do not add a sim egui minimap layer. The compositor is not redesigned in this plan; world meshes are for the main camera.

### C. Dev / options windows that run in play

Default visibility is off. The system still runs. Opening the panel paints egui during Simulation.

| Status | File · function | Class | Gate today | Replacement |
|:---|:---|:---|:---|:---|
| **PLANNED** gate | `src/gui/diagnostics_ui.rs` · `diagnostics_ui_system` | chrome (dev) | `in_simulation_or_editor`; paints if `visible` | `product_egui_shell_active`. Editor keeps F3. |
| **PLANNED** gate | `src/gui/faction_tools_ui.rs` · `faction_tools_ui_system` | chrome (dev) | same | Editor gate. |
| **PLANNED** gate | `src/gui/ai_explainability_ui.rs` · `ai_explainability_egui_system` | chrome (dev) | same | Editor gate. |
| **PLANNED** gate | `src/gui/pressure_tooling.rs` · `pressure_composer_egui_system` | chrome (dev) | same | Editor gate. Bevy text sync (`sync_pressure_bevy_overlay_text`) stays if a sim readout is still required. |
| **PLANNED** gate | `src/gui/logistics_targets_panel.rs` · `logistics_targets_panel_ui` | chrome (dev) | same | Editor gate. Focus cycling in Update can stay. |
| **PLANNED** gate | `src/gui/agent_permissions_ui.rs` · `permissions_ui_system` | chrome (dev) | **no `run_if`** | Editor gate. |
| **PLANNED** gate | `src/gui/options_keybindings_ui.rs` · `keybindings_options_ui` | chrome | **no `run_if`**; paints if `visible` | Editor gate. Binding capture in Update can stay. A play-session keybindings sheet, if product needs one later, is a Bevy page on the existing pause menu — **DEFER** that page. |
| **PLANNED** gate | `src/gui/settlement_block_frame_debug.rs` · `draw_block_frame_debug_overlay` | world paint (debug) | **no `run_if`**. Calls `ctx_mut` before `block_frame_debug_may_draw`. Simulation draws when env-armed and visible | Editor gate on the schedule so Simulation never acquires the context. |
| **PLANNED** gate | `src/render/probes/debug_viewport_overlay.rs` · `draw_debug_viewport_overlay` | chrome (debug) | **no `run_if`**. Paints when the env flag is set | Editor gate, or the same env flag AND not Simulation. |
| **PLANNED** gate | `src/gui/assembly_snapshot_qc_ui.rs` · `draw_assembly_snapshot_qc_ui` | chrome (art QC) | **no `run_if`**. Body returns unless `product_egui_shell_active` | Put that predicate on the schedule. |
| **PLANNED** gate | `src/bevysubengines/world_generator_plugin.rs` · `world_gen_ui_system` | chrome (world-gen duplicate) | **no `run_if`**. Paints when `ui_visible`, including play | Gate with `world_gen_ui_chrome_visible` (or Editor/WorldGen only). Canonical generator UI stays `gui/editor/world_gen_ui.rs`. Collapsing the duplicate window is **DEFER**. |

### D. On the egui pass in play, not a draw

| Status | File · function | Note |
|:---|:---|:---|
| **PLANNED** move | `src/gui/map_view/presentation/stability.rs` · `sync_shell_layout_drag_gate` | Calls `ctx_mut` every frame. No `run_if`. In play, shell drag comes from Bevy `Interaction` on the sim shell. Keep the egui read behind `product_egui_shell_active`. |
| **PLANNED** move | `src/gui/map_view/presentation/stability.rs` · `clear_active_map_view_input_before_map_widgets` | No egui context. Move to Update. |
| **PLANNED** move | `src/gui/map_presentation_diagnostics.rs` · `sync_map_fit_transform_components` | No egui context. Scheduled on the egui pass with `validate_map_fit_system`. Move both to Update. |
| **PLANNED** move | `src/gui/map_view/debug/map_fit_validator.rs` · `validate_map_fit_system` | Same. |

---

## Inventory — already off Simulation (keep)

Do not unschedule these from the editor. Do not point them at Simulation to “get the lines back.”

| Status | File · function | Class | Gate |
|:---|:---|:---|:---|
| **SHIPPED** | `src/gui/hud/hud_root_tick.rs` · `hud_product_shell_egui_root` | chrome root | `product_egui_shell_active` |
| **SHIPPED** | children of that root: `draw_hud_overlay_tray_egui`, `draw_hud_command_shell_egui`, `draw_hud_dock_minimized_strip_egui`, `draw_hud_side_status_panel_egui`, `draw_simulation_minimap_egui`, `draw_transmission_shell_egui`, `draw_overlay_shell_egui`, `draw_stage7_ui_shell_egui`, `draw_pending_construction_queue_egui`, `draw_organic_growth_inspector_egui`, `draw_hud_dev_overlay_egui` | chrome | same root |
| **SHIPPED** | `simulation_shell_phase2` · `finalize_simulation_map_pointer_gate_egui_system`, `sync_minimap_chrome_root_system`, `sync_minimap_chrome_layout_system`, `minimap_bevy_interaction::draw_simulation_minimap_topology_legend_egui_system` | chrome | `minimap_chrome_egui_sync_active` is false in Simulation. PostUpdate Bevy minimap chrome is the play path. |
| **SHIPPED** | `src/gui/in_game_pause_menu.rs` | chrome | Bevy pause menu. No egui draw. |
| **SHIPPED** | `src/construction/build_toolbox.rs` · `draw_build_toolbox_egui` | chrome | `product_egui_shell_active` (also calls `draw_r4_corridor_tray_legend`) |
| **SHIPPED** | `src/construction/placement_debug.rs` · `draw_construction_placement_debug_overlay` | chrome (debug) | `product_egui_shell_active` |
| **SHIPPED** gate · **PLANNED** sim mesh | `src/construction/visual_authority.rs` · `draw_construction_visual_requests_egui` | world paint | Editor. Draws paths, zone tiles, control points, footprints, corridor paths (`draw_corridor_phase_paths`), lock ring. Sim replacement is the network mesh plugin, reading the same buffer. |
| **SHIPPED** gate · **PLANNED** sim `Text2d` | `src/construction/phase_visual.rs` · `draw_construction_phase_labels_egui` | world labels | Editor. Sim labels are `Text2d` on the terrain plane, fed by `ConstructionSite` phase. |
| **SHIPPED** gate | `src/construction/build_footprint_overlay.rs` · `build_footprint_validity_overlay_egui` | world paint | Editor. Sim cells are the footprint mesh. |
| **SHIPPED** gate | `src/construction/power_lines/ghost.rs` · `draw_power_line_path_ghost_egui` | world paint | Editor. Sim preview is `preview_segments` → power mesh (section B). |
| **SHIPPED** gate | `src/construction/zones/ghost.rs` · `draw_admin_zone_ink_egui` | world paint | `product_egui_shell_active` only (separate system). Sim admin ink is the zone mesh. |
| **SHIPPED** | `src/gui/editor/map_editor/mod.rs` · `map_editor_minimap_window`, `map_editor_palette_system`, `scenario_editor_tools_entry_window`, `scenario_script_panel_system` | editor chrome | `map_editor_chrome_active` |
| **SHIPPED** | `src/gui/editor/world_preview/plugin.rs` · `display_world_preview` | world-gen chrome | `world_preview_chrome_active` |
| **SHIPPED** | `src/gui/editor/world_gen_ui.rs` · `world_gen_ui_system` | world-gen chrome | `world_gen_ui_chrome_visible` |

### Unscheduled egui draws — do not add them to Simulation

| Status | File · function | Class | Fate |
|:---|:---|:---|:---|
| **DEFER** delete | `src/construction/roads/ghost.rs` · `draw_road_path_ghost_egui` | world paint | Unscheduled. Buffer path is `sync_road_visual_requests`. |
| **DEFER** delete | `src/construction/rail/ghost.rs` · `draw_rail_path_ghost_egui` | world paint | Unscheduled. Buffer path is `sync_rail_visual_requests`. |
| **DEFER** delete | `src/construction/zones/ghost.rs` · `draw_zone_paint_ghost_egui` | world paint | Unscheduled. Buffer path is `sync_zone_visual_requests`. |
| **DEFER** delete | picker submenus `draw_residential_submenu`, `draw_industrial_submenu`, `draw_utilities_submenu`, `draw_commercial_submenu`, `draw_mock_shapes_submenu`, `draw_sim_build_rail_submenus_egui` | chrome | Called from the egui picker, which returns under the Bevy flag. |
| **DEFER** delete | `src/construction/roads/popup.rs` · `draw_road_tool_popup_egui` | chrome | Editor branch of the road sheet only. |
| **SHIPPED** | `src/gui/hud/minimap_topology_legend.rs` · `draw_minimap_topology_legend_gpu_chrome` | chrome | Bevy/GPU minimap legend. The egui twin stays behind the minimap egui sync gate. |

### Buildings already on the terrain plane

| Status | File · function | Class |
|:---|:---|:---|
| **SHIPPED** | `src/construction/world_map_frame.rs` · `sync_world_map_footprints` | World `Mesh2d` quads, one material, cells from footprint requests and `SiteFootprint`. Simulation only. Despawns those entities in other states. **This plan does not edit that file.** Validity colors, lock rings, and phase words are the sibling work in EGR-4a after the lock lifts, or `Text2d` / extra materials in the network plugin if they use a different component so they do not dual-write `WorldMapFootprintTile`. |

---

## World geometry contract (roads, rails, zones, power, buildings)

One plugin, new file, not a UI overlay:

`src/render/world_map_network_mesh.rs` · `WorldMapNetworkMeshPlugin`  
Registered in `src/engine/engine_with_worldgen.rs` beside `BuildPlanningPlugin`.  
No edit to `construction/mod.rs` or `world_map_frame.rs` while those files are locked.

| Mesh set | Component (new) | Reads | Plane |
|:---|:---|:---|:---|
| Road ribbons | `WorldMapRoadSeg` | `ConstructionVisualRequests.paths` where `VisualPathKind::Road`, including committed (`sync_executed_road_paths`) and the live ghost | `world_map_tile_center` / segment endpoints already in world XZ. Y slightly above the terrain sprite so the world camera sees them. Width from `PathLineRequest.width`. Valid / committed pick `bevy::Color` next to the existing `ghost_visual` tokens. |
| Rail ribbons | `WorldMapRailSeg` | `paths` where `VisualPathKind::Rail` | Same. |
| Zone cells | `WorldMapZoneCell` | `zone_tiles` (paint ghost, site stub, road-tile fallback quads already pushed as zone tiles) | Cell center is `ZoneTileRequest.center`. Color converted from `egui::Color32` at spawn. |
| Corridor | `WorldMapCorridorSeg` | `corridor_paths` | Same ribbon as roads, corridor material. |
| Power | `WorldMapPowerSeg` | power edges in `InfrastructureOverlayDrawRequests` + `preview_segments` | Endpoints are world positions the egui painter already projects. Skip the screen projection. |
| Logistics | `WorldMapLogisticsSeg` | `LogisticsIntentDrawFrameRes` | Same. |
| Phase word | `WorldMapPhaseLabel` | site phase on `ConstructionSite` | `Text2d` at the footprint center. |

Single writer per component. `sync_world_map_footprints` keeps `WorldMapFootprintTile`. The network plugin does not spawn that component.

Order in Update, after the existing chain that begins with `clear_construction_visual_requests`:

```text
clear_construction_visual_requests
  → sync_road / rail / zone / footprint / place_feedback / site_stub / corridor
  → collect_infrastructure_overlay_edges + sync_power_line_preview_overlay
  → sync_logistics_intent_draw_frame (PostUpdate today; mesh reads the frame next frame or moves nothing if the mesh system is PostUpdate .after that sync)
  → WorldMapNetworkMeshPlugin sync (spawn / despawn only the delta key)
```

The mesh system runs in Simulation. In Editor it despawns its entities so the editor egui painter is the only picture there. That matches `sync_world_map_footprints`.

Rebuild when a content key changes (segment count + quantized endpoints + color kind), the same stamp pattern as the footprint mesh. No per-frame entity churn on a still map.

---

## Implementation phases

Each phase is playable when it finishes. A Bevy widget or a world mesh exists before the egui system that painted the same thing leaves Simulation.

### EGR-0 — **SHIPPED** (do not redo)

Editor shell gate, Bevy pause menu, Bevy build picker, Bevy road/power sheets, Bevy build-tray body, GPU minimap compositor, PostUpdate minimap chrome, world-painter `run_if(product_egui_shell_active)`, footprint `Mesh2d` on the terrain sprite.

Acceptance already in tree: `product_egui_shell_base_active(Simulation) == false`, `SIM_BUILD_PICKER_USE_BEVY`, `SIM_ROAD_POWER_SHEETS_USE_BEVY`, `CONTEXT_TRAY_BUILD_USE_BEVY`.

### EGR-1 — **PLANNED** witness

**Goal.** The sim witness counts egui context acquisition in play, not only the product-shell counter.

**Files.** `src/gui/hud/shell_diagnostics.rs`, `src/gui/hud/simulation_shell_phase2.rs` (the writer `write_ui_shell_migration_live_proof`), `src/dev/ui_shell_witness_spec_v1.md` (one new field, spec owner may fold it later).

**Field.** `sim_egui_context_acquires` on `debug_runs/ui_shell_migration_live.json`. Increment when a system in Simulation successfully calls `EguiContexts::ctx_mut`. Required value at exit: `0`. Keep `egui_pass_count_in_sim` as the historical shell counter so old proofs still parse.

**Authority.** `ProductShellDiagnostics` remains the only counter writer. ⊚proof JSON ⊰ that resource.

**Rollback.** Drop the new field if the increment site double-counts. Do not change the old formula until EGR-6.

### EGR-2 — **PLANNED** drop no-op sheets from the sim pass

**Blocked on** release of `src/construction/mod.rs`.

**Goal.** Systems that already return under a Bevy flag, or that already return in Simulation, lose the Simulation predicate.

**Rows.** `draw_sim_build_picker_sheet_egui`, `draw_sim_road_tool_sheet_egui`, `draw_sim_power_tool_sheet_egui`, `draw_context_tray_build_body_egui`, `draw_staged_placements_panel_egui_system`, `draw_tool_hints_egui`.

**Gate after.** `product_egui_shell_active` (editor). The road sheet’s editor branch keeps `draw_road_tool_popup_egui`.

**Playability.** Bevy picker, sheets, and tray stay scheduled in Update (`simulation_shell_phase2` already does this). Do not delete those modules. Do not delete the egui functions in this slice (witness `include_str!` greps still name them).

**Acceptance.** In a sim-state schedule dump, those six names are absent from `EguiPrimaryContextPass`. `cargo test -p proc_A_dine01 --lib sim_build_picker` and the road/power sheet tests stay green.

### EGR-3 — **PLANNED** Bevy chrome, then unschedule (strict order)

3a **Power repairs** before anything else in this phase, because it paints whenever the Logistics tray is open. Add rows to the existing context tray (`ContextTrayBodyLine` pattern in `simulation_shell_phase2`). Source data: `PowerRepairQueue`, `PowerLineDamageBook`. Then change `draw_context_tray_power_repair_egui` to the editor gate (same `mod.rs` lock as EGR-2; the Bevy body can land first in `src/gui/hud/`).

3b **Plant card.** `PlantFocusCardSnapshot` is already filled. Spawn a shell `Node`. Hide it when `PowerNodeHoverState.card_visible` is the same rule the egui card uses. Then editor-gate the egui function.

3c **Power hover.** Same shell, fed by `PowerNodeHoverState`. Screen position from the current map projection helper, converted to shell pixels. Then editor-gate `draw_power_node_hover_egui`.

3d **Crosshair.** Shell `Node` at `gate.cursor` while `os_cursor_visible` is false. Then remove `draw_simulation_unified_cursor_egui_system` from the sim pass (`simulation_shell_phase2` is not the locked file).

**Authority.** One Bevy parent per card under the sim shell root that already exists. The egui function is not a second writer of `PowerRepairQueue` or the hover snapshot; those resources stay owned by their current sim systems.

**Rollback.** If a Bevy card misses a field the player uses (repair “queue all”, plant diesel hours), leave the egui function on the sim predicate until the field exists. Do not run both in Simulation (two copies of the same card).

### EGR-4 — **PLANNED** world meshes, then unschedule world egui

Land the plugin in `src/render/world_map_network_mesh.rs` and register it from `engine_with_worldgen.rs`.

4e **Power first.** It is the world paint players still see. Mesh live in play, then `draw_power_map_overlay_egui` leaves `EguiPrimaryContextPass` (plugin file `infrastructure_overlay.rs`, not the locked construction module).

4f **Logistics** next, same rule, in `economy/logistics/mod.rs`.

4b **Roads**, 4c **Rails**, 4d **Zones**, corridor segments. These are dark in play today. Adding meshes makes them visible. Do not re-enable `draw_construction_visual_requests_egui` in Simulation to “bridge” the gap.

4a **Footprint color, lock ring, phase `Text2d`.** After the world-map owner releases `world_map_frame.rs`, that owner adds color kinds on the existing footprint entities. Until then the network plugin may spawn lock-ring and phase-label entities under **different components**. It must not spawn a second quad on the same cell with `WorldMapFootprintTile`.

**Acceptance.** Sim scene with one road segment, one rail segment, one zone cell, one power edge, one haul: each has a mesh entity, and the two egui painters report no `ctx_mut` in Simulation. `cargo test -p proc_A_dine01 --lib construction::visual_authority` stays green (request buffer unchanged).

### EGR-5 — **PLANNED** dev windows and pass hygiene

Apply the editor gates in section C. Move the four non-draw systems in section D off `EguiPrimaryContextPass`.

`sync_shell_layout_drag_gate`: Simulation reads Bevy `Interaction` on shell nodes; Editor keeps the egui pointer read. ⊚`MapShellPointerGate` has one writer per state, not both in the same state.

Subengine `world_gen_ui_system` cannot open over a play session.

**Acceptance.** Schedule query: no `EguiPrimaryContextPass` system with a condition true in `BaseState::Simulation`.

### EGR-6 — **PLANNED** exit

`sim_egui_context_acquires == 0` for a Simulation session in `debug_runs/ui_shell_migration_live.json`.  
Editor session may be non-zero. World-gen session may be non-zero.  
`bevy_egui` stays a dependency (**DEFER** crate removal).

Lib filter for the coder, via `validate-report cargo` rather than a raw wall:

```text
cargo test -p proc_A_dine01 --lib ui_shell
cargo test -p proc_A_dine01 --lib construction::visual_authority
cargo test -p proc_A_dine01 --lib hud::sim_build_picker
```

---

## ECS schedule

```text
Update
  MapCameraSystemSet::ApplyInput → DeriveDesired → …          (existing, GU-CAM)
  BuildPlanning chain: tools, clear visual requests, sync roads/rails/zones/footprints/corridor
  InfrastructureOverlay: collect edges, power preview segments
  Sim shell: ops strip, context tray, picker, road/power sheets, plant snapshot   (Bevy UI)
  WorldMapNetworkMeshPlugin::Sync   .after(sync_zone_visual_requests)
                                    .after(sync_power_line_preview_overlay_system)
                                    .run_if(in_state(Simulation))

PostUpdate
  sync_logistics_intent_draw_frame          (existing)
  logistics mesh sync .after(that)
  minimap Bevy chrome + compositor consumers (existing, Simulation)
  sync_world_map_footprints                  (existing, locked)

EguiPrimaryContextPass
  .run_if(product_egui_shell_active)     editor shell + editor world painters
  .run_if(world_gen_ui_chrome_visible)   generator
  .run_if(world_preview_chrome_active)   preview
  .run_if(map_editor_chrome_active)      map editor
  Simulation: empty
```

Mesh sync is view/render. It does not run inside `EguiPrimaryContextPass`. Extraction keeps reading transforms the mesh spawn wrote earlier in the frame. No write to `ViewManager`, `ResolvedViewports`, or the minimap compositor.

---

## Diagnostics

| Check | Where |
|:---|:---|
| `sim_egui_context_acquires` | `debug_runs/ui_shell_migration_live.json` |
| `egui_pass_count_in_sim` | same file, historical shell counter, stays `0` |
| Network mesh counts | one witness object on the shell proof or a small `debug_runs/world_map_network_mesh_live.json`: `road_segs`, `rail_segs`, `zone_cells`, `power_segs`, `logistics_segs` non-zero when the matching request buffer is non-empty |
| Dual-writer | debug assert: network plugin never inserts `WorldMapFootprintTile` |

---

## Edge cases

- **Zoom.** Mesh size is world units. The camera ortho already scales the picture. Do not multiply by `map_zoom_screen_scale` (that helper exists for egui screen strokes).
- **Editor enter / sim exit.** Despawn network meshes and footprint meshes when leaving Simulation, as `sync_world_map_footprints` already does for footprints.
- **Empty buffers.** Despawn when the key goes empty (tool cancelled, overlay toggled off).
- **Pointer.** After EGR-5, a sim click must not require an egui context to decide shell drag vs map pick. `SimulationMapPointerGate` stays the pick authority.
- **Pause.** Bevy pause menu stays above the shell. World meshes stay visible under it (simulation may be paused; the picture does not switch to egui).
- **Chunk stream.** Ribbons store world endpoints. A streamed-out chunk does not delete the ribbon by entity distance inside this plan; the request buffer is the lifetime. If the buffer drops a segment, the key change despawns it.
- **Replay.** Mesh sync is derived each frame from the buffers. Replay that restores roads and the utility graph restores the picture without an egui recording.
- **Minimap.** Compositor unchanged. If power lines are absent on the minimap after they exist as world meshes, that is a compositor sampling gap and a separate RN-MMC slice. It is not an excuse to call `draw_power_strokes_on_minimap` during Simulation.

---

## Open questions

1. When does the world-map owner release `construction/mod.rs`? EGR-2 and the editor-gate half of EGR-3 sit behind that. EGR-3 Bevy bodies, EGR-4 meshes, EGR-1, and EGR-5 do not.
2. Should footprint validity colors land in `world_map_frame` (locked owner) or as a second material on a new component? Prefer the locked owner, so one cell has one quad.
3. Is the subengine world-generator window still an operator entry, or only `world_gen_ui.rs`? This plan gates the subengine off Simulation and defers deleting it.
4. Phase words: `Text2d` on the plane is the sim label. If glyph cost shows up in the frame budget, drop to a color-only phase material and keep the word on the Bevy plant/site card. Do not fall back to `draw_construction_phase_labels_egui` in play.

---

## Remaining work (reconciled 2026-09-27)

The inventory above described the schedule before the editor gates landed. Live `EguiPrimaryContextPass` rows for product chrome, world painters, dev windows, the subengine generator, and the shell drag gate now use `product_egui_shell_active` (Editor, and not WorldGen), `map_editor_chrome_active`, or `subengine_world_gen_editor_only`. `WorldGenShellRoot` stays the WorldGen chrome. This pass does not move it back, and it does not delete the `bevy_egui` crate.

`sim_egui_context_acquires` already exists on `ProductShellDiagnostics`. Exit value in a Simulation session stays `0`. Proof filter: `cargo test -p proc_A_dine01 --lib egr_simulation_schedule_omits_drained_egui_systems`.

### Play map — one sprite, two pixel modes

`TileWorldFallbackSprite` is the play map. Its center is `(width/2, height/2, 0)`. Tile identity is `world_map_tile_center` (`tile + 0.5` on that plane, z = 1). `MainWorldCamera` ortho scale is the only zoom. The GPU minimap compositor is the only extra view.

| Mode | What the player sees | Where the color comes from |
|:---|:---|:---|
| **Default (visible on enter)** | Current play map | `tile_world_fallback` reads `MapViewInstances` simulation layers and, when the base is empty, `replace_base(PreviewLayers::BIOME)`. `raster_sim_minimap_layered_to_subregion` then shades every cell with `landform_relief_rgba` (slope and curvature) and the play hydro tint. |
| **Toggle** | Height and slope on that same sprite | Preview coloring already implemented for the world-gen raster: `height_to_color` (`PreviewLayers::HEIGHT`) and `slope_grade_to_color` via `DERIVED_SLOPE_OVERLAY` in `world_preview/layers.rs`. The play raster already honors a HEIGHT base and does not yet blend the slope overlay. The toggle sets those two bits on the simulation layer mask and repaints the existing image. |

The world-preview window and `WorldGenShellRoot` layer buttons stay on world-gen. They are not the play toggle and not a second map. The old egui tile stamp is not a third view.

### One attachment

Each layer reads sim state and places itself with `world_map_tile_center` or `map_plane_horizontal_xy` → `construction_point_on_world_map`. It does not build a screen projection, and it does not rewrite pixels because the camera scale changed.

These systems still project on their own:

| System | What it invents | Play today |
|:---|:---|:---|
| `tile_world_fallback` raster (`fire_boost_main` from `px_per_tile`) | Heat strength on the sprite changes with zoom | Live on the play image |
| `map_tile_raster::stamp_haul_cells_subregion` (called from that raster) | A second haul picture in sprite texels | Live beside `sync_world_map_logistics_meshes` |
| `map_tile_raster::stamp_town_centers_subregion` | Town mark in texels | **OWNED ELSEWHERE** (town names) |
| `fire_vfx::emit::emit_world_fire_particles_from_projection` and `gpu_fire_particle_raster::fire_particle_view_proj` | Clip matrix and scatter from `ExtractedCameraMetrics`, plus `px_per_tile` cull | **OWNED ELSEWHERE** (particles, zoom lock) |
| `intent_draw::draw_logistics_intent_overlay_egui` (`world_to_egui_rendered`) | Screen strokes | Editor only (`product_egui_shell_active`, body requires Editor) |
| `power_map_overlay_draw::draw_power_map_overlay_egui` | Screen strokes | Editor only. Play lines are `sync_world_map_power_meshes` |
| `visual_authority::draw_construction_visual_requests_egui` and the road/rail/zone/power ghosts (`map_zoom_screen_scale`) | Screen quads | Editor or unscheduled. Play geometry is the footprint mesh and `WorldMapNetworkMeshPlugin` |

`sync_world_map_footprints`, `sync_world_map_path_meshes`, `sync_world_map_power_meshes`, and `sync_world_map_logistics_meshes` (`ribbon_transform`) already use the plane. Leave them. Do not add a second fire extract; heat on the sprite keeps reading `SharedOverlayFieldBuffers`.

### Simulation `ctx_mut`, painters, and pointer reads

“Runs in Simulation” means the schedule condition is true when `BaseState::Simulation`. A function that still contains `ctx_mut` but fails that condition is listed so a later edit does not drop the gate.

**Plugin, every state including play.** `EguiPlugin::default()` in `src/engine/engine.rs` and `src/engine/engine_with_worldgen.rs`. No user painter is attached in play. The plugin’s own pre-update still sees the pointer. That is the remaining menu/cursor capture. The first slice keeps the crate.

**No user `ctx_mut` is scheduled to run in Simulation.** These are the functions that still call it, and the gate that keeps the counter at 0:

| Function | File | Schedule in play |
|:---|:---|:---|
| `finalize_simulation_map_pointer_gate_egui_system` | `simulation_pointer_gate.rs` | `in_simulation_or_editor` **and** `minimap_chrome_egui_sync_active` (false in Simulation). Body still calls `ctx_mut` with no Simulation return — the second `run_if` is the cut |
| `draw_simulation_unified_cursor_egui_system` | same | `product_egui_shell_active` |
| `draw_simulation_minimap_topology_legend_egui_system` | `minimap_bevy_interaction.rs` | same pair of `run_if` as finalize |
| `hud_panel_click_outside_collapse_egui_system` | `panel_state.rs` | `not(in_state(Simulation))` |
| `apply_egui_theme_system` | `style/theme.rs` | `not(in_state(Simulation))`, and a WorldGen return before `ctx_mut` |
| `sync_shell_layout_drag_gate` | `map_view/presentation/stability.rs` | `product_egui_shell_active` |
| Product painters (`hud_product_shell_egui_root`, picker, road/power sheets, context tray, plant card, power hover, toolbox, staged panel, tool hints, placement debug, phase labels, footprint overlay, power ghost, visual-authority draw, admin-zone ink) | `construction/mod.rs`, `hud_root_tick.rs` | `product_egui_shell_active` |
| `draw_power_map_overlay_egui` | `infrastructure_overlay.rs` | `product_egui_shell_active` |
| `draw_logistics_intent_overlay_egui` | `economy/logistics/mod.rs` | `product_egui_shell_active` |
| Dev windows (`diagnostics_ui_system`, `faction_tools_ui_system`, `ai_explainability_egui_system`, `pressure_composer_egui_system`, `logistics_targets_panel_ui`, `permissions_ui_system`, `keybindings_options_ui`, `draw_block_frame_debug_overlay`, `draw_debug_viewport_overlay`, `draw_assembly_snapshot_qc_ui`) | matching `src/gui/*.rs` and `debug_viewport_overlay.rs` | `product_egui_shell_active` |
| Subengine `world_gen_ui_system` | `bevysubengines/world_generator_plugin.rs` | `subengine_world_gen_editor_only` (Editor, and off while `AppState::WorldGen`) |
| Map editor windows | `gui/editor/map_editor/mod.rs` | `map_editor_chrome_active` (off for InGame, Paused, WorldGen) |

Unscheduled, so they do not acquire in play: `draw_road_path_ghost_egui`, `draw_rail_path_ghost_egui`, `draw_zone_paint_ghost_egui`, `draw_render_surface_debug_overlay`, `configure_visuals_system`, `update_ui_scale_factor_system`.

**Pointer path (do not add a commit event).** `simulation_tool_pointer` / `SimulationMapPointerGate`. `sync_simulation_map_pointer_gate_system` clears `egui_blocks` in Simulation and does not call `ctx_mut`. Commit systems keep reading that gate.

### Ordered slices UPM-1..5 — **DONE** (do not redo)

`egr_`, `wgen_`, `wmap_`, `cityfix_`, and `upm_` already passed. Do not re-run the full lib suite to re-prove them.

| Slice | Status | What landed |
|:---|:---|:---|
| **UPM-1** | **DONE** | `PlayMapPixelMode` default is biome plus relief. Ops-strip HEIGHT writes `height_slope` and `layers_for_play_sprite` repaints `TileWorldFallbackSprite` with HEIGHT + `DERIVED_SLOPE_OVERLAY`. |
| **UPM-2** | **DONE** | `play_sprite_fire_heat_boost()` is `1.0`. Sprite heat reads `SharedOverlayFieldBuffers`. Camera scale is the zoom. |
| **UPM-3** | **DONE** | Haul pixels are off the sprite. Hauls are `sync_world_map_logistics_meshes` (`WorldMapLogisticsSeg`) on `world_map_tile_center` via `ribbon_transform`. |
| **UPM-4** | **DONE** | `omit_egui_pointer_preupdate_in_simulation` in `src/engine/states.rs` (called from `engine.rs` and `engine_with_worldgen.rs`). `EguiPreUpdateSet::ProcessInput` does not run in `BaseState::Simulation`. |
| **UPM-5** | **DONE** | Finalize and the minimap egui chain in `simulation_shell_phase2` are `product_egui_shell_active` only. |

`bevy_egui` removal, deleting unscheduled ghost functions, and a Bevy keybindings page stay **DEFER**.

### City-bug pass — **DONE** (do not redo)

- Sparks hidden below heat `0.12` (`CHUNK_FIRE_OVERLAY_DISPLAY_MIN` in `emit_world_fire_particles_from_projection`).
- Crosshair only when a tool is armed and `SimulationMapPointerGate` has a known cursor (`sync_sim_play_crosshair`, `apply_simulation_unified_cursor_system`).
- One power sheet (`sync_sim_power_sheet_bevy`).
- Out-of-map power, road, and rail rejected (`path_samples_leave_generated_map` / `sync_world_map_power_meshes`).
- Town `Text2d` at `world_map_tile_center` (`WorldMapTownLabel` in `sync_world_map_town_names_and_zone_shade`).
- SHADE zone tint on the map plane (`WorldMapZoneTint`, `ZoneShadeToggle`).
- Footprint fill and phase color stay on `world_map_frame.rs` / `phase_fill.rs`.

## PLN-1 → PLN-3 — SUPERSEDED (not queued)

**SUPERSEDED** — do not pick PLN-1..3 from this plan. Plugin-pass work (**PLN-1**) is covered by [`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md) (`bevy_egui` removed). **PLN-2 / PLN-3** are not re-seeded here (¬invent new work). Standing rules below (one play camera, tile identity, mesh/pointer authorities) still apply when touching those systems.

One play camera (`MainWorldCamera`). Tile identity is `world_map_tile_center` (`tile + 0.5` on the sprite, z = 1). Path endpoints stored as `(x, 0, row)` land on that plane through `map_plane_horizontal_xy` / `construction_point_on_world_map` / `ribbon_transform` (`PLANE_Z` = 1.04). `GhostBuildCursor` already uses the same cell formula. Roads, rails, power, zone cells, zone tints, logistics ribbons, town labels, and footprint quads already read that identity.

No slice from this plan writes `src/construction/mod.rs` or `src/construction/world_map_frame.rs`. Calling `world_map_tile_center` is a read.

### HEIGHT and SHADE stay on their writers

`play_map_height_slope_click_system` flips `PlayMapPixelMode.height_slope`. The main sprite reads it in `layers_for_play_sprite` (`sim_layers` only). `zone_shade_toggle_click_system` flips `ZoneShadeToggle.visible`. `sync_world_map_town_names_and_zone_shade` spawns `WorldMapZoneTint` only while that flag is set.

These toggles must not cover:

| Authority | Owner already in tree | What a later edit must leave alone |
|:---|:---|:---|
| Minimap | `MinimapCompositor`; CPU fallback uses `map_views.minimap.layers`, not `layers_for_play_sprite` | Do not thread `height_slope` or `ZoneShadeToggle` into the compositor or `minimap.layers`. |
| Placement tile | `build_sync_ghost_cursor_entity_system` writes `GhostBuildCursor` at `(origin.x + 0.5, origin.z + 0.5, 1)` | Do not move or retint that transform from either toggle. |
| Commit funnel | `execute_construction_plans_system` and `preview_commit_guard` | Do not write the queue, `ConstructionVisualRequests`, or a site from a HUD click. |
| Single fire extract | `FireVisualFrame` and `SharedOverlayFieldBuffers`. Sprite heat is `play_sprite_fire_heat_boost()` = 1.0 | Do not add a second extract. Do not let HEIGHT rewrite the heat field. |
| `(u32, f32)` logistics edge | `LogisticsVisualSnapshot.edge_rows` (`edge_id`, traffic) plus `haul_marker_t` | Play hauls are `LogisticsIntentDrawFrameRes` polylines. Do not rewrite `edge_rows` from HEIGHT, SHADE, or a mesh sync. |

### What already shares the tile and the camera

| Picture | Writer | Placement |
|:---|:---|:---|
| Buildings | `sync_world_map_footprints` | `WorldMapFootprintTile` at `world_map_tile_center` |
| Roads, rails, zones, corridor, power | `sync_world_map_path_meshes`, `sync_world_map_power_meshes` | `ribbon_transform` / `construction_point_on_world_map` |
| Logistics | `sync_world_map_logistics_meshes` | same ribbon, from `LogisticsIntentDrawFrameRes` |
| Zone tint | `sync_world_map_town_names_and_zone_shade` | `WorldMapZoneTint` from `DistrictRecord.tile_rect` |
| Town name | same system | `WorldMapTownLabel` at the town tile center |
| Sprite heat | `tile_world_fallback_rasterize` | overlay field, strength 1.0 |

### Systems that still use their own zoom, screen rect, or sprite bake

| System | What it invents | In this pass |
|:---|:---|:---|
| `emit_world_fire_particles_from_projection` | `ExtractedCameraMetrics.zoom_level` (`ZoomFrame.px_per_tile`) feeds `fire_particle_scatter_count` and `shape_fire_row_for_particle_lod`. Scatter offset is added to extract `world_xyz` (x and y). `world.z` stays the extract value, not the map plane. | **PLN-2 SUPERSEDED** (standing tile rule; ¬queued) |
| `stamp_town_centers_subregion` | Still called from `raster_tile_fallback_subregion` on both the play image and the CPU minimap image. Town `Text2d` already exists. | **PLN-3 SUPERSEDED** (standing play-sprite rule; ¬queued) |
| `apply_water_surface_overlay_subregion` | `river_overlay_pixel` / `lake_overlay_pixel` switch readability at `WATER_STRATEGIC_ZOOM_ALPHA` (D-W09). Motion does not scale with zoom. | Leave. D-W09 owns that threshold. |
| `stamp_tracer_cells_subregion`, `stamp_impact_cells_subregion`, `stamp_growth_origins_subregion` | Extra marks in sprite texels. No mesh twin yet. | Listed. No slice. |
| `bump_tile_raster_on_main_camera_zoom` / `raster_zoom_band` | Zoom-band dirty invalidates chunks. Leave it. It is the sprite’s cache, not a second camera. | No slice |
| `fire_particle_draw.wgsl` | `zoom_fade` is the constant `1.0`. `globals.zoom_alpha` is written and unused for fade. | No shader edit |
| Editor painters `draw_construction_visual_requests_egui`, road/rail/zone/power ghosts | `map_zoom_screen_scale` | Editor or unscheduled. They stay. |
| `tactical_viewport_screen_rect` | Minimap frame in screen pixels | Minimap. It stays its own view. |

### Simulation-time egui / pointer — historical (SUPERSEDED)

**SUPERSEDED by** [`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md) — `bevy_egui` removed. Pointer authority for play is Bevy shell / `SimulationMapPointerGate` (and related product gates). Do not re-add egui plugin passes or invent a PLN-1 gate slice.

Historical note (pre-drop): user `ctx_mut` systems were behind `product_egui_shell_active` / `map_editor_chrome_active` / `subengine_world_gen_editor_only` / `not(BaseState::Simulation)`; `EguiPlugin` default sets could still run in play before Cargo drop. That inventory is archive only.

### PLN-1 — stop the play egui plugin passes — **SUPERSEDED**

**SUPERSEDED by** [`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md) (`bevy_egui` removed · EGR-FULL-0..8 CLOSED). Do not implement the former `configure_sets` / Simulation enter-exit egui output clear. Do not re-add `bevy_egui`.

**Standing (still apply):** do not move `simulation_tool_pointer`; do not edit `construction/mod.rs` from this plan.

### PLN-2 — fire sparks on the same tile — **SUPERSEDED**

**SUPERSEDED** as a queue row from this plan (¬re-seed · ¬invent new work). Standing mesh/tile rule if `src/render/fire_vfx/emit.rs` is touched later: sparks that pass the heat floor place via `world_map_tile_center` + cell scatter with z on the map plane (`PLANE_Z` 1.04); no second fire extract; do not write `SharedOverlayFieldBuffers` or edit `construction/mod.rs` / `world_map_frame.rs` from this plan.

### PLN-3 — town mark is the Text2d only — **SUPERSEDED**

**SUPERSEDED** as a queue row from this plan (¬re-seed · ¬invent new work). Standing play-sprite rule if `src/render/tile_world_fallback.rs` is touched later: Simulation town picture is `WorldMapTownLabel`; do not dual-stamp play image via `stamp_town_centers_subregion`; leave CPU minimap / `LogisticsVisualSnapshot.edge_rows` / commit funnel / `MinimapCompositor` alone; do not edit `construction/mod.rs` from this plan.

## Historical slice list (closed by the 2026-09-27 reconcile)

EGR and UPM are closed. **PLN-1..3 SUPERSEDED** — do not re-open an EGR, UPM, or PLN row from this plan.

| Slice | Status |
|:---|:---|
| EGR-0 shell split, Bevy picker/sheets/tray, GPU minimap, footprint quads | **SHIPPED** |
| EGR-1 `sim_egui_context_acquires` | **SHIPPED** (exit value still proven by UPM-4 / UPM-5) |
| EGR-2 and EGR-3 egui sheets / plant / hover / crosshair off Simulation | **SHIPPED** gates. City cursor and one power sheet are **DONE** |
| EGR-4 power, logistics, road, rail, zone, corridor meshes; editor unschedule of those painters | **SHIPPED** (`WorldMapNetworkMeshPlugin`) |
| EGR-4a footprint color / lock / phase label | **DONE** (`world_map_frame.rs`, `phase_fill.rs`) |
| EGR-5 dev gates, drag-gate split, subengine world-gen off play | **SHIPPED** |
| EGR-6 `sim_egui_context_acquires == 0` | **SUPERSEDED** — Cargo drop removed `bevy_egui` ([`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md)) |
| Delete ghost fns, delete `bevy_egui`, Bevy keybindings page | **SUPERSEDED** — `bevy_egui` removed via EGR-FULL; keybindings via panel port |
| PLN-1 plugin-pass gates | **SUPERSEDED** by [`plan_egui_full_dep_drop_v1.md`](plan_egui_full_dep_drop_v1.md) |
| PLN-2 fire sparks tile align | **SUPERSEDED** (¬queued; standing tile rule retained) |
| PLN-3 play town stamp | **SUPERSEDED** (¬queued; standing play-sprite rule retained) |

No `@coder` pick from this plan. No slice writes `src/construction/mod.rs`.
