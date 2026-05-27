# PLAN-CONSTRUCTION-PARAM-001 — exec PR phases `v1`

| Field | Value |
|:---|:---|
| **Parent plan** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) (**SIGNED** 2026-05-26) |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) |
| **Rule** | ≤3 production files per PR; one witness boolean per PR where noted |

---

## Phase 1 — Weighted footprint spine

**Lanes:** CONSTRUCTION-PARAM-CODER-001 + CODER-003 (partial)

| PR | Files (max 3) | Exit |
|:---|:---|:---|
| **P1-A** | `src/construction/weighted_footprint.rs`, `src/construction/placement_scaling.rs`, `src/construction/mod.rs` | `cargo test -p proc_A_dine01 --lib weighted_footprint` |
| **P1-B** | `src/strategic/site/tile_occupation.rs`, `src/strategic/site/components.rs`, `src/strategic/site/mod.rs` | `TileOccupationBook` resource + tests |
| **P1-C** | `src/strategic/site/events.rs`, `src/strategic/site/systems.rs`, `src/construction/live_proof.rs` | `CommitConstructionSiteEvent` carries weights; witness partial |

**Witness flags:** `weighted_raster_tests_green`, `commit_carries_scale_and_weights`

---

## Phase 2 — Input + ghost UX

**Lanes:** CONSTRUCTION-PARAM-CODER-002 + CODER-005

| PR | Files (max 3) | Exit |
|:---|:---|:---|
| **P2-A** | `src/construction/build_state.rs`, `src/construction/build_interaction.rs`, `src/construction/build_tool_authority.rs` | Enter commits single ghost; Shift queue removed (buildings) |
| **P2-B** | `src/construction/visual_authority.rs`, `src/construction/tool_hints.rs`, `src/construction/mod.rs` | Partial-alpha tile weights in map overlay |

**Witness flags:** `shift_queue_building_removed`, `enter_commits_single_ghost`, `overlap_blocks_commit`

---

## Phase 3 — Staging panel

**Lanes:** CONSTRUCTION-PARAM-CODER-004 (+ design **CONSTRUCTION-PARAM-DESIGN-001** prereq)

| PR | Files (max 3) | Exit |
|:---|:---|:---|
| **P3-A** | `src/construction/staged_ghost_panel.rs`, `src/construction/pending_construction.rs`, `src/construction/mod.rs` | Toggle + list + Build approved/all |
| **P3-B** | `src/gui/...` (tray hook only if required) | Designer PASS checklist |

**Witness flags:** `staging_toggle_wired`, `build_approved_drains_staged`

---

## Phase 4 — Economy + tradeoffs

**Lane:** CONSTRUCTION-PARAM-CODER-006

| PR | Files (max 3) | Exit |
|:---|:---|:---|
| **P4-A** | `src/economy/activation/scale.rs`, `src/economy/activation/bridge.rs`, `src/economy/activation/mod.rs` | `economy_scale_non_unity` test |
| **P4-B** | `assets/configs/buildings/` (one exemplar JSON) optional | Catalog `placement_scaling` sample |

**Witness flags:** `economy_scales_at_activation` → rollup `construction_parametric_placement_001.green`

---

## Verify (rollup)

```powershell
cargo test -p proc_A_dine01 --lib construction::weighted_footprint construction::
```

Manual: staging OFF Enter place · staging ON Build approved · scale drag partial alpha.
