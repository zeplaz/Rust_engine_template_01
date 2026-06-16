# Construction system inventory and authority map

Scan date: 2026-05-14. Scope: repository-wide grep for construction, build, site, corridor, ghost placement, transport persistence, and related presentation hooks. Runbook cross-reference: [`docs/archive/2026-06-prompts-guides/runbooks/guides/infrastructure_construction_runbook_v1.md`](../../docs/archive/2026-06-prompts-guides/runbooks/guides/infrastructure_construction_runbook_v1.md).

This document inventories what exists today, classifies each piece into the authority buckets below, and records who mutates world state versus who should. It does not change UI or rendering code.

## Authority buckets

| Bucket | Meaning |
|--------|---------|
| **Intent** | Player or agent decision in UI/input; no authoritative world mutation |
| **Plan** | Authoritative gameplay record of what should be built (queues, books, events) |
| **Execution** | Simulation truth: ECS rows, transport topology, chunk overlays, terrain invalidation |
| **View** | Render/UI-only ghosts, overlays, snapshots, diagnostics |
| **Save** | Serialize/deserialize boundaries |

Proposed spine (target, not implemented as a single module yet):

- `ConstructionIntent` → `ConstructionPlan` → `ConstructionExecution` → `ConstructionView` / `ConstructionSnapshot`
- **Only one execution lane** should mutate world reality; everything else feeds **Plan** or derives **View**.

---

## 1. What exists (inventory)

### A. Strategic / simulation construction

| Symbol / system | Location | Bucket | Notes |
|-----------------|----------|--------|-------|
| `CorridorConstructionPhase`, `CorridorConstructionStatus`, `CorridorConstructionBook` | `src/strategic/construction_book.rs` | Plan | Per `TransportEdgeId` ledger; missing edge ⇒ completed (legacy default) |
| `CorridorConstructionTickConfig` | same | Plan | Sim-tick progression rate |
| `advance_corridor_construction_book_on_sim_tick` | same | Execution | **Only** mutates `CorridorConstructionBook` on sim ticks |
| `advance_corridor_construction_status` | same | Execution | Pure edge status transition helper |
| `plan_edge` | same | Plan | Inserts Planned row for an edge |
| `align_corridor_book_with_transport_directory`, `transport_directory_edge_signature` | same | Plan / Save | Reconciles book keys with live transport directory |
| `transport_construction_records_from_book`, `apply_corridor_book_from_transport_snapshot`, `corridor_phase_to_wire` / `from_wire` | same | Save | R8 construction slice on `TransportNetworkSnapshot` |
| `SiteConstructionPhase`, `SiteConstructionStatus`, `SiteConstructionBook`, `SiteId`, `SiteIdIssuer` | `src/strategic/site/resources.rs` | Plan | Per operational site |
| `FootprintTiles`, `site_phase_from_corridor_coarse` | same | Plan | Site footprint + coarse corridor mapping |
| `CommitConstructionSiteEvent` | `src/strategic/site/events.rs` | Plan | Message boundary into site commit |
| `commit_construction_site_system` | `src/strategic/site/systems.rs` | **Execution** | Spawns site ECS bundle, writes `SiteConstructionBook`, marks `NetworkDirtyMask`, **invalidates `WorldPreviewState`** |
| `footprint_affected_chunk_coords`, `footprint_tiles` | same | Plan | Chunk/tile math for sites |
| `validate_committed_site_terrain_system` | same | Execution | Stub terrain validation on committed sites |
| `site_advance_planned_to_under_construction_system` | same | Execution | Planned → UnderConstruction + manifest seed |
| `site_construction_progression_system` | `src/strategic/site/logistics.rs` | Execution | UnderConstruction readiness; mirrors into `SiteConstructionBook` |
| `site_provisioning_system` | `src/strategic/site/` (wired in `plugin.rs`) | Execution | Provisioning phase (not fully expanded here) |
| `apply_corridor_construction_book_to_entities` | `src/strategic/transport_bridge.rs` | Execution | Copies book → `CorridorConstructionStatus` on corridor entities |
| `rebuild_logistics_graph_from_transport` (uses book traffic factors) | same | Execution | Logistics graph capacity from construction phases |
| `inject_transport_scalar_fields_into_overlays` | same | **Execution** | Writes congestion/EW scalars into `ChunkStrategicOverlay` using `CorridorConstructionBook` |
| `sync_construction_book_after_transport_changes` | `src/strategic/plugin.rs` | Plan | Aligns corridor book when transport directory changes |
| `BuildOrder`, `BuildOrderQueue`, `ApprovedBuildOrders`, `process_build_order_queue_system` | `src/strategic/build_order.rs` | Plan | Faction/mission proposals; bounds check only |
| `BuildSiteTile`, `StructureType`, `BuildReason` | same | Plan | Shared tile coords for orders and sites |
| `ConstructionAiConfig`, `construction_ai_shared_validation_probe_system` | `src/ai/construction/mod.rs` | Intent → Plan | Emits `CommitConstructionSiteEvent` when probe passes |
| `ConstructionStates` (entity flags) | `src/entities/types/e_flagz.rs` | Plan / Execution | ECS flag enum; parallel to site components |
| `ConstructionStatus` (progress/time) | `src/entities/components.rs` | Execution | Generic entity construction progress |
| Strategic plugin schedule | `src/strategic/plugin.rs` | — | `InfrastructureSiteSet::{Planning, Logistics, Construction, Provisioning}`; corridor tick + transport sync |

