## Bevy 0.18 — schedule refactor runbook

Cross-plugin **`SystemSet`** ordering and plugin inventory: **[`../guides/ecs_systems_schedule_runbook_v1.md`](../guides/ecs_systems_schedule_runbook_v1.md)** · step packs [`runbook/README.md`](runbook/README.md).

---

## Bevy 0.10.x → 0.18 migration plan (validated)

> **STATUS:** ✅ **Active game/editor stack targets Bevy 0.18** (`bevy` 0.18 + `bevy_egui` 0.39). Core migration (schedules, states, input, egui pass, messages, `Camera2d`, `NextState::set_if_neq` on menu/splash/transitions, `AppExit::Success`) is applied. **Optional later:** Phase G feature flags, custom assets/render paths, `bevy-inspector-egui` when a matching release is chosen, warning cleanup (`cargo fix`).

Source notes used:
- Official Bevy **0.17 → 0.18** migration guide excerpts (provided verbatim by the user; treated as authoritative).
- **Crate pins:** read repo root `Cargo.toml` — this plan’s historical bullets may predate current versions; never assume versions from this doc alone.

> **Prompt use:** Cross-check `designer_questions/tools_ui/` + **`tools_ui/spec/README.md`** for UI coupling · verify phase checklists with `cargo check` + targeted `rg` · cite Bevy guide URL/version when quoting API moves · `ASK:` if a phase policy conflicts.

Validation outcome: **the previous version of this plan was directionally correct but incomplete and partly speculative**. Many items it listed as "verify" are now confirmed concrete API moves; several large-impact 0.18 changes were missing. This document replaces it.

> ⚠ Important: the leap is **0.10 → 0.18**, i.e. 8 minor versions. The 0.17→0.18 notes alone are not enough — every Bevy minor in between has its own breaking changes. The plan below splits the work into version hops, then layers the 0.18-specific items as the final pass.

---

## 0) Strategy: stepwise hops, not a single bump

Do **not** jump from 0.10 directly to 0.18. Per Bevy convention, migrate version-by-version:

```
0.10 → 0.11 → 0.12 → 0.13 → 0.14 → 0.15 → 0.16 → 0.17 → 0.18
```

For each hop:

1. Branch (`bevy-x.y`).
2. Bump only `bevy`, `bevy_egui`, `bevy-inspector-egui` to the matching pinned versions.
3. `cargo check` → fix mechanical errors only.
4. Smoke run (`cargo run --bin world_generator`).
5. Commit, then bump to next minor.

Do NOT mix gameplay refactors into version-bump commits.

---

## 1) High-impact API milestones between 0.10 and 0.18

These are the boulders to clear before 0.18 release-note items even matter:

### 1.1  Schedules rewrite (≈0.11–0.12)
- `add_system`, `add_system_set`, `SystemSet::new().with_system`, `OnUpdate(State::X)`, `in_base_set` are all **gone**.
- Replacement:
  - `app.add_systems(Update, my_system)`
  - `app.add_systems(Update, my_system.run_if(in_state(BaseState::Simulation)))`
  - `app.add_systems(Update, (a, b).chain())`

Repo hot-spots (active and legacy alike — must be cleaned even in legacy files because 0.11+ won’t compile any of them):

| File | Pattern | Action |
|---|---|---|
| `src/engine/engine.rs` | `add_plugin`, `add_state(...)`, commented `add_system_set` blocks | rewrite to `add_plugins`, `init_state`, `add_systems(...).run_if(in_state(...))` |
| `src/engine/sets.rs` | `app.add_system(my_system.in_base_set(MySet::BeforeRound))` | `add_systems(Update, my_system.in_set(MySet::BeforeRound))` |
| `src/gui/main_menu.rs` | `app.add_state::<MainMenuState>()`, `.in_set(OnUpdate(BaseState::MainMenu))` | `init_state::<MainMenuState>()`, `.run_if(in_state(BaseState::MainMenu))` |
| `src/systems/production/power_systems.rs` (LEGACY) | `add_system_set(SystemSet::new()...)` | already legacy-tagged; rewrite or delete during 0.11 hop |
| `src/main.rs`, `src/bin/world_generator.rs`, `src/engine/engine_with_worldgen.rs` | `.add_plugin(...)` chain | replace with `.add_plugins((...))` |

