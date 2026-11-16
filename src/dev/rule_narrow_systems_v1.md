# RULE-NARROW-SYSTEMS-v1 — no god-systems, no bundles to dodge the param cap

| Field | Value |
|:---|:---|
| **Id** | **RULE-NARROW-SYSTEMS-v1** |
| **Status** | **SIGNED** (standing rule) |
| **Date** | 2026-11-16 |
| **Siblings** | [`plan_anti_clone_kernel_v1.md`](plan_anti_clone_kernel_v1.md) (no clone farms) · `bevy-simulation-grade` invariant 6 |

## 1. Rule

1. **One domain per system.** A system reads and writes one domain (water, fire, construction, logistics, …). It may read **published read-only surfaces** of at most 2–3 other domains.
2. **At most 8 params.** If you reach Bevy's 16-param cap, the design is wrong. It is not a packaging problem.
3. **`#[derive(SystemParam)]` / `#[derive(QueryData)]` only group one domain's params.** `WaterReads { field, reservoir, gauges }` is fine. `ShellSyncA/B/C`, or a bundle that exists only to fit under 16, is banned. A bundle named `…A/B/C`, `…Extras` or `…Params2` is always a violation.
4. **The owner writes.** Cross-domain writes go through a request (`Message` or queue resource) that the owning system drains. They are never extra `ResMut`s on the caller.
5. **Gate.** A PR that adds a param to a system already at 8+, or adds a cross-domain bundle, must cite this rule and choose one of the patterns below.

**Why.** Wide systems serialize the schedule: every `ResMut` blocks parallelism. They also create hidden second writers, and they grow with every feature until they hit the cap. That is how we end up with `A/B/C` bundles.

## 2. Pick the pattern by symptom

| Symptom | Pattern | Section |
|:---|:---|:---|
| Debug dump or diagnostics read every domain | **Per-domain reporter** | 3.1 |
| Gameplay step must change things in other domains | **Request → owner drains** | 3.2 |
| One happening fans out to many domains | **Message per consumer / observers** | 3.3 |
| Decision (AI, scoring, UI card) needs facts from many domains | **Published read-only frames + staged pipeline** | 3.4 |
| Many `Option<Res<…>>` for optional features | **Run conditions + per-feature systems** | 3.5 |
| Many queries over the same entities | **One `QueryData` query** | 3.6 |
| Save/load, scenario setup, world bootstrap touching everything | **Registry of per-domain adapters over `&mut World`** | 3.7 |
| Test or harness needs lots of state | **Drive the `World` directly** | 3.8 |
| UI panel shows many domains | **Per-panel view model built from frames** | 3.9 |

## 3. Patterns

### 3.1 Per-domain reporter (dumps, metrics, diagnostics)

Bad: one `write_sim_run_dump` that reads 40 resources across 8 bundles.

```rust
#[derive(Resource, Default)]
pub struct SimRunDump { sections: BTreeMap<&'static str, serde_json::Value> }
impl SimRunDump { pub fn put(&mut self, k: &'static str, v: impl Serialize) { … } }

#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub struct SimDumpSet; // runs only when dump is armed (run condition)

// in water's own plugin (debug cfg only):
fn dump_water(field: Res<SurfaceWaterField>, pool: Res<ReservoirLevelPool>, mut dump: ResMut<SimRunDump>) {
    dump.put("water", WaterDumpCounts::from(&*field, &*pool));
}
app.add_systems(PostUpdate, dump_water.in_set(SimDumpSet));

// central writer: ONE param
fn write_dump_json(dump: Res<SimRunDump>) { … }
```

Several systems write the same `SimRunDump`, each to its own key. That is acceptable only for this debug-only accumulator. The `ResMut` serializes them, which is fine because they are cheap. If ordering matters, chain them in `SimDumpSet`. A new domain adds a reporter system; it never edits the central writer.

### 3.2 Request → owner drains (cross-domain writes)

Bad: `step_drone_missions` holding `ResMut<BallisticFlightBook>`, `ResMut<AudioOneShotQueue>`, `ResMut<…>`.

```rust
#[derive(Message)] pub struct LaunchMissile { site: Entity, target: Vec2, warhead: WarheadId }

fn step_drone_missions(…, mut launch: MessageWriter<LaunchMissile>) { launch.write(LaunchMissile { … }); }

// combat owns the flight book; it is the only writer
fn apply_missile_launches(mut rx: MessageReader<LaunchMissile>, mut book: ResMut<BallisticFlightBook>, mut audio: MessageWriter<PlayOneShot>) { … }
```

Already in the repo: `MissileSortieLaunchQueue`, `BlastIntentQueue`, `LogisticsOrderRequest`, `CommitConstructionSiteEvent`. Use a `Message` when ordering within the frame does not matter, and a queue resource when it must be deterministic or seeded (a `Vec` drained in a fixed order).

### 3.3 Fan-out: one message, one reader per consumer

Bad: `on_dam_breach` updating water, power, cooling, trees, audio and HUD in one system.

```rust
#[derive(Message)] pub struct DamBreached { dam: Entity, cell: IVec2, q_peak_m3s: f32 }

fn hydro_on_breach(mut rx: MessageReader<DamBreached>, mut dam: Query<&mut DamBody>) { … }      // dam domain
fn power_on_breach(mut rx: MessageReader<DamBreached>, mut plants: Query<&mut PlantOutput>) { … } // power domain
fn hud_on_breach(mut rx: MessageReader<DamBreached>, mut toasts: ResMut<HudToasts>) { … }        // UI
```

Each reader has its own cursor, so there is no coupling between them. For entity-scoped reactions, use an observer (`commands.trigger(…)` + `app.add_observer`).

### 3.4 Many-domain decisions: frames + stages

