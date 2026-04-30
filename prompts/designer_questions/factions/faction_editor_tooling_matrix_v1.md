# Faction editor — tooling matrix & implementation breakdown `v1`

**Status:** ⏳ **Specs complete (00–05 + README)** — implementation in `src/` still partial (agents/permissions UI only; faction tools plugin TBD).

**Round:** 2026-04-27 · **For:** LLM + engineers + designer

**Paired matrices (no single `matrix/factions/` folder):** `prompts/matrix/repo/repo_boundary_matrix_v1.md` (layers) + `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md` + `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md`.

---

## Purpose

Rework **faction design** from a single bullet into an **implementable tree**: each row becomes a ticket / milestone with clear **Serializable / ECS / ToolsUI** ownership (see `prompts/matrix/repo/repo_boundary_matrix_v1.md`).

---

## Canonical folder layout (`faction_editor/`)

Index + read order: `prompts/designer_questions/factions/faction_editor/README.md`.

```
prompts/designer_questions/factions/faction_editor/
  README.md                — spec index + repo tool patterns
  00_scope.md              — goals, non-goals, MP implications
  01_data_model.md         — FactionBlueprint, tags, diplomacy DTOs, versioning
  02_ui_egui_panels.md     — FactionToolsUi plugin, panels, hotkeys, gating
  03_persistence.md        — scenario vs save vs assets vs live overlay
  04_validation.md         — names, colors, graph rules, UX at scale (no hard cap)
  05_integration_tests.md — headless round-trip, import/export tests
```

📎 **Designer decision:** keep specs under `prompts/designer_questions/factions/faction_editor/` until features ship; mirror summaries into `prompts/matrix/` if a dedicated **factions** matrix is added later.

---

## Build matrix (draft)

| ID | Subsystem | Serializable | ECSRuntime | ToolsUI (egui) | Depends on |
|:---|:---|:---:|:---:|:---:|:---|
| FE-01 | Faction definition DTO (`FactionBlueprint`) | ✅ | ⏳ optional mirror | — | `idgen`, save schema |
| FE-02 | Tag bag + trait placeholders | ✅ | ✅ runtime view | editor | FE-01 |
| FE-03 | Hue / emblem / naming | ✅ | — | color picker | FE-01 |
| FE-04 | Scenario embed vs procedural rules | ✅ | — | wizard | `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md` |
| FE-05 | Permissions + **v1 diplomacy layer** (stance graph, treaties, interlocking modifiers) | ✅ | ⏳ | panel + graph inspector | `src/systems/agents/permissions.rs` (`DiplomaticRelations`), serialization matrix |
| FE-06 | Export/import `.ron` | ✅ | — | file dialog | `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md` |

---

## v1 product decisions (canonical)

1. **Faction count — no arbitrary max.** Add or retire factions via **world/scenario authoring**, **tools**, and **live** edits where simulation authority allows (MP rules TBD per `implementation_questions_v1.md`). Engineering may use **soft** limits only when needed (UI paging, perf profiling, save-size hints)—not a fixed “designer cap.”
2. **Player-facing evolution.** Factions (and related knobs) change over time through **tech / research / point systems**; some budget is **chosen at start** or stored as **saved player preference** — design those pipelines here, persist per serialization matrix.
3. **AI factions.** Prefer **many prebuilt** `FactionBlueprint` / archetype assets (same schema as humans; optional compact `archetype_id` → shared defaults for disk and load time).
4. **Relationships — v1, not deferrable.** A **non-trivial** relationship layer: stances (alliance, war, trade, …), **interdependent** rules and side effects (one treaty affects trade/military/intel), editor + runtime. **Code today:** `PermissionDomain::DiplomaticRelations` in `src/systems/agents/permissions.rs` and UI toggles in `src/gui/agent_permissions_ui.rs` — the **serialized pairwise/typed graph + sim systems** are still to land; do not strip this from v1 planning.

## Residual 📎 (only where still unset)

1. Mid-game **create/remove faction**: exact **server authority** + replication event shape (pair with MP note in `implementation_questions_v1.md`).
2. Export/import UX: native file dialog vs crate (platform detail).

---

## Repo touchpoints (when implementing)

- Agents / permissions (includes diplomatic *permission* domain): `src/systems/agents/permissions.rs`, `src/gui/agent_permissions_ui.rs`
- Stance / treaty **data + systems** (to add): keep Serializable vs ECS split per `prompts/matrix/repo/repo_boundary_matrix_v1.md`
- Persistence direction: `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md`
- Parent design: `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md` (factions §)
- Implementation checklist: `prompts/designer_questions/factions/implementation_questions_v1.md`
- **Editor specs (01–05):** `prompts/designer_questions/factions/faction_editor/README.md`
