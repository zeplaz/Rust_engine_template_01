---
name: mcp-asset-pipeline
description: >-
  Orchestrate deterministic asset production end-to-end — spec is authority, MCP/CLI
  executes, validators gate, promotion copies into the engine registry. Use when
  driving an art job from request to shipped asset, routing tile vs geometry lanes, or
  sequencing the spec→tool→validate→promote phases. Triggers: asset pipeline, AssetSpec,
  staging, promote, geometry job, atlas, module kit, art lane, G0-G5 gates.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# mcp-asset-pipeline — spec → tool → validate → promote

`◉Q🎯 deterministic · 🏛 spec=authority` — single execution path, gated at every hop.

## Pattern (form A — gated pipeline)

```text
◎AssetSpec🏛 ▷⊳ ▢MCP/CLI-tool ─⬡[one-path]▶ ◎staging/ ─⬡[validate★]▶ ⇧promote ▷⊳ ◎registry★
   │                                                  │
   └─⊰ structured JSON ¬freeform                       └─🔴[validate✗]⊸ ¬promote
```

```text
🏛 spec=authority    agent authors JSON · tool executes · ¬freeform mesh/texture (⤴ mcp-production-rules)
   one-path          MCP tools ∧ micro-CLI call same functions ⇒ CLI-proven ≡ MCP-behavior
⛓ staging-boundary   tools write only staging/<job_id>/ · promote = separate validated copy
⬡ gate-every-phase   validate✗ ⊸ promote · ¬designer-signoff ⊸ tool-exec
```
Transfers: define spec schema · force one execution path · gate the staging→ship hop.

## In this repo — G0–G5 (form A · graph algebra)

```text
⟨G:art⟩ ≝ G0 ⨟ G1 ⨟ G2 ⨟ G3 ⨟ G4 ⨟ G5      (each Gn begins when Gn−1 closes ★)

G0 order-critique + rules-audit   ⤳ @designer-mcp
G1 spec valid       ─⬡▶ validate-report mcp_spec
G2 tool runs        ─⬡▶ run-geometry / tile-spine-run
G3 validate green   ─⬡▶ validate-report asset_glb
G4 staging sign-off ─⬡▶ @designer-mcp (list-staging)
G5 promote+register+witness ▷⊳ promote / library-register / write-witness ▷⊳ ◎registry★
   owners: @designer-mcp authors · @coder-mcp builds · @orchestrator-mcp sequences
   ¬promote ∵ ¬(G3★ ∧ G4★)
```

Drive via the agent-lang driver (verified commands):

```bash
node .claude/skills/agent-lang/driver.mjs list-staging                                   # what's staged (G4)
node .claude/skills/agent-lang/driver.mjs job-status wall_brick_1u_lod0_run001           # a job's state
node .claude/skills/agent-lang/driver.mjs validate-report asset_glb assets/staging/wall_brick_1u_example/model.glb --compress 4   # G3
```

Lane router (form K):

```text
◎request ═[kind=geom]▶ ▢geometry-lane (Blender modules) ⤴ [blender-geometry](../blender-geometry/SKILL.md)
         ═[kind=tile]▶ ▢tile-lane (iso state machines)   ⤴ [tile-generation](../tile-generation/SKILL.md)
```
Tool registry: [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](../../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md). SHIPPED ≠ PLANNED — ¬plan on `tile.generate` execution (PLANNED) as if shipped.

## Gotchas

```text
🏛 consumer≠builder   @coder/@designer/@planner USE these tools · only -mcp agents build tools/mcp/ · ¬paste bpy as consumer
   list-staging quirk  glb_count: 0 for container dirs (assemblies, tiles) = ¬failure · those hold sub-jobs
⬡ promote≠idempotent  promote copies validated staging → registry + auto library_register · validate★ first
```

## Source

Cursor original: [.cursor/skills/mcp-asset-pipeline/](../../../.cursor/skills/mcp-asset-pipeline/) · toolchain README [`tools/mcp/README.md`](../../../tools/mcp/README.md).

```text
⟦/mcp-asset-pipeline⟧ NEXT ⚑ G0 critique → G1–G3 validate → G4 sign → G5 ⇧promote ▷⊳ ◎registry★
```
