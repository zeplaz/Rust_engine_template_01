# Faction editor — tools UI (egui) `02`

**Goal:** dev-time / authorized in-game tooling with the same **ToolsUI** boundary as production and worldgen (`ProductionToolsUiPlugin`, `WorldGenerationToolsUiPlugin`).

## Plugin skeleton

- **Name 📎:** e.g. `FactionToolsUiPlugin` in `src/gui/` or `src/systems/factions/` (follow `src/systems/production/tools_ui.rs`: `init_resource` + systems in `Plugin::build`).
- **State resource:** `FactionToolsState { enabled: bool, selected_faction_id: Option<…>, active_panel: FactionToolsPanel }`.
- **Schedule:** egui pass aligned with `agent_permissions_ui` — `EguiPrimaryContextPass` + `EguiContexts` (see `permissions_ui_system`).

## Panels (v1)

| Panel | Purpose | Data source |
|:---|:---|:---|
| **Roster** | Search, sort, filter; add/remove faction (authority-gated); duplicate from archetype | Blueprint store / scenario |
| **Blueprint inspector** | Name, codes, HSL picker, tags, emblem path | `FactionBlueprint` |
| **Diplomacy matrix** | Pairwise or row/column grid; stance dropdown; treaty attach | Graph DTO + validation §04 |
| **Import / export** | Single blueprint `.ron` / bundle 📎; dry-run diff | FE-06 |
| **Agent link** | Jump to selected faction’s agents; deep-link to **Agent Permissions** window | `AgentManager`, `permissions_ui` |

## Hotkeys & discoverability 📎

- Toggle tools window (F-key or menu) — match project convention in `ui_windows` / main menu.
- “Apply” vs “staging” for live edits: server / single-player rules in §03.

## Gating

- **Dev-only** vs **in-game admin:** reuse patterns from `prompts/designer_questions/tools_ui/debug_perf_ui_split_v1.md` if applicable; otherwise document one gate resource.

## Overlap

- **Permissions:** do not duplicate grant matrices here; link out to `Agent Permissions` for per-agent `DiplomaticRelations` access. This editor owns **faction data + graph**, not per-user grants.
