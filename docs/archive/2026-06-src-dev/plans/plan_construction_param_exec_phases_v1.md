# PLAN-CONSTRUCTION-PARAM-001 — exec PR phases `v1`

| Field | Value |
|:---|:---|
| **Parent plan** | [`plan_construction_parametric_placement_v1.md`](plan_construction_parametric_placement_v1.md) (**SIGNED** 2026-05-26) |
| **Product spec** | [`construction_parametric_placement_spec_v1.md`](construction_parametric_placement_spec_v1.md) |
| **Parallel expand** | **PLAN-CONSTRUCTION-PARAM-P3P4-001** (2026-05-26) — staging + economy detail for **B-C4** / **B-C5** |
| **Coder matrix** | [`coder_fleet_multistage_matrix_v1.md`](coder_fleet_multistage_matrix_v1.md) **B-C4**, **B-C5** |
| **Design prereq** | [`construction_parametric_staged_panel_v1.md`](construction_parametric_staged_panel_v1.md) (**CONSTRUCTION-PARAM-DESIGN-001**) |
| **Rule** | ≤3 production files per PR; one witness boolean per PR where noted |

**Do not re-open:** PLAN-CONSTRUCTION-PARAM-001 product spec — this doc is **exec expansion only**.

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

## Phase 3 — Staging panel (expanded — **B-C4**)

**Lanes:** **CONSTRUCTION-PARAM-CODER-004** · **B-C4** in multistage matrix  
**Design:** [`construction_parametric_staged_panel_v1.md`](construction_parametric_staged_panel_v1.md) + [`construction_parametric_staging_ux_v2.md`](construction_parametric_staging_ux_v2.md) — **PASS** (2026-05-26)

### Authority

| Resource | Writer | Readers |
|:---|:---|:---|
| `StagedPlacementBook` (new) | `staged_ghost_panel` systems | commit funnel, witness |
| `PendingConstructionQueue` | existing / extend | staging drain |
| `BuildGhostState` | `build_interaction` | panel preview only — **no commit** from panel |
| `TileOccupationBook` | P1-B | overlap check on approve |

### P3-A — Core staging book + toggle

| File | Change |
|:---|:---|
| `src/construction/staged_ghost_panel.rs` | `StagedPlacementBook`, row model, toggle `Stage placements` |
| `src/construction/pending_construction.rs` | queue rows: scale, rotation, footprint ref, `allows_commit`, `approved` |
| `src/construction/mod.rs` | export + plugin registration |

**Row model (implement fully):**

```rust
pub struct StagedPlacementRow {
    pub id: u64,
    pub catalog_id: String,
    pub anchor_tile: IVec2,
    pub scale: f32,
    pub rotation_quarter_turns: u8,
    pub approved: bool,
    pub validity: StagedValidity, // Ok | Warn | Bad
    pub footprint_weights: Vec<f32>, // 4x4 raster ref
}
```

**Behaviors:**

- LMB on map with staging ON → append row from active ghost (no immediate commit)
- `Stage placements` OFF + `staged_count > 0` → panel stays visible (designer rule)
- **Build approved** → commit only `approved && allows_commit` rows through **single execute funnel**
- **Build all valid** → auto-approve valid rows then commit
- **Clear unapproved** → remove unapproved or `Bad` rows

**Lib tests:**

```text
staging_toggle_wired: toggle flips resource
enter_does_not_commit_when_staging_on: ghost remains after Enter
build_approved_drains_staged: approved row count → 0 after commit
build_approved_skips_unapproved: unapproved rows remain
```

**Witness:** `construction_stage_live.json` → `staging_toggle_wired`, `build_approved_drains_staged`

---

### P3-B — Tray hook + validity badges

| PR | Files (max 3) | Exit |
|:---|:---|:---|
| **P3-B** | `src/gui/hud/construction_tray.rs` (or parametric tray host), `src/construction/staged_ghost_panel.rs`, `src/construction/tool_hints.rs` | Tray body ≥156px when staging active; validity badges OK/Warn/Bad |

**Witness flags:** `staging_panel_visible`, `staging_validity_badges_wired`

**Designer exit:** **DESIGN-PARAM-STAGING-POLISH-002 PASS** — queue must **not** use `blocked_design` on CODER-004

**If stuck (B-C4):** fall back to **B-C5** economy slice (disjoint files) per matrix

---

## Phase 4 — Economy + scale at activation (expanded — **B-C5**)

**Lane:** **CONSTRUCTION-PARAM-CODER-006** · **B-C5**  
**Design:** [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md) — **PASS** (2026-05-26)  
**Depends on:** P1-A `placement_scaling` + P3 rows carrying `scale` into commit event

### Authority

| Domain | Writer | Notes |
|:---|:---|:---|
| `placement_scaling` on commit | construction commit funnel | carries `scale` + weights |
| `EconomyActivationScale` | `economy/activation/scale.rs` | applies to throughput/cost curves |
| `IndustrialActivation` bridge | `economy/activation/bridge.rs` | reads scale at facility spawn |

### P4-A — Scale application at activation

| File | Change |
|:---|:---|
| `src/economy/activation/scale.rs` | `apply_placement_scale_to_facility`, non-unity test vectors |
| `src/economy/activation/bridge.rs` | read scale from `CommitConstructionSiteEvent` or staged row payload |
| `src/economy/activation/mod.rs` | wire systems in activation plugin |

**Scale semantics (locked for v1):**

| Scale | Effect |
|:---|:---|
| `1.0` | Baseline catalog cost / output |
| `> 1.0` | Footprint area ∝ scale² for occupancy; **cost** ∝ scale^2.2 (designer tunable constant in `scale.rs`) |
| `< 1.0` | Min clamp `0.25` — warn validity in staging panel |

**Lib tests:**

```text
economy_scales_at_activation: scale 1.24 → output != baseline
economy_scale_clamp: scale 0.1 → clamped 0.25
staged_scale_flows_to_activation: end-to-end staged commit
```

**Witness:** `construction_stage_live.json` → `economy_scales_at_activation`

---

### P4-B — Catalog exemplar + rollup

| PR | Files (max 3) | Exit |
|:---|:---|:---|
| **P4-B** | `assets/configs/buildings/` (one JSON), `src/construction/live_proof.rs`, `src/construction/placement_scaling.rs` | `placement_scaling` in catalog; rollup green |

**Witness rollup:**

```text
construction_parametric_placement_001.green :=
  weighted_raster_tests_green
  AND enter_commits_single_ghost
  AND overlap_blocks_commit
  AND staging_toggle_wired
  AND build_approved_drains_staged
  AND economy_scales_at_activation
```

**B-C6 PARAM rollup:** refresh `construction_stage_live.json` all booleans + manual playtest note in HANDOFF

---

## Phase 3–4 witness block (`construction_stage_live.json`)

| Key | Phase | Predicate |
|:---|:---:|:---|
| `staging_toggle_wired` | P3 | bool true |
| `build_approved_drains_staged` | P3 | bool true |
| `staging_panel_visible` | P3-B | bool true when count > 0 |
| `economy_scales_at_activation` | P4 | bool true |
| `construction_parametric_placement_001` | rollup | object `.green` true |

```powershell
cargo test -p proc_A_dine01 --lib construction
cargo test -p proc_A_dine01 --lib construction::staged
cargo test -p proc_A_dine01 --lib economy::activation
```

---

## Verify (rollup)

```powershell
cargo test -p proc_A_dine01 --lib construction::weighted_footprint construction::
```

Manual: staging OFF Enter place · staging ON Build approved · scale drag partial alpha.
