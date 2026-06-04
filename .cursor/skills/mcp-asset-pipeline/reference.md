# MCP Asset Pipeline — Reference

Merged from [`prompts/MCP/mcp_drafts.md`](../../prompts/MCP/mcp_drafts.md) and [`prompts/MCP/rules_skills_draft.md`](../../prompts/MCP/rules_skills_draft.md), aligned with shipped [`tools/mcp/`](../../tools/mcp/).

## Core flow

```text
Agent (LLM policy layer)
   ↓
MCP Request (JSON)
   ↓
Python Toolchain Layer (tools/mcp/python/)
   ↓
Blender / Material / Tile Generator
   ↓
Asset Pack (GLB + PNG tiles + metadata)
   ↓
Bevy Import + ECS registry
```

## Agent execution model (draft)

```text
Agent → Load Skills → Apply Rules → Select Tools
  → Generate MCP Request → Execute Toolchain
  → Validate Output → Register Asset
```

Agents are **not** asset generators — they are policy + skill routing + tool selection.

## MCP tool categories

| Category | Tools (draft) | Repo status |
|----------|---------------|-------------|
| Geometry | `blender.generate_building`, `geometry_run_job` | **Partial** — module ops shipped |
| Tile | `tile.generate`, `tile_batch` | **Planned** |
| Prop | `prop.generate` | **Planned** |
| Material | Material Maker CLI | **Planned** (Tier 3) |
| Atlas | `atlas_packer` | **Planned** |
| Reference | OSM / manual cite | **Planned** (read-only) |

## Shipped MCP tools (Cursor)

From [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md):

| MCP tool | CLI | Purpose |
|----------|-----|---------|
| `ping` | `ping` | Health + paths |
| `spec_write` / `spec_validate` | write/validate spec | AssetSpec JSON |
| `geometry_run_job` | `run-geometry` | Blender headless |
| `validate_glb_asset` | `validate-glb` | GLB checks |
| `promote_staging_module` | `promote` | → modules/ |
| `geometry_job_status` | `job-status` | Status file |

## Python toolchain layout (target)

```text
tools/
  mcp/                    # SHIPPED
    python/rust_engine_mcp/
    blender/scripts/
  tile/                   # PLANNED (draft)
    tile_generator.py
    tile_batch.py
  props/                  # PLANNED
    cargo_generator.py
  utils/                  # PLANNED
    atlas_packer.py
```

## Bevy integration model

Promotion target: `assets/models/modules/` + RON sidecar → `BuildingDefinition` / `StylePack` / `RepresentationResult`.

Tile lane (planned): `TileVariant` component → atlas sprite handle swap on sim state change.

## Future: Rust Skill Runtime Engine (draft only)

Draft structs from `rules_skills_draft.md`:

- `Agent { id, skills, rules, tools, memory_scope, output_format }`
- `Skill { id, inputs, outputs, process_steps }`
- `Rule { id, enforcement, conditions }`
- `Tool { id, input_schema, execution, output }`

`SkillRuntimeEngine` enforces rules pre-execution, dispatches CLI tools, hashes requests for reproducibility, validates artifacts.

**Not implemented in repo** — Cursor skills + `tools/mcp/python` are the current runtime.

## Future: Visual Skill Graph Editor (draft only)

Graph nodes: Skill → Rule → Tool → MCP Request → Execution → Artifact.

File format: `{ graph_id, nodes[], edges[] }` — Blueprint-style designer authoring.

## Key insight

You are **not** generating assets with an LLM. You are generating:

**STATE → TOOL → BATCHED ART OUTPUT**

LLM controls: parameters · variation · requests · routing.

Everything else is deterministic.

## Related skills

- [mcp-production-rules](../mcp-production-rules/SKILL.md) — hard constraints
- [tile-generation](../tile-generation/SKILL.md) — tile state machine
- [blender-geometry](../blender-geometry/SKILL.md) — geometry jobs