### B. Transport / infrastructure graph

| Symbol / system | Location | Bucket | Notes |
|-----------------|----------|--------|-------|
| `TransportConstructionRecord`, `TransportNetworkSnapshot.construction` | `src/systems/transport/snapshot.rs` | Save | Optional R8 slice for corridor phases |
| `transport_network_snapshot_from_world_with_construction` | same | Save | Embeds construction rows in snapshot |
| `transport_network_persistence_on_load` | `src/systems/transport/persistence.rs` | Save → Execution | Hydrates topology/fields; calls `apply_corridor_book_from_transport_snapshot` |
| `TransportNetworkPersistencePlugin` | same | Save | `LoadTransportNetworkSnapshotFromDisk` |
| Map editor save/load transport | `src/gui/editor/map_editor/mod.rs` | Intent / Save | Reads `CorridorConstructionBook`, builds snapshot with `transport_construction_records_from_book` |
| `InfrastructureGraph`, `LogisticsGraph`, `SpatialNetworkGraph` | `src/strategic/` | Execution | Downstream of transport + construction factors |

### C. GUI build / construction UX

| Symbol / system | Location | Bucket | Notes |
|-----------------|----------|--------|-------|
| `BuildPlanningPlugin` | `src/gui/build/mod.rs` | — | Registers build resources + systems |
| `BuildStripState`, `ToolContext`, `cycle_build_planning_tool_system` | `src/gui/build/build_strip.rs` | Intent | Active planning tool |
| `BuildGhostState`, `BuildPlacementPreview`, `BuildCommandActor`, `BuildGhostRoot` | `src/gui/build/build_state.rs` | Intent / View | Ghost cursor state |
| `GhostBuildCursor` | `src/gui/build/build_ghost.rs` | View | ECS marker for ghost entity |
| `build_pick_ghost_tile_system`, `build_drag_paint_queue_system`, `build_rotate_mirror_ghost_system`, `build_cancel_ghost_system` | `src/gui/build/build_interaction.rs` | Intent | Pointer → ghost / queue |
| `build_refresh_placement_validation_system` | same | View | Calls `evaluate_site_placement_at_world_tile` |
| `build_confirm_site_system` | same | Intent → Plan | Drains approved `PendingConstructionQueue`, emits `CommitConstructionSiteEvent` |
| `build_queue_blueprint_on_shift_click_system`, `build_clear_pending_queue_system` | same | Intent | Queue mutations |
| `build_sync_ghost_cursor_entity_system` | same | View | Syncs ghost ECS entity from resources |
| `queue_commit_construction_site` | `src/gui/build/build_commit.rs` | Plan | Thin writer for `CommitConstructionSiteEvent` |
| `PendingBuildBlueprint`, `PendingConstructionQueue` | `src/gui/build/pending_construction.rs` | Plan (UI-held) | Defers commit until approval |
| `ConstructionQueueIntent`, `ConstructionQueuePanelView`, `sync_construction_queue_panel_view`, `apply_construction_queue_intents` | `src/gui/build/construction_queue_intent.rs` | Intent | Panel → queue mutations |
| `draw_pending_construction_queue_egui` | `src/gui/build/pending_construction_panel.rs` | View | HUD shell window; emits intents only |
| `BuildOverlayVisibility` | `src/gui/build/build_overlays.rs` | View | Toggle flags (terrain/network/cost) |
| `build_footprint_validity_overlay_egui` | `src/gui/build/build_footprint_overlay.rs` | View | Validity hint over **simulation map viewport** (not minimap/world preview) |
| `BlueprintPresetCollectionR8`, `blueprint_collection_from_pending` | `src/gui/build/blueprint_preset.rs` | Save / View | RON preset export from pending queue |
| `validate_planned_site_stubs` | `src/gui/build/build_validation.rs` | View | Stub validation helpers |
| `HudWidgetId::ConstructionQueue` | `src/gui/hud/shell_framework.rs` | View | Shell slot for construction panel |
| `in_game_hud` build strip / pending read | `src/gui/in_game_hud.rs` | View | Bevy UI shell mirrors tool context |
| `PermissionDomain::RoadConstruction`, `RailConstruction` | `src/gui/agent_permissions_ui.rs` | Intent | Agent permission UI only |
| Diagnostics corridor phase editor | `src/gui/diagnostics_ui.rs` | View / **risk** | Can edit `CorridorConstructionStatus` in dev UI |

