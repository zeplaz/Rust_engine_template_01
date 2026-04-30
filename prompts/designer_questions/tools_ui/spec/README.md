# Tools & debug UI — spec index

**Topic docs:** [`../debug_perf_ui_split_v1.md`](../debug_perf_ui_split_v1.md), [`../tooling_cross_domain_v1.md`](../tooling_cross_domain_v1.md)  
**Checklist:** [`../implementation_questions_v1.md`](../implementation_questions_v1.md)  
**Guides:** `prompts/guides/ui_boundary_guide_v1.md` · **Matrix:** `prompts/matrix/engine_bevy/`, `prompts/matrix/strategic_platforms/` (inspectors)

| # | File | Contents |
|:---|:---|:---|
| 00 | [`00_scope.md`](00_scope.md) | Dev vs player UI, feature gates |
| 01 | [`01_plugin_schedule_patterns.md`](01_plugin_schedule_patterns.md) | Plugins, `EguiPrimaryContextPass`, ordering |
| 02 | [`02_egui_bevy_split.md`](02_egui_bevy_split.md) | HUD vs tools, input focus |
| 03 | [`03_cross_domain_panels.md`](03_cross_domain_panels.md) | World gen, production, vehicles, platforms |
| 04 | [`04_metrics_diagnostics.md`](04_metrics_diagnostics.md) | Frame time, chunk stats, memory 📎 |
| 05 | [`05_integration_tests.md`](05_integration_tests.md) | Hotkey, toggle, no panic in headless |
| 06 | [`06_asset_content_studio_workflow_v1.md`](06_asset_content_studio_workflow_v1.md) | Python asset editor: JSON, Tiled, variants, placeholders (artist workflow) |
| 07 | [`07_asset_editor_dual_chain_audit_v1.md`](07_asset_editor_dual_chain_audit_v1.md) | `utils/asset_tools` vs `src/utils/asset_tools` — canonical chain, ported features |
| 08 | [`08_world_gen_desktop_tool_v1.md`](08_world_gen_desktop_tool_v1.md) | Asset editor **World Gen** page ↔ `world_gen_tuning.json` |

**Code patterns:** `src/systems/production/tools_ui.rs`, `src/gui/agent_permissions_ui.rs`, `src/terrain/generation/world_generation_plugin.rs` (`WorldGenerationToolsUiPlugin`).
