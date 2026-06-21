---
name: blender-geometry
description: >-
  Author headless Blender geometry jobs — footprint + style + state as JSON → bpy op
  graph → validated GLB → promote. Use for procedural module kits (walls, roofs, doors,
  windows, props), geometry job specs, or building visual layers. Triggers: blender,
  bpy, geometry job, GLB, module kit, wall/roof/door/window/prop, mesh export, headless
  bake, LOD, collision.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# blender-geometry — geometry jobs → GLB

`◉Q🎯 reproducible · 🏛 job=artifact` — declarative JSON, executed headless · ¬paste bpy in chat.

## Pattern (form A — pipeline)

```text
◎geometry_job_v1 JSON ▷⊳ ▢bpy-op-graph ▷⊳ ▮headless-Blender ▷⊳ ◎GLB ─⬡[validate]▶ ⇧promote
   (footprint + style + state)
   job = hashable · reproducible · reviewable · ops registered/named/grid-anchored (fixed unit, consistent pivot ⇒ snap together)
```
Adapt op catalog + grid unit to your kit; the JSON ▷⊳ headless ▷⊳ validate ▷⊳ promote spine is constant.

## In this repo — shipped ops + paths

```text
ops⊳ module_wall · module_roof · module_door · module_window · module_prop    (1u grid · bottom-center pivot)
schema⊳ tools/mcp/schemas/geometry_job_v1.schema.json · examples⊳ tools/mcp/schemas/examples/
out⊳ assets/staging/<job_id>/model.glb · status⊳ tools/mcp/jobs/<job_id>.status.json
promote⊳ ⇧ assets/models/modules/ + RON sidecar
```

Verified end-to-end this session (bake ▷⊳ status ▷⊳ validate):

```bash
# run a job — NOTE: the path is relative to tools/mcp/python (the CLI cwd), so use ../schemas/...
node .claude/skills/agent-lang/driver.mjs run-geometry ../schemas/examples/wall_brick_1u_lod0_run001.json
node .claude/skills/agent-lang/driver.mjs job-status wall_brick_1u_lod0_run001
node .claude/skills/agent-lang/driver.mjs validate-report asset_glb assets/staging/wall_brick_1u_lod0_run001/model.glb --compress 4
```

```text
🟢✅🔬 run-geometry ▷⊳ {"status":"done","outputs":[".../model.glb"]}
🟢✅🔬 validate-report asset_glb ▷⊳ {"status":"passed","summary":"... verts=24 tier=smoke arch=module_wall profile=brick"}
🟢✅🔬 pipeline-preflight ▷⊳ blender_ok:true   (Steam Blender path from tools/mcp/config.defaults.json)
```

## Gotchas — TWO path conventions ⚠

```text
◆ which path? ⊗ easy to trip
   ═[run-geometry job-file]▶ resolved vs tools/mcp/python (CLI cwd) ⟶ pass ../schemas/... ∨ absolute
   ═[validate-report asset_glb]▶ repo-relative (resolved internally)
   ☍ NOT the same — a repo-relative path to run-geometry ⟶ tools/mcp/python/<that> ⟶ 🔴 No such file or directory
```

```text
⛓ needs-Blender-on-disk   set via BLENDER_EXE ∨ tools/mcp/config.local.json · pipeline-preflight ▷⊳ blender_ok / blender_error
⬡ promote-after-validate  promote only after validate-report asset_glb passes (the [mcp-production-rules](../mcp-production-rules/SKILL.md) gate)
```

## BUILD-GRAMMAR◈ → MODULE-RUNS◈ (v0 hook)

Grammar picks **massing** + **module slots**; this skill authors the **module GLBs** those slots resolve to.

```text
SHAPE-GRAMMAR◈(massing id) ▷⊳ facade/roof/detail slots ▷⊳ MODULE-RUNS◈ ▷⊳ geometry_job_v1
   βmod ↑ ⇒ denser module-run coverage (PG-2) · ¬new bpy ops in v0
```

| Grammar field | Geometry job input |
|:---|:---|
| `facade.wall_slot` / `door_slot` / `window_slot` | `module_wall` · `module_door` · `module_window` |
| `roof.default_slot` / `by_massing` | `module_roof` |
| `detail.prop_slot` | `module_prop` |
| `district_styles[].material_profiles` | style params on job JSON (profile id) |

**Refs:** $ref:src/dev/arch_build_grammar_v0_baseline_v1.md§6 · $ref:assets/configs/buildings/grammars/industrial_warehouse_v1.ron

## Source

Cursor original: [.cursor/skills/blender-geometry/](../../../.cursor/skills/blender-geometry/) · shipped ops in [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](../../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) Tier 2.

```text
⟦/blender-geometry⟧ NEXT ⚑ author geometry_job_v1 → run-geometry → job-status → validate-report asset_glb → ⇧promote
```