### 1.2  States rework (0.12–0.13)
- Resource `State<S>` / `ResMut<State<S>>` no longer used to set state.
- Read with `Res<State<S>>`; write with `ResMut<NextState<S>>` + `next_state.set(...)`.
- 0.18 then adds: `set` always triggers `OnEnter`/`OnExit` even on same state — use `set_if_neq` if you depend on the old behavior.

Repo touch points: `src/engine/states.rs`, every plugin using `OnUpdate`, `gui/main_menu.rs`.

### 1.3  Input rename (0.14)
- `Res<Input<KeyCode>>` → `Res<ButtonInput<KeyCode>>`.

Repo (active) hot-spots:
- `src/bevysubengines/world_generator_plugin.rs:165`
- `src/systems/agents/agent_plugin.rs:35`
- `src/terrain/generation/world_generation_plugin.rs:41`
- `src/gui/ui_windows.rs:47`
- `src/engine/transitions.rs:20`

### 1.4  `add_plugin` deprecated → `add_plugins` (0.15–0.16, removed in 0.17)
- All `.add_plugin(X)` chains must become `.add_plugins(X)` or `.add_plugins((X, Y, Z))`.

Repo: `engine_with_worldgen.rs`, `engine/engine.rs`, `bin/world_generator.rs`, `main.rs`.

### 1.5  Bundles → required components (0.15)
- Existing tuple bundles (`commands.spawn((A, B, C))`) keep working.
- Old derived `#[derive(Bundle)]` and `Camera3dBundle`-style spawn helpers are removed/renamed; many become required components.

Repo: `Camera2dBundle::default()` in `bin/world_generator.rs` — needs `Camera2d` + required components in 0.15+.

### 1.6  Time API (0.13)
- `Time::delta_seconds()` → `Time::delta_secs()` (deprecation chain stabilized by 0.16+).

Repo:
- `src/entities/production/core/production_care.rs:60`
- `src/systems/navigation/road_vehicles_motion.rs:32`

### 1.7  Reflect / `register_type`
- `#[derive(Reflect)]`, `#[reflect(Component)]`, `App::register_type::<T>()` mostly survive but their imports moved (`bevy::reflect`, `bevy_reflect`).

Repo (legacy): `entities/production/concrete/sys.rs`, `entities/production/aluminum/production_sys.rs`. These are already `LEGACY MODULE`; rewrite during the 0.13 hop or keep gated.

### 1.8  Image / render asset textures
- `Image::new(...)` and `TextureDescriptor` constructors are unchanged in shape but `view_formats: &[]` and `sample_count` semantics tightened.
- `Image::reinterpret_size`/`reinterpret_stacked_2d_as_array` return `Result` (0.18 — see §2).

Repo: `src/gui/editor/world_preview.rs` — already uses `view_formats: &[]`, will need 0.18 `Result` handling.

---

## 2) 0.17 → 0.18 specific items (final pass)

These are the items from the user-supplied 0.18 release notes. Grouped by impact for this repo.

### 2.1  Hard renames / mechanical updates

