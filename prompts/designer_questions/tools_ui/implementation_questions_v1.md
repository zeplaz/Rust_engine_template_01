# Tools & debug UI — implementation questions `v1`

**Pair with:** [`spec/README.md`](spec/README.md), `debug_perf_ui_split_v1.md`, `tooling_cross_domain_v1.md`, `prompts/guides/ui_boundary_guide_v1.md`.

## Enablement (build / runtime)

1. **`devtools` cargo feature**: which crates define it — root package only vs workspace?
2. **Runtime `DiagnosticsConfig`**: fields for enabling tabs without recompile — serde from RON in dev?

## Scheduling & integration

3. `DiagnosticsUiPlugin` **ordering** vs `EguiPrimaryContextPass` — document in `EnginePlugin` graph.
4. Hotkey handling: `ButtonInput` consumer order vs egui **wants keyboard**.

## Metric resources

5. **Resource list v0** — `FrameTimeStats`, `ChunkStreamerStats`, `EcsEntityCounts`: exact struct fields + update systems’ schedule.
6. Memory: RSS-only probe vs **optional** tracy — feature-gated dependency in `Cargo.toml`.

## Bevy HUD (player)

7. Sparkline: **texture** ring buffer vs **`egui`** embed for v0 only?
8. Alert list: **virtualization** approach for Bevy UI (if >200 rows).

## Code hygiene

9. **Conditional compilation**: `#[cfg(feature = "devtools")]` on entire plugin module to keep release builds lean — acceptable binary size trade?

## Engine stack (code-synced)

10. **`EnginePlugin`** in `src/engine/engine_with_worldgen.rs` currently registers **`DefaultPlugins`**, **`EguiPlugin`**, **`WorldGenToolsPlugin`**, **`ProductionRuntimePlugin`**, **`ProductionSerializationPlugin`**, **`ProductionToolsUiPlugin`**, **`RoadVehicleToolsUiPlugin`** — document this as the **reference dev stack** until `main.rs` diverges.
11. **World gen:** **`WorldGenToolsPlugin`** + **F8** (`world_generation_plugin.rs` / `world_gen_key_input`) — list in cross-domain tooling matrix; conflicts with other F-keys 📎?
12. **`Agent Permissions`** window — `agent_permissions_ui.rs` + `PermissionsUiState`; ordering vs other egui windows.
