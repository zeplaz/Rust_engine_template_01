# MCP grammar · building-set guards `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-MCP-GRAMMAR-BUILD-SET-GUARDS-001** |
| **Owner** | @planner-mcp (plan) → @coder-mcp (tools) → @coder (Rust guards) |
| **Parent** | $ref:src/dev/plan_operator_build_readability_exec_001_v1.md |
| **Grammar baseline** | $ref:src/dev/arch_build_grammar_v0_baseline_v1.md |
| **Registry** | $ref:tools/mcp/MICRO_TOOLS_REGISTRY_v1.md |
| **Date** | 2026-06-13 |
| **Status** | **SIGNED** (plan only) |

---

## Why this plan exists

Recent work converged on **grammar + ARCH-DNA + pilot catalog**, but the repo still carries **warehouse-shaped debt**:

| Symptom | Where | Risk |
|:---|:---|:---|
| Single grammar pilot (`logistics_rail_warehouse_v0`) | `_pilot_catalog.ron` | Tray + witnesses test one story |
| `IndustrialWarehouse` dominates diversity witness | `grammar_diversity_witness.json` | Metrics green on one archetype only |
| Rust branches on warehouse id | `pilot_catalog.rs`, `parametric_commit.rs`, `site_stub_overlay.rs` | Every new pilot = copy-paste branch |
| Examples named `*_warehouse_*` | `tools/mcp/schemas/examples/` | Agents copy warehouse JSON as “the” template |
| OPS-007 / MCP-PILOT-GRAMMAR-001 paused on warehouse Track B | queues | Production lane frozen on one building |

**Goal:** MCP tools + CI guards that **insure building sets** (coverage, parity, teachable examples) and **block single-archetype drift** before it hardens into Rust.

---

## Design principles

```text
1. Catalog is authority     — pilots/grammars/presets live in RON/JSON manifests, not if pilot.id == "…"
2. Examples teach logic     — every schema example declares teaches[] axes, not one building name
3. Sets not singletons      — min N grammar pilots + min axis coverage before ship gates
4. Verify before integrate  — grammar/gen tools return ValidationReport, not prose
5. User path = brief + gate  — APS/operator sees compressed brief + plain-language FAIL
```

**BLANG hooks:**

| Token | Tool |
|:---|:---|
| `BLANG:GRAMMAR` | `grammar_set_brief` · `grammar_eval_sweep` |
| `BLANG:GUARD` | `pilot_hardcode_lint` · `building_set_coverage_report` |
| `BLANG:INTEGRATE` | `complex_building_brief` · `grammar_integration_validate` |

---

## Tool catalog (new — Tier 1g)

### A — Grammar generation & verification

| CLI / MCP | Purpose | Input | Output |
|:---|:---|:---|:---|
| **`grammar_set_brief`** | Compressed inventory: pilots, grammars, presets, gaps | optional `--set-id` | ≤50 lines: counts, missing links, axis coverage |
| **`grammar_preset_pair_validate`** | `arch_dna` preset ↔ `grammar_id` ↔ pilot row | preset json or id | ValidationReport pass/fail |
| **`grammar_eval_sweep`** | Seed sweep massing/roof distribution | archetype + district + seeds | brief + optional witness JSON |
| **`grammar_pilot_parity`** | MCP wrapper for `pilot_catalog_parity` | — | ValidationReport (registry + footprint parity) |
| **`grammar_integration_validate`** | Snapshot path: DNA + grammar + site + material_profiles | assembly_snapshot | ValidationReport + plain sentences |

**Composes existing:** `validate-report arch_build_grammar`, `snapshot_digest`, `validate_p0_gate_plain`, `witness_brief`.

### B — Building-set assurance (“insure sets”)

| CLI / MCP | Purpose | Rule |
|:---|:---|:---|
| **`building_set_manifest_validate`** | Validate `building_set_manifest_v1.json` | ≥2 grammar pilots, ≥2 distinct `arch_dna.F`, linked tile batches optional |
| **`building_set_coverage_report`** | Axis coverage table (F, L, I, massing modes) | FAIL if any axis has 0 presets |
| **`building_set_health_brief`** | Rollup for OPS brief | pilots / grammars / examples / hardcode hits |

**New schema:** `tools/mcp/schemas/building_set_manifest_v1.schema.json`  
**Manifest file:** `assets/configs/buildings/_building_sets.ron` (or JSON twin for MCP)

### C — Anti-hardcoding guards

| CLI / MCP | Purpose | Fail condition |
|:---|:---|:---|
| **`pilot_hardcode_lint`** | Scan `src/`, `tests/`, `tools/mcp/schemas/examples/` | String literal `logistics_rail_warehouse_v0` or `IndustrialWarehouse` **outside** allowlist (catalog loaders, one canonical example dir) |
| **`single_archetype_ratio_guard`** | Ratio of grammar test/example refs per archetype | Any archetype > **40%** of counted refs without `building_set` manifest exception |
| **`example_teachable_audit`** | `tools/mcp/schemas/examples/*.json` | Missing `_meta.teaches` with ≥2 logic axes |
| **`warehouse_track_guard`** | New `*warehouse*` paths in examples/staging | No row in `building_set_manifest` + no `teaches` including `grammar_eval` or `pilot_catalog` |

