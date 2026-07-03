---
name: aps-design-ux
description: >-
  Art Pipeline Suite (APS) designer UX — tag vocabulary, Variants live draft
  preview, tooltips, generation trace, operator rubric. Use when authoring APS
  copy/specs, reviewing Variants/Assembly tag pickers, mandate vs semantic tags,
  reaction-territory sessions, or APS-UX-AUDIT deliverables. Triggers: APS
  variants, apply layers, mandate tags, aps_tooltips, tag vocabulary, generation
  trace, reaction event filter, APS artist tool.
---

# aps-design-ux — Art Pipeline Suite presentation layer

**Authority:** specs in `src/dev/design_aps_*.md` · copy pack `prompts/designer_questions/aps_tooltip_copy_v1.md` · brief `prompts/designer_questions/aps_ux_audit_brief_v1.md`

**Split:** `@designer` charters copy + interaction rules in `src/dev/` · `@coder-mcp` wires `tools/mcp/art_pipeline_suite/` and `rust_engine_mcp/aps_tag_vocabulary.py`

---

## Three tag surfaces (never conflate)

| Surface | UI location | Ships on | Example ids |
|:---|:---|:---|:---|
| **Semantic tags** | Assembly · per placement | `assembly_snapshot` | `street_facing`, `loading_dock` |
| **Assembly variant tags** | Assembly · per piece | placement `variant_tags` | Clean, Night read, Fire damage |
| **Mandate tags** | Variants · per `variant_key` | `variant_set` row | Cultural survival, Burn origin |

Taxonomy: `tools/mcp/schemas/examples/aps_tag_taxonomy_v1.json`  
Mandate families: `rust_engine_mcp.reaction_territory.TAG_FAMILIES`

---

## Tag vocabulary rules (2026-06)

1. **No raw snake_case** in checkboxes — use `aps_tag_vocabulary.mandate_tag_label()` / taxonomy `label`
2. **Every mandate tag** needs artist label + hint in `MANDATE_TAG_VOCAB` (`aps_tag_vocabulary.py`)
3. **Context line** on Variants when toggling tags or changing reaction filter
4. **Tooltips:** dynamic keys `var_mandate_tag:{id}`, `asm_semantic_tag:{id}`, `asm_variant_tag:{id}` via `aps_tooltips.py`
5. **Audit test:** `pytest tools/mcp/python/tests/test_aps_tag_vocabulary.py -q` → `tag_vocabulary_audit()["green"]`

---

## Variants tab interaction model

| Control | Behavior |
|:---|:---|
| Layer dropdowns | **Live draft preview** (debounced) — preview merges form, not saved row |
| Mandate tag checkboxes | Live draft + context line |
| **Apply layers to selected** | **Commit** form → selected `variant_key` row (required before Save / tile batch) |
| Draft strip | `Draft — not saved on row` when form ≠ saved row |
| Reaction event filter | Shows catalog `tag_anchors` + `preview_states` as human text |

Spec: `src/dev/design_aps_variants_live_preview_v1.md`

---

## Generation trace strip

Read-only lineage on Assembly + Variants: archetype · district · seed · grammar steps · **Approve snapshot** checkbox.

Spec: `src/dev/design_aps_gen_step_exposure_v1.md`  
State: `SuiteState.assembly_generation_approved`

---

## Designer deliverable template

```markdown
| Field | Value |
| **ID** | **DES-APS-…-001** |
| **Verdict** | **PASS** or **PASS (qualified)** |
| **Handoff** | @coder-mcp row id |
| Exit predicate | spec PASS + pytest or operator rubric row |
```

Register in `tools/orchestrator/queues/designer_signoff_registry.json`.

---

## Tier-2 backlog (creative pass)

From `design_aps_tag_vocab_creative_pass_v1.md`:

- `DES-APS-TAG-TIER2-001` — district flavor tags, sim-coupled tags, archetype tag presets
- Operator tag rubric walk before expanding taxonomy

---

## Related skills

| Skill | When |
|:---|:---|
| [mcp-asset-pipeline](../mcp-asset-pipeline/SKILL.md) | variant_set → tile batch → atlas |
| [validation-first](../validation-first/SKILL.md) | witness JSON WIT-HON |
| [agent-lang](../agent-lang/SKILL.md) | queue + witness envelope |

---

## Quick verify

```bash
cd tools/mcp/python
python -m pytest tests/test_aps_tag_vocabulary.py tests/test_variants_layer_context.py -q
```
