---
name: tile-generation
description: >-
  Treat isometric tiles as deterministic state machines, not standalone textures —
  variant specs, keyframe bake spine, and atlas batch output. Use for isometric tile
  variants, damage/power/fill/lighting states, vehicle facings, or atlas packing.
  Triggers: tile, isometric, atlas, tilemap, variant, keyframe bake, sprite sheet,
  tile state, damage/power states, facings.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# tile-generation — tiles are state machines

`◉Q🎯 deterministic+batched` — a tile is a state machine baked from a spec, packed into an atlas · ¬one-off image.

## Pattern (form A/E — state → keyframe → atlas)

```text
◎STATE-spec (variant axes) ▷⊳ ▢KEYFRAME-stills ▷⊳ ▢TILEMAPGEN ▷⊳ ◎ATLAS ⇧▶ ◎ENGINE
   variant axes ¬hand-drawn frames: base-material × state(damage/power/fill/lighting)
     vehicles = N facings / sheet · buildings = 1 iso + state rows
```

```text
⬡ production⊨keyframe_pack   bake_source: keyframe_pack (proven still pipeline)
   smoke=ortho               headless ortho bake = smoke/CI only · never shipped art (a [production rule](../mcp-production-rules/SKILL.md))
⚖ deterministic+batched      seeded variation · atlas budget planned up front
```
The state-machine ▷⊳ stills ▷⊳ atlas model transfers to any tilemap; variant axes + bake source adapt to your art tooling.

## In this repo — spine + state machine (form E)

```text
SPINE (authoritative): ◎keyframe ▷⊳ ▢tilemapgen ▷⊳ ◎atlas
SMOKE: smoke_ortho_headless
PLANNED ⊘ (¬treat as shipped): tile.generate execution · TileAtlasRegistry + PT-4 resolver
axes⊳ variant_key · base(wood/stone/concrete) · state/damage/power/fill/lighting
```

Driver commands (validate + status; `tile-spine-run` / `tile-atlas-pack` drive bakes):

```bash
node .claude/skills/agent-lang/driver.mjs variant-set-validate <path/to/variant_set.json>   # takes a variant_set_v1 .json/.ron
node .claude/skills/agent-lang/driver.mjs tile-batch-status <batch_id>                       # status for a batch
node .claude/skills/agent-lang/driver.mjs validate-report tile_promotion <path>              # promotion gate
```

```text
🟢✅🔬 arg shapes via --help: variant-set-validate <path> · tile-batch-status <batch_id>
       tile_promotion / tile_batch = validator ids in validate-report --help
```
Keyframe assets: `utils/Light_keysshotsetup.blend` · `utils/keyframe_render.py` ▷⊳ `utils/tilemapgen`.

## BUILD-GRAMMAR◈ → WEATHERING◈ (v0 hook)

Grammar `age` bands + `ARCH-DNA.A` feed tile **state rows** — not separate art paths.

```text
MATERIAL◈(district_styles) ▷⊳ WEATHERING◈ ▷⊳ variant_key / damage / lighting axes ▷⊳ ATLAS
```

| Grammar source | Tile axis |
|:---|:---|
| `age.bands[].variant_tags` | `clean` · `weathered` · `abandoned` · `damaged` |
| `arch_dna.A` | `new`→`clean_day` · `weathered`→damage row · `ruined`→`abandoned` |
| `βroof` (high) | extra roof-state variants if atlas budget allows |

**Rule:** plan atlas budget for all `age` bands up front — $ref:src/dev/arch_build_grammar_v0_baseline_v1.md§6

## Gotchas

```text
⬡ ortho ≠ production   tile-batch-validate enforces ship: true ⇒ keyframe_pack · a shipped tile baked via ortho = 🔴 rule violation
⚖ state ⇒ plan-atlas    tiles encode state · plan atlas budget for all states up front · adding a damage row later re-bakes the sheet
```

## Source

Cursor original: [.cursor/skills/tile-generation/](../../../.cursor/skills/tile-generation/) (reference.md has the full enum axes + engine swap contract). Legacy pipeline: [`utils/LEGACY_ART_PIPELINE_README.md`](../../../utils/LEGACY_ART_PIPELINE_README.md).

```text
⟦/tile-generation⟧ NEXT ⚑ STATE spec → variant-set-validate → keyframe bake → atlas → validate-report tile_promotion → ⇧ENGINE
```
