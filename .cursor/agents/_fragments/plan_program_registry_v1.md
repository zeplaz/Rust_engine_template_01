# plan_program_registry_v1 — shared fragment (referenced by agents; owned by @plan-orchestrator)
# Updated 2026-07-03 (MIG-V1 green + deferral registry). One table = current program set + global pick order.
# Authority: src/dev/development_plan_index.md + plan_deferral_registry_v1.md + HANDOFF.md lease blocks.

## Shared map (read before planning/sweeping ANYTHING)

- `src/dev/codebase_index_v1.md` — coded module index (CO-*/SIM-*/RN-*/GU-*/CB-*…) + concepts K01–K24.
  Rule: cite entry codes and file:line from the index; re-derive only what the index doesn't cover.
- `src/dev/plan_deferral_registry_v1.md` — **DR-*** cross-program deferrals + `unblock_when` predicates.
- `tools/orchestrator/queues/defer_registry.json` — machine queue defer rows (orchestrator scripts).
- `debug_runs/mig_bevy_019/mig_v1_gate.json` — migration gate truth.

## Active programs (P-order; human P0 lease always wins)

| Pri | Program id | File (src/dev/) | Codes | One-line scope |
|:--|:--|:--|:--|:--|
| P0 | PLAN-BEVY-019-MIG-v1 | plan_bevy_019_migration_v1.md | MIG-A tail | **MIG-V1 DONE** · RTT/VFX operator verify · incremental MIG-A deep |
| P1 | PLAN-CLEANUP-v1 Phase 0 | plan_cleanup_v1.md | R/S/P/T/D | hygiene now; Phase 2+ → **DR-CLEANUP-P2** |
| P2 | PLAN-CITY-GRAMMAR-v1 | plan_city_grammar_upgrade_v1.md | CITY-G/C/P | block/town tier; **owns BSN product architecture** § BSN ASSEMBLY CHARTER |
| P2 | PLAN-BUILDING-QUALITY-v1 | plan_building_quality_v1.md | BQ-F/C/A/H/K/Q | building-level jank fix; integration hub for APSR |
| P2 | PLAN-APS-REFACTOR-v1 | plan_aps_refactor_v1.md | APSR-T/S/P/D/Q | APS services/state/panels; sequenced via BQ hub |
| P3 | PLAN-SCHEDULE-SYNC-v1 | plan_schedule_sync_v1.md | SCH-E/A/T/P/D | Wave 1 OK; Wave 2+ → **DR-SCHED-W2** |
| P1 | PLAN-GPU-TERRAIN-EXEC | plan_gpu_terrain_production_exec_001_v1.md | PERF-GPU | P0-C tilemap → **DR-MIG-TILEMAP**; use P0-C′ instanced |

## Cross-program ownership locks (do not double-pick)

- plan_cleanup S11/S1c → owned by CITY-G0a/G0b · SCH-P1 dormant plugins → cleanup Phase-0 lane
- BQ owns building-level quality · CITY owns block/town tier · APSR owns the tool surface
- BSN / `WorldAssetRoot` → **plan_city_grammar** § BSN ASSEMBLY CHARTER (**DR-CITY-C6-***) — MIG-A9 handoff complete
- GPU terrain P0-C blocked until **DR-MIG-TILEMAP** — do not bump `bevy_ecs_tilemap` Cargo without steward sign-off

## Verified facts agents must not re-litigate (2026-07-03 audits)

- **Bevy 0.19 on master** — MIG-V1 gate_pass true; do not re-pick MIG-G/M/R/V1 or shipped MIG-A core.
- Schedule: ambiguity detection OFF; 233 bare-fn `.after` anchors; strategic agents ignore pause (SCH-T1);
  verified global Update order lives in plan_schedule_sync + skill 07-repo-authority-map.
- Buildings: style-blind `prefer_stylepack_tier` (module_index.rs) · zero adjacency constraints ·
  roof bake floats 0.1m · no brick/wood/concrete roofs exist (kit holes).
- APS: 47 unguarded SuiteState mutation sites · stale buildings-panels on lane switch · IA/jobs/headless are GOOD (keep).
- **MIG-A9 BSN** — migration handoff **complete**; BSN product owner = `plan_city_grammar_upgrade_v1.md` § BSN ASSEMBLY CHARTER
- Tilemap: `bevy_ecs_tilemap` 0.18.1 only — **DR-MIG-TILEMAP** blocks default adapter enable.
