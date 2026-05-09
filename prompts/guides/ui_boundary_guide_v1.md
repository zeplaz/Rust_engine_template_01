## UI Boundary Guide v1

> **STATUS:** ✅ **Rules stable** · **Shell refactor (2026-05):** main menu = Bevy (`app_shell.rs`); dev egui (F3/F4/…) gated to simulation/editor via `ui_gates.rs`.

Version: `v1.0.2`
Audience: agents and contributors deciding which UI system to use for a given feature. Strategic layout and mockup epics: [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md) · layer index: [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md).

---

## Decision rule

```
In-game or player-facing?  → Bevy native UI  (Node + components)
Dev tooling / editor?      → egui            (EguiContexts)
```

Do NOT mix the two in a single plugin.

---

## Bevy native UI — when and how

### Use for
- Splash screen, main menu, loading screen
- In-game HUD: health, resources, minimap, notifications, tooltip overlays
- Build/unit placement UIs
- Overlay panels that respond to ECS game state directly
- Buttons, text, images that belong to the game experience

### Key 0.15+ patterns (aligned with 0.18 docs)
```rust
// Spawning a box
commands.spawn(Node {
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    align_items: AlignItems::Center,
    justify_content: JustifyContent::Center,
    ..default()
});

// Text in UI (not Text2d)
commands.spawn((
    Text::new("Health: 100"),
    TextColor(Color::srgb(0.9, 0.9, 0.9)),
    TextLayout::new_with_justify(Justify::Center),
    Node { ..default() },
));

// Button with observer (preferred over Interaction query)
parent.spawn(button_bundle()).observe(on_button_hover);

// Image (no ImageBundle)
parent.spawn((Node { width: Val::Px(200.0), ..default() }, UiImage::new(handle)));

// Color — always sRGB
Color::srgb(r, g, b)
// Not Color::rgb (removed in 0.14)

// Despawn all entities with a tag component
fn despawn_screen<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q { commands.entity(e).despawn_recursive(); }
}
```

### What NOT to do
- `NodeBundle { style: Style { ... } }` — removed in 0.15
- `Style { size: Size::new(...) }` — `Size` removed in 0.11, `Style` renamed to `Node` in 0.15
- `ImageBundle`, `TextBundle`, `ButtonBundle` — removed in 0.15
- `Color::rgb(...)` — renamed to `Color::srgb(...)` in 0.14
- `Msaa::Sample4` — use `Msaa::default()` in 0.14+
- Direct mutation of `UiTransform` / `UiGlobalTransform` — always set through Node layout

### Interaction patterns (two choices, prefer Observers for 0.18)
```rust
// Observers (preferred 0.18+):
entity.observe(|trigger: On<Pointer<Click>>, mut cmds: Commands| { ... });

// Interaction query (compatible 0.14+):
fn button_system(q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>) { ... }
```

---

## egui — when and how

### Use for
- World generation inspector / parameter tuning
- Production domain inspector (concrete, power, aluminum)
- Terrain/biome visualizer / preview
- Noise stack configuration
- Debug overlays during development
- Agent permissions editor

### Active plugins that use egui (keep here)
| Plugin | File |
|---|---|
| `WorldGeneratorSubenginePlugin` | `src/bevysubengines/world_generator_plugin.rs` |
| `WorldGenUiPlugin` | `src/gui/editor/world_gen_ui.rs` |
| `WorldPreviewPlugin` | `src/gui/editor/world_preview.rs` |
| `AgentPermissionsUiPlugin` | `src/gui/agent_permissions_ui.rs` |
| `ProductionToolsUiPlugin` | `src/systems/production/tools_ui.rs` |

### Current egui pattern (Bevy 0.18 / bevy_egui 0.39)
```rust
// Import: EguiContexts, EguiPrimaryContextPass, EguiPlugin
// Source: https://github.com/vladbat00/bevy_egui

fn my_tool_system(mut contexts: EguiContexts, ...) -> Result {
    egui::Window::new("Tool Name").show(contexts.ctx_mut()?, |ui| {
        ui.label("...");
    });
    Ok(())
}

// Plugin registration:
app.add_plugins(EguiPlugin::default())
   .add_systems(EguiPrimaryContextPass, my_tool_system);  // NOT Update
```
**Breaking changes in 0.39:**
- `EguiPlugin` → `EguiPlugin::default()`
- Systems must be in `EguiPrimaryContextPass`, not `Update`
- `ctx_mut()` returns `Result` → use `ctx_mut()?`
- System functions must return `Result`
- `ResMut<EguiContext>` (pre-0.14 API) is fully gone — use `EguiContexts`

---

## Implementation alignment (shell vs dev workspace)

- **Application shell:** `app_shell.rs` — main menu + load stub (Bevy UI only). World-gen flow may still open **egui** tooling while in `BaseState::MainMenu` when `WorldGenFlowState` is active (preview/full pipeline).
- **Gameplay HUD:** `in_game_hud.rs` — logistics / placeholders only; **spawns when entering `BaseState::Simulation`** (not on main menu). No duplicate dev toolbars.
- **Dev workspace (egui):** diagnostics, faction tools, logistics list — `EguiPrimaryContextPass` systems use `in_simulation_or_editor` (`ui_gates.rs`). Hotkeys (F3, F4, …) only affect tooling while in those modes.
- **Strategic / field overlays:** prefer GPU/render layers (e.g. weather-fire compute field) over stacking **egui** windows for map truth.

---

## GUI module layout (canonical)

```
src/gui/
  splash.rs          → Bevy native UI  (splash screen)
  app_shell.rs       → Bevy native UI  (main menu + load stub shell)
  main_menu.rs       → states + shared UiState (no layout)
  ui_gates.rs        → run_if helpers (shell vs dev separation)
  in_game_ui.rs      → LEGACY (rewrite using native Bevy UI + Node)
  in_game_hud.rs     → Bevy native UI  (simulation HUD; logistics + tactical placeholders)
  ui_windows.rs      → shared Bevy UI helpers + egui scale config
  gui_assets.rs      → asset loading only
  gui_sets.rs        → SystemSet definitions only
  agent_permissions_ui.rs → egui tooling
  editor/
    world_gen_ui.rs  → egui tooling
    world_preview.rs → egui tooling
```

---

## Migration path for `in_game_ui.rs`

The file is currently a `LEGACY MODULE (not actively wired)`. When rewriting:

1. Replace `Visibility` query (old `Visible`) with the current `Visibility` / `InheritedVisibility` components.
2. Replace `in_game_menu_state.current()` with `in_game_menu_state.get()`.
3. Replace `add_system(x.run_if())` with `add_systems(Update, x.run_if(in_state(S)))`.
4. Build the actual HUD as `Node`-based entities in `in_game_hud.rs`.

---

## Prompt fragment for future agents

1. Read `prompts/guides/ui_boundary_guide_v1.md`.
2. In-game UI → Bevy native `Node` + components.
3. Tooling UI → `egui` with `EguiContexts`.
4. Never use `NodeBundle`, `TextBundle`, `ImageBundle`, `ButtonBundle` (removed 0.15+).
5. Never use `Color::rgb(...)` — use `Color::srgb(...)`.
6. Always use `despawn_recursive()` to clean up tag-component screen entities.
