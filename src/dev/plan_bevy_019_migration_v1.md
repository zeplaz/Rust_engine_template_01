# BEVY 0.18 → 0.19 MIGRATION + MITIGATION PLAN v1 — agent-executable
# Generated 2026-07-03. Sources: https://bevy.org/news/bevy-0-19/ ·
#   https://bevy.org/learn/migration-guides/0-18-to-0-19/ (canonical — re-fetch per slice, entries below are the working checklist)
# Companions: codebase_index_v1.md (entry codes) · plan_cleanup_v1.md · plan_schedule_sync_v1.md · plan_city_grammar_upgrade_v1.md
# Issue codes: MIG-G# (gates) · MIG-M# (mechanical, lesser-agent OK) · MIG-R# (render, coder-grade)
#              MIG-E# (ECS/resources) · MIG-A# (post-migration adoption wins)

# ═════════════════════════════════════════════════════════════════════
# PROGRAM METADATA
# ═════════════════════════════════════════════════════════════════════
# id:           PLAN-BEVY-019-MIG-v1
# status:       ACTIVE P0 — Phase 0 gates; HANDOFF primary lease 2026-07-03
# priority:     P0 PRIMARY — supersedes multi-parallel as global pick; defers cleanup/schedule Phase 2+ while MIG in flight
# index:        development_plan_index.md + HANDOFF.md § PLAN-BEVY-019-MIG-v1 (linked 2026-07-03)
# active_phase: Phase 0 — MIG-P0-G1-001 (steward) then MIG-P0-G2-001 (coder)
# owner:        @sim-steward gates + sequencing · @coder for MIG-R# render slices ·
#               lesser agents (coder_a/coder_b/general) for MIG-M# mechanical slices
# branch:       master (single canonical worktree — NO separate migration branch)
# regression:   validate-report cargo (compression 3) per slice · cargo test --lib per phase ·
#               stage5 witness + full --test matrix (weather|fire|visual) at phase gates
# rollback:     Cargo.lock snapshot committed at P0; any phase gate red >2 sessions → park MIG slices,
#               restore lockfile from snapshot if needed, file debug-intelligence packet, do NOT force
# conflicts:    FREEZE plan_cleanup Phase 2+ and plan_schedule_sync Wave 2+ code slices while P1–P3
#               are in flight on master (same-file ownership via HANDOFF tandem matrix); Phase-0 hygiene OK.
#               plan_city_grammar_upgrade CITY-G0..G2 may run in parallel when files don't overlap MIG slices.
#
# EXPOSURE SNAPSHOT (measured against src/, 2026-07-03):
#   custom render-graph nodes (Node/ViewNode/RenderLabel): 37+20+16 hits in 10 files  ← BIGGEST ITEM
#   bevy_egui imports: 131 files (ecosystem gate) · bevy_vector_shapes: 1 file · tilemap: feature-off
#   SceneRoot: 2 files · TextFont: 5 files · bevy_default(): 5 files · ExtractSchedule: 4 hits (1 file)
#   RenderStartup already used: 10 files (good — 0.19 continues this) · Msaa: 10 hits
#   Messages API already 0.18-style (add_message=83, MessageReader/Writer=115; EventReader/Writer=0) ✓
#   Query<Entity> bare: 1 hit · derive(Component,Resource) doubles: 0 · ReflectResource: 0 ·
#   non_send_resource: 0 · ExecutorKind: 0 · ShaderStorageBuffer: 0 · Frustum: 0 · Skybox: 0 ✓
#   own rand=0.8.5/noise=0.8.2 are independent of bevy's rand 0.10 — no forced upgrade [verify no interop]

# ═════════════════════════════════════════════════════════════════════
# GROUND RULES FOR EXECUTING AGENTS (read before any slice)
# ═════════════════════════════════════════════════════════════════════
# 1. One slice = one commit = one issue code. Never batch unrelated renames.
# 2. Mechanical slices (MIG-M#): grep the OLD pattern first, record hit count in the commit
#    message, replace, re-grep to prove 0 remaining, cargo check. No judgment calls — if a hit
#    doesn't match the recipe exactly, STOP and hand to @sim-steward with file:line.
# 3. Never fix unrelated warnings/smells in a migration slice (that's plan_cleanup's lane).
# 4. Reason on validate-report output, not raw cargo stderr [validation-first].
# 5. Migration guide is authority; this checklist is the index into it. When guide and this file
#    disagree, guide wins — note the discrepancy in HANDOFF.