**CI placement:** `pytest tools/mcp/python/tests/test_build_set_guards*.py` + optional `cargo test` hook calling Python guard via `BLANG:GUARD`.

### D — Complex building user integration

| Surface | Tool / UX | User sees |
|:---|:---|:---|
| **APS Grammar tab** | `grammar_set_brief` in status strip | “2/5 F-axis presets · missing: manufacturing” |
| **APS DNA panel** | preset picker from `list_preset_ids()` only | No free-text warehouse id |
| **Iterate panel** | `grammar_eval_sweep` on demand | Massing histogram plain text |
| **Operator build tray** | pilot catalog only | Shape QA + grammar pilots from RON |
| **QC HUD (Bevy)** | `complex_building_brief` path | grammar chain + site zones + β snapshot |
| **P0 gate** | `grammar_integration_validate` | Artist sentences from validator map |

---

## `_meta.teaches` contract (examples)

Every file under `tools/mcp/schemas/examples/` that is not a **canonical pilot fixture** must include:

```json
{
  "_meta": {
    "teaches": ["arch_dna", "pressure_field", "pilot_catalog"],
    "not_a_ship_target": true,
    "building_set_id": "industrial_west_v0"
  }
}
```

| `teaches` value | Means example demonstrates |
|:---|:---|
| `arch_dna` | DNA + β fields |
| `pilot_catalog` | Catalog id indirection |
| `grammar_eval` | Massing sweep / generate |
| `site_composition` | Site zones stub |
| `tile_state_machine` | Variant axes / age bands |
| `module_runs` | Geometry job composition |

**Rule:** Examples with only `warehouse` in filename must either migrate to `building_set` manifest or gain `teaches` ≥2 non-warehouse-specific axes.

---

## Building-set manifest (v1 shape)

```ron
(
    set_id: "industrial_west_v0",
    label: "Industrial west pilot set",
    min_grammar_pilots: 2,
    pilots: [
        "logistics_rail_warehouse_v0",
        // "manufacturing_factory_v0",  // BUILD-SET-PILOT-002
    ],
    grammar_ids: ["industrial_warehouse_v1"],
    arch_dna_presets: ["logistics_rail_warehouse_v0"],
    required_f_functions: ["logistics", "manufacturing"],
    tile_batches_optional: [],
)
```

**Exit for “set insured”:** `building_set_coverage_report` green + `pilot_hardcode_lint` zero violations outside allowlist.

---

## Phased delivery

| Phase | ID span | Owner | Delivers |
|:---:|:---|:---|:---|
| **0** | PLAN-MCP-GUARD-000 | @planner-mcp | This doc + todos + queue rows (**done**) |
| **1** | MCP-GRAMMAR-SET-001…004 | @coder-mcp | `grammar_set_brief`, `grammar_preset_pair_validate`, `grammar_eval_sweep`, `grammar_pilot_parity` |
| **2** | MCP-BUILD-SET-001…003 | @planner-mcp + @coder-mcp | manifest schema + `building_set_manifest_validate` + `building_set_coverage_report` |
| **3** | MCP-GUARD-001…004 | @coder-mcp | hardcode lint + teachable audit + ratio guard + witness |
| **4** | BUILD-SET-PILOT-002 | @designer-mcp + @coder | Second grammar pilot (manufacturing or housing) in catalog |
| **5** | CODER-PILOT-REFACTOR-001 | @coder | Remove warehouse-only branches; data-driven parity tests |
| **6** | MCP-INTEGRATE-001…002 | @coder-mcp | `complex_building_brief` + APS status strip + `grammar_integration_validate` |
| **7** | OPS-BUILD-SET-001 | @coder-mcp | `ops_get_build_set_health` section in `ops_project_brief_v1` |

---

## Registry append (planned Tier 1g)

Add to `MICRO_TOOLS_REGISTRY_v1.md` when Phase 1 lands:

```text
Tier 1g — grammar/build-set guards
  grammar_set_brief · grammar_preset_pair_validate · grammar_eval_sweep
  grammar_pilot_parity · building_set_manifest_validate · building_set_coverage_report
  pilot_hardcode_lint · example_teachable_audit · single_archetype_ratio_guard
  complex_building_brief · grammar_integration_validate · building_set_health_brief
```

---

## Acceptance (program done)

| # | Criterion |
|:---:|:---|
| 1 | ≥2 grammar pilots in `_pilot_catalog.ron` with distinct `arch_dna.F` |
| 2 | `pilot_hardcode_lint` green on `src/construction/` |
| 3 | `building_set_coverage_report` wired to `pipeline_preflight` optional flag |
| 4 | All new schema examples have `_meta.teaches` ≥2 |
| 5 | `grammar_diversity_witness` reports per-archetype rows, not only `IndustrialWarehouse` |
| 6 | OPS brief includes `build_set_health` block |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-13 | Post warehouse-refactor lesson plan |
