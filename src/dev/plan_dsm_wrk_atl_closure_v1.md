# PLAN-DSM-WRK-ATL-001 — Track A node closure criteria `v1`

| Field | Value |
|:---|:---|
| **Slice ID** | **PLAN-DSM-WRK-ATL-001** |
| **Chain** | MAT★ → APS★ → SNAP★ → WRK★ → ATL★ → RT★ (closed 2026-06-08 · $ref:debug_runs/dsm_signoff_001_live.json) |
| **Parent** | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) · [`plan_mcp_productivity_chain_v1.md`](plan_mcp_productivity_chain_v1.md) |
| **Status** | **SIGNED** |
| **Date** | 2026-06-03 |

**Rule:** This doc names **witness keys + PASS table** only — does not duplicate MCP bpy specs or Tk layout.

---

## DSM snapshot (v19 → v20 signoff 2026-06-08)

```text
MAT★ → APS★ → SNAP★ → WRK★ → ATL★ → RT★
```

| Node | v20 | Close to ★ when |
|:---|:---:|:---|
| **MAT★** | profiles + category tree on disk | maintain |
| **APS★** | UX audit PASS; Tk phases 1–9 largely done | maintain |
| **SNAP★** | grammar iter E2E green | maintain |
| **WRK★** | `build_worker_001_live.json` green | maintain |
| **ATL★** | **SIGNED 2026-06-08** | `atl_sign_001` + production v2 preview + legend + atlas brief |
| **RT★** | **SIGNED 2026-06-08** | `rt_registry_001` lookup stamp + `rt_eng_001` + runtime_lookup_brief |

**Signoff witness:** [`dsm_signoff_001_live.json`](../debug_runs/dsm_signoff_001_live.json) · tensor [`master_chain_tensor_v1.json`](../../tools/orchestrator/queues/master_chain_tensor_v1.json) v1.1.0

---

## WRK★ — closure table

| Witness | Key | PASS |
|:---|:---|:---:|
| [`build_worker_001_live.json`](../debug_runs/build_worker_001_live.json) | `ok` | `true` |
| | `materials.ok` | `true` |
| | `authority` | contains `snapshot material_profile` |
| | `ship_policy` | rejects headless minimum as ship keyframe |
| APS UI | worker status line | plain sentence: `Build done — materials applied from snapshot` |
| MCP | `snapshot_digest` | lists `material_profiles.missing: 0` |

**Owner:** @coder-mcp (APS status text) + maintain BUILD-WORKER-001 witness on regression.

**Does not require:** Track B manual keyframe ship.

---

## ATL○ → ATL★ — closure table

| Witness | Key | PASS |
|:---|:---|:---:|
| [`aps_atlas_preview_002_live.json`](../debug_runs/aps_atlas_preview_002_live.json) | `uv_grid_overlay` | `true` |
| | `plain_language[]` | non-empty on fail |
| | `green` | `true` on **production** atlas folder (schema v2 + facings 4/8) |
| [`aps_atlas_legend_001_live.json`](../debug_runs/aps_atlas_legend_001_live.json) | legend wired | after APS-ATLAS-LEGEND-001-IMPL |
| MCP | `atlas_meta_brief` | returns grid + missing lookups ≤40 lines — spec $ref:docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md§3 · review $ref:docs/archive/2026-06-src-dev/plans/plan_p1_atl_closure_review_v1.md |
| APS UI | validate atlas meta | inline FAIL text + color (not modal) |

**Current blockers on disk:** pilot folder still schema v1 — `green: false` is **honest**. ATL★ closes on **production** atlas path, not pilot greybox folder.

**Owner:** @coder-mcp Phases: legend impl, `atlas_meta_brief`, APS-UX-NONBLOCK inline QC.

---

## SNAP★ — paired closure (same week)

| Item | PASS |
|:---|:---:|
| `validate_p0_gate_plain` MCP + APS inline | shipped per [`aps_validator_plain_signoff_v1.md`](aps_validator_plain_signoff_v1.md) |
| `grammar_iter_001_e2e_live.json` | `green: true` (maintain) |

---

## RT○ — defer criteria

Promote RT★ when:

1. WRK★ + ATL★ signed in this table — **DONE 2026-06-08**
2. `tile-atlas-register` witness includes lookup table stamp — **DONE** $ref:debug_runs/rt_registry_001_live.json
3. Engine registry load path — **DONE** $ref:debug_runs/procedural_tiles_runtime_live.json `rt_eng_001.green`

---

## Orchestrator paste

```text
DSM closure refs: plan_dsm_wrk_atl_closure_v1.md

AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL★ ⇢ RT★ (signed 2026-06-08)

WRK★ — maintain build_worker_001_live.json
ATL★ — maintain atl_sign_001 + production atlas witnesses
RT★ — maintain rt_registry_001 + rt_eng_001

Do not un-defer Track B warehouse without operator sign-off.
```
