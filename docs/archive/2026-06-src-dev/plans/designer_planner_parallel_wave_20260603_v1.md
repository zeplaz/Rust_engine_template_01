# Designer + Planner parallel wave — June 2026 `v1`

| Field | Value |
|:---|:---|
| **Wave ID** | **DESIGN-PLANNER-PARALLEL-20260603** |
| **Rule** | **Coders execute** — designer/planner **unblock** with specs, copy, schemas, sign-offs |
| **No Rust** | @designer · @planner-mcp — docs, wireframes, JSON schemas only |
| **No Tk/Python** | @designer does not implement APS — @coder-mcp does |
| **Warehouse keyframe** | **PAUSED** — not a designer/planner gate |

**Dispatch hubs:** [`grammar_iter_agent_orders_v1.md`](grammar_iter_agent_orders_v1.md) · [`bevy_hud_lanes_agent_orders_v1.md`](bevy_hud_lanes_agent_orders_v1.md) · [`plan_aps_artist_tool_exec_v1.md`](plan_aps_artist_tool_exec_v1.md)

---

## Already done (do not re-assign)

| ID | Owner | Artifact | Verdict |
|:---|:---|:---|:---:|
| APS-UX-AUDIT-001 | @designer | [`design_aps_ux_audit_v1.md`](design_aps_ux_audit_v1.md) | **PASS** (lead) |
| SIM-HUD-PRODUCT-001 | @designer | [`design_sim_hud_product_signoff_v1.md`](design_sim_hud_product_signoff_v1.md) | **PASS (qualified)** |
| APS-UX-TOOLTIPS-002 | @designer | [`aps_tooltip_copy_v1.md`](../prompts/designer_questions/aps_tooltip_copy_v1.md) | PASS |
| APS-ATLAS-LEGEND-001 | @designer | [`design_aps_atlas_preview_legend_v1.md`](design_aps_atlas_preview_legend_v1.md) | PASS |
| APS-MAT-IA-001 | @designer | [`design_aps_materials_tab_ia_v1.md`](design_aps_materials_tab_ia_v1.md) | PASS |
| APS-UX-POLISH-001-SIGNOFF | @designer | [`design_aps_ux_polish_signoff_v1.md`](design_aps_ux_polish_signoff_v1.md) | PASS WITH NOTES |
| DESIGN-WEATHER-PLAYER-READ-001 | @designer | [`design_weather_player_read_v1.md`](design_weather_player_read_v1.md) | PASS |
| GRAMMAR-ITER-001-UI | @designer | [`design_grammar_iter_ui_v1.md`](design_grammar_iter_ui_v1.md) | PASS WITH NOTES |
| APS-BEVY-QC-HUD-001-DESIGN | @designer | [`design_aps_bevy_qc_hud_v1.md`](design_aps_bevy_qc_hud_v1.md) | PASS WITH NOTES |
| SIM-HUD slices (OPS/DOCK/MINIMAP/BUILD) | @designer | `design_sim_hud_*.md` | specs ready for @coder |
| GRAMMAR-ITER-001-SPEC | @planner-mcp | [`grammar_iter_001_spec_v1.md`](grammar_iter_001_spec_v1.md) | SPEC READY |
| PG-MODULE-AUDIT-001 | @designer | [`pg_module_audit_warehouse_v1.md`](pg_module_audit_warehouse_v1.md) | done |

---

## @designer — active queue (priority order)

| P | ID | Deliverable | Unblocks | Hours |
|:---:|:---|:---|:---|:---:|
| — | *(idle)* | Implementation review when @coder notifies PR | — | on-call |

### Optional follow-up (not blocking)

| ID | Note |
|:---|:---|
| **SIM-HUD-SLICE-MINIMAP-CODER** | Witness optional after qualified program close |
| **SIM-HUD-SLICE-BUILD-CODER** | Witness optional after qualified program close |
| **APS-BEVY-QC-HUD-001-V2** | Designer review after @coder lands row highlight |

### Designer — not this wave

- `@designer-mcp` — MCP batches, G4 ship, warehouse keyframe (**PAUSED**)
- Rust / egui implementation — @coder
- Tk APS implementation — @coder-mcp

---

## @designer — closed this wave (2026-06-03)

| ID | Artifact |
|:---|:---|
| APS-UX-TOOLTIPS-002 | [`aps_tooltip_copy_v1.md`](../prompts/designer_questions/aps_tooltip_copy_v1.md) |
| APS-ATLAS-LEGEND-001 | [`design_aps_atlas_preview_legend_v1.md`](design_aps_atlas_preview_legend_v1.md) |
| APS-MAT-IA-001 | [`design_aps_materials_tab_ia_v1.md`](design_aps_materials_tab_ia_v1.md) |
| APS-UX-POLISH-001-SIGNOFF | [`design_aps_ux_polish_signoff_v1.md`](design_aps_ux_polish_signoff_v1.md) |
| DESIGN-WEATHER-PLAYER-READ-001 | [`design_weather_player_read_v1.md`](design_weather_player_read_v1.md) |

---

## @planner-mcp — active queue (priority order)

