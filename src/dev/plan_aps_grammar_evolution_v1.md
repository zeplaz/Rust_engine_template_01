# PLAN-APS-GRAMMAR-EVOLUTION-001 — Grammar maturity drives APS exposure & preview

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-APS-GRAMMAR-EVOLUTION-001** |
| **Status** | **ACTIVE — drain queue issued** |
| **Machine queue** | [`aps_grammar_evolution_queue.json`](../tools/orchestrator/queues/aps_grammar_evolution_queue.json) (15 rows) |
| **Agent board** | [`aps_grammar_evolution_agent_todos_v1.md`](aps_grammar_evolution_agent_todos_v1.md) |
| **Dispatch** | [`aps_grammar_evolution_dispatch_orders_v1.md`](../tools/orchestrator/queues/aps_grammar_evolution_dispatch_orders_v1.md) |
| **Witness** | [`plan_aps_grammar_evolution_witness_v1.md`](plan_aps_grammar_evolution_witness_v1.md) |
| **Presence correction** | [`planner_routing_aps_presence_v1.md`](planner_routing_aps_presence_v1.md) · [`design_aps_default_presence_audit_v1.md`](design_aps_default_presence_audit_v1.md) |
| **Date** | 2026-06-07 · **G3 reconcile** 2026-06-21 |
| **Owner** | @planner (plan) → @designer (IA) → @coder-mcp (APS/MCP) → @coder-mcp + @designer-mcp (content) |
| **Parent** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) · [`plan_mcp_grammar_build_set_guards_v1.md`](plan_mcp_grammar_build_set_guards_v1.md) |
| **APS UX** | [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) · [`aps_design_system_v1.md`](aps_design_system_v1.md) |
| **Grammar schema** | [`arch_build_grammar_001_schema_v1.md`](arch_build_grammar_001_schema_v1.md) · [`arch_build_grammar_v0_baseline_v1.md`](arch_build_grammar_v0_baseline_v1.md) |
| **Landscape mirror** | [`design_aps_grammar_panel_v1.md`](design_aps_grammar_panel_v1.md) · [`plan_landscape_grammar_exec_001_v1.md`](plan_landscape_grammar_exec_001_v1.md) |

---

## Why this plan exists

APS **grammar tooling** and **grammar content** matured at different speeds. The repo now ships **four building grammars** and four ARCH-DNA presets; APS cold-starts at **G3 — layer depth** (`refresh_grammar_tier_from_registry()`). Earlier plan copy still described **G0 singleton** — that was **pilot-era baseline**, not today's disk.

**Rule for this program:** what APS **shows**, **previews**, and **asks the artist to do next** must **track grammar-set maturity** — not a fixed chrome layout from the pilot era.

```text
Grammar content matures (G0 → G4)
        ↓
grammar_set_brief / coverage guards report tier
        ↓
APS unlocks panels, preview fidelity, pipeline steps
        ↓
Witness JSON proves tier — not “UI feels fine”
```

---

## North star (artist path)

```text
Pick building type + district  (grammar authority)
        ↓
Generate Assembly            (snapshot + grammar_rule_chain)
        ↓
Inspect why                  (inspector ↔ footprint highlight)
        ↓
Assign materials             (authority — unchanged)
        ↓
Preview                      (fidelity rises with tier)
        ↓
Ship check → Variants → Atlas
```

**Manual footprint / style pack** remains a **fallback lane** — never the default when grammars exist.

---

## Two lanes (do not collapse)

| Lane | Delivers | Paths | Agent |
|:---|:---|:---|:---|
| **A — Content** | Grammars, districts, DNA presets, module kit gaps | `assets/configs/buildings/grammars/*.ron`, `tools/mcp/schemas/examples/`, `_module_index.ron` | @designer-mcp → @coder-mcp |
| **B — Tooling** | APS exposure tiers, preview ladder, pipeline spine, guards | `tools/mcp/art_pipeline_suite/`, `rust_engine_mcp/building_grammar.py`, witnesses | @designer → @coder-mcp |

**Gate:** tooling slice **B-n** must not claim “done” unless content tier **A-n** exists **or** the UI explicitly shows a **tier-locked** empty state (not a sparse broken dropdown).

---

## Grammar set maturity model (G0–G4)

Reported authoritatively by `grammar_set_tier()` + `grammar_set_brief` + `building_set_coverage_report` (see [`plan_mcp_grammar_build_set_guards_v1.md`](plan_mcp_grammar_build_set_guards_v1.md)).