### D. Presentation / render (construction-adjacent)

| Symbol / system | Location | Bucket | Notes |
|-----------------|----------|--------|-------|
| `publish_logistics_visual_snapshot`, `LogisticsVisualSnapshot` | `src/render/visual_domain_snapshots.rs` | View | Reads `CorridorConstructionBook` → visual snapshot |
| `ghost_band_neighbor_coords_for_preview` | `src/io/streaming/preview_ghost.rs` | View | **Streaming residency**, not build ghost |
| World preview raster/GPU chunk iteration | `src/gui/editor/world_preview/render_raster.rs`, `gpu_preview.rs` | View | Uses streaming ghost bands for chunk sampling |

### E. Persistence / IO

| Symbol / system | Location | Bucket | Notes |
|-----------------|----------|--------|-------|
| Transport R8 save/load (map editor + G4) | `map_editor/mod.rs`, `transport/persistence.rs` | Save | Construction slice on transport snapshot |
| `TRANSPORT_OVERLAY_NAME`, `transport_overlay_ref` | `src/io/save/transport_overlay.rs` | Save | Named overlay ref for save pipeline |
| No dedicated site-book save module | — | Gap | `SiteConstructionBook` / pending queue not found in `src/io/save/` |

---

## 2. Bucket summary (counts)

| Bucket | Primary owners today |
|--------|----------------------|
| **Intent** | Build strip/ghost input, construction queue panel intents, AI probe config, agent permissions, map editor transport tools |
| **Plan** | `CorridorConstructionBook`, `SiteConstructionBook`, `PendingConstructionQueue`, `BuildOrderQueue` / `ApprovedBuildOrders`, `CommitConstructionSiteEvent` |
| **Execution** | `commit_construction_site_system`, site phase/provisioning systems, `advance_corridor_construction_book_on_sim_tick`, transport hydrate, overlay injection, logistics graph rebuild, entity component sync |
| **View** | Ghost entity + footprint egui overlay, construction HUD panel, logistics visual snapshot, diagnostics corridor editor, build validation preview |
| **Save** | `TransportNetworkSnapshot.construction`, map editor + G4 load, blueprint RON presets |

