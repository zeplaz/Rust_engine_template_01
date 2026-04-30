# Factions — implementation questions `v1`

**Pair with:** `faction_editor_tooling_matrix_v1.md`, **`faction_editor/README.md`** (data + tools specs `01`–`05`), `src/systems/agents/permissions.rs`, `src/gui/agent_permissions_ui.rs`.

## Data

0. **Dynamic roster:** no fixed max — scenario, tools, and authorized live create/retire; document soft UX/perf guardrails only (`faction_editor_tooling_matrix_v1.md` § v1 product decisions).
1. **`FactionBlueprint`**: single RON file per faction vs directory of partials?
2. **Tag bag** storage: `Vec<String>` vs interned `FactionTagId` for save size?
3. **Color**: HSL in file → `LinearRgba` at load — where converted?
4. **Relationship graph v1:** pairwise (or typed) stances + interlocking modifiers — Serializable DTO + ECS/query layer + events; pair with FE-05 in tooling matrix.
5. **Tech / research / points:** where faction-affecting unlocks live (same blueprint revision vs runtime overlay) — 📎 if split undecided.

## Editor (egui)

6. Plugin name / schedule: align with `ProductionToolsUiPlugin` pattern?
7. **Import/export** pipeline: file dialog crate vs native (📎).

## MP & persistence

8. Faction definitions **replicated** on join vs referenced by `scenario_version` hash?
9. Mid-game **faction create**: server authority + broadcast event shape?

## Integration

10. Link **permissions** matrix rows to faction tags — mapping table location (`assets/configs/`).
11. **DiplomaticRelations** grants vs automated stance rules — who wins when in conflict (📎 policy).

## Agents runtime (`src/systems/agents/` — code-synced)

12. **`AgentManager`** (`human_players` / `ai_players` lists) vs future **faction roster** resource — one source of truth or mirror 📎?
13. **`AddPlayerCommand`** / **`CreateAgentCommand`** — fields `faction_id: Option<EntityId>` today; document join flow when `FactionBlueprint` id model lands.
14. **`MultiplayerManager`** (`multiplayer.rs`) — how faction assignments replicate when diplomacy graph changes mid-game 📎?
