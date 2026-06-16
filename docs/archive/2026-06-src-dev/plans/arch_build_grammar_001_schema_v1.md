# ARCH-BUILD-GRAMMAR-001 — `BuildingGrammar` schema contract

| Field | Value |
|:---|:---|
| **ID** | ARCH-BUILD-GRAMMAR-001 |
| **Status** | **done** |
| **Schema** | [`tools/mcp/schemas/building_grammar_v1.schema.json`](../../tools/mcp/schemas/building_grammar_v1.schema.json) |
| **Example** | [`tools/mcp/schemas/examples/building_grammar_industrial_warehouse_v1.json`](../../tools/mcp/schemas/examples/building_grammar_industrial_warehouse_v1.json) |
| **Pilot RON** | [`assets/configs/buildings/grammars/industrial_warehouse_v1.ron`](../../assets/configs/buildings/grammars/industrial_warehouse_v1.ron) |

---

## Hierarchy (evaluation order)

```text
district_style + seed
  → archetype (bounds, usage)
  → massing (strategy + footprint_mode + W×D×floors)
  → roof (slot overrides for R token)
  → facade (W/D/C slot overrides + placement_tags)
  → detail (prop density / tags)
  → age (variant_tags band)
```

## Public API (Rust + Python parity)

```text
generate(archetype_id, district_style, seed) -> GrammarGenerateResult
```

| Output field | Use |
|:---|:---|
| `width`, `depth`, `floors` | Footprint grid before module fill |
| `massing_strategy` | Massing rule id (`long_hall`, …) |
| `footprint_mode` | `rect` \| `l_shape` \| `yard_interior` |
| `style_pack_id` | From `district_styles[]` |
| `slot_overrides` | Map footprint slot_key → style-pack slot key |
| `placement_tags`, `variant_tags` | ARCH-003 / APS-TAGS downstream |
| `rule_chain` | Grammar inspector (APS-GRAMMAR-INSPECTOR-001) |

## Scope boundary

- **In:** T1 `building_grammar.rs`, T3 `building_grammar.py` + `assembly.py` hook.
- **Out:** tile atlas, validators, Bevy thumb preview, categorized APS tags (APS-TAGS-001).
- **Massing placement (A vs B):** [`arch_pbg_massing_placement_v1.md`](arch_pbg_massing_placement_v1.md) — perimeter grid is v1; mesh-face instancing requires ARCH-PBG-MASSING-002 gate before generator changes.

## Next

| ID | Owner | Note |
|:---|:---|:---|
| ARCH-BUILD-GRAMMAR-002 | @coder | **done** — `src/construction/procedural/building_grammar.rs` |
| ARCH-BUILD-GRAMMAR-003 | @coder-mcp | **done** — `rust_engine_mcp/building_grammar.py` |
