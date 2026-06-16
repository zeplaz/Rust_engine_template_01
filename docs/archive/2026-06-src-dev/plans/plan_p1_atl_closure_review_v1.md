# PLAN-P1-ATL-REVIEW-001 — P1 plan vs ATL closure (planner maintenance) `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-P1-ATL-REVIEW-001** |
| **Trigger** | @planner maintenance after ⟨MCP-PRODUCTIVITY-P1-PLAN⟩ landed |
| **P1 plan** | $ref:docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md |
| **ATL authority** | $ref:src/dev/plan_dsm_wrk_atl_closure_v1.md§ATL○→ATL★ |
| **Legend design** | $ref:docs/archive/2026-06-src-dev/plans/design_aps_atlas_preview_legend_v1.md |
| **Planner** | **SIGNED** — maintenance only |
| **Date** | 2026-06-07 |

**Orient:** `agent_doc_touch(prompts/llm_agent_brief.md, intent=orient)` — ledger `debug_runs/agent_ops/doc_reads.jsonl`

---

## Verdict

| Area | φ | Note |
|:---|:---:|:---|
| P1 §3 `atlas_meta_brief` vs ATL closure table | 🟡 | **Aligned** on keys — 3 gaps below |
| P1 §2 `tile_spine_run` vs WRK★ | 🟢 | `assembly_build` step → `build_worker_001_live.json`; honest bake matches `ship_policy` |
| P1 §1 grammar loop vs SNAP★ | 🟢 | No planner delta — maintain shipped ritual |
| ATL★ gate | ○ | Still blocked on production atlas + legend impl — P1 does not claim ATL★ |

**Planner:** approve P1 for @coder-mcp implement with **§3 amendments** in witness contract.

---

## Key mapping — ATL closure ↔ P1 §3

| ATL closure row | P1 §3 field | PASS when |
|:---|:---|:---|
| `atlas_meta_brief` ≤40 lines | whole response | `ok` + `artist_messages[]` on fail |
| `uv_grid_overlay` (APS witness) | `uv_grid_summary` | MCP brief **references** grid dims; overlay stays Tk — not duplicated in MCP |
| `plain_language[]` on fail | `artist_messages[].sentence` | non-empty when `ok: false` |
| `green` on **production** folder | `ok` + `atlas_meta_schema` | `ok: true` only when schema **v2** + facings 4/8 on production path |
| `aps_atlas_legend_001` wired | — | **Gap** — separate ⟨APS-ATLAS-LEGEND-001-IMPL⟩; MCP brief must not replace legend |

---

## Gaps (planner amendments — @coder-mcp)

| # | Gap | Amendment |
|:---:|:---|:---|
| G1 | P1 example uses `atlas_meta_schema: "v1"` | Witness fixture must include **v1 fail** + **v2 pass** paths per closure §60 |
| G2 | `missing_lookups` vs legend cell ids | Map to legend plain-error codes in $ref:docs/archive/2026-06-src-dev/plans/design_aps_atlas_preview_legend_v1.md — add `legend_code` optional field |
| G3 | `tile_spine_run` `atlas_validate` cites `tile_promotion_honest_check` | **PLANNED** in P2 per parent chain — spine step may stub honest check with `warn` until MCP-HONEST-BAKE ships; document in step result |
| G4 | Pilot folder `tile_warehouse_industrial_west_pilot_v1` | `atlas_meta_brief` default test → pilot **expect `ok: false`**; production folder for ATL★ green |

---

## WRK★ cross-check (P1 §2)

| WRK closure key | P1 spine step | φ |
|:---|:---|:---:|
| `build_worker_001_live.json` `ok` | `assembly_build` witness_path | 🟢 |
| `ship_policy` rejects headless minimum | `honest_bake: true` + `ship: false` default | 🟢 |
| `snapshot_digest` materials | `p0_gate` + digest optional step | 🟢 |

No planner wait on spine impl — WRK★ already signed; spine is **agent ergonomics**, not WRK gate.

---

## SNAP★ cross-check (P1 §1)

| SNAP row | P1 | φ |
|:---|:---|:---:|
| `validate_p0_gate_plain` shipped | in grammar ritual + spine `p0_gate` | 🟢 |
| `grammar_iter_001_e2e_live.json` | not in P1 scope — maintain | 🟢 |

---

## ⟨BP:SHARE⟩ marker joint

```text
mirror: "P1 plan landed; planner ATL review signed with G1–G4 amendments"
scan: "BLANG:DOC orient llm_agent_brief · dim 🟡🟢○"
joint: "@coder-mcp — atlas_meta_brief witness: include v1 pilot fail + v2 production pass fixtures?"
joint: "@designer-mcp — artist_messages tone matches aps_validator_plain + legend 6 mapped codes?"
delta_wf: ΔWF→@coder-mcp ⟨MCP-SPINE-CHAIN-001⟩ then ⟨MCP-ATLAS-BRIEF-001⟩ per $ref:docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md§6
why: "ATL★ still ○ — P1 brief is necessary not sufficient"
```

---

## Planner Q+ state

| Queue | ready | Pick |
|:---|:---:|:---|
| `planner_active_queue.json` | PLAN-TERRITORY-MATRIX-002 (A2) | **defer** — not blocking ATL/P1 |
| grammar | 0 planner rows | idle OK |

**EXIT:** `planner idle — drain is @coder-mcp P1 impl + operator G-PLAY`

---

## Changelog

| Ver | Date |
|:---|:---|
| v1.0.0 | 2026-06-07 |
