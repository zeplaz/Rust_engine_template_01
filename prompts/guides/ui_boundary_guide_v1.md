## UI Boundary Guide v1

> **STATUS:** ✅ **Bevy-only (2026-10-10)** · Program [`plan_egui_full_dep_drop_v1.md`](../../src/dev/plan_egui_full_dep_drop_v1.md) removes `bevy_egui` from Cargo. All UI = native Bevy `Node` trees.

Version: `v1.1.0`
Audience: agents and contributors deciding which UI system to use for a given feature. Strategic layout and mockup epics: [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md) · layer index: [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md).

---

## Decision rule

```
Any UI (player, editor, WorldGen, diagnostics)  → Bevy native UI  (Node + Text + Button + ImageNode)
```

Do **not** add `bevy_egui`, `EguiContexts`, or `EguiPrimaryContextPass`. World paint uses meshes / compositor — never egui strokes.

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

## egui — retired

`bevy_egui` is **not** a dependency. Do not reintroduce it. Editor / WorldGen / diagnostics panels use Bevy `Node` trees (see `simulation_shell_phase2`, `WorldGenShellRoot`, pause menu Bevy patterns).

Shared colors for construction/render helpers: `gui::ui_types::UiRgba` (not `egui::Color32`).

---

## Implementation alignment (shell vs workspace)

- **Application shell:** `app_shell.rs` — main menu + load stub (Bevy UI only).
- **Gameplay HUD:** `in_game_hud.rs` + `simulation_shell_phase2` — Bevy; spawns on enter `BaseState::Simulation`.
- **Editor / WorldGen / diagnostics:** Bevy Node panels; hotkeys via `ui_gates.rs` / session chrome.
- **Strategic / field overlays:** GPU/render layers / meshes — never stroke APIs from a second UI crate.
- **Pointer:** sole authority `SimulationMapPointerGate` + `TacticalMapPickFrame`.

---

## GUI module layout (canonical)

```
src/gui/
  splash.rs          → Bevy native UI
  app_shell.rs       → Bevy native UI
  main_menu.rs       → states + shared UiState
  ui_gates.rs        → run_if helpers
  ui_types.rs        → UiRgba / logical rect (no egui)
  in_game_hud.rs     → Bevy simulation HUD
  hud/               → shells, pointer gate, Bevy panels
  editor/            → Bevy editor / WorldGen chrome
```

---

## Prompt fragment for future agents

1. Read `prompts/guides/ui_boundary_guide_v1.md`.
2. All UI → Bevy native `Node` + components.
3. Never add `bevy_egui` / `EguiContexts`.
4. Never use `NodeBundle`, `TextBundle`, `ImageBundle`, `ButtonBundle` (removed 0.15+).
5. Never use `Color::rgb(...)` — use `Color::srgb(...)`.
6. Always use `despawn_recursive()` to clean up tag-component screen entities.