| Item | Action in this repo |
|---|---|
| `Gizmos::cuboid` → `Gizmos::cube` | none yet (no usage) |
| `bevy::ptr::dangling_with_align()` → `NonNull::without_provenance()` | none yet (no usage) |
| `clear_children` / `clear_related` / `remove_children` / `remove_child` → `detach_*` | grep before each hop; none today |
| `get_many_mut` → `get_disjoint_mut` (and friends) | none today |
| `BorderRect` directional fields → `min_inset` / `max_inset` `Vec2` | check when UI is added |
| `Atmosphere`: most fields → `medium: Handle<ScatteringMedium>` | not used |
| `MaterialDrawFunction` → split per RenderPhase | not used (no custom materials yet) |
| `MaterialPlugin::prepass_enabled/shadows_enabled` → `Material::enable_prepass()/enable_shadows()` | not used |
| `TrackedRenderPass::set_index_buffer` no longer takes offset | not used |
| `ImageRenderTarget::scale_factor` is now `f32` (no `FloatOrd`) | not used |
| `LineHeight` is its own component (removed from `TextFont`) | check when text is added |
| `AnimationTarget` split into `AnimationTargetId` + `AnimatedBy` | not used |
| `bevy::scene` / `bevy_asset` no longer re-export `ron` | add `ron` direct dep if/when needed |
| `bevy_gizmos` rendering split → `bevy_gizmos_render` feature | enable feature if `default-features = false` |
| `bevy_input` source features (`mouse`, `keyboard`, `gamepad`, `touch`, `gestures`) | not affected if `bevy_window` is enabled |
| `animation` feature renamed to `gltf_animation`; picking backend feature renames | reflect in `Cargo.toml` |
| `#[reflect{...}]` / `#[reflect[...]]` → `#[reflect(...)]` only | follow when re-enabling reflect on legacy production files |
| `BevyManifest::shared` → scope-style API | only relevant if writing macros |

### 2.2  Behavior changes (likely silent breakage)

| Item | Risk | Mitigation |
|---|---|---|
| **Same-state transitions now fire `OnEnter`/`OnExit`** | High | Audit every `next_state.set(...)`; switch to `next_state.set_if_neq(...)` where the old "no-op when equal" behavior was assumed. Repo: anywhere we add state machines (terrain gen, menus). |
| **System combinators (`and`/`or`/etc.) treat failures as `false` instead of propagating errors** | Medium | Review any `run_if(a.and(b))` chains added during the upgrade. |
| **`DragEnter` now fires on drag start over already-hovered entity** | Low | Only relevant if drag-drop UI is built. |
| **Text-only areas of `Text` nodes are pickable, non-text areas are not** | Low | Add intercepting parent node if old behavior is needed. |
| **`Aabb` updates automatically when a mesh/sprite changes** | Low | Remove any old `entity.remove::<Aabb>()` workarounds; or add `NoAutoAabb` if you want manual control. |

### 2.3  ECS / world surface changes (most relevant once on 0.17)

| Item | What to do |
|---|---|
| `Internal` component removed | If a future entity inspector relies on it, define your own categorization. |
| `Entities` API rework (alloc/free/flush gone, `EntityRow` → `EntityIndex`, `EntityNotSpawnedError`, etc.) | Fix per compile error; affects any low-level entity APIs (we don’t use them today, but `idgen.rs`/agents may once they grow). |
| `QueryEntityError::EntityDoesNotExist` → `NotSpawned` | Mechanical rename. |
| `get_components`/`get_components_mut_unchecked` → `Result<_, QueryAccessError>` | Mechanical. |
| Entity pointers (`EntityRef`, `EntityWorldMut`) assume entity is spawned | Use `EntityWorldMut::is_spawned` if needed. |
| `Bundle::component_ids` / `get_component_ids` return iterators | Only matters if implementing `Bundle` manually. |
| `ArchetypeQueryData` trait (split off from `QueryData`) | Bound `ExactSizeIterator` queries with `ArchetypeQueryData` instead of `QueryData`. |
| `ThinSlicePtr::get` → `ThinSlicePtr::get_unchecked` | Mechanical. |
| `Replaced Column with ThinColumn` | Only matters at low-level ECS. |
| `Derive(Resource)` rejects non-`'static` lifetimes | Audit any `Resource` with lifetimes (none today). |

### 2.4  Rendering