| Tier | Content bar | Example today (2026-06-21) |
|:---|:---|:---|
| **G0 — Pilot singleton** | 1 archetype, ≥1 district, grammar generates placements | **Fixture only** — CI matrix wiring; not cold-start tier |
| **G1 — Family seed** | ≥3 archetypes *or* ≥3 districts in one lineage; JSON mirrors for MCP | **Met** — 4 archetypes on disk |
| **G2 — Axis coverage** | ARCH-DNA axes represented across set (F,L,C,D,W… per v0 baseline) | **Met** — 4 F-axis presets (fuel, logistics, manufacturing, power) |
| **G3 — Layer depth** | `grammar_rule_chain` includes facade + detail + age layers in snapshots | **Current cold-start** — DNA + iterate visible; kit hint off; set-health promoted |
| **G4 — Production set** | Diversity witness green; module audit gaps closed; G4 bake path unblocked | **Blocked** — `building_set_coverage` not green; `pilot_hardcode_green` false |

**Live tier witness:** `debug_runs/grammar_set_tier_live.json` → **G3** (4 archetypes, 5 districts, 4 presets).

**G4 gap (honest):** pilot counts align (`APS-GUARD-BRIEF-PARITY-001` green) but coverage/hardcode guards still red — tier reasons include `building_set_coverage not green for G4`.

**APS reads tier at runtime** (cached from last `grammar_set_brief` or witness) — do not hardcode “we have grammar” from a single file on disk.

---

## APS exposure map (what unlocks when)

**Designer-owned IA contract** — canonical matrix: [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) (`@designer`). Wireframes: [`design_aps_grammar_tier_wireframes_v1.md`](design_aps_grammar_tier_wireframes_v1.md). Implementation gates in `assembly_panel.py` + pipeline spine.

### Always visible (all tiers)

| Surface | Purpose |
|:---|:---|
| **Generate** — building type + district (human labels) | Primary path |
| **Footprint grid + placement list** | Spatial truth after generate |
| **Material library + slot edit** | Ship authority |
| **Grammar inspector** (collapsed default) | Read-only rule chain after generate |
| **Pipeline status bar** | Single “what’s next” spine |

### Tier-gated surfaces

| Surface | G0 | G1 | G2 | G3 | G4 |
|:---|:---:|:---:|:---:|:---:|:---:|
| District / archetype dropdowns | 1–2 values + **kit hint** | Multi-value, grouped by lineage | + DNA axis badges on rows | — | — |
| **Kit grammar reference** (build-set brief / sweep) | Advanced, collapsed | Advanced | **Promoted** — “Set health” strip | Same + sweep required before ship | CI-linked |
| **Shape bias / ARCH-DNA** (`GrammarDnaPanel`) | Hidden | Collapsed advanced | **Visible** — preset picker | β sliders active | Saved on snapshot by default |
| **Tweak one layer** (`GrammarIteratePanel`) | Hidden | Collapsed | Collapsed | **Visible** — massing/facade iterate | + diff overlay default |
| **Manual style pack + footprint** | Collapsed “Manual fallback” | Same | Same | Same | Deprecated banner if grammar covers style |
| **Assembly preview thumb** | P1 slot + P2 quick assembly | P2 | P2 + rule highlight | P3 grammar-aware | P4 ship fidelity |
| **Variants / Atlas steps** | Spine visible; bake may warn | Same | Same | Preview hints reference grammar tags | Full ship path |

### Anti-patterns (forbid)

- Showing **five grammar panels** at full width before **G1**
- Engineer labels (`IndustrialWarehouse`) in primary combos
- Second pipeline walkthrough in Generate *and* status bar
- Landscape footprint widgets on Buildings tab (already forbidden — keep)

---

## Preview fidelity ladder (buildings)

Coupled to tier — see also [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) APS-PREVIEW-*.

| Level | ID | What artist sees | Unlocks |
|:---|:---|:---|:---|
| **P0** | `PREVIEW-FOOTPRINT` | Token heatmap W/D/C/R/Y | G0 |
| **P1** | `PREVIEW-SLOT` | Isolated module + material thumb | G0 |
| **P2** | `PREVIEW-ASSEMBLY` | Combined assembly thumb (browser / worker) | G0 (partial today) |
| **P3** | `PREVIEW-GRAMMAR` | Click inspector row → highlight placements + “why” tooltip | **G2** |
| **P4** | `PREVIEW-SHIP` | Keyframe / ship render fidelity chip | **G4** |

**Implementation hook:** `grammar_rule_chain` step ids must map to `module_placements` keys for P3 (today inspector lists chain but grid does not link back).

---

## Pipeline steps (evolve with tier)

Current Buildings spine (conceptual):

```text
Catalog → Assembly → Variants → Atlas
```

**Target spine copy** (artist-facing, tier-aware):

