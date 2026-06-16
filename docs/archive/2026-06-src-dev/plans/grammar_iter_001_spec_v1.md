# GRAMMAR-ITER-001 — Iterative grammar authoring spec `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **GRAMMAR-ITER-001** |
| **Owner (spec)** | **@planner-mcp** |
| **Implement** | @coder (Rust partial regen) · @coder-mcp (Python/APS) · @designer (UX wireframe) |
| **Track** | **C — Content / grammar** (parallel — **not** blocked by warehouse keyframe or broken ship art) |
| **Parent** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) · [`plan_bevy_hud_grammar_parallel_v1.md`](plan_bevy_hud_grammar_parallel_v1.md) |
| **Status** | **SPEC READY** — planner-mcp sign-off; implementation phases open |
| **Date** | 2026-06-03 |

---

## Problem (why this exists)

Today the artist workflow is **seed-only reroll**:

```text
Generate snapshot → like massing → tweak facade → full regen → lose massing
```

Republic-style authoring needs **layer-scoped iteration**: change one grammar stratum while keeping others stable, with **constant visual feedback** (APS slot preview + Bevy assembly preview).

**GRAMMAR-001/002** mature rule *content* (massing strategies, facade/roof tables).  
**GRAMMAR-ITER-001** matures the **interaction model** — how artists *drive* those rules without Blender or full reroll.

---

## Goals

| Goal | Success signal |
|:---|:---|
| **Small deltas** | Change massing OR facade OR roof OR material strategy without full snapshot discard |
| **Deterministic** | Same inputs → same output; overrides are explicit fields, not hidden state |
| **Inspectable** | Grammar inspector shows *why* + *which layer* last changed |
| **Authority preserved** | `assembly_snapshot` remains ship contract (ARCH-MAT-001) |
| **Preview loop** | Any iteration step refreshes APS-PREVIEW-001 + assembly Bevy preview in &lt;5s typical |

## Non-goals

| Item | Lane |
|:---|:---|
| Mesh-face massing (PBG model B) | ARCH-PBG-MASSING-002 gate — not GRAMMAR-ITER |
| Warehouse keyframe ship / G4 | Track B — paused independently |
| Replacing `building_grammar_v1` schema wholesale | GRAMMAR-001/002 content work |
| Blender as authoring surface | ARCH-MAT-001 |

---

## Layer model (iteration scope)

Evaluation order is **fixed** (from [`arch_build_grammar_001_schema_v1.md`](arch_build_grammar_001_schema_v1.md)):

```text
seed + archetype + district
  → massing     (footprint W×D×floors, footprint_mode, strategy id)
  → roof        (slot overrides for R token)
  → facade      (W/D/C slots, placement_tags)
  → detail      (prop density / tags)
  → age         (variant_tags band)
  → material    (district material_profiles → placement.material_profile)
```

### Iteration tiers

| Tier | ID | What changes | Geometry regen? | Material regen? |
|:---|:---|:---|:---:|:---:|
| **T0** | `full` | Seed or archetype or district | Yes (all layers) | Yes |
| **T1** | `massing` | Massing strategy, W×D, floors, footprint_mode | Yes (footprint + placements) | Re-apply district defaults |
| **T2** | `roof` | Roof rule / slot override | Roof cells only | Roof profiles only |
| **T3** | `facade` | Facade slots, exterior tags | Facade ring cells | Facade profiles |
| **T4** | `detail` | Prop density, detail tags | Detail placements add/remove | — |
| **T5** | `age` | Age band / variant_tags | — | — |
| **T6** | `material_strategy` | District profile map or per-slot override | No | Yes |
| **T7** | `placement` | Single cell module_id / tags | Local cell | Local profile |

**Rule:** Higher tier invalidates lower tiers on downstream geometry (T1 wipes T2–T4 placements; T2 does not wipe massing footprint).

---

## API contract (planner-mcp — implement in Phase 2)

### Request: `GrammarIterateRequest`

```json
{
  "schema": "grammar_iterate_request_v1",
  "base_snapshot_path": "assets/staging/assemblies/industrial_west_8x9_s43_f75a.json",
  "mode": "massing",
  "seed": 43,
  "archetype_id": "IndustrialWarehouse",
  "district_style": "industrial_west",
  "overrides": {
    "massing_strategy": "double_hall",
    "footprint": { "width": 10, "depth": 6, "floors": 2 }
  },
  "preserve_layers": ["district_style", "age"],
  "parent_lineage_id": "industrial_west_8x9_s43_f75a"
}
```

| Field | Required | Notes |
|:---|:---:|:---|
| `mode` | yes | One of `full` \| `massing` \| `roof` \| `facade` \| `detail` \| `age` \| `material_strategy` \| `placement` |
| `base_snapshot_path` | yes* | *Or inline `base_snapshot` object |
| `overrides` | per mode | Layer-specific keys (see table below) |
| `preserve_layers` | no | Hints for partial regen; validator rejects impossible combos |
| `parent_lineage_id` | recommended | Branch tracking for "Save as variant" |

### Response: `GrammarIterateResult`