| Item | What to do |
|---|---|
| **`RenderTarget` is now a required component on `Camera`** | When adding cameras: `commands.spawn((Camera3d::default(), RenderTarget::Image(handle.into())))`. |
| `AmbientLight` split into component + `GlobalAmbientLight` resource | If we add lighting: insert `GlobalAmbientLight` resource, optional `AmbientLight` per-camera. |
| `RenderPipelineDescriptor` / `ComputePipelineDescriptor` now take `BindGroupLayoutDescriptor` | Only relevant for custom pipelines. |
| `BindGroupLayout` labels are now required in `AsBindGroup` | If we add custom materials. |
| `Image::reinterpret_size` / `reinterpret_stacked_2d_as_array` return `Result` | Wrap in `?` / handle. |
| `MeshletMesh` asset format changed → regenerate | Not used. |
| `TilemapChunk` origin moved to bottom-left | Not used. |
| Mesh getters → `try_*` variants when `Assets<Mesh>` keeps `RENDER_WORLD`-only meshes | Use `mesh.try_attribute(...)` etc. when interacting with `Assets<Mesh>`. |

### 2.5  Assets

| Item | What to do |
|---|---|
| `AssetSourceBuilder::with_watcher` channel: `crossbeam_channel::Sender` → `async_channel::Sender` | Use `send_blocking(...)`. |
| Custom asset sources require a reader (`AssetSourceBuilder::new(reader_factory)`) | Update if we register custom sources. |
| `AssetServer::new` / `AssetProcessor::new` take `Arc<AssetSources>` | Wrap in `Arc::new(sources)`. |
| `LoadContext::asset_path()` removed; `LoadContext::path()` now returns `AssetPath` | `load_context.asset_path()` → `load_context.path()`; old `path()` → `path().path()` (but prefer `AssetPath` directly). |
| `AssetLoader`/`AssetTransformer`/`AssetSaver`/`Process` require `TypePath` | Add `#[derive(TypePath)]` to loader structs. |
| `Reader::seekable` required; `AsyncSeekForward` deleted | Implement `seekable` on custom readers. |
| `ProcessContext::asset_bytes` removed; use `asset_reader` + `read_to_end` | Mechanical update. |
| `AssetPlugin` gains `use_asset_processor_override: Option<...>` | Use `..Default::default()` to stay forward-compat. |
| `ImageLoaderSettings::array_layout` available | New feature; opt-in. |

### 2.6  glTF (only if we ship glTF assets)

- `GltfPlugin::use_model_forward_direction` → `convert_coordinates: GltfConvertCoordinates { rotate_scene_entity, rotate_meshes }`.
- Default is **off**; we don’t need to do anything unless we want forward-direction conversion.

### 2.7  Misc / niche

- `WinitPlugin` and `EventLoopProxyWrapper` are no longer generic over `M: Message`. Use `WinitUserEvent::WakeUp`. We don’t use this today.
- `FontAtlasSets` removed; `FontAtlasSet` is a resource newtyping a map.
- Schedule cleanup (`topsort_graph`, `ConflictingSystems`, `ScheduleBuildError` enum churn) — only relevant to custom schedule builders.
- `FunctionSystem` gains `In` generic parameter — only relevant to highly generic system code.
- `SimpleExecutor` removed → use `SingleThreadedExecutor`.
- `Assets<Mesh>` now retains render-world-only meshes; `try_*` mesh fns required (see §2.4).
- `BorderRadius` moved off `BorderRadius` component into `Node.border_radius`.

---

## 3) Repo execution checklist

### Phase A — Stabilize pre-migration ✅
- [x] Boundary refactor (Serializable / ECSRuntime / ToolsUI / Legacy).
- [x] `build.rs` banned-import gate.
- [x] Confirmed codebase was NOT on 0.10: mixed 0.9–0.12 state.

