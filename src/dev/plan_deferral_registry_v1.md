# Cross-program deferral registry `v1`

**Authority chain (read in order):**

1. [`debug_runs/mig_bevy_019/mig_v1_gate.json`](../debug_runs/mig_bevy_019/mig_v1_gate.json) — migration gate + `deferral_taxonomy` + `pick_now`
2. [`plan_bevy_019_migration_v1.md`](plan_bevy_019_migration_v1.md) § **AGENT ROUTING** — MIG-* pick rules
3. **This file** — cross-program deferrals (city, terrain, cleanup, schedule, MCP, …)
4. [`tools/orchestrator/queues/defer_registry.json`](../tools/orchestrator/queues/defer_registry.json) — machine queue defer rows (orchestrator scripts)

**Rule for every plan author:** Any slice marked DEFER / BLOCKED / POST-MIG / “after MIG-V1” MUST cite a **DR-*** row here (or add one) with `unblock_when` predicates. Agents promote a defer row to active only when **all** predicates in `unblock_when` are true.

**Rule for agents:** Before picking a deferred slice, run the unblock check below — do not assume “still deferred” from stale plan prose.

---

## Unblock check (copy per session)

```text
1. Read mig_v1_gate.json → gate_pass, deferral_taxonomy, pick_now
2. Read this file → find DR-* id cited by your plan slice
3. Evaluate unblock_when predicates (witness paths, crate versions, steward sign-off)
4. If ALL true → pick slice · refresh witness · update defer_registry status
5. If ANY false → pick from pick_now / active queue instead
```

---

## Taxonomy (shared with MIG plan)

| Class | Meaning | Agent action |
|:---|:---|:---|
| **ecosystem_blocked** | Upstream crate/API does not exist yet | Wait · monitor crates.io · do not bump Cargo |
| **product_blocked** | No product charter / zero code hooks yet | Wait · or pick unblocked partial (see row) |
| **closed_wont_adopt** | Intentional — will not implement | Never pick |
| **closed_handoff** | Migration slice complete — product plan owns continuation | Never pick from MIG queue; route to owner plan |
| **incremental_ok** | Large slice; scaffolds done | Pick one bounded sub-slice + regression |
| **verify** | Code landed; operator witness stale | Run command in `unblock_when` |
| **program_gate** | Cross-program sequencing (file ownership) | Pick when conflict matrix row clear |

---

## DR-* rows (cross-program)

### Migration / Bevy 0.19

| ID | Item | Class | unblock_when | Owner plan |
|:---|:---|:---|:---|:---|
| **DR-MIG-V1** | Bevy 0.18 → 0.19 mechanical + render port | — | **DONE** — `mig_v1_gate.json` → `gate_pass: true` | `plan_bevy_019_migration_v1.md` |
| **DR-MIG-TILEMAP** | `bevy_ecs_tilemap` 0.19 + `bevy_tilemap_adapter` default | ecosystem_blocked | `crates.io/bevy_ecs_tilemap` publishes **0.19.x** + compat row in `compat_matrix_g1.json` + steward sign-off | `plan_gpu_terrain_production_exec_001_v1.md` PERF-GPU-TERRAIN P0-C |
| **DR-MIG-A8** | SettingsPlugin replaces shell persistence | closed_wont_adopt | Never — audit only (`mig_a_a8_settings_coexistence_audit.json`) | `plan_bevy_019_migration_v1.md` A8 |
| **DR-MIG-A9** | BSN / WorldAssetRoot | closed_handoff | **DONE** — pilot + witness; **product owner = plan_city_grammar § BSN ASSEMBLY CHARTER** | `plan_city_grammar_upgrade_v1.md` |
| **DR-MIG-A15** | Morph targets / MorphWeights | product_blocked | `grep MorphWeights src/` non-zero **OR** procedural skinning plan signed | `plan_bevy_019_migration_v1.md` A15 |
| **DR-MIG-A11-DEEP** | Stock depth prepass merge | **POST-MIG perf** (not migration) | `plan_gpu_terrain_production_exec_001_v1.md` — migration audit closed |
| **DR-MIG-A13-DEEP** | GPU light clustering replaces CPU fire extract | **POST-MIG perf** (not migration) | Fire/perf lane — CPU path shipped for 0.19 |
| **DR-MIG-A17-DEEP** | Stock mesh collection in RN-* draw | **POST-MIG perf** (not migration) | `terrain_instanced_draw` shipped · deep merge = perf only |
| **DR-RTT-VR16** | Sparks/particles operator verify | verify | `cargo run --release -- --test vfx` on display **AND** refresh `stage5_full_app_live.json` / triage witness | `visual_run_blockers.md` VR-16 |

