# Orchestrator → coders dispatch — post planner schemas `v1`

| Field | Value |
|:---|:---|
| **Date** | 2026-06-03 |
| **Lang** | [`agent_lang_v1.md`](agent_lang_v1.md) — `⟨ID⟩` · `$ref:` · BLANG |
| **Prerequisite** | @planner-mcp schemas/maps **DONE** |
| **Warehouse keyframe** | **⏸** — $ref:tools/orchestrator/queues/defer_registry.json |
| **Designer parallel** | ⟨GRAMMAR-ITER-001-UI⟩ 🟢 — $ref:docs/archive/2026-06-src-dev/plans/design_grammar_iter_ui_v1.md |

**Ritual:** `BLANG:PRE → BLANG:Q+ → work → BLANG:WIT → BLANG:Q✓` · resolve reads via `$ref:` not full Read.

---

## Planner inputs (landed — use these)

| ⟨ID⟩ | $ref |
|:---|:---|
| ⟨GRAMMAR-ITER-SNAPSHOT-001⟩ | $ref:tools/mcp/schemas/assembly_snapshot_v1.schema.json |
| ⟨GRAMMAR-ITER-RESULT-001⟩ | $ref:tools/mcp/schemas/grammar_iterate_result_v1.schema.json |
| ⟨GRAMMAR-ITER-REQUEST⟩ | $ref:tools/mcp/schemas/grammar_iterate_request_v1.schema.json |
| ⟨APS-VALIDATOR-PLAIN-001⟩ | $ref:docs/archive/2026-06-src-dev/plans/aps_validator_plain_language_v1.md |
| ⟨APS-MAT-CATEGORY-SCHEMA-001⟩ | $ref:assets/materials/profiles/material_category_tree_v1.json · $ref:tools/mcp/schemas/material_category_tree_v1.schema.json |
| ⟨GRAMMAR-002-SLICE-001⟩ | $ref:docs/archive/2026-06-src-dev/plans/grammar_002_slice_001_v1.md |

---

## Assignment matrix

| Order | ⟨ID⟩ | Agent | Task | COMMIT:WIT |
|:---:|:---|:---|:---|:---|
| **1** | ⟨GRAMMAR-ITER-001-API⟩ | @coder-mcp | `iterate_grammar()` + CLI `grammar-iterate`; modes: massing, material_strategy, placement | $ref:debug_runs/grammar_iter_001_massing_live.json |
| **1b** | ⟨GRAMMAR-ITER-001-RUST⟩ | @coder | Rust `iterate_grammar` massing parity (parallel) | `BLANG:S5` building_grammar |
| **2** | ⟨APS-VALIDATOR-PLAIN-002⟩ | @coder-mcp | $ref:docs/archive/2026-06-src-dev/plans/aps_validator_plain_language_v1.md → P0 inline (sentence first) | $ref:debug_runs/aps_validator_plain_002_live.json |
| **3** | ⟨APS-MAT-003⟩ | @coder-mcp | $ref:assets/materials/profiles/material_category_tree_v1.json → Materials tab | $ref:debug_runs/aps_mat_003_category_tree_live.json |
| **4** | ⟨GRAMMAR-002-G2S-2⟩ | @coder | After massing 🟢 — $ref:docs/archive/2026-06-src-dev/plans/grammar_002_slice_001_v1.md | $ref:debug_runs/grammar_002_roof_facade_live.json |
| **5** | ⟨GRAMMAR-002-G2S-3⟩ | @coder-mcp | Python roof/facade modes (after G2S-2) | same witness |
| **—** | ⟨GRAMMAR-ITER-001-APS1⟩ | @coder-mcp | Inspector lineage + footprint diff | $ref:debug_runs/grammar_iter_001_aps1_live.json |

---

## Paste — @coder-mcp (primary session)

```text
BLANG:PRE → BLANG:Q+("coder-mcp")
$ref:docs/archive/2026-06-fleet-drain/dev_dispatch/orchestrator_coder_dispatch_20260603_v1.md
$ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md
$ref:tools/mcp/schemas/grammar_iterate_request_v1.schema.json
$ref:docs/archive/2026-06-src-dev/plans/aps_validator_plain_language_v1.md

ΔWF:
  ⟨GRAMMAR-ITER-001-API⟩  → COMMIT:WIT $ref:debug_runs/grammar_iter_001_massing_live.json
  ⟨APS-VALIDATOR-PLAIN-002⟩ → $ref:docs/archive/2026-06-src-dev/plans/aps_validator_plain_signoff_v1.md
  ⟨APS-MAT-003⟩           → COMMIT:WIT $ref:debug_runs/aps_mat_003_category_tree_live.json
  ⟨GRAMMAR-ITER-001-APS1⟩ parallel · $ref:docs/archive/2026-06-src-dev/plans/design_grammar_iter_ui_v1.md

⏸ warehouse keyframe · $ref:tools/orchestrator/queues/defer_registry.json
BLANG:PY → BLANG:WIT → BLANG:Q✓
```

---

## Paste — @coder (Rust — parallel then G2S-2)

```text
BLANG:Q+("coder") · territory: src/construction/procedural/
$ref:docs/archive/2026-06-fleet-drain/dev_dispatch/orchestrator_coder_dispatch_20260603_v1.md
$ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md
$ref:tools/mcp/python/rust_engine_mcp/building_grammar.py

⟨GRAMMAR-ITER-001-RUST⟩ parallel:
  iterate_grammar massing · grammar_lineage/overrides serde
  BLANG:S5 construction::procedural::building_grammar

🔴 ⟨GRAMMAR-002-G2S-2⟩ 🧩 ⟨GRAMMAR-ITER-001-API⟩ 🟢
  $ref:docs/archive/2026-06-src-dev/plans/grammar_002_slice_001_v1.md
  COMMIT:WIT $ref:debug_runs/grammar_002_roof_facade_live.json

NO: tools/mcp/ · Tk APS · ⏸ keyframe
```

---

## Paste — @designer (parallel — not blocking coders)

```text
⟨GRAMMAR-ITER-001-UI⟩ 🟢 $ref:docs/archive/2026-06-src-dev/plans/design_grammar_iter_ui_v1.md

On-call: $ref:docs/archive/2026-06-src-dev/plans/designer_planner_parallel_wave_20260603_v1.md
  ⟨APS-UX-TOOLTIPS-002⟩ · ⟨APS-ATLAS-LEGEND-001⟩ · ⟨APS-MAT-IA-001⟩
```

---

## Dependency graph

```text
planner schemas DONE
    ├─► @coder-mcp GRAMMAR-ITER-001-API ──► grammar_iter_001_massing_live.json
    ├─► @coder     GRAMMAR-ITER-001-RUST (parallel)
    ├─► @coder-mcp APS-VALIDATOR-PLAIN-002 (independent)
    ├─► @coder-mcp APS-MAT-003 (independent)
    └─► @coder-mcp GRAMMAR-ITER-001-APS1 (parallel)

grammar_iter_001_massing_live.json GREEN
    └─► @coder GRAMMAR-002 G2S-2 ──► @coder-mcp G2S-3 roof/facade modes
```

---

## Queue authority

Status rows: `$ref:tools/orchestrator/queues/grammar_continuation_queue.json` — **do not edit queue from this doc**; use `BLANG:Q✓` after COMMIT:WIT.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Post-planner coder dispatch |
| v1.1.0 | 2026-06-03 | ⟨AGENT-LANG-002-REF⟩ $ref + ⟨⟩ delta pass |
