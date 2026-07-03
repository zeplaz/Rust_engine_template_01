# Designer parallel workboard `v1` (while coders execute)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-PARALLEL-WAVE-001** |
| **Version** | `1.2.0` |
| **Date** | 2026-05-27 |
| **Status** | **P2 drained** — queue `active` empty |
| **Coder matrix** | [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md) |
| **Rule** | **No Rust** — mocks, PASS records, readability contracts |

**WSS-DESIGN-GATE-001** and **CONSTRUCTION-PARAM-DESIGN-001** are **done** — this board is **forward prep**, not re-litigation.

---

## P0 — unblocks coders in next 2–4 cycles

| ☐ | ID | Deliverable | Unblocks coder | When |
|:---:|:---|:---|:---|:---|
| ☑ | **DESIGN-CONTAMINATION-001** | [`wss_contamination_visual_language_v1.md`](wss_contamination_visual_language_v1.md) **PASS** | A-W2 atmos clipmap | 2026-05-26 |
| ☑ | **DESIGN-SMOKE-LAYER-AB-001** | [`wss_smoke_layer_a_b_visual_v1.md`](wss_smoke_layer_a_b_visual_v1.md) **PASS** | A-V3 smoke bridge | 2026-05-26 |
| ☑ | **DESIGN-PARAM-STAGING-POLISH-002** | [`construction_parametric_staging_ux_v2.md`](construction_parametric_staging_ux_v2.md) **PASS** | B-C4 staging panel | 2026-05-26 |
| ☑ | **DESIGN-PARAM-SCALE-HUD-001** | [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md) **PASS** | B-C5 economy readout | 2026-05-26 |

**Source:** [`construction_parametric_design_signoff_v1.md`](construction_parametric_design_signoff_v1.md) — extend, do not contradict.

---

## P1 — WSS player read (parallel A-W2 / B-H1)

| ☐ | ID | Deliverable | Pairs with |
|:---:|:---|:---|:---|
| ☑ | **DESIGN-HYDRO-PLAYER-READ-001** | [`wss_hydrology_player_read_v1.md`](wss_hydrology_player_read_v1.md) — ocean mask, river ribbon, minimap dim | B-H1 hydro |
| ☑ | **DESIGN-ATMOS-CLIPMAP-READ-001** | Zoom bands L0–L3 vs render clipmap — what player sees | A-W2 |
| ☑ | **DESIGN-WSS-DIAGNOSTICS-PASS-002** | [`wss_diagnostics_pass_002.md`](wss_diagnostics_pass_002.md) **PASS** | 2026-05-27 |
| ☑ | **DESIGN-DUAL-WRITE-UX-001** | [`wss_dual_write_transition_ux_001.md`](wss_dual_write_transition_ux_001.md) **PASS (qualified)** | 2026-05-27 |

---

## P1 — close wave 6 queue rows (already on disk, may need PASS upgrade)

| ☐ | ID | Record | Action |
|:---:|:---|:---|:---|
| ☑ | **DESIGN-UI-P6-MULTIVIEW-001** | [`ui_phase6_multiview_readability_v1.md`](ui_phase6_multiview_readability_v1.md) | **PASS (qualified)** 2026-05-26 |
| ☑ | **DESIGN-VFX-CAPTURE-WAVE5-001** | [`vfx_capture_status_wave5.md`](../assets/vfx/reference/review_captures/vfx_capture_status_wave5.md) | **PASS (qualified)** 2026-05-26 |
| ☑ | **DESIGN-OPERATOR-VISUAL-BUNDLE-001** | [`operator_visual_signoff_design_checklist_v1.md`](operator_visual_signoff_design_checklist_v1.md) | Operator handoff polish |
| ☑ | **DESIGN-THEME-COLLAGE-001** | [`ui_theme_collage_delta_v1.md`](ui_theme_collage_delta_v1.md) | Marked done in registry |
| ☑ | **DESIGN-WP-QUALIFIED-UPGRADE-001** | [`world_preview_visual_upgrade_checklist_v1.md`](world_preview_visual_upgrade_checklist_v1.md) | Marked done in registry |

---

## P2 — witness batch (2026-05-27, closed)

| ☐ | ID | Deliverable | Witness | Verdict |
|:---:|:---|:---|:---|:---|
| ☑ | **DESIGN-F2-EXTRACT-READ-001** | [`fire_f2_extract_readability_pass_001.md`](fire_f2_extract_readability_pass_001.md) | `stage5_full_app_live.json` → `f2_extract_witness` (`green`, `fire_instance_buffer_rows: 1`) | **PASS** |
| ☑ | **DESIGN-DUAL-WRITE-UX-001** | [`wss_dual_write_transition_ux_001.md`](wss_dual_write_transition_ux_001.md) | ECS+slab coexist copy (PR-2); `dual_write_shim` keys pending | **PASS (qualified)** |
| ☑ | **DESIGN-R4-MV-POST-PARAM-001** | [`construction_r4_mv_post_param_001.md`](construction_r4_mv_post_param_001.md) | `construction_stage_live.json` → parametric + `construction_r4_mv_ghost_001` green | **PASS** |
| ☑ | **DESIGN-WSS-DIAGNOSTICS-PASS-002** | [`wss_diagnostics_pass_002.md`](wss_diagnostics_pass_002.md) | `wss_substrate_live.json` → `green: true` | **PASS** |

