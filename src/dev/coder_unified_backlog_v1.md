# Coder unified backlog `v1` — dual track, no toe-stepping

| Field | Value |
|:---|:---|
| **ID** | **CODER-UNIFIED-BACKLOG-001** |
| **Date** | 2026-06-02 |
| **Workload queue** | [`fleet_coder_workload_queue_20260602_v1.md`](fleet_coder_workload_queue_20260602_v1.md) |
| **Snapshot** | [`fleet_snapshot_20260602_v3.md`](fleet_snapshot_20260602_v3.md) |
| **Alignment** | [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) |
| **Rule** | **Both programs run.** Pick next `ready` row for your lane (A/B). **≤3 files per PR.** Never edit another lane's files in the same PR. |

**Pull policy:** Finish `coder_* .active[]` first, then top of your column below. Mark done in [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) and move to `done_*`.

---

## Owner split (file territories)

| Coder | Owns (default) | Do not touch without handoff |
|:---|:---|:---|
| **A** | `src/construction/site_stage*.rs`, `strategic/site/components.rs` (P2 progress), `src/infrastructure/profiles/`, `src/infrastructure/transport/spline.rs`, `src/infrastructure/transport/graph.rs`, `src/gui/editor/` authoring, `src/render/` overlay, `src/systems/navigation/`, `construction/live_proof.rs`, `src/engine/` test harness boundaries | `placement_scaling.rs`, `parametric_commit.rs`, `economy/logistics/` (B owns) |
| **B** | `placement_scaling.rs`, `build_validation.rs`, `parametric_commit.rs`, `src/construction/site_stage_tick.rs`, `strategic/site/systems.rs` commit, `src/infrastructure/utility/`, `src/infrastructure/transport/junction.rs`, `snapshot.rs` hydrate, `src/io/save/`, `economy/logistics/`, `economy/activation/` | `src/gui/editor/` spline tool (A owns) |

---

## Coder A — ordered backlog

| # | ID | Program | Files hint | Blocked by |
|:---:|:---|:---|:---|:---|
| 1 | CON-P2-001 | Construction P2 | `SiteStageProgress`, commit → Planned | — |
| 2 | CON-P2-003 | Construction P2 | witness `construction_site_stage_pipeline_001` | CON-P2-001 |
| 3 | INFRA-E0-001 | Infra E0 | `src/infrastructure/profiles/`, assets RON | — |
| 4 | INFRA-E0-003 | Infra E0 | remove legacy transport stubs | — |
| 5 | INFRA-E1-001 | Infra E1 | `TransportGraph` resource | INFRA-E0-001 |
| 6 | INFRA-E1-002 | Infra E1 | spline subdivide | INFRA-E1-001 |
| 7 | INFRA-E2-001 | Infra E2 | corridor authoring tool | INFRA-E1-002 |
| 8 | INFRA-E2-002 | Infra E2 | map editor bake v2 | INFRA-E2-001 |
| 9 | INFRA-E3-003 | Infra E3 | transport_network_live witness | INFRA-E1-004 (B) |
| 10 | INFRA-E4-002 | Infra E4 | utility flow hook | INFRA-E4-001 (B) |
| 11 | INFRA-E5-002 | Infra E5 | logistics graph-only paths | INFRA-E1-004, CON-P7 gate |
| 12 | INFRA-E6-001 | Infra E6 | material tags from profiles | INFRA-E0-001 |
| 13 | INFRA-E6-002 | Infra E6 | nav on TransportTopology | INFRA-E1-001 |
| 14 | INFRA-E6-004 | Infra E6 | debug overlays | INFRA-E6-003 (B) |
| 15 | CON-P3-S1-S3 | Construction P3 | scaling audit (A half) | CON-P2-003 |
| 16 | CONTAIN-MINIMAP-001 | Fleet P2 | minimap shim retire | — |
| 17 | STAB-CI-001 | Fleet P2 | `-D warnings` CI | — |
| 18 | DEHACK-ENV-002 | Fleet P2 | env sunset PRs | — |
| 19 | INFRA-E2-004 | Infra E2 | rail tool | INFRA-E2-001 |
| 20 | PLAN-SETTLEMENT-HIERARCHY-005 | Construction P5 | Town book schema (A drafts) | CON-P2-003 |

---

## Coder B — ordered backlog

