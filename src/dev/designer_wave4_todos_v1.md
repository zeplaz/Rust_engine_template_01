# Designer wave 4 todos `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.3.0` |
| **Date** | 2026-05-26 |
| **Trigger** | Coder dual-queue closure — [`coder_fleet_return_recap_wave3_v1.md`](coder_fleet_return_recap_wave3_v1.md) |
| **Coder backlog** | [`coder_wave3_full_todos_v1.md`](coder_wave3_full_todos_v1.md) |
| **Workboard** | [`stage_designer_workboard_v1.md`](stage_designer_workboard_v1.md) |
| **Machine queue** | [`tools/orchestrator/queues/designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) |

**Rule:** Design / review / sign-off records only. No Rust.

**Already SIGNED (coders use as-is — do not re-open unless PASS fails):**

| ID | Witness |
|:---|:---|
| **FIRE7-DESIGN-001** | [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) |
| **S7P-DESIGN-002** | [`s7p_grid_overload_ux_note_v1.md`](s7p_grid_overload_ux_note_v1.md) |
| **DESIGN-CONSTRUCTION-MV-001** | [`construction_multiview_ghost_readability_v1.md`](construction_multiview_ghost_readability_v1.md) |
| **DESIGN-M3-UNITS-001** | [`minimap_unit_marker_visual_spec_v1.md`](minimap_unit_marker_visual_spec_v1.md) |
| **DESIGN-M3-REPLAY-001** | [`minimap_replay_scrub_visual_spec_v1.md`](minimap_replay_scrub_visual_spec_v1.md) |
| **DESIGN-VFX-VISUAL-ACCEPT-001** | [`vfx_visual_acceptance_record_v1.md`](vfx_visual_acceptance_record_v1.md) — **PASS (qualified)** |
| **DESIGN-WP-VISUAL-ACCEPT-001** | [`world_preview_visual_acceptance_record_v1.md`](world_preview_visual_acceptance_record_v1.md) — **PASS (qualified)** |
| **DESIGN-S7P-TOAST-PASS-001** | [`s7p_grid_overload_ux_pass_record_v1.md`](s7p_grid_overload_ux_pass_record_v1.md) — **PASS (qualified)** |
| **DESIGN-F7-B-DEBUG-001** | [`fire_streaming_debug_overlay_names_v1.md`](fire_streaming_debug_overlay_names_v1.md) |
| **DESIGN-BQ128-APPLY-UX-001** | [`bq128_apply_ghost_ux_review_v1.md`](bq128_apply_ghost_ux_review_v1.md) — **PASS (qualified)** |
| **DESIGN-IND-E03-OPS-001** | [`ind_e03_ops_strip_polish_v1.md`](ind_e03_ops_strip_polish_v1.md) — **PASS (qualified)** |
| **DESIGN-VFX-CAPTURE-ROUND-002** | [`vfx_capture_status_wave4.md`](../assets/vfx/reference/review_captures/vfx_capture_status_wave4.md) — **PASS (qualified)** |

Wave 4 was **acceptance + new visual specs** for coder wave 3. **Complete (rows 1–10)** on disk + in `designer_active_queue.json` → `done`.

---

## Master board (wave 4)

| ☐ | # | ID | Unblocks coder | If blocked → start |
|:---:|:---:|:---|:---|:---|
| ☑ | 1 | **DESIGN-S7P-TOAST-PASS-001** | **S7P-GRID-UX-UI-001** (B #3) | — |
| ☑ | 2 | **DESIGN-CONSTRUCTION-MV-001** | **CONSTRUCTION-MV-SIM-001** (B #2) **CLOSED** | — |
| ☑ | 3 | **DESIGN-M3-UNITS-001** | **UI-P3-M3-UNITS-001** (B #5) | — |
| ☑ | 4 | **DESIGN-M3-REPLAY-001** | **UI-P3-M3-REPLAY-001** (B #6) | — |
| ☑ | 5 | **DESIGN-VFX-VISUAL-ACCEPT-001** | **VFX-VISUAL-SIGNOFF-001** (A #2) | — *(qualified PASS — **not** blocked on `--test visual`)* |
| ☑ | 6 | **DESIGN-WP-VISUAL-ACCEPT-001** | **UI-WP-VISUAL-001** (A #6) | — *(qualified PASS — lib witness + layout refs)* |
| ☑ | 7 | **DESIGN-F7-B-DEBUG-001** | **FIRE7-F7-B-001** optional F3 labels | — |
| ☑ | 8 | **DESIGN-BQ128-APPLY-UX-001** | **UX-E02-APPLY-POLISH-001** (B #9) | — |
| ☑ | 9 | **DESIGN-IND-E03-OPS-001** | **IND-E03-SIM-UX-001** (B #11) | — |
| ☑ | 10 | **DESIGN-VFX-CAPTURE-ROUND-002** | operator captures (**VX-P0-04**) | — |

---

## P1 — detail (rows 1–6)

| ☐ | # | Queue ID | Deliverable | Unblocks coder |
|:---:|:---:|:---|:---|:---|
| ☑ | 1 | **DESIGN-S7P-TOAST-PASS-001** | [`s7p_grid_overload_ux_pass_record_v1.md`](s7p_grid_overload_ux_pass_record_v1.md) — **PASS (qualified)** | **S7P-GRID-UX-UI-001** (B #3) |
| ☑ | 2 | **DESIGN-CONSTRUCTION-MV-001** | [`construction_multiview_ghost_readability_v1.md`](construction_multiview_ghost_readability_v1.md) | **CONSTRUCTION-MV-SIM-001** — witness green |
| ☑ | 3 | **DESIGN-M3-UNITS-001** | [`minimap_unit_marker_visual_spec_v1.md`](minimap_unit_marker_visual_spec_v1.md) | **UI-P3-M3-UNITS-001** |
| ☑ | 4 | **DESIGN-M3-REPLAY-001** | [`minimap_replay_scrub_visual_spec_v1.md`](minimap_replay_scrub_visual_spec_v1.md) | **UI-P3-M3-REPLAY-001** |
| ☑ | 5 | **DESIGN-VFX-VISUAL-ACCEPT-001** | [`vfx_visual_acceptance_record_v1.md`](vfx_visual_acceptance_record_v1.md) | **VFX-VISUAL-SIGNOFF-001** |
| ☑ | 6 | **DESIGN-WP-VISUAL-ACCEPT-001** | [`world_preview_visual_acceptance_record_v1.md`](world_preview_visual_acceptance_record_v1.md) | **UI-WP-VISUAL-001** |

---

## P2 — optional / polish (rows 7–10)

| ☐ | # | Queue ID | Deliverable | Unblocks coder |
|:---:|:---:|:---|:---|:---|
| ☑ | 7 | **DESIGN-F7-B-DEBUG-001** | [`fire_streaming_debug_overlay_names_v1.md`](fire_streaming_debug_overlay_names_v1.md) | Optional F3 wire |
| ☑ | 8 | **DESIGN-BQ128-APPLY-UX-001** | [`bq128_apply_ghost_ux_review_v1.md`](bq128_apply_ghost_ux_review_v1.md) — **PASS (qualified)** | **UX-E02-APPLY-POLISH-001** |
| ☑ | 9 | **DESIGN-IND-E03-OPS-001** | [`ind_e03_ops_strip_polish_v1.md`](ind_e03_ops_strip_polish_v1.md) — **PASS (qualified)** | **IND-E03-SIM-UX-001** |
| ☑ | 10 | **DESIGN-VFX-CAPTURE-ROUND-002** | [`vfx_capture_status_wave4.md`](../assets/vfx/reference/review_captures/vfx_capture_status_wave4.md) — **PASS (qualified)** | Operator PNG upgrade optional |

---

## Blocked — honest

| Blocked ID | Waits on | Designer start instead |
|:---|:---|:---|
| ~~**DESIGN-S7P-TOAST-PASS-001**~~ | — | **CLOSED** — witness + PASS record on disk |
| **DESIGN-VFX-VISUAL-ACCEPT-001** | ~~visual run~~ | **CLOSED** — qualified PASS on witness + interim PNGs |
| **DESIGN-WP-VISUAL-ACCEPT-001** | ~~visual run~~ | **CLOSED** — qualified PASS on `wave_p_live.json` |
| ~~**DESIGN-F7-B-DEBUG-001**~~ | — | **CLOSED** — F3 label contract on disk |

---

## Remaining designer work (2026-05-26)

**Wave 4 complete (10/10).** No open designer rows on this board.

---

## Orchestrator sync

| Source | P1 open | P2 open |
|:---|:---:|:---:|
| [`designer_active_queue.json`](../tools/orchestrator/queues/designer_active_queue.json) `active` | **0** | **0** |
| [`designer_signoff_registry.json`](../tools/orchestrator/queues/designer_signoff_registry.json) | Wave 4 done rows registered | — |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.3.0 | 2026-05-26 | P2 rows 7–10 ☑ — **wave 4 closed** |
| v1.2.0 | 2026-05-26 | **DESIGN-S7P-TOAST-PASS-001** PASS — P1 complete |
| v1.1.0 | 2026-05-26 | Master board sync; rows 2–6 ☑; #5/#6 not blocked on visual run |
| v1.0.0 | 2026-05-26 | 10 designer rows; acceptance-heavy wave 4 |