| P | ID | Deliverable | Unblocks | Hours |
|:---:|:---|:---|:---|:---:|
| **1** | **GRAMMAR-ITER-SNAPSHOT-001** | Extend `assembly_snapshot_v1.schema.json` — optional `grammar_lineage`, `grammar_overrides` | @coder snapshot serde | 1–2 |
| **2** | **GRAMMAR-ITER-RESULT-001** | `tools/mcp/schemas/grammar_iterate_result_v1.schema.json` | @coder-mcp API witness | 1 |
| **3** | **APS-VALIDATOR-PLAIN-001** | [`aps_validator_plain_language_v1.md`](aps_validator_plain_language_v1.md) — map P0 / assembly_p0 / material codes → artist sentences | APS-MAT-AUTH-UI-001 | 2–3 |
| **4** | **APS-MAT-CATEGORY-SCHEMA-001** | `material_library_v1.json` or extend `material_profiles_v1.json` with `category` tree | APS-MAT-002 scale | 2 |
| **5** | **GRAMMAR-002-SLICE-001** | Thin plan: facade + roof partial regen (T2/T3) — deps on GRAMMAR-001 | @coder Track C | 2–4 |

### Planner — on-call only

- New mega-plans / warehouse replan
- PBG mesh-face generator (**ARCH-PBG-MASSING-002**) — only if orchestrator requests

---

## Paste — @designer (open this chat)

```text
You are @designer on Rust_engine_template_01 — NO Rust, NO tools/mcp Python.

READ: docs/archive/2026-06-src-dev/plans/designer_planner_parallel_wave_20260603_v1.md

SESSION GOAL — pick 1–2 items:

1) APS-UX-TOOLTIPS-002 (highest leverage)
   - Source: design_aps_ux_audit_v1.md § prioritized fixes + tab walkthrough
   - Output: prompts/designer_questions/aps_tooltip_copy_v1.md
   - Format: tab → control_id → tooltip string (≤120 chars) + optional on-screen hint
   - Cover: Catalog, Assembly, Materials, Variants, Atlas, Flow bar, Iterate grammar (wireframe copy)

2) APS-ATLAS-LEGEND-001
   - Output: docs/archive/2026-06-src-dev/plans/design_aps_atlas_preview_legend_v1.md
   - UV grid legend, cell hover text, atlas_meta validate errors in plain English

3) APS-MAT-IA-001
   - Output: docs/archive/2026-06-src-dev/plans/design_aps_materials_tab_ia_v1.md
   - Industrial→Steel/Corrugated tree; link to Assembly slot apply

SIM-HUD specs already on disk — wait for @coder slices before SIGNOFF-001.

Do NOT: designer-mcp batches, warehouse keyframe, Tk implementation.
```

---

## Paste — @planner-mcp

```text
You are @planner-mcp on Rust_engine_template_01 — schemas + thin plans only, NO implementation.

READ: docs/archive/2026-06-src-dev/plans/designer_planner_parallel_wave_20260603_v1.md
READ: docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md

SESSION GOAL — pick 2–3 items:

1) GRAMMAR-ITER-SNAPSHOT-001
   - Extend tools/mcp/schemas/assembly_snapshot_v1.schema.json (optional lineage fields)
   - Example snapshot snippet in spec

2) GRAMMAR-ITER-RESULT-001
   - tools/mcp/schemas/grammar_iterate_result_v1.schema.json matching spec § Response

3) APS-VALIDATOR-PLAIN-001
   - docs/archive/2026-06-src-dev/plans/aps_validator_plain_language_v1.md
   - Map: StylePackDrift, FootprintTooSmall, GrammarChainMissing, missing material_profile → artist text

4) APS-MAT-CATEGORY-SCHEMA-001
   - Category tree for 300 profiles (Industrial/Residential/…)

5) GRAMMAR-002-SLICE-001 (if time)
   - Facade/roof partial regen — what GRAMMAR-001 must land first

Update grammar_continuation_queue.json rows → done when shipped.
NO warehouse keyframe sequencing.
```

---

## Paste — @orchestrator

```text
While coders run parallel lanes, assign:

DESIGNER (docs only):
- APS-UX-TOOLTIPS-002 → aps_tooltip_copy_v1.md
- APS-ATLAS-LEGEND-001 → atlas UV legend
- APS-MAT-IA-001 → Materials tab IA

PLANNER-MCP (schemas + plain language):
- GRAMMAR-ITER-SNAPSHOT-001 + RESULT schema
- APS-VALIDATOR-PLAIN-001
- APS-MAT-CATEGORY-SCHEMA-001
- GRAMMAR-002-SLICE-001 optional

Board: docs/archive/2026-06-src-dev/plans/designer_planner_parallel_wave_20260603_v1.md

Designer SIM-HUD + GRAMMAR-ITER + QC + APS audit = DONE on disk.
Coder queues: SIM-HUD-*-CODER, GRAMMAR-ITER-APS1/API, APS-BEVY-QC-V2, APS artist tool phases.
```

---

## Coder alignment (who consumes designer/planner output)

| Designer/planner output | Consumer |
|:---|:---|
| `aps_tooltip_copy_v1.md` | @coder-mcp → `aps_tooltips.py` |
| `design_aps_atlas_preview_legend_v1.md` | @coder-mcp → Atlas tab overlay |
| `design_aps_materials_tab_ia_v1.md` | @coder-mcp → Materials tab |
| `grammar_lineage` schema | @coder + @coder-mcp snapshot load/save |
| `grammar_iterate_result_v1.schema.json` | @coder-mcp CLI + tests |
| `aps_validator_plain_language_v1.md` | @coder-mcp P0 gate UI + pipeline bar |
| `design_sim_hud_*.md` | @coder → `in_game_hud.rs` (in progress) |
| `design_grammar_iter_ui_v1.md` | @coder-mcp → Iterate panel (in progress) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Designer/planner wave while coders execute |
