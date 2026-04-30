# Designer questions (nested by subsystem)

**Scope:** Questions here support **building the simulation engine and game systems** — implementation order, data shapes, schedules, tests, and tooling — with **matrix-backed boundaries** so work ships as **Serializable / ECS / ToolsUI**-correct, testable features. For product/business topics, keep **outside** this tree.

> **Prompt routing:** Open subsystem **`README.md`** → **paired matrix** (see table below) → intent in topic `*_v1.md` and/or **`spec/`** `00–05` → **`implementation_questions_v1.md`** to implement. Mirrors **`prompts/matrix/README.md`**.

## Agent flow (4 steps)

1. **Pick subsystem** (one row in the table below).
2. **Read paired matrix/matrices** — STATUS and domain rows are authoritative for **Serializable / ECS / ToolsUI**.
3. **Design input:** answer open 📎 in topic docs or `spec/`; reflect decisions back into prompts when stable.
4. **Implementation:** work through **`implementation_questions_v1.md`** with **`rg`/`cargo check`**; update matrix **Applied** rows when code lands.

**Modes:** see `prompts/llm_agent_brief.md` § **Workflow — design Q&A vs implementation** (supervisor vs expensive LLM).

## Subsystem → paired matrix / matrices

| Subsystem folder | Primary matrix doc(s) | Designer entry |
|:---|:---|:---|
| **`terrain_world/`** | `matrix/terrain_biome/terrain_biome_migration_matrix_v1.md` + `serialization/` + `assets/` when I/O or sprites | `terrain_world/README.md` → **`spec/README.md`** |
| **`navigation/`** | `matrix/repo/` (layers only; nav matrix TBD) | `navigation/README.md` → **`spec/README.md`** |
| **`production_economy/`** | `matrix/production/` + `serialization/` + `assets/` | `production_economy/README.md` → **`spec/README.md`** |
| **`strategic_platforms/`** | `matrix/strategic_platforms/` + cross-read `terrain_biome/` for theatre | `strategic_platforms/README.md` → **`spec/README.md`** |
| **`tools_ui/`** | `matrix/engine_bevy/` + `matrix/strategic_platforms/` (inspectors) + **`guides/ui_boundary_guide_v1.md`** | `tools_ui/README.md` → **`spec/README.md`** |
| **`factions/`** | `matrix/repo/` + `serialization/` + `assets/` (no single matrix folder) | `factions/README.md` → **`faction_editor/README.md`** |
| **`_legacy/`** | — (archive only) | `_legacy/README.md` |

Each folder has **`README.md`**, topic **`_v1.md`** files, optional **`spec/`** (00–05 stubs), and **`implementation_questions_v1.md`**.

| Subsystem | `README.md` |
|:---|:---|
| Terrain & world | [`terrain_world/README.md`](terrain_world/README.md) |
| Navigation | [`navigation/README.md`](navigation/README.md) |
| Production & economy | [`production_economy/README.md`](production_economy/README.md) |
| Strategic platforms & EW | [`strategic_platforms/README.md`](strategic_platforms/README.md) |
| Tools & debug UI | [`tools_ui/README.md`](tools_ui/README.md) |
| Factions | [`factions/README.md`](factions/README.md) |
| Legacy | [`_legacy/README.md`](_legacy/README.md) |

Parent: [`../README.md`](../README.md) · LLM: [`../llm_agent_brief.md`](../llm_agent_brief.md) · Matrices: [`../matrix/README.md`](../matrix/README.md)