Bad: an AI or score system reading raw state from 10 domains.

1. Each domain publishes a small **read-only frame** with one builder, e.g. `SimAttentionFrame`, `AtmosphereFrame`, `WaterFrame`. The builder is that frame's sole writer.
2. Split the decision into stages, each with its own output resource:
   `sense` (reads ≤3 frames → `ThreatPicture`) → `evaluate` (`ThreatPicture` + `EconomyFrame` → `Plan`) → `act` (`Plan` → requests per 3.2).
3. If one stage still needs 6+ frames, it is two decisions. Split it again.

A per-domain read bundle (`AiWorld { sites, hqs, fronts }`, all in the AI domain) is fine. A bundle that gathers fire, water, logistics and construction for one AI system is not. Publish frames instead.

### 3.5 Optional features: run conditions, not `Option<Res<_>>` piles

Bad: one system with 12 `Option<Res<Feature>>`, each guarded by `if let Some`.

```rust
app.add_systems(Update, (
    sample_fire_overlay.run_if(resource_exists::<FireField>),
    sample_heat_overlay.run_if(resource_exists::<DenseHeatField>),
).in_set(OverlaySampleSet));
```

Each feature gets its own small system that writes its own part of the output. `Option<Res>` stays for one or two truly optional inputs to a single algorithm.

### 3.6 Many queries over the same entities: one `QueryData`

Bad: `Query<&Chunk>`, `Query<&ChunkCellMatrix>`, `Query<&ChunkCellStand>`, … joined by entity id.

```rust
#[derive(QueryData)]
pub struct ChunkView { chunk: &'static Chunk, matrix: &'static ChunkCellMatrix, stand: Option<&'static ChunkCellStand> }
fn sample(q: Query<ChunkView>) { for c in &q { … } }
```

This is one param, one archetype walk and no lookups. Use `#[query_data(mutable)]` when it writes.

### 3.7 Whole-world operations: adapter registry over `&mut World`

Bad: save/load or scenario-setup systems with 20 params that grow with every domain.

```rust
pub trait SaveSlice: Send + Sync + 'static {
    const KEY: &'static str;
    fn save(world: &World) -> ron::Value;
    fn load(world: &mut World, v: &ron::Value);
}
// each domain: impl SaveSlice for SurfaceWaterOverlay { … } and app.register_save_slice::<SurfaceWaterOverlay>()
// pipeline: exclusive system iterates the registry. It is not hot-path, so &mut World is correct.
```

Exclusive `&mut World` systems are for **rare structural work**: load, setup, teardown. Never use them per tick.

### 3.8 Tests and harness

Don't build a `TestHarnessEnterWorld` mega-param. Tests and the harness use `World` directly: `world.resource_mut::<T>()`, `world.run_system_once(sys)`, `world.spawn(…)`. Each scenario step is a small function over `&mut World`, or it sends the same requests the game does (3.2).

### 3.9 UI panels

Each panel gets a **view model** resource, e.g. `PlantCardVm`, built by one system from ≤3 frames. The drawing system reads only the view model. If a panel needs 6 domains, it gets 2–3 view-model builders, each writing its own section.

## 4. Allowed exceptions

- A per-domain read or write bundle that genuinely belongs to one domain: `WaterTileProbe`, `ConstructionMapPick`, `ConstructionTerrainConform`.
- Render-world extraction systems that must read several main-world sources in one `ExtractSchedule` pass. They are read-only and can still use `QueryData`. Document them with a `ScaffoldContract` if they keep growing.
- Exclusive `&mut World` systems for load/save/setup (3.7).

## 5. Audit list (2026-11-16): existing `derive(SystemParam)` bundles to check

These come from a grep. They are **not verdicts**. Classify each one before changing it (cleanup-intelligence), then fix it with the pattern shown.

| Bundle(s) | File | Likely pattern |
|:---|:---|:---|
| `WorldParams`, `FireChainParams`, `GameParams` | `src/dev/sim_run_dump.rs` | 3.1 |
| `DamFloodDumpParams`, `FidelitySampleReads`, `PresentationSampleReads`, `QuadrantDumpParams` | `src/dev/sim_run_dump_*.rs` | 3.1 |
| `InventoryProbe` | `src/dev/test_run_instrumentation.rs` | 3.1 / 3.8 |
| `MissionStepWriters`, `MissionStepReads`, `MissionStepQueries` | `src/combat/drone_missions.rs` | 3.2 + 3.4 |
| `AiOrderSinks`, `AiWorld` | `src/ai/faction/mod.rs` | 3.2 / 3.4 (keep if all AI) |
| `ExecuteHydroOpt`, `ExecuteSideEffects` | `src/construction/construction_pipeline.rs` | 3.3 (side effects → messages; the execute funnel stays the single writer) |
| `ShellSyncA`, `ShellSyncB`, `ShellSyncC`, `MainMenuExit` | `src/gui/editor/world_preview/world_gen_bevy_shell.rs` | 3.9 / 3.5 (**A/B/C naming = violation**) |
| `TileFallbackGpuStampInputs`, `TileFallbackDualViewAtlasInputs`, `TileFallbackRasterExtras` | `src/render/tile_world_fallback.rs` | 3.5 / 3.6 |
| `TestHarnessEnterWorld` | `src/engine/test_harness.rs` | 3.8 |
| `MenuDebugBootstrap` | `src/gui/app_shell.rs` | 3.7 (or delete: no debug panels) |
| `VisualDiagCtx` | `src/render/probes/visual_diagnostics.rs` | 3.1 |
| others (29 files total) | `rg "derive\(SystemParam\)" src` | check against §1.3 |

Track fixes as cleanup items (CQ-xx) in [`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md) when they are picked up.