# ═════════════════════════════════════════════════════════════════════
# PHASE 0 — GATES (block everything; steward-owned)
# ═════════════════════════════════════════════════════════════════════

MIG-G1 | HARD | Ecosystem dependency gate — check crates.io/GitHub for 0.19-compatible releases:
  · bevy_egui (0.39 → 0.19-compatible line)   ← CRITICAL: 131 files, entire GUI. No release = no migration.
  · bevy_ecs_tilemap (optional feature, default OFF — may lag; acceptable to gate feature out temporarily)
  · bevy_hanabi (optional hanabi_l3 — acceptable to gate out temporarily)
  · bevy_vector_shapes (1 file: render/tactical_vector_overlay.rs — if it lags, vendor or stub the overlay behind a feature)
  · egui/image/arboard version chain pinned per bevy_egui's compat table (Cargo.toml comment already tracks this)
  EXIT: written compat matrix in HANDOFF; go/no-go per crate.

MIG-G2 | HARD | Baseline capture on 0.18 BEFORE M1 bump on master:
  cargo test --lib green list · stage5_full_app_live.json refresh · full --test matrix run ·
  frame_perf witness baseline (PERF_NO_VSYNC=1) — this is the perf comparison anchor for MIG-A#.
  Commit Cargo.lock snapshot. EXIT: baselines in debug_runs/ + HANDOFF note.

MIG-G3 | SOFT | Feature-flag audit for 0.19 Cargo changes (do in same slice as bump):
  · audio no longer implied by 2d/3d/ui — we don't use bevy audio: verify, then exclude via default-features
  · feature collections moved (bevy_window→common_api, custom_cursor→default_platform)
  · `multi_threaded` now required explicitly for parallel transform propagation — WE NEED THIS (add it)
  · occlusion culling no longer experimental — note for MIG-A

# ═════════════════════════════════════════════════════════════════════
# PHASE 1 — MECHANICAL RENAMES (MIG-M#, lesser-agent slices, XS–S each)
# Order within phase is free; all on master after the version bump compiles enough to iterate.
# Slice recipe format: OLD → NEW | scope | verify
# ═════════════════════════════════════════════════════════════════════

MIG-M1 | Cargo bump slice (steward pairs with an agent):
  bevy = "0.19" + feature adjustments (MIG-G3) + ecosystem crates per MIG-G1 matrix.
  Expect the build to break — that's the work queue for M2–M9/R1–R6. Record initial error count.

MIG-M2 | Scene → WorldSerialization renames (2 files + any RON/docs refs):
  SceneRoot → WorldAssetRoot · Scene → WorldAsset (type positions only) · DynamicScene → DynamicWorld (0 hits, verify)
  SceneInstanceReady → WorldInstanceReady (0 hits, verify) · SceneSpawner → WorldInstanceSpawner (0 hits, verify)
  scope: grep -rn "SceneRoot\|Handle<Scene>\|SceneInstanceReady" src | verify: 0 remaining, cargo check.
  NOTE: new `bevy_scene` = BSN next-gen scenes; do NOT confuse imports. Our procedural module scene
  catalog (RN-EXT procedural_module_extract) references scene handles — check those 2 files first.

MIG-M3 | Text API (5 files: gui/app_shell, in_game_hud, pause_menu_bevy, pressure_tooling, style/fonts):
  TextFont::font: Handle<Font> → FontSource (append .into() on existing handles)
  TextFont::font_size: f32 → FontSize::Px(value)
  TextLayout::new_with_justify/linebreak/no_wrap → TextLayout::justify/linebreak/no_wrap
  PositionedGlyph::span_index → section_index (grep, likely 0)
  verify: grep "font_size:\s*[0-9]" src → 0 remaining in bevy-text contexts (egui font sizes are unrelated — DO NOT touch egui code).