---

## 3. Who mutates world state today

| Mutator | What it changes | Should it? |
|---------|-----------------|------------|
| `commit_construction_site_system` | ECS site entities, `SiteConstructionBook`, `NetworkDirtyMask`, **`WorldPreviewState` invalidation** | **Yes** (site execution), but preview invalidation is a **presentation leak** |
| `advance_corridor_construction_book_on_sim_tick` | `CorridorConstructionBook` only | **Yes** (corridor execution) |
| `site_advance_planned_to_under_construction_system`, `site_construction_progression_system`, `site_provisioning_system` | Site ECS + `SiteConstructionBook` | **Yes** |
| `apply_corridor_construction_book_to_entities` | `CorridorConstructionStatus` components | **Yes** (derived from book) |
| `inject_transport_scalar_fields_into_overlays` | `ChunkStrategicOverlay` cell scalars | **Yes** (sim overlay), but also **feeds map/presentation consumers** |
| `transport_network_persistence_on_load` / map editor hydrate | Transport topology, fields, **`CorridorConstructionBook`** | **Yes** (load execution) |
| `sync_construction_book_after_transport_changes` | `CorridorConstructionBook` alignment | **Yes** (plan sync) |
| `build_confirm_site_system` / AI probe / `queue_commit_construction_site` | Emits `CommitConstructionSiteEvent` only | **Correct** (plan boundary) |
| `PendingConstructionQueue` / `ConstructionQueueIntent` handlers | UI queue entries only | **Correct** (plan held in UI resource until commit) |
| `process_build_order_queue_system` | `ApprovedBuildOrders` only | **Plan**; **no execution consumer found** |
| `publish_logistics_visual_snapshot` | `LogisticsVisualSnapshot` | **View only** |
| `build_*` ghost/pick/overlay systems | `BuildGhostState`, ghost entity, egui | **View / intent** (except confirm → event) |
| Diagnostics UI corridor phase radios | `CorridorConstructionStatus` on entities | **Should not** in production; bypasses book authority |

**Target execution writers (single spine):** one corridor execution lane (`CorridorConstructionBook` + transport directory) and one site execution lane (`CommitConstructionSiteEvent` → site systems). All other paths should enqueue or derive.

---

## 4. Duplication and coupling

### Multiple plan authorities

1. **`CorridorConstructionBook`** — transport-edge corridor phases (P2).
2. **`SiteConstructionBook`** — operational site phases (P2-A).
3. **`PendingConstructionQueue`** — UI-held blueprints before `CommitConstructionSiteEvent`.
4. **`BuildOrderQueue` / `ApprovedBuildOrders`** — faction/mission orders; **approved list has no downstream commit system in-repo**.
5. **`CommitConstructionSiteEvent`** — shared message bus (player, AI, batch approve).
6. **ECS `ConstructionSite` + `CorridorConstructionStatus` components** — mirrors of books (must stay derived).

There is **no single `ConstructionPlan` type**; corridor and site plans are separate ledgers plus a UI queue.

### Presentation vs simulation mixing (symptom alignment)

| Reported symptom | Likely construction-related leak |
|------------------|----------------------------------|
| Minimap affecting world / preview coupling | Not a dedicated minimap construction overlay; **shared map presentation** (`ChunkStrategicOverlay`, `WorldPreviewState` invalidation on site commit, logistics snapshot from corridor book) |
| World preview black / wrong on load | **Execution** invalidates preview terrain cache on commit; GPU/CPU preview reads shared world/overlay state while books and transport hydrate race |
| Drag crash / layout spikes | HUD construction panel uses same **shell layout / viewport** path as minimap; drag was mutating layout intent (addressed separately in HUD freeze work) |
| Zoom / scale instability | Not construction-specific; HUD density and map consumers share egui pass |
| MAP FIT MISMATCH | Independent fit validators vs painted rects; construction footprint overlay uses **simulation map viewport** projection, not the shared map-view consumer spine |