### Phase B — 0.10 → 0.12 (Schedules + States) ✅
- [x] `add_plugin` → `add_plugins` in `engine_with_worldgen`, `main`, `bin/world_generator`.
- [x] `app.add_state::<S>()` → `init_state::<S>()` in `main_menu.rs`.
- [x] `.add_system(x.in_schedule(OnEnter(...)))` → `add_systems(OnEnter(...), x)`.
- [x] `.add_system(x.in_set(OnUpdate(...)))` → `add_systems(Update, x.run_if(in_state(...)))`.
- [x] `insert_resource(NextState(Some(S)))` → `next_state.set(S)`.
- [x] `ResMut<State<S>>` (write) → `Res<State<S>>` + `state.get()`.
- [x] `.add_system(x.system())` → `add_systems(Update, x)` in `damage_system.rs`.
- [x] Legacy-tagged `render/light.rs` (pre-0.9 `AppBuilder`) and `gui/in_game_ui.rs` (`Visible`).

### Phase C — 0.12 → 0.14 (Input + Time + Camera) ✅
- [x] `Res<Input<KeyCode>>` → `Res<ButtonInput<KeyCode>>` in 5 active call sites.
- [x] `time.delta_seconds()` → `time.delta_secs()` in `production_care.rs`.
- [x] `Camera2dBundle::default()` → `Camera2d` in `bin/world_generator.rs`.
- [x] `ResMut<EguiContext>` → `EguiContexts` in 4 active UI files.
- [x] Removed OpenGL deps (imgui, glfw, glm, gl, glow) from `Cargo.toml`.
- [x] `Cargo.toml` bumped to `bevy = "0.14"`, `bevy_egui = "0.27"`, `bevy-inspector-egui = "0.22"`.

### Phase D — 0.14 → 0.16 (required components, UI)
- [x] `Color::rgb` → `Color::srgb` in active UI/main paths.
- [x] Native splash: `Node` + required components in `splash.rs` (no legacy bundles in active code).
- [x] `build.rs` banned-pattern gate for pre-0.14 Bevy UI / input / egui types.
- [x] `gui/editor/world_preview.rs`: `Image`/`TextureDescriptor`/`view_formats` reviewed; no `reinterpret_*` calls (N/A until used).
- [ ] Re-enable/fix `Reflect` on legacy production modules when those files are rewired.
- [x] `main.rs`: no invalid global `insert_resource(Msaa)` (0.18 — `Msaa` on `Camera`).

### Phase E — 0.16 → 0.18
- [x] `Cargo.toml`: `bevy = "0.18"`, `bevy_egui = "0.39"`.
- [x] `EguiPlugin::default()`, `EguiPrimaryContextPass`, `ctx_mut()?`, UI systems return `Result`.
- [x] `NextState::set_if_neq` for menus / splash / simulation toggle — use **`NextState::set_if_neq(&mut *res_mut, value)`** (UFCS): `ResMut<NextState<S>>::set_if_neq` is the change-detection API and shadows the inherent `NextState` method if called as `res_mut.set_if_neq(state)`.
- [x] `egui::ComboBox::from_id_salt`, `ui.close()` (egui deprecation cleanup).
- [x] World preview: `EguiTextureHandle::Strong` + `contexts.add_image`.
- [ ] `RenderTarget` on custom `Camera` spawn paths — verify when adding render-to-texture cameras.
- [ ] Mesh `try_*` / asset §2.5 items — when custom loaders or render-world meshes are used.
- [x] §2.1 mechanical renames — applied where the repo used old symbols (egui; no `Gizmos::cuboid` etc.).

