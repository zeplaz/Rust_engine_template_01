# Designer on-call absorption `v1`

| Field | Value |
|:---|:---|
| **Doc ID** | **DESIGNER-ONCALL-ABSORPTION-001** |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Mode** | On-call — **no** standing active queue |
| **Closed long-run** | P0–P5 in registry v2.3+ — **do not re-queue** |
| **No Rust** | Review / signoff / copy only |

---

## Status

| Lane | State |
|:---|:---|
| Six-phase proc/growth (P0–P5) | **CLOSED** — see [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| `designer_active_queue.json` `active` | `[]` |
| Implementation review | **On notification** from `@coder` |
| Optional tails | **Product-triggered only** |
| **PG-3 live W3** | **Pass** — [`pg3_w3_tactical_review_live.json`](../debug_runs/pg3_w3_tactical_review_live.json) + runbook [`pg3_w3_live_tactical_review_runbook_v1.md`](pg3_w3_live_tactical_review_runbook_v1.md) |

---

## PG-3 live tactical review (15 min — when spawn visible)

**Trigger:** `@coder` notifies that commit → `ProceduralBuildingRequest` → procedural assembly is **player-visible** in tactical view (not lib witness only).

**Rubric:** [`procedural_assembly_pg2_signoff.yaml`](../../debug_runs/art_pipeline/procedural_assembly_pg2_signoff.yaml) → `w3_live_tactical_review` + charter § PG-2 witness sign-off rubric (W2–W5).

| Step | Action |
|:---|:---|
| 1 | Same footprint for both packs — match witness (4×2) or rowhouse pilot (4×3×2) |
| 2 | Spawn **style_victorian** and **style_industrial_west** side-by-side at tactical zoom |
| 3 | Without labels: wall family, roof profile, door width readable? |
| 4 | Confirm no smoke/greybox; missing slots → gap not wrong mesh |
| 5 | Fill `w3_live_tactical_review.status`, screenshots, `reviewed_at`; revise `proceed_player_visible` only if live read fails |

**Desk sign-off (MCP-DUX-PG2-002) already `yes`** — unblocks PG-3 coder work; this live pass confirms or revokes before PG-4 / production swap.

---

## Implementation review (when @coder notifies)

Record as `IMPLEMENTATION-REVIEW-{slice}-{date}` in registry — **never** reopen closed design IDs.

| Coder slice | PASS design doc | Review for |
|:---|:---|:---|
| **CON-P2-*** | [`design_construction_site_stage_read_v1.md`](design_construction_site_stage_read_v1.md) | Phase labels, Clearing substeps, minimap icons, no instant Operational |
| **CON-P3-S*** | [`design_construction_scaling_read_v1.md`](design_construction_scaling_read_v1.md) | Overlap badges, S1–S6, partial-alpha |
| **INFRA-E6-003/004** | [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) | Colors, legend, PLAY-01 sim chrome |
| **PROC-PG-2-001** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) | Module IDs, greybox LOD0, W/D/C grammar |
| **PROC-OG-3-001** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) | **Dashed** proposals; approve policy; no zone→built |
| Settlement UI | [`design_settlement_hierarchy_read_v1.md`](design_settlement_hierarchy_read_v1.md) | District picker ≠ zone tool |

### Verdict rubric

| Verdict | When |
|:---|:---|
| **PASS** | Matches design doc; witness green |
| **PASS (qualified)** | Minor copy drift; file follow-up noted |
| **REVISE** | Violates invariant — cite doc § + required fix |

---

## Optional tails (trigger matrix)

| ID | Trigger | Action |
|:---|:---|:---|
| **DESIGN-HANABI-H-A2-PROD-001** | `hanabi_l3` on **default** binary chartered + merged | Production preset disposition doc |
| **DESIGN-PR4-RETIRE-UX smoke** | `hybrid_ecs_smoke_authoritative == true` | Extend pending rows only |
| **DESIGN-S7B-M4-PLAY-READ-001** | `@coder B` requests enqueue UX | Short read doc |
| **DESIGN-PROC-ART-ACCEPTANCE-001** | Art assets land in `assets/meshes/buildings/modules/` | [`design_procedural_art_acceptance_v1.md`](design_procedural_art_acceptance_v1.md) |

**2026-06-02 disk:** smoke auth **false** → PR4 retire upgraded to **PASS** (see [`wss_pr4_retire_cutover_ux_v1.md`](wss_pr4_retire_cutover_ux_v1.md) v1.1). Hanabi: `hanabi_l3_plugin_wired: false` → **HOLD**.

---

## Operator / product (not designer active)

| Lane | Owner |
|:---|:---|
| **G-PLAY-01** execution | `@operator` — [`play_scenario_acceptance_runbook_v1.md`](play_scenario_acceptance_runbook_v1.md) |
| **DESIGN-CONSTRUCTION-R4-PRODUCT-001** | **Planner** product board — not designer charter |

---

## Hard rules (all on-call work)

1. **10 modules per category** — not 200 buildings.
2. Growth **proposals** only — never instant zone → built.
3. Proposal ghosts **dashed**; player/parametric ghosts **solid**.
4. Bump `designer_signoff_registry.json` `_meta.version` on each new PASS.
5. **Do not** re-queue: DESIGN-PROC-MODULE-KIT-001, DESIGN-ORGANIC-GROWTH-UX-001, DESIGN-CONSTRUCTION-STAGE-READ-001, DESIGN-CONSTRUCTION-SCALING-READ-001, DESIGN-INFRA-NETWORK-OVERLAY-001, DESIGN-SETTLEMENT-HIERARCHY-READ-001.