| # | ID | Program | Files hint | Blocked by |
|:---:|:---|:---|:---|:---|
| 1 | CON-P2-002 | Construction P2 | `site_stage_tick.rs`, advance system | CON-P2-001 (A) |
| 2 | INFRA-E0-002 | Infra E0 | deprecate `TerrainFeatures.road/track` | — |
| 3 | INFRA-E1-003 | Infra E1 | junction detection | INFRA-E1-001 (A) |
| 4 | INFRA-E1-004 | Infra E1 | snapshot ↔ graph round-trip | INFRA-E1-001 (A) |
| 5 | INFRA-E2-003 | Infra E2 | road/rail → TransportEdgeRecord | CON-P2-001, INFRA-E1-004 |
| 6 | INFRA-E3-001 | Infra E3 | R8 schema v2 | INFRA-E1-004 |
| 7 | INFRA-E3-002 | Infra E3 | hybrid save slice | INFRA-E3-001 |
| 8 | INFRA-E4-001 | Infra E4 | UtilityNetworkSnapshot | INFRA-E0-001 |
| 9 | INFRA-E4-003 | Infra E4 | UtilityConnection buildings | INFRA-E4-001 |
| 10 | INFRA-E4-004 | Infra E4 | utility authoring | INFRA-E4-001 |
| 11 | INFRA-E5-001 | Infra E5 | settlement attach | CON Phase 5 schema |
| 12 | INFRA-E5-003 | Infra E5 | play scenario graph seed | INFRA-E5-002 (A) |
| 13 | INFRA-E6-003 | Infra E6 | network overlay render | INFRA-E1-001 |
| 14 | CON-P3-S4-S6 | Construction P3 | scaling audit (B half) | CON-P2-002 |
| 15 | PLAY-TRUTH-001-TAIL | Fleet P2 | play scenario seeds | — |
| 16 | DEHACK-WSS-002 | Fleet P2 | WSS slab authority | — |
| 17 | FEAT-WSS-HYDRO-READ-001 | Fleet P2 | hydrology read UX | — |
| 18 | CONSTRUCTION-R4-PRODUCT-001 | Construction R4 | one product slice | designer PASS |
| 19 | PLAN-CONSTRUCTION-SCALING-AUDIT-003 | Construction P3 | audit doc + witness | CON-P3-* |
| 20 | SET-P5-002 | Construction P5 | BlockBook + site linkage | SET-P5-001 (A) |
| 21 | ECON-OG-1-A | Econ growth | `actors.rs` + extended metrics | SET-P5-001 |
| 22 | ECON-OG-1-B | Econ growth | `pressure.rs` + `market.rs` | ECON-OG-1-A, PG-1 partial |
| 23 | ECON-OG-1-C | Econ growth | organic growth witness | ECON-OG-1-B |
| 24 | PROC-OG-2-001 | Proc/growth P6 | proposals → queue | ECON-OG-1-C |
| 25 | PROC-OG-3-001 | Proc/growth P6 | policy + approve UI | PROC-OG-2-001 |

---

## Horizon — procedural + organic (after P2 green)

**Index:** [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) · **Hub:** [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md)

| # | ID | Owner | Program | Blocked by |
|:---:|:---|:---|:---|:---|
| H1 | PROC-PG-1-001 | A | PG-1 archetype + StylePack RON | CON-P2-003 |
| H2 | PROC-PG-2-001 | A | PG-2 footprint grid + greybox extract | PROC-PG-1-001, DESIGN-PROC-MODULE-KIT-001 |
| H3 | PROC-PG-3-001 | B | PG-3 commit bridge | PROC-PG-2-001 |
| H4 | PROC-PG-4-001 | A | PG-4 grammar (optional) | PROC-PG-3-001 |
| H5 | PROC-OG-4-001 | A | OG-4 Town rollup | PLAN-SETTLEMENT-HIERARCHY-005 |

**Designer parallel (not in coder columns):**

| ID | When |
|:---|:---|
| **DESIGN-PROC-MODULE-KIT-001** | Step **4a** — parallel with **PROC-PG-1-001** (greybox OK) |
| **DESIGN-ORGANIC-GROWTH-UX-001** | Step **6b** — parallel with **PROC-OG-1-001** stub |

---

## Parallel work safe pairs (same week)

| Coder A | Coder B | Why safe |
|:---|:---|:---|
| CON-P2-001 | CON-P2-002 | A components + witness prep; B tick system |
| INFRA-E0-001 | INFRA-E0-002 | profiles vs terrain deprecation |
| INFRA-E1-002 | INFRA-E1-003 | spline vs junction |
| INFRA-E2-001 | INFRA-E4-001 | transport editor vs utility types |
| CONTAIN-MINIMAP-001 | PLAY-TRUTH-001-TAIL | disjoint modules |
| CON-P2-001 | CON-P2-002 | site progress vs tick (see construction exec) |
| PROC-PG-1-001 | DESIGN-PROC-MODULE-KIT-001 | data vs art kit — disjoint files |
| PROC-OG-1-001 | INFRA-E5-001 | settlement attach vs district metrics — coordinate ids only |

---

## Current drain (2026-06-02 reconcile)

**Coder A:** CON-P3-S1..WIT → INFRA-E0-003 + E1/E2/E3/E4/E5/E6 column → PROC-PG-2-TAIL → PROC-OG-4 → PT-5-002.

**Coder B:** CON-PARAM-PARTIAL-ALPHA → FIX-PROC-TEST-REGRESS → INFRA-E4/E5 tails → PROC-OG-UX-WIRE.

SET-P5 + ECON-OG + PROC-OG **closed on disk** — see `done_2026_06_02` in queue JSON.

---

## Changelog

| Version | Date | Notes |
| 1.1.0 | 2026-06-02 | Reconcile after large coder return; link workload queue v1 |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Dual-track backlog; maps construction 1→11 + infra E0–E6 + fleet P2 |
| v1.1.0 | 2026-06-02 | Procedural/organic horizon rows + designer parallel pairs |
