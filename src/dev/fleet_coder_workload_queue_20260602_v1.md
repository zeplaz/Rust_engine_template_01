# Fleet coder workload queue — 2026-06-02 `v1`

| Field | Value |
|:---|:---|
| **ID** | **FLEET-CODER-WORKLOAD-20260602** |
| **Snapshot** | [`fleet_snapshot_20260602_v3.md`](fleet_snapshot_20260602_v3.md) |
| **Machine queue** | [`coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Territories** | [`coder_unified_backlog_v1.md`](coder_unified_backlog_v1.md) |
| **Long-run prompts** | [`fleet_longrun_prompts_20260602_v1.md`](fleet_longrun_prompts_20260602_v1.md) `v1.2` |
| **Rule** | ≤3 files per PR · witness JSON wins · update queue row on done |

---

## Drain policy

1. Finish your lane `active[]` **in priority order** (lower number first).
2. Do not re-queue rows in `done_2026_06_02` unless witness regresses.
3. **CON-P3-S1–S3** blocks qualified close of Phase 3 even if rollup is green today.
4. MCP: **consume** `validate-report` only — do not edit `tools/mcp/` ([`agent_mcp_consumer_guide_v1.md`](agent_mcp_consumer_guide_v1.md)).

---

## Coder A — active workload (18 rows)

| P | ID | Program | Plan / exit |
|:---:|:---|:---|:---|
| 1 | **CON-P3-S1** | Construction P3 | `scaling_audit_s1_preset_matrix_match_green` — [`plan_construction_scaling_audit_exec_003_v1.md`](plan_construction_scaling_audit_exec_003_v1.md) |
| 2 | **CON-P3-S2** | Construction P3 | `scaling_audit_s2_occupied_tiles_wired_green` |
| 3 | **CON-P3-S3** | Construction P3 | `scaling_audit_s3_blocked_disables_commit_green` |
| 4 | **CON-P3-WIT** | Construction P3 | `construction_scaling_audit_001` includes S1–S3 fields; refresh `construction_stage_live.json` |
| 5 | **INFRA-E0-003** | Infrastructure E0 | Remove/gate `legacy_transport_stubs` `Road`/`Rail` from default build |
| 6 | **INFRA-E1-001** | Infrastructure E1 | `TransportGraph` resource + plugin init (verify vs existing `graph.rs`) |
| 7 | **INFRA-E1-002** | Infrastructure E1 | Spline subdivide → edge records |
| 8 | **INFRA-E2-001** | Infrastructure E2 | Corridor spline authoring tool (`src/gui/editor/`) |
| 9 | **INFRA-E2-002** | Infrastructure E2 | Map editor bake v2 |
| 10 | **INFRA-E3-003** | Infrastructure E3 | `debug_runs/transport_network_live.json` witness |
| 11 | **INFRA-E4-002** | Infrastructure E4 | Utility flow hook |
| 12 | **INFRA-E5-002** | Infrastructure E5 | Logistics graph-only paths (after E1-004 — **done** on B) |
| 13 | **INFRA-E6-001** | Infrastructure E6 | Material tags from profiles |
| 14 | **INFRA-E6-002** | Infrastructure E6 | Nav on `TransportTopology` |
| 15 | **INFRA-E6-004** | Infrastructure E6 | Debug overlays (pair E6-003 B) |
| 16 | **PROC-PG-2-TAIL-001** | Procedural | Tier filter + `mesh_tier_used` witness; fix lod0 index regressions — [`construction_procedural_growth_index_v1.md`](construction_procedural_growth_index_v1.md) § PG-2 |
| 17 | **PROC-OG-4-001** | Organic | Town rollup metrics — after SET-P5 closed |
| 18 | **PT-5-002** | Tile prod | Fire frame tick from `SimStepStamp` — [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) |

**Horizon (after P≤5):** **PROC-PG-4-001** grammar · **PT-5-003** map dirty rect · **INFRA-E2-004** polish (B done — A owns editor UX only if gap).

---

## Coder B — active workload (14 rows)

| P | ID | Program | Plan / exit |
|:---:|:---|:---|:---|
| 1 | **CON-PARAM-PARTIAL-ALPHA-001** | Construction | `construction_parametric_placement_001.partial_alpha: true` + green rollup — [`design_construction_scaling_read_v1.md`](design_construction_scaling_read_v1.md) |
| 2 | **FIX-PROC-TEST-REGRESS-001** | Procedural | `cargo test -p proc_A_dine01 --lib construction::procedural` **0 failed** (module index alias, tile quarantine tests) |
| 3 | **INFRA-E4-003** | Infrastructure E4 | `UtilityConnection` on buildings |
| 4 | **INFRA-E4-004** | Infrastructure E4 | Utility authoring UX |
| 5 | **INFRA-E5-003** | Infrastructure E5 | Play scenario graph seed |
| 6 | **INFRA-E3-WIT-001** | Infrastructure E3 | Hybrid save + `transport_network_live.json` green |
| 7 | **PT-4-005** | Tile prod | Damage scalar → damaged_* variant keys |
| 8 | **PROC-OG-UX-WIRE-001** | Organic | Growth proposal approve / reject HUD — [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) **PASS** |
| 9 | **FIX-BQ128-WIT-001** | Construction | `bq128_apply_live_witness_refresh_green` — empty JSON parse |
| 10 | **FIX-S7P-MV-PROOF-001** | Dev proof | `s7p_grid_ux_and_construction_mv_001_bundle` IND-E02 cluster seed |
| 11 | **INFRA-E6-004** | Infrastructure E6 | Overlay depth / legend (if not duplicated on A) |
| 12 | **ECON-OG-SAVE-001** | Organic | Town/District/Block book save slice round-trip (extend SET-P5) |
| 13 | **PT-4-004** | Tile prod | Power + day/night inputs (stub constants `cfg(test)` OK with witness keys) |
| 14 | **HANABI-WIT-001** | Render | Optional: wire `hanabi_witness` into stage5 proof if product asks |

**Blocked on A:** none for rows 1–2; row 6 may need A **INFRA-E3-003** transport graph witness first.

---

## Parallel safe pairs (same week)

| A | B | Why |
|:---|:---|:---|
| CON-P3-S1–S3 | CON-PARAM-PARTIAL-ALPHA-001 | placement_scaling vs visual_authority — coordinate witness only |
| INFRA-E2-001 | INFRA-E4-003 | editor transport vs utility types |
| PROC-PG-2-TAIL-001 | FIX-PROC-TEST-REGRESS-001 | procedural/ disjoint files |
| PT-5-002 | PT-4-005 | fire tick vs damage resolver |

---

## Copy-paste session openers

### @coder A

```text
Drain coder_active_queue.json coder_a.active[] in order (v5.4.0).
READ: fleet_coder_workload_queue_20260602_v1.md · fleet_snapshot_20260602_v3.md
START: CON-P3-S1 → S2 → S3 → CON-P3-WIT
THEN: INFRA-E0-003 → E1/E2 column → PT-5-002
DO NOT: site_stage_tick (B); Operational on commit.
REGRESSION: cargo test -p proc_A_dine01 --lib construction
```

### @coder B

```text
Drain coder_active_queue.json coder_b.active[] in order (v5.4.0).
READ: fleet_coder_workload_queue_20260602_v1.md · fleet_snapshot_20260602_v3.md
START: CON-PARAM-PARTIAL-ALPHA-001 → FIX-PROC-TEST-REGRESS-001
THEN: INFRA-E4-003/004 → E5-003 → PROC-OG-UX-WIRE-001
REGRESSION: cargo test -p proc_A_dine01 --lib construction
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-06-02 | Initial large workload after SET-P5/OG/PG return |
