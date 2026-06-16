# PLAN-PHASE3-EXEC-001 — POST-DRAIN Phase 3 (thin exec) `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-PHASE3-EXEC-001** |
| **Program** | POST-DRAIN-PHASE-3-001 |
| **Seeding** | $ref:src/dev/plan_queue_seeding_v1.md |
| **Queue** | $ref:tools/orchestrator/queues/post_drain_phase3_queue.json |
| **Planner** | **SIGNED** |
| **Date** | 2026-06-08 |

**Rule:** Witness keys + COMMIT:SPEC only. **Do not** assign EGUI-QC (lane 4 shipped). **Do not** reopen grammar / rowhouse / infra E-tail.

---

## Cycle 1 slices

### ⟨TRIAGE-FIRE-EXTRACT-FINAL-001⟩ — @coder · Lane H_INFRA

| Field | Value |
|:---|:---|
| **COMMIT:SPEC** | Harden `VisibleFireChunkSet` — per-view visible fire extraction final |
| **Source** | `stage5_triage_backlog.md` · TRIAGE-FIRE-EXTRACT · `base_finsh_5.md` §3 |
| **Territory** | $sym:fire_view_extract@src/render/fire_view_extract.rs · $sym:fire_visual_extract@src/render/extraction/fire_visual_extract.rs |
| **Witness** | $ref:debug_runs/stage5_full_app_live.json |

**Problem (plain):** F2-EXTRACT proved projection-graph instances; **final** closes per-view visible set — no camera-global shortcuts, bounded extract per `ViewId`.

**Deliverables:**

| # | Task | Exit |
|:---:|:---|:---|
| 1 | `VisibleFireChunkSet` derives from `ViewProjectionAuthority` + view policy only | witness key below |
| 2 | Minimap/preview caps respected (VM-11 parity) | `minimap_cap_respected` · `preview_cap_respected` |
| 3 | No full-world fire scan in extract path | `full_world_scan_absent: true` |
| 4 | Merge witness block into stage5 live proof writer | lib test refresh |

**Witness keys** — `stage5_full_app_live.json` → `triage_fire_extract_final_001`:

| Key | Target |
|:---|:---|
| `gate` | **`TRIAGE-FIRE-EXTRACT-FINAL-001`** |
| `green` | **true** |
| `per_view_visible_derived_from_projection` | **true** |
| `no_camera_global_shortcut` | **true** |
| `f7_a_per_view_extract_bounded` | **true** |
| `minimap_cap_respected` | **true** |
| `preview_cap_respected` | **true** |
| `projection_fire_source` | **`WorldMain`** (or documented policy string) |
| `visible_fire_chunk_set_authoritative` | **true** |
| `full_world_scan_absent` | **true** |

**Maintain:** `f2_extract_witness.green: true` · `fire_f2_004_001.green: true`

**Regression:**

```powershell
cargo test -p proc_A_dine01 --lib fire_visual_extract fire_view_extract stage5
```

**COMMIT:WIT** `debug_runs/stage5_full_app_live.json`

**Do not:** Second global extract · minimap ECS fire query · Stage 5 gate reopen

---

### ⟨SIM-HUD-PRODUCT-CLOSE-001⟩ — @designer · Lane I_PRODUCT

| Field | Value |
|:---|:---|
| **COMMIT:SPEC** | Close lane 5 program — **Bevy sim chrome only** (NOT egui QC lane 4) |
| **Orders** | $ref:docs/archive/2026-06-src-dev/plans/bevy_hud_lanes_agent_orders_v1.md |
| **Prior** | PASS **(qualified)** — $ref:docs/archive/2026-06-src-dev/plans/design_sim_hud_product_signoff_v1.md |

**Mission:** Upgrade qualified close → **full program PASS** — all five slice witnesses green + designer sign-off v2.

**Slice witnesses (all must be green):**

| Slice | File | Current |
|:---|:---|:---:|
| PLAY-01 | `sim_hud_play01_live.json` | 🟢 |
| DOCK | `sim_hud_slice_dock_live.json` | 🟢 |
| OPS | `sim_hud_slice_ops_live.json` | 🟢 |
| MINIMAP | `sim_hud_slice_minimap_live.json` | 🟢 |
| BUILD | `sim_hud_slice_build_live.json` | 🟢 |

**Designer deliverables:**

| # | Task | Exit |
|:---:|:---|:---|
| 1 | Playtest polish — ops/dock/minimap/build readability at 1080p | notes in sign-off doc |
| 2 | Confirm boundary: lane 4 egui QC separate | checklist ✓ |
| 3 | Update `design_sim_hud_product_signoff_v1.md` → **PASS (full)** | verdict row |
| 4 | Optional: refresh `ui_shell_migration_live.json` `sim_hud_product_001` rollup | if drift vs slice JSONs |

**Rollup witness keys** — new or refreshed `sim_hud_product_close_001` block (any one file — prefer `ui_shell_migration_live.json` or dedicated JSON):

| Key | Target |
|:---|:---|
| `gate` | **`SIM-HUD-PRODUCT-CLOSE-001`** |
| `green` | **true** |
| `slices_green_count` | **5** |
| `play01_green` | **true** |
| `dock_green` | **true** |
| `ops_green` | **true** |
| `minimap_green` | **true** |
| `build_green` | **true** |
| `lane4_egui_qc_separate` | **true** |
| `designer_signoff_full_pass` | **true** |

**COMMIT:SPEC** `docs/archive/2026-06-src-dev/plans/design_sim_hud_product_signoff_v1.md` (v1.2 full PASS)

**Do not:** Tk APS · egui sim product shell · merge lane 4 into lane 5

**ΔWF→@coder:** only if polish finds witness regression — otherwise designer-only close

---

### ⟨G-PLAY-01⟩ — Operator · Lane G_OPS

| Field | Value |
|:---|:---|
| **COMMIT:SPEC** | $ref:src/dev/plan_g_play_close_001_checklist_v1.md |
| **Runbook** | $ref:docs/archive/2026-06-src-dev/plans/play_scenario_acceptance_runbook_v1.md |
| **Blocks** | PLAN-AUDIT-020 |

Preconditions: release build · no `--test visual` · no harness seed env.

---

## Cycle 2 (queued — not Cycle 1)

| ⟨ID⟩ | Agent | Note |
|:---|:---|:---|
| TRIAGE-FIRE-LOD-TIERS-001 | @coder | planner exec keys first |
| TRIAGE-PHASE-F-CULL-001 | @coder | stage5 witness extend |
| APS-ARTIST-TOOL-E2E-REVIEW-001 | @designer-mcp | defer review |

---

## Cancelled (do not assign)

| ⟨ID⟩ | Reason |
|:---|:---|
| EGUI-QC-IMPL-001 | Lane 4 shipped — `aps_bevy_qc_hud_001_v2_live.json` green |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-08 | Cycle 1 — FIRE-EXTRACT-FINAL + SIM-HUD-CLOSE witness keys |
