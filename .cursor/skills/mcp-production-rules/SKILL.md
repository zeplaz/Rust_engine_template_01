---
name: mcp-production-rules
description: >-
  The non-negotiable constraints for procedural-asset production — no AI-generated
  final art, deterministic seeded output, batch/atlas processing, and grid alignment.
  Attach BEFORE any MCP/CLI art or geometry call to emit a pass/block verdict. Use
  whenever someone proposes "quick texture", a diffusion image, a one-off tile, a
  free-rotated prop, or pasting bpy in chat. Triggers: production rules, deterministic,
  seed, no AI art, batch, atlas, grid alignment, asset gate, art constraints.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# mcp-production-rules — the four hard gates

`◉Q🎯 reproducible-at-batch-scale` — enforced as code ¬vibes. 4 ⬡ gates fire BEFORE the tool; any 🔴 = hard stop.

## Pattern (form B — 4 gates before execution)

```text
◎proposal ─⬡[no_ai_generated_images]▶ ─⬡[deterministic_output]▶ ─⬡[batch_processing]▶ ─⬡[grid_alignment]▶ ▢tool-call ▷⊳ ◎asset
              │                            │                        │                      │
              └─🔴⊸ base64/diffusion        └─🔴⊸ unseeded/random      └─🔴⊸ one-off single     └─🔴⊸ free-rotated/off-grid
   ∀⬡★ ⇒ 🟢 proceed   ·   ∃⬡🔴 ⇒ 🔴 stop ⤴ propose compliant alternative
```

| Rule ⬡ | Constraint | Blocks |
|:--|:--|:--|
| `no_ai_generated_images` | ¬diffusion/LLM-freeform for **final** assets | base64 textures, "generate an image", diffusion output |
| `deterministic_output` | same input + same seed ⇒ same output | unseeded variation, random-per-run |
| `batch_processing` | process groups/atlases ¬ad-hoc singles | one-off tile, single-prop bakes as production |
| `grid_alignment` | fixed grid unit, ¬free rotation | free-rotated props, off-grid placement |

Emit a compressed verdict **before** executing — never silently proceed:

```yaml
rules_check: { passed: 🟢|🔴, blocked_by: [<rule_id>...], seed: "<seed>" }
```
🔴 ⟶ propose the compliant alternative ("describe mesh in chat" ⟶ a `geometry_job_v1` JSON; "quick texture" ⟶ keyframe render pack). Rules are env-independent; the *blocked patterns* adapt to your toolchain.

## In this repo — policy gate over the art lane

Wraps [mcp-asset-pipeline](../mcp-asset-pipeline/SKILL.md) · [blender-geometry](../blender-geometry/SKILL.md) · [tile-generation](../tile-generation/SKILL.md). Enforcement points:

```text
◆ job declares variation-set ∧ ¬seed         ─▶ 🔴 block (deterministic_output⊸)
◆ promote attempted ∧ ¬(validate-report★)    ─▶ 🔴 block (see below)
◆ ortho headless bake offered as production    ─▶ 🔴 block — production ⊨ bake_source: keyframe_pack ; ortho = smoke/CI only
```

Schema validation is where rules become real:

```bash
node .claude/skills/agent-lang/driver.mjs validate-report mcp_job <path/to/job.json>
node .claude/skills/agent-lang/driver.mjs variant-set-validate <path/to/variant_set.json>
```

## Gotchas

```text
⬡ gate-before-tool   🔴 = hard stop · ¬"just this once" a one-off — point is reproducibility at batch scale
⚖ deterministic⊨seed  seed IS the variation source · vary w/o recorded seed ⟶ non-reproducible even if it looks fine once
```

## Source

Cursor original: [.cursor/skills/mcp-production-rules/](../../../.cursor/skills/mcp-production-rules/) (reference.md lists every blocked pattern). Consumer/builder split in [.cursor/skills/README.md](../../../.cursor/skills/README.md).

```text
⟦/mcp-production-rules⟧ NEXT ⚑ ∀⬡ check → rules_check verdict → 🟢 tool ∨ 🔴 ⤴ compliant alt
```