MIG-M4 | Render misc renames (mechanical, no logic):
  TextureFormat::bevy_default() → ExtractedView::target_format (5 files — needs the view in scope;
    if no view available, STOP → hand to MIG-R owner, it's a real refactor at that site)
  PipelineCacheError → ShaderCacheError · ShaderStorageBuffer → ShaderBuffer (0 hits, verify)
  Hdr import path bevy_render → bevy_camera · ExtractedView::hdr → ExtractedCamera::hdr
  DataFormat → TextureChannelLayout (grep, likely 0)

MIG-M5 | ECS misc (tiny):
  Query<Entity> bare (1 hit) → add Without<IsResource> filter (resources-as-components conflict)
  World::remove_resource_by_id return type Option<()> → bool (grep, likely 0)
  DefaultErrorHandler → FallbackErrorHandler (grep) · System::type_id() → system_type() (0 hits ✓)
  Then AUDIT: grep -n "Query<(\s*)>\|Query<EntityMut>\|Query<EntityRef>" src → each hit gets Without<IsResource>.

MIG-M6 | UI/window/input follow-ups:
  Window exit systems moved to Last + ExitSystems set — verify no conflict with our Last-schedule
    dev writers (DV, orchestrator_health): they must run before ExitSystems or be order-independent.
  InputFocus.0 field access → .get()/.set()/.clear() (grep input_focus, likely 0 — we're egui-first)
  UiWidgetsPlugins/InputDispatchPlugin now in DefaultPlugins — remove manual adds if any (grep).

MIG-M7 | Math/util:
  Affine3::to_transpose/inverse_transpose_3x3 → import bevy::math::Affine3Ext (grep Affine3)
  PlaneMeshBuilder.subdivisions = n → .subdivisions_x/_z(n) (grep, likely 0)
  bevy::render::define_atomic_id → bevy::utils::define_atomic_id (0 hits ✓)

MIG-M8 | Reflection/serde:
  Add #[reflect(Asset)] where Handle<T> fields are serialized (audit IO-SAV dto/wire_format — our
    save pipeline serializes IDs not handles; verify and record NO-CHANGE if true).
  bevy_reflect module reorg — follow compiler hints only; no proactive edits.

MIG-M9 | Msaa/camera audit (10 hits): confirm per-camera Msaa component usage still compiles;
  Camera3d transmission fields → ScreenSpaceTransmission component (grep screen_space_specular, likely 0).

# ═════════════════════════════════════════════════════════════════════
# PHASE 2 — RESOURCES-AS-COMPONENTS FALLOUT (MIG-E#, S, needs judgment — coder_a/b OK with review)
# ═════════════════════════════════════════════════════════════════════

MIG-E1 | ResMut mutability bounds in generic code:
  grep -rn "R: Resource\|T: Resource" src — any generic fn taking ResMut<R> needs
  R: Resource<Mutability = Mutable>. Our dev witness plumbing (DV-WIT common.rs) and validation
  helpers are the likely sites. Fix per compiler errors, don't pre-empt.

MIG-E2 | Broad-query semantic audit (beyond M5's mechanical fix):
  Resources are now entities: any Query<&Transform>-style broad query stays fine, but debug/inventory
  systems that iterate ALL entities (DV sample_ecs_resource_inventory, engine_deep_debug entity
  inventory, stage5 entity counts) will now SEE resource entities — counts shift. Update witness
  expectations + add Without<IsResource> where the intent is "world entities only".
  EXIT: --test matrix entity-count witnesses re-baselined with a one-line note in each.

MIG-E3 | World::clear_entities now clears resources — grep clear_entities (test harness world resets);
  replace with targeted despawns if resource survival is assumed.

# ═════════════════════════════════════════════════════════════════════
# PHASE 3 — RENDER GRAPH → SYSTEMS (MIG-R#, coder-grade, the big one)
# The old Node/ViewNode/RenderLabel architecture is REMOVED. Render passes become ECS systems in
# Core3d/Core2d schedules on the render world, ordered via Core3dSystems::MainPass etc.
# 10 files, all in RN-PRT/RN-WPT/RN-WSV/RN-TRA/RN-MMC territory (see codebase_index):
#   gpu_fire_particle_raster.rs · gpu_particle_draw.rs · gpu_spark_compute.rs · gpu_tile_debug_draw.rs
#   gpu_water_particle_draw.rs · gpu_water_particle_raster.rs · gpu_water_surface_draw.rs
#   gpu_weather_fire_field.rs · minimap_compositor/gpu_compute.rs · terrain_instanced_draw.rs
# ═════════════════════════════════════════════════════════════════════

MIG-R1 | Conversion recipe (write ONCE against the simplest file — gpu_tile_debug_draw.rs — then
  the recipe becomes the template for R2–R6):
  · impl Node/ViewNode { run(...) } → fn system(...) with render-world SystemParams
  · RenderLabel derives + add_render_graph_node/edges → add_systems(Core3d, sys.after(Core3dSystems::MainPass))
    (pick anchor per pass intent: prepass/main/EarlyPostProcess/PostProcess — note PostProcess split;
    prefer batched depth-only prepass hooks per Bevy 0.19 — see MIG-A11)
  · RenderSystems::ManageViews (1 hit) → CreateViews/Specialize/PrepareViews split
  · systems touching MeshPipeline/MeshPipelineViewLayouts must run after MeshPipelineSystems (RenderStartup-created)
  · RenderMeshInstance field access → accessor methods (mesh_asset_id() / set_mesh_asset_id())
  · BaseMeshPipelineKey::from_primitive_topology → from_primitive_topology_and_strip_index
  · SortedRenderPhase::add → add_transient/add_retained (choose: our per-frame draws = add_transient;
    static terrain/tile bulk → evaluate add_retained + NoCpuCulling per MIG-A2)
  · shadow_pass split (per_view vs shared) — only if we hook shadows (grep shadow_pass, likely 0)
  · POST-CONVERT: evaluate folding GpuIndirectDrawSpine CPU batch prep into stock GPU MDI bin unpack (MIG-A10)
  EXIT: recipe doc committed as src/dev/mig_r_render_node_recipe_v1.md + tile debug pass renders in --test visual.

MIG-R2..R6 | Apply recipe per subsystem (one slice each, stage5/VT witness after each):
  R2 fire particles (raster + draw + spark compute — 3 files, same slice, shared bind groups)
  R3 water (particle draw + raster + surface draw — 3 files)
  R4 terrain_instanced_draw + gpu_weather_fire_field
  R5 minimap_compositor/gpu_compute (coordinate with RN-MMC witness M1-M4)
  R6 sweep: grep render_graph|RenderLabel|ViewNode → 0 remaining; delete dead label types.
  RISK: this is where visual regressions live. Each slice gates on: --test visual + fire streaming
  witness + vt_ci_matrix + full_render_diagnostic diff vs MIG-G2 baseline.

# ═════════════════════════════════════════════════════════════════════
# PHASE 4 — VERIFY + ADOPT (MIG-A#, the payoff; post-green only)
# ═════════════════════════════════════════════════════════════════════

MIG-V1 | Full gate: cargo test --lib all green · full --test matrix · stage5 witness green ·
  frame_perf vs MIG-G2 baseline (expect render-thread WIN from 0.19 GPU batching/culling work —
  many_cubes-class scenes ~2.2-2.6x; record actuals) · merge to master.

# ═════════════════════════════════════════════════════════════════════
# RENDER PERFORMANCE FOCUS — Bevy 0.19 upstream wins → this repo (MIG-A10+)
# ═════════════════════════════════════════════════════════════════════
# The migration is NOT "compile green only." Phase 4 must deliberately adopt 0.19 renderer
# improvements where our hot path overlaps Bevy's mesh/instancing pipeline — not only custom
# RN-* graph ports (MIG-R#). Authority unchanged: sim → snapshot → extract; adoption is render-thread.
#
# Repo render hot path today (pre-migration):
#   · RN-TWF tile_world_fallback + terrain_instanced_draw (many static/dirty tiles)
#   · RN-PRT/RN-WPT gpu_*_particle_* + GpuIndirectDrawSpine (instanced quads)
#   · RN-EXT fire_visual_extract CPU light clustering → RequestLocalLight
#   · GU-REP WorldLodBand / visibility_for_band (CPU LOD hints today)
#   · 10 custom RenderLabel/ViewNode files (MIG-R1–R6) — must convert BEFORE trusting stock wins
#
# Upstream win (Bevy 0.19)              | Repo relevance | Adoption slice
# --------------------------------------|----------------|------------------------------------------
# GPU bin-unpack for multi-draw-indirect  | HIGH           | MIG-A10 — after R4/R6: route terrain +
#   batch sets (was CPU prep)             |                | particle indirect through stock MDI path;
#                                         |                | drop duplicate CPU batch-set prep in
#                                         |                | GpuIndirectDrawSpine where redundant.
#                                         |                | Bench: stage5 indirect_count + frame_perf.
# Batched depth-only prepasses            | MED            | MIG-A11 — audit custom passes post-R1;
#   (normals/motion, no material)         |                | fold depth-only prepasses into Bevy batch
#                                         |                | groups; likely hits gpu_tile_debug_draw,
#                                         |                | minimap depth if any remain.
# Sparse mesh uniform buffer uploads      | HIGH           | MIG-A12 — static building modules + terrain
#   (changed uniforms only)               |                | tiles: mark bulk meshes static; verify
#                                         |                | upload_bytes/frame drops in frame_perf.
# GPU clustering (lights/probes/decals)   | HIGH           | MIG-A13 — replace/supplement CPU
#                                         |                | build_fire_light_clusters scratch path
#                                         |                | (fire_visual_extract.rs) with Bevy GPU
#                                         |                | clustering where LocalLightExtractSet
#                                         |                | can consume stock clusters; measure many_lights
#                                         |                | class scene if we add a micro-bench witness.
# Increased render system parallelism     | AUTO           | MIG-V1 note only — record render-thread
#                                         |                | wall time delta vs MIG-G2 baseline.
# Visibility ranges checked on GPU      | MED–HIGH       | MIG-A14 — align GU-REP LOD bands with
#                                         |                | Bevy visibility ranges; reduce CPU
#                                         |                | visibility_for_band work per frame.
# Batched morph targets (storage buf)     | LOW            | MIG-A15 — defer unless procedural
#                                         |                | modules ship morph targets; grep morph.
# NoCpuCulling on static bulk meshes      | HIGH           | MIG-A2 (expanded) — tile atlas quads,
#                                         |                | terrain instances, module LOD0 shells;
#                                         |                | skip CPU culling when GPU path owns vis.
# Previous-transform copy only on mutate  | MED            | MIG-A16 — pairs with A1/A2; static tiles
#                                         |                | should not pay prev-transform writes.
# Mesh collection GPU shared memory         | MED            | MIG-A17 — after R1 recipe: prefer Bevy
#                                         |                | mesh collection over bespoke gather loops
#                                         |                | in custom draw files where possible.
# Change lists vs full-tick specialization| AUTO           | MIG-V1 — note in perf report; helps
#                                         |                | large entity counts in editor/worldgen.
# Clustering heuristic uses last frame    | AUTO (Bevy)    | MIG-A13 benefit if we adopt stock lights.
# Direct memcopy vs encase for clustering | AUTO (Bevy)    | bundled into A13 measurement.
# Parallel mesh collection gather         | MED–HIGH       | MIG-A18 — benchmark terrain_instanced_draw
#                                         |                | + tile fallback after migration; target
#                                         |                | scenes with 50k–200k moving tile cells.
# Dirty transform tree buffered channels  | HIGH           | MIG-A1 synergy — StaticTransformOptimizations
#                                         |                | + many static chunks = largest win for
#                                         |                | our iso map (mostly static geometry).
# Entity removal scans from list end      | AUTO           | minor; note in MIG-V1 changelog only.
#
# MEASUREMENT CONTRACT (render adoption slices):
#   · Reuse MIG-G2 frame_perf baseline (PERF_NO_VSYNC=1) — compare upd_* on render substages, not STALL spam.
#   · Per MIG-A10–A18 slice: one witness JSON under debug_runs/mig_bevy_019/ with before/after ms.
#   · Gate: stage5_full_app_live.json green + --test visual no regression vs MIG-G2 screenshots hash optional.
#   · Do NOT adopt stock MDI/GPU clustering until MIG-R6 grep-zero (custom graph removed) — else double work.
#
# PRIORITY ORDER (post MIG-V1 green):
#   1. MIG-A1 + A2 + A16 (static transform / NoCpuCulling / prev-transform) — cheapest, iso-map shaped
#   2. MIG-A10 + A12 (MDI batch unpack + sparse mesh uniforms) — particle + terrain spine
#   3. MIG-A13 + A14 (GPU light clustering + visibility ranges) — fire/local lights + LOD
#   4. MIG-A11 + A17 + A18 (prepass batching, mesh collection, parallel gather) — audit-driven
#   5. MIG-A3 + A4 (DiagnosticsOverlay, RenderErrorHandler) — plan_cleanup D3 synergy

Adoption backlog (each its own post-merge slice; ordered by value/effort):
MIG-A1 | StaticTransformOptimizations::Enabled — our tile/terrain/building entities are overwhelmingly
        static; near-free transform propagation win. (bevy_city uses exactly this.)
MIG-A2 | NoCpuCulling on static bulk meshes (tile fallback sprites, terrain instances) once GPU
        culling covers them — measure with frame_perf, don't assume.
MIG-A3 | DiagnosticsOverlayPlugin — replaces part of our custom HUD dev overlay / frame budget panels;
        feeds plan_cleanup D3 (gate ours out, adopt built-in for the basics).
MIG-A4 | RenderErrorHandler recovery policies (DeviceLost → Recover) — our GPU teardown/stall_watch
        lane gets a principled backend; ties RN-GPU gpu_surface_teardown.
MIG-A5 | Remote entity reservation — Wave C streaming (IO-STR task_pool) can pre-allocate entity IDs
        on worker threads; removes a main-thread sync point in chunk apply [K04].
MIG-A6 | contiguous_iter()/contiguous_iter_mut() SIMD on hot sim loops — pairs with plan_cleanup P1
        (ember scan) and P3 (atmosphere fill); measure per loop.
MIG-A7 | Observer run_if + delayed commands — simplify our cadence latches where they exist only to
        defer one-shot work (LiveProofCadence writers, staged spawn messages).
MIG-A8 | SettingsPlugin — replace hand-rolled settings persistence (shell_persistence, options UI backing).
MIG-A9 | BSN scenes + SceneComponent — DO NOT retrofit engine-wide; adopt first in the procedural
        building assembly lane where composable/patchable scenes fit naturally → handed to
        plan_city_grammar_upgrade_v1 CITY-C6. EditableText similarly: only if we move any HUD off egui.
# Render perf adoption (see RENDER PERFORMANCE FOCUS table above):
MIG-A10 | GPU MDI batch-set bin unpack — adopt stock path for terrain/particle indirect; retire redundant CPU prep.
MIG-A11 | Batched depth-only prepasses — audit post-R1; batch norm/depth-only custom passes.
MIG-A12 | Sparse mesh uniform uploads — static module/tile meshes; measure upload_bytes/frame.
MIG-A13 | GPU light/probe clustering — evaluate vs CPU fire_visual_extract clustering.
MIG-A14 | GPU visibility ranges — align WorldLodBand hints with Bevy visibility ranges.
MIG-A15 | Batched morph targets — defer; grep-driven.
MIG-A16 | Previous-transform write only on mutate — static tile bulk.
MIG-A17 | Stock mesh collection / shared memory — reduce bespoke gather in RN-* draw files.
MIG-A18 | Parallel mesh collection benchmark — terrain_instanced + high cell-count scenes.

# ═════════════════════════════════════════════════════════════════════
# QUEUE SEED (Phase 0 + 1; later phases seeded at phase gate)
# ═════════════════════════════════════════════════════════════════════
# id                | issue  | owner       | effort | exit
# MIG-P0-G1-001     | MIG-G1 | sim-steward | S      | compat matrix in HANDOFF; go/no-go
# MIG-P0-G2-001     | MIG-G2 | coder       | S      | baselines committed (lockfile + witnesses)
# MIG-P1-M1-001     | MIG-M1 | coder       | S      | master compiles far enough to enumerate errors; count recorded
# MIG-P1-M2-001     | MIG-M2 | coder_a     | XS     | grep-zero SceneRoot/Scene-as-asset; cargo check
# MIG-P1-M3-001     | MIG-M3 | coder_a     | S      | 5 text files migrated; egui untouched
# MIG-P1-M4-001     | MIG-M4 | coder_b     | S      | renames done or STOPped-with-note per site
# MIG-P1-M5-001     | MIG-M5 | coder_b     | XS     | broad-query filters added
# MIG-P1-M6..M9     | …      | coder_a/b   | XS ea  | grep-zero per recipe
# Phase 1 gate: error count from M1 reduced to render-graph-only errors → seed Phase 2/3 queues.

# ═════════════════════════════════════════════════════════════════════
# REVIEW NOTES (2026-07-03)
# ═════════════════════════════════════════════════════════════════════
# Strengths: exposure snapshot is accurate (37+ RenderLabel hits, bevy_egui gate, Messages already 0.18).
# MIG-R1–R6 correctly identified as biggest risk; recipe-first approach is right.
# Gap closed: RENDER PERFORMANCE FOCUS section maps Bevy 0.19 upstream renderer wins (MDI bin unpack,
# sparse mesh uniforms, GPU clustering, NoCpuCulling, parallel mesh collection, etc.) to MIG-A10–A18.
# Critical sequencing: MIG-R6 (custom graph zero) BEFORE MIG-A10/A13 — otherwise duplicate CPU+GPU paths.
# Ecosystem: bevy_egui remains hard gate (MIG-G1); tilemap/hanabi can feature-gate temporarily.
# Perf proof: extend MIG-G2 baseline with render substage upd_* + optional debug_runs/mig_bevy_019/.
# Conflict: freeze plan_cleanup Phase 2+ and plan_schedule_sync Wave 2+ while MIG-P1–P3 in flight on master.
# HANDOFF + development_plan_index linked 2026-07-03 as P0 PRIMARY.