### Phase F — Companion crates
- [x] `bevy_egui` 0.39 paired with Bevy 0.18 (per [compatibility table](https://github.com/vladbat00/bevy_egui)).
- [ ] `bevy-inspector-egui` — not in `Cargo.toml`; add when a 0.18-compatible release is selected.
- [ ] `bevy_puffin` — re-evaluate when compatible with 0.18.

### Phase G — `Cargo.toml` housekeeping (optional)
- [ ] High-level Bevy feature collections (`2d`, `3d`, `default_app`, …) if trimming default features.
- [ ] Direct `ron` dep if scene loaders need it.

### Phase H — Definition of done (practical)
- [x] `cargo build` / `cargo check` succeed for default targets (warnings may remain).
- [x] `world_generator` bin builds; spawns `Camera2d`.
- [x] Active plugins use `add_plugins` / `add_systems` / `init_state` / `MessageReader`&`MessageWriter`.
- [x] `build.rs` scan: no banned patterns in non-legacy `src/**/*.rs` (legacy `engine/engine.rs`, `render/light.rs`, `gui/in_game_ui.rs`, `bin/world_generator.rs` exempt).
- [x] Legacy `power_systems.rs` reduced to a documented stub (not in `mod.rs`).

---

## 4) Validation notes vs. the previous plan

| Previous plan claim | Status | Correction |
|---|---|---|
| "Replace deprecated `add_plugin` with `add_plugins`" | **Confirmed** | OK; phrased correctly. |
| "0.18 uses Schedule-centric APIs" | **True but incomplete** | The schedules rewrite landed in 0.11–0.12, not 0.18. |
| "ResMut<State<S>> for state transitions: in newer Bevy, state API changed" | **Vague** | Concrete: use `Res<State<S>>` + `ResMut<NextState<S>>` (0.12–0.13); 0.18 also forces `set_if_neq` for old "no-op on equal" behavior. |
| "delta_seconds — verify" | **Confirmed** | Renamed to `delta_secs` (deprecation chain ending 0.16). |
| "Input<KeyCode> → ButtonInput" | **Confirmed** | Landed in 0.14. |
| "Reflect paths may have moved" | **Mostly OK** | Real changes: `#[reflect(...)]` only allows parens (0.18); module relocations from earlier hops. |
| "Render graph changes are highest churn" | **True** | Add: `RenderTarget` is now a separate required component (0.18); mesh `try_*` API; pipeline descriptors take `BindGroupLayoutDescriptor`. |
| "Strategy: get 0.10 green first" | **Confirmed and reinforced** | Adopted as Phase A. |

Missing items added in this revision:
- Same-state transition behavior change.
- `AssetServer`/`AssetProcessor` `Arc<AssetSources>` requirement.
- `LoadContext::path()` semantics change.
- `Reader::seekable` requirement.
- `Atmosphere` / `ScatteringMedium`.
- `Camera`'s `RenderTarget` extraction.
- `AnimationTarget` split.
- `bevy_gizmos` render split.
- `BorderRect`, `BorderRadius`, `LineHeight` UI changes.
- `ProcessContext::asset_reader`.
- `AssetPlugin::use_asset_processor_override`.
- Many ECS error/type renames.

---

## 5) Upgrade log (fill in as we go)

| Date | Hop | `bevy` | `bevy_egui` | `bevy-inspector-egui` | Notes |
|---|---|---|---|---|---|
| 2026-04-26 | 0.10 (stated) → 0.14 (target) | 0.14 | 0.27 | 0.22 | Phase B+C applied in one pass. Confirmed codebase was actually a mixed 0.9–0.12 state, not 0.10. Applied: add_plugins, ButtonInput, EguiContexts, Camera2d, delta_secs, NextState::set, init_state, add_systems(OnEnter). Legacy-tagged render/light.rs (AppBuilder), in_game_ui.rs (Visible). Removed OpenGL deps. |
| 2026-04-30 | stabilize 0.18 app | 0.18 | 0.39 | _(not in Cargo.toml)_ | `set_if_neq` on menus/splash/transitions; `AppExit::Success` in `exit_game`; egui `from_id_salt`/`close`; `power_systems.rs` stub; `Cargo.toml` comments; migration checklist consolidated. |
| 2026-04-26 | 0.14 → 0.18 (jump) | 0.18 | 0.39 | _(optional)_ | Egui pass, `ctx_mut()?`, world preview texture handle, `main_menu` patterns, docs. Inspector not pinned. |