### Overlap: transport construction vs strategic construction

- **Same book:** `CorridorConstructionBook` is both strategic ledger and transport snapshot slice.
- **Map editor** writes construction rows into transport save; **G4 load** restores book from snapshot.
- **Site** construction does **not** serialize through the same R8 slice today (site book / ECS only in memory).

### Overlap: UI preview vs sim ghosting

- **Sim ghost:** `BuildGhostState` + `GhostBuildCursor` entity + `build_footprint_validity_overlay_egui` (sim map viewport).
- **Queue preview:** `ConstructionQueuePanelView.ghost_valid` from pending queue sync (HUD panel).
- **World preview / minimap:** no build-tool overlay; world preview “ghost” naming is **chunk streaming ghost bands**, unrelated to build placement.

### Overlap: minimap vs world preview construction visuals

- **No** minimap-specific construction markers found.
- Corridor construction affects **logistics visual snapshot** and **chunk overlays**, which downstream map/world consumers may read indirectly.

---

## 5. Current problem (diagnosis)

Construction gameplay is split across **two authoritative books** (corridor + site), a **UI pending queue**, an **orphaned approved build-order list**, and **ECS mirrors**. Execution paths also **poke presentation caches** (`WorldPreviewState`, strategic overlays) from site commit and transport injection.

That matches the reported instability: presentation and simulation share mutable state instead of a single plan → execution → view pipeline.

---

## 6. Missing spine (recommended target)

| Layer | Canonical owner (recommended) | Today |
|-------|------------------------------|--------|
| `ConstructionIntent` | Input + HUD intents + AI probe | `BuildGhostState`, `ConstructionQueueIntent`, build strip |
| `ConstructionPlan` | **Single merged plan store** (or strict hierarchy: site plan + corridor plan with one commit API) | `CorridorConstructionBook`, `SiteConstructionBook`, `PendingConstructionQueue`, `ApprovedBuildOrders` |
| `ConstructionExecution` | Transport topology + site ECS + overlay/chunk effects | Split across `commit_construction_site_system`, corridor tick, transport hydrate, overlay injection |
| `ConstructionView` | Derived snapshots + ghosts + HUD | Ghost entity, footprint egui, logistics snapshot, diagnostics |
| `ConstructionSnapshot` | Transport R8 + (future) site book | Transport snapshot construction slice only |

**Collapse rule:** pick **one plan write API** per domain (corridor edge vs site tile), **one execution mutator** per domain, and make preview/minimap/logistics **read committed execution or published snapshots only**—never invalidate presentation from commit paths.

---

## 7. Next engineering step (after this inventory)

Do **not** refactor UI or rendering until plan/execution boundaries are agreed:

1. Document commit order: `CommitConstructionSiteEvent` vs corridor `plan_edge` / editor transport edits.
2. Wire or delete **`ApprovedBuildOrders`** consumers.
3. Remove or gate **diagnostics direct mutation** of `CorridorConstructionStatus`.
4. Move **`WorldPreviewState` invalidation** out of site commit into a presentation subscriber on committed execution revision.
5. Define **site persistence** alongside transport R8 or explicit snapshot extension.

---

## 8. Quick file index

| Area | Paths |
|------|--------|
| Corridor book | `src/strategic/construction_book.rs` |
| Site systems | `src/strategic/site/` |
| Build UX | `src/gui/build/` |
| Transport save | `src/systems/transport/snapshot.rs`, `persistence.rs` |
| Map editor transport + construction | `src/gui/editor/map_editor/mod.rs` |
| AI construction | `src/ai/construction/mod.rs` |
| Build orders | `src/strategic/build_order.rs` |
| Visual snapshot | `src/render/visual_domain_snapshots.rs` |
| Runbook | `docs/archive/2026-06-prompts-guides/runbooks/guides/infrastructure_construction_runbook_v1.md` |