Registry + `designer_active_queue.json` `done` rows updated 2026-05-27. **Do not re-run.**

---

## P2 — extended (post witness batch, closed)

| ☐ | ID | Deliverable | Trigger |
|:---:|:---|:---|:---|
| ☑ | **DESIGN-HANABI-BOUNDS-001** | [`hanabi_event_vfx_style_bounds_v1.md`](hanabi_event_vfx_style_bounds_v1.md) **PASS (qualified)** | 2026-05-27 |
| ☑ | **DESIGN-M3-REPLAY-PASS-002** | [`minimap_replay_pass_002_v1.md`](minimap_replay_pass_002_v1.md) **PASS** | 2026-05-27 |

---

## P2 — identity guard (light touch, high value)

| ☐ | ID | Deliverable |
|:---:|:---|:---|
| ☑ | **DESIGN-IDENTITY-CHECKPOINT-001** | [`project_identity_guard_rail_v1.md`](project_identity_guard_rail_v1.md) **PASS** | 2026-05-27 |

Use when reviewing coder Hybrid Assessments — designer does not block PRs, but records **TUNE** vs **ACCEPT**.

---

## Do not re-open

WSS-DESIGN-GATE-001 · CONSTRUCTION-PARAM-DESIGN-001 · wave 4/5 signoffs · FIRE7-DESIGN-001

---

## Next designer work

**Gang rollup:** [`designer_gang_lane_status_v1.md`](designer_gang_lane_status_v1.md) — cross-lane PICK snapshot.

**Skill:** [`.cursor/skills/aps-design-ux/SKILL.md`](../../.cursor/skills/aps-design-ux/SKILL.md) — APS tags · Variants draft preview · tooltips.

Queue `active` is empty — see [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) and optional / `routed_to_coder` lanes only.

### P0 — city grammar (designer-mcp — blocked on G0)

| ID | Action | Owner | Status |
|:---|:---|:---|:---|
| **DES-CITY-BLOCK-RECIPE-001** | [`design_city_block_recipe_v1.md`](design_city_block_recipe_v1.md) DRAFT | designer-mcp | **blocked** on G0c · critique → PASS |
| **DES-CITY-PALETTE-VARIATION-001** | Kit × palette variation charter (CITY-C5) | designer-mcp | **blocked** until G1 gate |

Queue: [`tools/orchestrator/queues/city_grammar_queue.json`](../../tools/orchestrator/queues/city_grammar_queue.json) · Plan: [`plan_city_grammar_upgrade_v1.md`](plan_city_grammar_upgrade_v1.md)

**Note:** APS building-tier specs (tier-2 tags, UX audit, G0/G1 empty copy) remain valid for **APS panels**; they do **not** cover block-tier town grammar — new designer-mcp charters above.

### P0 — open design (2026-06-02 scan)

| ID | Action | Owner |
|:---|:---|:---|
| ~~**DES-APS-TAG-TIER2-001**~~ | **PASS (qualified)** — [`design_aps_tag_tier2_v1.md`](design_aps_tag_tier2_v1.md) | done |
| ~~**APS-UX-AUDIT-001**~~ | **PASS WITH NOTES** v2 — [`design_aps_ux_audit_v2.md`](design_aps_ux_audit_v2.md) | done |
| ~~**DES-APS-GRAM-TIER-004**~~ | G0/G1 empty copy — [`design_aps_grammar_tier_g1_empty_v1.md`](design_aps_grammar_tier_g1_empty_v1.md) | done |
| ~~**DESIGN-WEATHER-PLAYER-READ-001**~~ | Charter in `src/dev/` — registry signed | done |
| **DES-APS-TAG-RUBRIC-001** | Operator tier-1 walk — **READY** | @operator |
| **MCP-PROD-ROWHOUSE-SIGNOFF** | Production pilot G4 review | designer-mcp tail (blocked on TILE) |
| **BUILD-READ-DESIGN-001/002** | **PASS on disk** — post_drain queue synced | done |

### P0 — APS presence (designer closed, coder-mcp picks)

| ID | Status |
|:---|:---|
| DES-APS-DEFAULT-PRESENCE-AUDIT-001 | **PASS (qualified)** |
| DES-APS-ASSEMBLY-EMPTY-G2-001 | **PASS** — copy only; coder-mcp wires label |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Parallel designer wave |
| v1.1.0 | 2026-05-26 | P0 parallel batch PASS (param staging/HUD + WSS contamination/smoke) |
| v1.2.1 | 2026-05-27 | P2 witness batch (F2 / dual-write / R4 MV / WSS diagnostics) documented closed |
| v1.2.2 | 2026-06-02 | APS UX v2 audit · tier-2 tags · G0/G1 empty · weather charter · operator rubric |
