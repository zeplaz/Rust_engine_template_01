# Coder orders — settlement + econ growth + organic queue `v1`

**Date:** 2026-06-02  
**Exec:** [`plan_econ_growth_actors_exec_001_v1.md`](plan_econ_growth_actors_exec_001_v1.md) **SIGNED**  
**Settlement:** [`plan_settlement_hierarchy_exec_005_v1.md`](plan_settlement_hierarchy_exec_005_v1.md)  
**Organic:** [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) v1.1  
**Queue:** [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json)

**Closed (do not re-pick):** CON-P2-001/002/003 · INFRA-E0-001/002 (both coders).

---

## @coder A — orders (fill queue)

```text
You are Coder A on Rust_engine_template_01 (master). Drain active[] in priority order. ≤3 files per PR. Witness JSON wins.

MCP: consumer only — after each cargo test:
  python -m rust_engine_mcp.cli validate-report cargo --compress 3
  python -m rust_engine_mcp.cli validate-report bevy -p proc_A_dine01 --compress 3
Do NOT edit tools/mcp/python/

READ:
- src/dev/plan_construction_scaling_audit_exec_003_v1.md
- src/dev/plan_settlement_hierarchy_exec_005_v1.md
- src/dev/plan_procedural_build_gen_exec_001_v1.md § PG-1
- src/dev/plan_econ_growth_actors_exec_001_v1.md (unblocks B — you own SET-P5-001 + PG-1 partial)

DRAIN (in order):

1) CON-P3-S1 — scaling audit preset matrix
2) CON-P3-S2 — occupied tiles wired
3) CON-P3-S3 — blocked disables commit
4) CON-P3-WIT — construction_scaling_audit_001.green on disk

5) SET-P5-001 — BLOCKS B ECON LANE
   TownBook + DistrictBook + DistrictRecord.style_rules + archetype_caps in RON
   Files: src/strategic/settlement/ (ids, town, district, loader)
   Design: design_settlement_hierarchy_read_v1.md
   Exit: town_book_loaded + district_book_loaded witness keys

6) PROC-PG-1-001 — PG-1 partial (3 archetypes minimum)
   BuildingArchetype + BuildingUsage on RON; procedural_archetypes_loaded
   Unblocks B market saturation by archetype id

7) SET-P5-003 — settlement witness + play seed default_town.ron
   Gate: construction_settlement_hierarchy_001.green

8) INFRA-E0-003 — legacy transport stubs (when unblocked)

PARALLEL OK: SET-P5-001 with CON-P3 tail (disjoint files).

DO NOT: site_stage_tick.rs (B); Operational on commit.

REGRESSION each PR:
cargo test -p proc_A_dine01 --lib construction construction_scaling_audit settlement
```

---

## @coder B — orders (fill queue)

```text
You are Coder B on Rust_engine_template_01 (master). Drain active[] in priority order. ≤3 files per PR.

MCP: validation-first only — no tools/mcp/ edits.

READ:
- src/dev/plan_settlement_hierarchy_exec_005_v1.md § SET-P5-002
- src/dev/plan_econ_growth_actors_exec_001_v1.md (PRIMARY — ECON-OG-1-A/B/C)
- src/dev/plan_organic_growth_exec_001_v1.md
- src/dev/design_organic_growth_ux_v1.md (PASS — OG-3)
- src/dev/construction_economy_growth_vision_v1.md

PREREQ FROM A (ping if not merged):
- SET-P5-001 DistrictBook + style_rules
- PROC-PG-1-001 partial archetypes (for archetype_caps tests)

CON-P2-002: DONE — do not re-pick.

DRAIN (in order):

1) SET-P5-002 — BlockBook + tile_to_block + site linkage on commit
   Blocked until A merges SET-P5-001

2) ECON-OG-1-A — actors.rs + extended DistrictMetrics in district.rs
   GrowthActorLayer; BuildingUsage; employment_demand, housing_deficit, freight_access, utility_service, civic_pressure
   Market layer NEVER commits (G-ACTOR-01)
   Fixture tests OK before A lands; wire to DistrictBook after

3) ECON-OG-1-B — pressure.rs + market.rs
   compute_district_pressure_system + compute_market_saturation_system
   priority = pressure × (1 - saturation) × transport × utilities
   LIB TEST REQUIRED: commercial_saturated — 3 shops at cap → 4th rejected

4) ECON-OG-1-C — witness construction_organic_growth_001
   Keys: market_saturation_active, employment_demand_wired, execute_via_pipeline true
   Extend construction/live_proof.rs only — no hand-edit JSON

5) PROC-OG-2-001 — GrowthProposal + GrowthReasonCode → ConstructionPlanQueue
   No Operational on enqueue; reason_codes not LLM prose
   Filter proposals against district style_rules

6) PROC-OG-3-001 — AutoBuildPolicy + growth_inspector + dashed proposal ghosts
   visual_authority.rs — distinct from player/parametric ghosts

7) INFRA-E2-004 — rail tool (infra tail when growth queue drained)

GROWTH INVARIANTS:
- State/player factories = GrowthActorLayer::State via CON-P2 (already green)
- Private infill = Growth layer → queue only
- Market = scoring only — never ConstructionPlanQueue writer

REGRESSION each PR:
cargo test -p proc_A_dine01 --lib organic_growth settlement market_saturation construction
python -m rust_engine_mcp.cli validate-report cargo --compress 3
python -m rust_engine_mcp.cli validate-report bevy -p proc_A_dine01 --compress 3
```

---

## Coordination

| Week | A | B |
|:---|:---|:---|
| Now | CON-P3 → **SET-P5-001** ASAP | **SET-P5-002** after A-001; start **ECON-OG-1-A** with fixtures |
| Next | **PROC-PG-1-001** | **ECON-OG-1-B/C** |
| Then | SET-P5-003 | **PROC-OG-2-001** → OG-3 |

**Safe parallel:** A SET-P5-001 + B ECON-OG-1-A types (fixtures) — merge integration when DistrictBook exists.

---

## Changelog

| v | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-06-02 | PLAN-ECON-GROWTH-ACTORS SIGNED; queue populated |