| Step | G0 copy emphasis | G2+ copy emphasis |
|:---|:---|:---|
| **Catalog** | Pick modules for kit | Tags are hints; grammar filters later |
| **Assembly** | Generate from building type | Tune shape bias; inspect rule chain |
| **Assembly** | Assign materials on cells | Same + “layers affected” from inspector |
| **Variants** | Optional states | Variant tags ↔ grammar age/detail rules |
| **Atlas** | Bake when ship check passes | Diversity note if grammar sweep stale |

**Machine fields** (pipeline pills): add `grammar_set_tier` + `grammar_sweep_stale` to `SuiteState` / witness — spine grays **Pack atlas** when tier &lt; G4 and ship check failed, not silently.

---

## Landscape grammar (parallel, do not merge UI)

Buildings grammar evolves on **Assembly**; landscape on **Presets → Grammar → States → Atlas** ([`design_aps_grammar_panel_v1.md`](design_aps_grammar_panel_v1.md)).

| Concern | Buildings | Landscape |
|:---|:---|:---|
| Authority object | `assembly_snapshot` | `landscape_grammar_v0` preset |
| Preview | Footprint grid | Topology graph canvas |
| Shared code | `GrammarInspectorPanel` branch only | No `FootprintCanvas` |

**Cross-lane rule:** shared components (`GrammarInspectorPanel`, labels in `aps_grammar_labels.py`) stay **read-only** and **context-branched** — never one combo row for both domains.

---

## Execution phases

### Phase 0 — UX consolidation (partial ✓)

| ID | Task | Status |
|:---|:---|:---:|
| APS-GRAM-UX-001 | Grammar on by default; human labels; kit hint when G0 | **done** |
| APS-GRAM-UX-002 | Build-set panel → advanced collapsible | **done** |
| APS-THEME-001 | Dark theme + readable combobox/list/text | **done** |
| APS-GRAM-UX-003 | Single pipeline “what’s next” owner (spine vs Generate hint) | verify |

### Phase 1 — Tier registry & dynamic chrome (tooling)

| ID | Owner | Task | Depends |
|:---|:---|:---|:---|
| **APS-GRAM-TIER-001** | @coder-mcp | `grammar_set_tier()` in `rust_engine_mcp/grammar_build_set.py` — returns G0–G4 + reasons[] | `grammar_set_brief` |
| **APS-GRAM-TIER-002** | @coder-mcp | `AssemblyPanel.apply_grammar_tier(tier)` — show/hide DNA, iterate, build-set per exposure table | APS-GRAM-TIER-001 |
| **APS-GRAM-TIER-003** | @designer | Sign-off wireframes per tier (G0 minimal, G2 expanded) — amend `design_aps_assembly_density_v1.md` | APS-GRAM-TIER-002 mock |
| **APS-GRAM-TIER-004** | @coder-mcp | Pipeline bar reads tier; step copy from table above | APS-GRAM-TIER-001 |
| **APS-GRAM-REG-001** | @coder-mcp | Archetype/district combos driven from registry only — no fallback `["IndustrialWarehouse"]` in UI | `list_archetype_ids()` |

**Exit (Phase 1):** Launch APS → `refresh_grammar_tier_from_registry()` applies **live tier** (G3 today); tier chip shows `G3 — layer depth`; pytest `test_aps_grammar_tier_gates.py` — **live** witness at `aps_grammar_tier_gates_live.json`, **G0 fixture** at `aps_grammar_tier_gates_g0_fixture_live.json` only.

### Phase 2 — Content G1 (family seed)

| ID | Owner | Task | Depends |
|:---|:---|:---|:---|
| **GRAM-CONTENT-001** | @designer-mcp | Second archetype spec (e.g. `FactoryCluster` or `RailEdge`) — massing strategies + district | ARCH-BUILD-GRAMMAR-001 |
| **GRAM-CONTENT-002** | @coder-mcp | Add `*.ron` + JSON mirror under `grammars/` + `schemas/examples/` | GRAM-CONTENT-001 |
| **GRAM-CONTENT-003** | @coder-mcp | `grammar_labels_v1.json` + `aps_grammar_labels.py` entries for new ids | GRAM-CONTENT-002 |
| **GRAM-CONTENT-004** | @coder-mcp | `building_set_coverage_report` → G1 green witness | GRAM-CONTENT-002 |

**Exit:** Dropdowns show ≥3 meaningful choices; kit hint removed or downgraded; `debug_runs/grammar_set_tier_g1.json`. **Status:** content bar met; live tier is G3.

### Phase 3 — Inspector ↔ preview coupling (P3)