```json
{
  "schema": "grammar_iterate_result_v1",
  "ok": true,
  "snapshot": { },
  "diff": {
    "cells_added": 12,
    "cells_removed": 4,
    "cells_changed": 6,
    "layers_touched": ["massing", "roof", "facade"]
  },
  "grammar_rule_chain": { },
  "lineage": {
    "parent_id": "industrial_west_8x9_s43_f75a",
    "iteration_mode": "massing",
    "seed": 43
  }
}
```

### Overrides by mode

| mode | `overrides` keys |
|:---|:---|
| `massing` | `massing_strategy`, `footprint.{width,depth,floors}`, `footprint_mode` |
| `roof` | `roof_slot`, `roof_rule_id` |
| `facade` | `wall_slot`, `door_slot`, `window_slot`, `placement_tags[]` |
| `detail` | `prop_slot`, `density`, `detail_tags[]` |
| `age` | `age_band_id` |
| `material_strategy` | `district_material_profiles{}` or `slot_material_overrides{}` |
| `placement` | `node_id`, `module_id`, `material_profile`, `semantic_tags` |

### Rust + Python parity

| Surface | Path |
|:---|:---|
| Rust | `src/construction/procedural/building_grammar.rs` — `iterate_grammar(...)` |
| Python | `tools/mcp/python/rust_engine_mcp/building_grammar.py` — `iterate_grammar(...)` |
| CLI | `rust_engine_mcp.cli grammar-iterate` (coder-mcp) |
| APS | `assembly_panel.py` — Iterate panel (coder-mcp + designer wireframe) |

**Determinism:** `_mix_seed(seed, layer_salt)` pattern already in `building_grammar.py` — reuse per layer; overrides bypass weighted pick for pinned fields only.

---

## Snapshot extensions (authority)

Add optional top-level fields on `assembly_snapshot_v1` (backward compatible):

```json
{
  "grammar_lineage": {
    "parent_assembly_id": "industrial_west_8x9_s43_f75a",
    "iteration_mode": "massing",
    "iteration_seq": 2,
    "pinned_layers": ["district_style", "age"]
  },
  "grammar_overrides": {
    "massing_strategy": "double_hall"
  }
}
```

| Field | Purpose |
|:---|:---|
| `grammar_lineage` | Branch history; APS "revert to parent" |
| `grammar_overrides` | Explicit artist pins — inspector shows pinned vs rolled |
| `grammar_rule_chain` | **unchanged** — flattened chain for APS inspector |

**Schema work (@planner-mcp):** extend `assembly_snapshot_v1.schema.json` + example — do not break existing validators (fields optional).

---

## APS UX — Iterate panel (designer wireframe → coder-mcp)

**Location:** Assembly tab — new `LabelFrame` **"Iterate grammar"** below Generate, above footprint grid.

```text
┌─ Iterate grammar ─────────────────────────────────────────────┐
│ Mode: [Massing ▼]   Seed: [43]   [Apply iteration]          │
│                                                              │
│ Massing strategy:  ( ) long_hall  (•) double_hall  …        │
│ Footprint W×D:     [10] x [6]   Floors: [2]                 │
│                                                              │
│ ☐ Pin district style   ☐ Pin age band                        │
│                                                              │
│ Last diff: +12 −4 ~6 cells · layers: massing, facade         │
│ [Preview assembly]  [Save branch]  [Revert to parent]       │
└──────────────────────────────────────────────────────────────┘
```

| Control | mode | Preview on Apply |
|:---|:---|:---|
| Mode dropdown | switches override fields | — |
| Strategy radios | `massing` | Footprint grid diff highlight (new/changed/removed cells) |
| W×D/Floors spinners | `massing` | Same |
| Roof slot combo | `roof` | Slot preview + roof cells |
| Facade slots | `facade` | Facade ring highlight |
| Material strategy | `material_strategy` | Material thumbs only (no geom) |
| Selected cell | `placement` | Existing slot preview panel |

**Designer deliverable:** `prompts/designer_questions/grammar_iter_wireframe_v1.md` — @designer after planner-mcp spec review.

**Live loop (Phase 2):** Apply → `iterate_grammar` → reload UI → `slot_preview` + optional `assembly_preview.on_preview()` auto-trigger (toggle "Auto-preview on iterate").

---

## Grammar inspector enhancements

Extend [`grammar_inspector.py`](../../tools/mcp/art_pipeline_suite/grammar_inspector.py):

| Addition | Content |
|:---|:---|
| **Pinned vs rolled** | Show `grammar_overrides` keys in bold |
| **Lineage** | Parent id + iteration_seq + mode |
| **Per-cell why** | Already partial — add `iteration_mode` that last touched cell |
| **Human labels** | Map `long_hall` → "Long Hall" (glossary JSON from planner-mcp) |

**Glossary file:** `assets/configs/buildings/grammars/grammar_labels_v1.json` — @planner-mcp maintains ids → display strings.

---

## Implementation phases

### Phase 0 — Spec + schema (@planner-mcp) — **this doc**