### City / grammar

| ID | Item | Class | unblock_when | Owner plan |
|:---|:---|:---|:---|:---|
| **DR-CITY-C6-VIS** | CITY-C6 visual (ECS/recipe street furniture + corridors) | — | **CLOSED** 2026-07-03 — C6 BSN + rollout witnesses green | `city_c6_bsn_001_live.json` |
| **DR-CITY-C6-BSN** | BSN scene assembly (block/building composed visuals) | — | **PILOT CLOSED** — `block_street_visual.rs`; expansion = BQ-K / designer charter | `city_c6_bsn_001_live.json` |
| **DR-CITY-P1** | StaticTransformOptimizations at block scale | — | **CLOSED** 2026-07-03 | `city_p1_001_live.json` |
| **DR-CITY-P2** | Block LOD impostor (C8 GLB) | — | **CLOSED** 2026-07-03 | `city_p2_001_live.json` |

### Program gates (were “frozen during MIG”)

| ID | Item | Class | unblock_when | Owner plan |
|:---|:---|:---|:---|:---|
| **DR-SCHED-W2** | Schedule sync Wave 2+ (fire authority SCH-E2…) | program_gate | Wave 1 gate recorded **AND** no file conflict with RTT lane per HANDOFF matrix | `plan_schedule_sync_v1.md` |
| **DR-CLEANUP-P2** | Cleanup Phase 2+ authority/perf splits | program_gate | Phase 0 started **OR** PERF baseline captured **AND** steward sign-off on slice | `plan_cleanup_v1.md` |
| **DR-GPU-TERRAIN-P0C** | Enable tilemap adapter as sim default | ecosystem_blocked | **DR-MIG-TILEMAP** all predicates **OR** P0-C′ instanced path signed without tilemap | `plan_gpu_terrain_production_exec_001_v1.md` |

---

## Plan integration (required footer)

Every active plan under `src/dev/plan_*.md` SHOULD include:

```markdown
**Deferrals:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — cite DR-* ids in DEFER/BLOCKED rows.
**Migration truth:** [`mig_v1_gate.json`](../debug_runs/mig_bevy_019/mig_v1_gate.json)
```

When a defer **closes**, update: this file (status column) · `defer_registry.json` · citing plan · HANDOFF lease block.

---

## Stale phrase guide (do not use in new plan text)

| Stale | Replace with |
|:---|:---|
| “0.18-safe until M1 bump” | “Bevy **0.19** on master (MIG-V1 green)” |
| “after MIG-V1 merge” | “**UNBLOCKED** if DR-MIG-V1 done; else see DR-* row” |
| “defer while MIG in flight” | “see DR-SCHED-W2 / DR-CLEANUP-P2 — MIG-V1 **done**” |
| “POST-MIG BSN” / “MIG-A9 blocked” | **DR-CITY-C6-BSN** (product) · MIG-A9 = **closed_handoff** |
| “pick MIG-A9” | Route to **plan_city_grammar_upgrade_v1** § BSN ASSEMBLY CHARTER |
| “enable bevy_tilemap in default” | “blocked — **DR-MIG-TILEMAP** until crates.io 0.19” |