| ID | Owner | Task | Depends |
|:---|:---|:---|:---|
| **APS-GRAM-P3-001** | @coder-mcp | Inspector row click → `FootprintCanvas` selection + diff style | G2 tier |
| **APS-GRAM-P3-002** | @coder-mcp | Snapshot carries `placement_rule_id` per module (if missing, extend schema + generator) | ARCH-ASSEMBLY-GRAPH-002 |
| **APS-GRAM-P3-003** | @designer | Copy for “why this module” tooltips — link `aps_grammar_labels` + `grammar_why_detail` | APS-GRAM-P3-001 |

**Exit:** Click `long_hall` in inspector → cells highlight; witness `aps_grammar_p3_live.json`.

### Phase 4 — ARCH-DNA & iterate (G2–G3)

| ID | Owner | Task | Depends |
|:---|:---|:---|:---|
| **APS-GRAM-DNA-001** | @coder-mcp | Promote `GrammarDnaPanel` per tier; human labels on presets | G2 |
| **APS-GRAM-ITER-001** | @coder-mcp | Iterate panel: facade/detail modes when chain supports layer | G3 |
| **APS-GRAM-SWEEP-001** | @coder-mcp | Eval sweep stale flag → warn on ship check if sweep older than grammar file mtime | `grammar_eval_sweep` |

### Phase 5 — Production alignment (G4)

| ID | Owner | Task | Depends |
|:---|:---|:---|:---|
| **APS-GRAM-G4-001** | @coder-mcp + @coder | P4 preview chip + ship check ties to grammar diversity witness | PG-QUALITY-001, PILOT-GRAMMAR-001 |
| **APS-GRAM-G4-002** | @orchestrator-mcp | MCP promote path blocked when `building_set_coverage_report` &lt; G4 | guards plan |

---

## Witness & validation contracts

| Witness | Proves |
|:---|:---|
| `debug_runs/grammar_set_tier_live.json` | **Live** tier + counts from `grammar_set_tier()` |
| `debug_runs/grammar_set_brief_live.json` | Brief green + pilot inventory |
| `debug_runs/aps_grammar_tier_gates_live.json` | **Live** UI exposure matches registry tier (anti-fake-green) |
| `debug_runs/aps_grammar_tier_gates_g0_fixture_live.json` | **Fixture** — G0 matrix wiring only; not cold-start truth |
| `debug_runs/aps_session_presence_live.json` | Bundled presence rollup (tier + brief + guards + ui_presence) |
| `debug_runs/grammar_set_tier_g{N}.json` | Tier milestone snapshots (G1 historical) |
| `debug_runs/aps_grammar_p3_live.json` | Inspector ↔ grid link |
| `debug_runs/grammar_diversity_witness.json` | Content diversity (existing) |
| `pytest -k aps` | No regression on generate callbacks |

**Validation-first:** `validate-report witness_honesty` on live witnesses — agents reason on JSON, not Tk screenshots.

---

## Suggested work order (next 2–3 sessions)

```text
1. G4 guard green     pilot_hardcode_lint + building_set_coverage → tier G4 honest
2. DES-APS-SESSION-DUMP-001   bundled aps_session_presence_live.json
3. OVR-APS-PRESENCE-OPERATOR-001   operator rubric walk + HANDOFF attach
4. APS-GRAM-G4-001    P4 preview chip + ship check ↔ diversity witness
```

**Do not:** re-open tier exposure IA — [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) PASS at G3.

---

## Agents

| Role | Responsibility |
|:---|:---|
| **@planner** | Keep this plan + parent PLAN-BUILDING-GRAMMAR-001 aligned |
| **@designer** | Exposure table sign-off; tier wireframes; plain-language copy |
| **@coder-mcp** | APS tiers, registry, inspector coupling, MCP brief/sweep |
| **@designer-mcp** | New grammar RON specs + module kit gaps per archetype |
| **@coder** | Rust evaluator parity if placement_rule_id or layers extend |
| **@sim-steward** | Witness honesty — no green when tier UI mismatches content |

**Orchestrator paste:**

```text
Execute PLAN-APS-GRAMMAR-EVOLUTION-001: Phase 1 tier gates → Phase 2 G1 content → Phase 3 inspector-preview link.
Do not add grammar panels without tier gate. Content and tooling PRs separate.
Parent: plan_building_grammar_evolution_v1.md · Guards: plan_mcp_grammar_build_set_guards_v1.md
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-06-21 | **G3 reconcile** — maturity table, witness split (live vs G0 fixture), Phase 1 exit, work order → G4 path · `PLAN-APS-PRESENCE-PLAN-EDIT-001` |
| v1.0.0 | 2026-06-07 | Initial plan — maturity-driven APS exposure + preview ladder |