| Deliverable | Status |
|:---|:---:|
| GRAMMAR-ITER-001 spec | **done** |
| `grammar_iterate_request_v1.schema.json` | **done** |
| `grammar_labels_v1.json` pilot (IndustrialWarehouse) | **done** — review @designer |
| Snapshot lineage fields in `assembly_snapshot_v1` | **pending** @coder / @planner-mcp |
| Designer wireframe | **assigned** — GRAMMAR-ITER-001-UI |

### Phase 1 — Read-only APS (@coder-mcp, 1–2 days)

| Task | Acceptance |
|:---|:---|
| Show `grammar_lineage` / `grammar_overrides` when present on load | Inspector + metadata panel |
| Footprint diff highlight (compare before/after in memory) | Heatmap legend: green=added, red=removed, yellow=changed |
| No `iterate_grammar` yet — UI disabled with "Phase 2" tooltip | — |

### Phase 2 — Partial regen API (@coder + @coder-mcp)

| Task | Owner | Acceptance |
|:---|:---|:---|
| `iterate_grammar` Python for modes `massing`, `material_strategy`, `placement` | @coder-mcp | Determinism tests |
| Rust `iterate_grammar` for `massing` parity | @coder | `cargo test` grammar iterate |
| Modes `roof`, `facade`, `detail`, `age` | @coder | After GRAMMAR-002 tables exist |
| CLI `grammar-iterate` | @coder-mcp | JSON in/out |
| APS Apply iteration wired | @coder-mcp | Witness below |

**Witness:** `debug_runs/grammar_iter_001_massing_live.json`

```json
{
  "program_id": "GRAMMAR-ITER-001",
  "mode": "massing",
  "parent": "industrial_west_8x9_s43_f75a",
  "child": "industrial_west_10x6_s43_iter1",
  "diff": { "cells_added": 8, "cells_removed": 2 },
  "determinism": "pass",
  "preview": "bevy_worker",
  "green": true
}
```

### Phase 3 — Branch + diversity (@coder-mcp + @planner)

| Task | Acceptance |
|:---|:---|
| Save branch → new `assembly_id` with lineage | Revert restores parent snapshot |
| PG-QUALITY sweep on iterate branches | Diversity metrics don't collapse to single silhouette |

---

## Dependencies

| ID | Relationship |
|:---|:---|
| ARCH-BUILD-GRAMMAR-001/002/003 | **done** — base evaluator |
| APS-PREVIEW-001 | **done** — slot feedback |
| APS-PREVIEW-002/004 | **done** — assembly Bevy preview |
| GRAMMAR-001 | Massing content depth — parallel, not blocking Phase 2 `massing` mode |
| GRAMMAR-002 | Facade/roof partial regen — blocks Phase 2 modes T2/T3 |
| ARCH-MAT-001 | Material strategy iteration uses snapshot profiles only |
| APS-UX-AUDIT-001 | Iterate panel layout should align with designer audit |

---

## Orchestrator queue rows

```text
GRAMMAR-ITER-001-SPEC    @planner-mcp   Phase 0 schemas + glossary     READY (this doc)
GRAMMAR-ITER-001-UI      @designer      Wireframe + control labels     after spec
GRAMMAR-ITER-001-APS1    @coder-mcp     Phase 1 inspector + diff UI    ready
GRAMMAR-ITER-001-API     @coder-mcp     Phase 2 Python iterate + CLI   after schemas
GRAMMAR-ITER-001-RUST    @coder         Phase 2 Rust massing iterate   parallel
GRAMMAR-ITER-001-E2E     @coder-mcp     Phase 2 witness JSON           after API
```

**Does not wait on:** MCP-PILOT-GRAMMAR-001 B2, keyframe PNGs, G4, or production atlas.

---

## Acceptance (program close)

| Criterion | Proof |
|:---|:---|
| Artist changes massing without changing seed and keeps district pin | APS Apply + witness |
| Material strategy change updates profiles, not geometry | Validator + preview |
| Single placement swap is local | Diff shows 1 cell |
| Full regen still works (T0) | Existing grammar E2E green |
| Inspector answers "why" + "what changed" | Designer sign-off optional |

---

## @planner-mcp immediate actions

1. Add `tools/mcp/schemas/grammar_iterate_request_v1.schema.json`  
2. Extend `assembly_snapshot_v1.schema.json` with optional `grammar_lineage`, `grammar_overrides`  
3. Add `assets/configs/buildings/grammars/grammar_labels_v1.json` (IndustrialWarehouse pilot)  
4. Open PR/note linking GRAMMAR-001 slice plan — which massing strategies get iteration first  
5. Hand off wireframe brief to @designer  

---

## References

- [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) — Track C  
- [`pilot_grammar_001_grammar_e2e_live.json`](../../debug_runs/pilot_grammar_001_grammar_e2e_live.json) — baseline grammar E2E  
- [`arch_mat_001_material_authority_v1.md`](arch_mat_001_material_authority_v1.md)  
- Rust evaluator: `src/construction/procedural/building_grammar.rs`  
- Python mirror: `tools/mcp/python/rust_engine_mcp/building_grammar.py`  

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Initial @planner-mcp spec — iterative layers, API, APS UX, phases |
