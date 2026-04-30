# LLM agent brief — read first

`v1` · goal: **min tokens**, **max signal** · repo: `Rust_engine_template_01`

---

## Legend (use in commits / replies)

| Sym | Meaning |
|:---:|:---|
| ✅ | Claimed done — **verify** path before relying |
| ⏳ | Open, partial, or needs sizing |
| ❌ | Banned pattern / do not reintroduce |
| ⚠️ | Read constraint before edit |
| 📎 | **Ask user** — do not invent |

---

## Prompt contract (production / token-sensitive)

1. **Pair docs:** **Mode A:** matrix + `designer_questions/<subsystem>/README.md`. **Mode B:** same + `spec/` as needed + `implementation_questions_v1.md` before editing `src/`.
2. **Verify “Applied” / ✅:** `grep` / read `src/` — matrix rows go stale when code moves.
3. **Reply shape:** cite `` `path` + `Symbol` `` and ≤~10 lines when needed; do not paste large existing code blocks “for context.”
4. **Unset numbers & policy:** `ASK:` or ask the user — no invented Hz, caps, paths, or crate versions.

---

## Hard rules

1. **📎 No guessing** on numbers (LOD tiers, tick Hz, player caps), netcode, or **paths not in repo**. If missing → ask or mark `ASK:` in notes.
2. **Prefer code cites** over paraphrase: `` `src/…/file.rs` `` (line refs after you read the file).
3. **Layer stack** ⚠️: `Serializable` → may use ← `ECSRuntime` → may use ← `ToolsUI`. Never invert. Detail: `prompts/matrix/repo/repo_boundary_matrix_v1.md`.
4. **Banned imports** — `build.rs` scans `src/**/*.rs` (excludes tagged legacy).

---

## Workflow — design Q&A vs implementation

Use the **same doc chain**; depth depends on mode.

| Mode | Who | Read (min → max) | Output |
|:---|:---|:---|:---|
| **A — Design / answer questions** | Supervisor, designer, cheaper model | Matrix STATUS + designer `README.md` → topic `*_v1.md` → optional `spec/` `00–05` | Update prompts with decisions; mark unset as 📎 / `ASK:` |
| **B — Implementation** | Stronger model, engineer | **A** + `implementation_questions_v1.md` + verify ✅ in `src/` | Code + tests; refresh matrix rows if ownership changed |

**Expensive LLM (B):** always include **paired matrix** + **implementation_questions** in the prompt so scope stays bounded. **Do not** paste whole matrices — cite section or row id (e.g. FE-05, domain table row).

---

## Minimal load order (pick by task)

| Task | Read |
|:---|:---|
| Any code change | `llm_agent_brief.md` + `prompts/matrix/repo/repo_boundary_matrix_v1.md` |
| Terrain / world / theatre | `prompts/matrix/terrain_biome/terrain_biome_migration_matrix_v1.md` + `prompts/designer_questions/terrain_world/README.md` → **`terrain_world/spec/README.md`** for world gen, politics, cities, logistics |
| Production | `prompts/matrix/production/production_migration_matrix_v1.md` + `prompts/designer_questions/production_economy/README.md` (+ `production_economy/spec/README.md` when scoping tools/saves) |
| Navigation / pathing | `prompts/matrix/repo/repo_boundary_matrix_v1.md` (layers) + `prompts/designer_questions/navigation/README.md` + `navigation/spec/README.md` — *no dedicated nav matrix yet* |
| Factions / diplomacy UI | `prompts/matrix/repo/` + `serialization/` + `assets/` + `prompts/designer_questions/factions/README.md` → **`faction_editor/README.md`** — *no single matrix folder* |
| Bevy API bump | `prompts/matrix/engine_bevy/bevy_0_18_migration_plan.md` + `prompts/designer_questions/tools_ui/README.md` |
| Save / RON / binary | `prompts/matrix/serialization/serialization_hybrid_migration_matrix_v1.md` + save-touching designer READMEs (`terrain_world`, `production_economy`, `factions`) |
| Strategic / EW / munitions | `prompts/matrix/strategic_platforms/strategic_platforms_matrix_v1.md` + `prompts/designer_questions/strategic_platforms/README.md` + **`strategic_platforms/spec/README.md`** |
| Config assets | `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md` + `terrain_world` / `production_economy` / `factions` as needed |
| Tools / debug UI | `prompts/guides/ui_boundary_guide_v1.md` + `prompts/designer_questions/tools_ui/README.md` + **`tools_ui/spec/README.md`** |
| **Implement a feature** | Paired **matrix** + designer **`spec/`** or topic docs + **`implementation_questions_v1.md`**; confirm rows in `src/` |

Full tree: `prompts/README.md` · Matrix ↔ designer pairing: `prompts/matrix/README.md` + `prompts/designer_questions/README.md`.

---

## Hot paths (bookmark)

- **Engine plugin**: `src/engine/engine_with_worldgen.rs`
- **Canonical IDs**: `src/idgen.rs` (`EntityId`)
- **World gen UI**: `src/terrain/generation/world_generation_plugin.rs`, `src/gui/editor/world_gen_ui.rs`
- **Assets root** (Bevy): `assets/`
- **Schemas (non-runtime)**: `schemas/flatbuffers/`

---

## Example: good vs bad prompt hygiene

✅ *"Add field X to `ConcreteProductionConfig` in `src/entities/production/concrete/components.rs`, serde + manifest entry per `matrix/production/` doc."*

❌ *"Add a config probably in some production file"* (no path, will waste turns)

---

## When stuck

1. `grep` / search `src/` for symbol.
2. Check relevant **matrix** row (Applied / Pending) and **`prompts/matrix/README.md`** ↔ **designer_questions/README.md** pairing.
3. Still unclear → **ask user**; do not fabricate design.
