# Construction coder drain order `v1`

| Field | Value |
|:---|:---|
| **ID** | **CONSTRUCTION-DRAIN-ORDER-001** |
| **Date** | 2026-06-03 |
| **Scope** | **Construction program only** — not infra E0–E6, tile PT, weather, fleet stab |
| **Witness** | `debug_runs/construction_stage_live.json` |
| **Index** | [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) |
| **Machine queue** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) → `construction_program` |

**Rule:** One slice per PR (≤3 files). `SiteConstructionPhase` only. Growth/procedural → `ConstructionPlanQueue` → commit — never instant Operational.

---

## Closed on disk (do not re-queue)

| Phase | IDs | Witness keys |
|:---|:---|:---|
| **P1 Parametric** | PARAM-CODER-001..006 | `construction_parametric_placement_001.green` |
| **P2 Staged pipeline** | CON-P2-001..003 | `construction_site_stage_pipeline_001`, `construction_site_stage_tick_002` |
| **P3 Scaling audit** | CON-P3-S1..S6, CON-P3-WIT | `construction_scaling_audit_001` (S1–S6 + partial_alpha) |
| **P4 Procedural spine** | PROC-PG-1-001, PG-2-TAIL, PG-3 witness | `construction_procedural_build_001` |
| **P5 Settlement** | SET-P5-001..003 | `construction_settlement_hierarchy_001` |
| **P6 Organic growth sim** | ECON-OG-1-A/B/C, PROC-OG-2/3, PROC-OG-UX | `construction_organic_growth_001`, `construction_growth_inspector_001` |
| **P6+ Coder A drain** | PROC-OG-4-ROLLUP-001, CON-P2-CLEARING-WIT-001, PROC-PG-4-001 | `construction_town_rollup_001`, `construction_site_stage_pipeline_001.clearing_substeps_seen`, `construction_shape_grammar_001` |
| **R4 product** | R4 corridor/MV/prep | `construction_r4_*` blocks |

**Lib gate:** `cargo test -p proc_A_dine01 --lib construction` — **144/144 green** (2026-06-03).

---

## Gap (code vs product)

| Gap | Today | Target |
|:---|:---|:---|
| **Approve → execute** | Inspector removes proposal from queue; no `ConstructionPlanQueue` write | Approve enqueues `PendingBuildBlueprint` / staged row → same funnel as player |
| **Settlement persist** | Witness `save_roundtrip_ok: true` stub | RON slice round-trip for Town/District/Block books |

**Closed (Coder A drain 2026-06-03):** town rollup from block site counts; clearing substeps witness Trees/Stumps; PG-4 grammar stable layout (`building_grammar.rs` + `industrial_warehouse_v1.ron`).

---

## Drain order — @coder B (growth execute spine)

| P | ID | Files hint | Exit |
|:---:|:---|:---|:---|
| 1 | **ECON-OG-SAVE-001** | `src/io/save/`, `strategic/settlement/` | Settlement books persist + reload; witness `save_roundtrip_ok` from real round-trip |
| 2 | **PROC-OG-APPROVE-001** | `gui/construction_growth_inspector.rs`, `construction/pending_construction.rs` or `build_interaction.rs` | Approve → `ConstructionPlanQueue` / pending blueprint; lib test + witness `execute_via_pipeline` from real wiring |
| 3 | **PROC-OG-POLICY-001** | `strategic/settlement/policy.rs`, `growth.rs` | `AutoBuildPolicy::AutoCommercial` / `AutoAll` drains proposals to queue on tick (still Planned on commit) |

**Do not:** spawn Operational on approve; duplicate queue writers outside construction funnel.

---

## Drain order — @coder A (rollup + grammar + stage witness)

**Status:** **DONE** (2026-06-03) — rows below closed on disk; horizon → BUILD-WORKER-001 (Bevy preview slice).

| P | ID | Files hint | Exit |
|:---:|:---|:---|:---|
| 1 | **PROC-OG-4-ROLLUP-001** | `town_rollup.rs`, `settlement/mod.rs` | `construction_town_rollup_001` + `town_rollup_wired` on organic growth |
| 2 | **CON-P2-CLEARING-WIT-001** | `site_stage_tick.rs` (witness helpers only), `witness_collectors.rs` | `clearing_substeps_seen` Trees/Stumps in pipeline witness |
| 3 | **PROC-PG-4-001** | `building_grammar.rs`, `assets/configs/buildings/grammars/` | `proc_pg4_001_shape_grammar_witness_green` + `construction_shape_grammar_001` |

**Parallel safe with B:** A row 1 while B row 1 (disjoint: town rollup vs save IO).

---

## Horizon (construction roadmap — after drain above)

| Phase | ID | Owner | When |
|:---|:---|:---|:---|
| **P7 Logistics hook** | CON-P7-LOGISTICS-001 | B | After approve pipeline green — facility activation reads graph-only paths |
| **P8 GIS** | deferred | planner | Town stable + save slice |
| **Grammar pilot ship** | PILOT-GRAMMAR-001 | A + coder-mcp | Designer `proceed_ship: yes` |

**Not in this drain:** INFRA-E*, PT-*, weather, Hanabi — pull from `infrastructure_program` / art lanes separately.

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib construction
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
```

---

## Copy-paste

### @coder B

```text
Construction drain only — docs/archive/2026-06-src-dev/plans/construction_coder_drain_order_v1.md
1) ECON-OG-SAVE-001
2) PROC-OG-APPROVE-001
3) PROC-OG-POLICY-001
Skip closed P1–P6 rows. Update coder_active_queue.json construction_program on done.
```

### @coder A

```text
Construction drain only — docs/archive/2026-06-src-dev/plans/construction_coder_drain_order_v1.md
1) PROC-OG-4-ROLLUP-001
2) CON-P2-CLEARING-WIT-001
3) PROC-PG-4-001 (optional until grammar pilot ship)
Infra rows are parallel lane — not this drain.
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-06-03 | Post P1–P6 witness green; construction-only next drain |
