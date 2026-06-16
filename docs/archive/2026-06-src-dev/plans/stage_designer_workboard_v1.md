# Designer workboard — on-call

| Field | Value |
|:---|:---|
| **Version** | `3.4.0` |
| **Date** | 2026-06-03 |
| **Mode** | **On-call** — industrial/military tile lanes + PG-3 W3 **pass** |
| **Playbook** | [`designer_oncall_absorption_v1.md`](designer_oncall_absorption_v1.md) |
| **Registry** | [`designer_signoff_registry.json`](../../tools/orchestrator/queues/designer_signoff_registry.json) v2.5 |

**Rule:** design/spec/signoff only; no Rust. **Do not** re-queue P0–P5 long-run IDs.

---

## On-call — first applicable row

| Priority | Lane | Action | Status |
|:---:|:---|:---|:---|
| 0 | **PG-3 live W3** | Victorian vs `style_industrial_west` — witness [`pg3_w3_tactical_review_live.json`](../../debug_runs/pg3_w3_tactical_review_live.json) · captures [`w3_captures/`](../../debug_runs/art_pipeline/w3_captures/) | **Done** — `w3_live_tactical_review.status: pass` |
| 0 | **APS-UX-AUDIT-001** | Art Pipeline Suite Phase 0 gate | **Done** — lead **PASS** 2026-06-03 |
| 0b | **SIM-HUD-PRODUCT-001** | PLAY-01 + product HUD program close | **Done** — **PASS (full)** |
| 0b1 | **SIM-HUD-PRODUCT-CLOSE-001** | Rollup + sign-off v1.2 | **Done** — `sim_hud_product_close_001_live.json` |
| 0c | **DESIGN-WEATHER-PLAYER-READ-001** | Weather player read charter | **Done** — PASS |
| 0d | **EGUI-DEV-UX-001** | APS Bevy QC HUD V2 polish sign-off | **Done** — PASS |
| 0e | **DESIGN-WX-HUD-IMPL-001** | Weather HUD §Implementation → @coder C | **Done** — PASS |
| 1 | **Implementation review** | When `@coder` notifies PR — slice→doc map in playbook | **Idle** |
| 2 | **Absorption** | **DESIGN-PR4-RETIRE-UX-001** qualified→**PASS** (smoke witness) | **Done** 2026-06-02 |
| 3 | **Production pilot** | **MCP-PROD-ROWHOUSE-SIGNOFF** — rowhouse production G4 only ([`mcp_fleet_production_pilot_rowhouse_v1.md`](mcp_fleet_production_pilot_rowhouse_v1.md)) | **READY** (after coder-mcp TILE) |
| 4 | **Artist (hold)** | **DESIGN-PROC-ART-ACCEPTANCE-001** — full 50 modules | **HOLD** until rowhouse pilot closes |
| 5 | **Operator** | **G-PLAY-01** — `@operator` runs runbook | **Pending** |
| — | **Hold** | Hanabi H-A2 prod, S7B play read, R4 product board | See below |

---

## Long-run six phases — closed (do not reopen)

| P | ID | Deliverable | Verdict |
|:---:|:---|:---|:---|
| P0 | **DESIGN-PROC-MODULE-KIT-001** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) | **PASS** |
| P1 | **DESIGN-ORGANIC-GROWTH-UX-001** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) | **PASS** |
| P2 | **DESIGN-CONSTRUCTION-STAGE-READ-001** | [`design_construction_site_stage_read_v1.md`](design_construction_site_stage_read_v1.md) | **PASS** |
| P3 | **DESIGN-CONSTRUCTION-SCALING-READ-001** | [`design_construction_scaling_read_v1.md`](design_construction_scaling_read_v1.md) | **PASS** |
| P4 | **DESIGN-INFRA-NETWORK-OVERLAY-001** | [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) | **PASS** |
| P5 | **DESIGN-SETTLEMENT-HIERARCHY-READ-001** | [`design_settlement_hierarchy_read_v1.md`](design_settlement_hierarchy_read_v1.md) | **PASS** |

---

## Holds (product-triggered)

| ID | Trigger | Status |
|:---|:---|:---|
| **DESIGN-HANABI-H-A2-PROD-001** | `hanabi_l3` on default binary + merged | **HOLD** (`hanabi_l3_plugin_wired: false`) |
| **DESIGN-PR4-RETIRE-UX smoke tail** | `hybrid_ecs_smoke_authoritative == true` | **Closed** — absorbed to full PASS |
| **DESIGN-S7B-M4-PLAY-READ-001** | `@coder B` requests enqueue UX | **DEFER** |
| **DESIGN-CONSTRUCTION-R4-PRODUCT-001** | Planner product board | **Planner only** |

---

## Implementation review map (idle until notify)

| Slice | PASS doc |
|:---|:---|
| CON-P2-* | [`design_construction_site_stage_read_v1.md`](design_construction_site_stage_read_v1.md) |
| CON-P3-S* | [`design_construction_scaling_read_v1.md`](design_construction_scaling_read_v1.md) |
| INFRA-E6-003/004 | [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) |
| PROC-PG-2-001 | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) |
| PROC-OG-3-001 | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) |
| Settlement UI | [`design_settlement_hierarchy_read_v1.md`](design_settlement_hierarchy_read_v1.md) |

---

## Prior batches

Stability P1–P4, wave4/5/6, proc long-run — registry `signoffs` / queue `done`.
