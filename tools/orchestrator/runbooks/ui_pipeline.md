# UI pipeline runbook

## File map

| Area | File |
|------|------|
| Map spine | `src/gui/map_view/mod.rs` |
| HUD shell | `src/gui/hud/hud_root_tick.rs` |
| World preview | `src/gui/editor/world_preview/` |
| Viewport measure | `src/gui/authoritative_viewport.rs` |

## egui / schedule order (`MapViewPlugin`)

1. `Update`: `sync_resolved_map_view_frames` (after render `ViewportPipelineSet::Resolve`)
2. `PostUpdate`: `update_world_preview_view` → `update_minimap_view` → interaction commit
3. `EguiPrimaryContextPass`: `clear_active_map_view_input` **before** `hud_product_shell_egui_root` **before** `display_world_preview`
4. `sync_map_fit_transform_components` → `validate_map_fit_system`

## IN PROGRESS

- `map_view` presentation spine (`@orchestrator-status IN_PROGRESS`)
- `sim_view_sync_debug` instrumentation

## Agents

- `ui_layout_agent` — map_view, egui ordering
- `viewport_migration_agent` — measure + semantic authority
