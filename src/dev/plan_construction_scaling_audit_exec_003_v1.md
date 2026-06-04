# PLAN-CONSTRUCTION-SCALING-AUDIT-003 — Phase 3 scaling audit exec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-CONSTRUCTION-SCALING-AUDIT-003** |
| **Parent** | [`construction_product_roadmap_phases_2_10_v1.md`](construction_product_roadmap_phases_2_10_v1.md) § Phase 3 |
| **Alignment** | [`planner_program_alignment_v1.md`](planner_program_alignment_v1.md) |
| **Designer input** | [`design_construction_scaling_read_v1.md`](design_construction_scaling_read_v1.md) **PASS** |
| **Scale HUD baseline** | [`construction_parametric_scale_hud_v1.md`](construction_parametric_scale_hud_v1.md) |
| **Prereq** | **CON-P2-001..003** green on disk |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` → `@coder A` / `@coder B` |
| **Status** | **SIGNED — READY** |
| **Horizon** | **1–2 weeks** (4 PRs: S1–S3 + witness refresh) |

**Hard rules:** Preview never commits. Tray widget resize ≠ building `scale_factor`. No new `ConstructionStage` enum. Witness JSON wins over markdown.

---

## 1. Disk truth at sign-off

| Witness key | On disk (`construction_stage_live.json`) | Code |
|:---|:---|:---|
| `construction_site_stage_pipeline_001.green` | **true** | CON-P2 closed |
| `construction_parametric_placement_001.green` | **false** (`partial_alpha: false`) | PARAM mostly wired |
| `construction_scaling_audit_001` | **absent** | B-half S4–S6 in `scaling_audit.rs`; collector wired |

**Gate satisfied:** CON-P2 closed → Phase 3 coder pulls authorized.

---

## 2. Problem

Parametric placement (scale, rotation, weighted footprint) is partially landed. Product requires a **paired designer + coder audit** (S1–S6) proving:

- Ghost footprint cells match commit cells at all scale presets
- Occupied / blocked / terrain-mod tiles are visible before confirm
- Overlap disables commit and drives partial-alpha ghosts (designer PASS)
- Scale + rotation persist on committed site
- HUD tray bounds stay independent of building scale clamp

Today `construction_parametric_placement_001` is red on disk; B-half helpers exist but live JSON not refreshed.

---

## 3. Out of scope

| Item | Where |
|:---|:---|
| Module kit / procedural meshes | Phase 4 / PG-2 |
| Staged construction tick | CON-P2 (done) |
| New catalog rows per scale step | Use continuous `scale_factor` only |
| GIS / town books | SET-P5 |

---

## 4. Audit matrix (S1–S6)

| # | Designer acceptance | Coder verify | Owner | Disk field |
|:---:|:---|:---|:---:|:---|
| **S1** | Presets 1×1…12×12 readable on tray | `scale_factor` clamp + matrix cell count; ghost cells == commit cells | **A** | `s1_preset_matrix_match` |
| **S2** | Occupied tiles yellow footprint | `FootprintTileWitness` / `FootprintTileColorKind::Risky` for occupied | **A** | `s2_occupied_tiles_wired` |
| **S3** | Blocked red; commit disabled | `allows_commit == false`; cannot execute | **A** | `s3_blocked_disables_commit` |
| **S4** | Terrain mod token in legend | mud/cut/fill or validity badges | **B** | `s4_terrain_mod_legend` |
| **S5** | Rotation + scale on site after commit | `BuildingScaleParams` / `commit_carries_scale_and_weights` | **B** | `s5_scale_persists_on_site` |
| **S6** | Tray resize ≠ building scale | `DEFAULT_SCALE_MIN/MAX` independent of panel bounds | **B** | `s6_tray_independent_of_building_scale` |

**Overlap UX (designer):** badge priority Blocked > Overlap > Terrain > Clear; partial-alpha table in [`design_construction_scaling_read_v1.md`](design_construction_scaling_read_v1.md) → witness `partial_alpha_wired`, `overlap_badge_wired`.

---

## 5. PR train

### CON-P3-S1 — Preset matrix match (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `src/construction/placement_scaling.rs`, `src/construction/parametric_commit.rs`, `src/construction/scaling_audit.rs` (extend A-half) |
| **Tests** | `scaling_audit_s1_preset_matrix_match` — for scale factors mapping to 1×1…12×12 effective cells, ghost `FootprintMatrix` cell count == post-commit `FootprintTiles` |
| **Exit** | `scaling_audit_s1_preset_matrix_match_green()` true |

### CON-P3-S2 — Occupied tile flags (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `src/construction/footprint_tile_instances.rs`, `src/construction/visual_authority.rs` |
| **Tests** | Place ghost overlapping existing site tile → witness sees occupied/risky flag |
| **Exit** | `scaling_audit_s2_occupied_tiles_wired_green()` true |

### CON-P3-S3 — Blocked commit gate (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `src/construction/build_validation.rs` or strategic overlap helpers, `staged_ghost_panel.rs` |
| **Tests** | Invalid/blocked footprint → `allows_commit` false; execute rejected |
| **Exit** | `scaling_audit_s3_blocked_disables_commit_green()` true; extends `overlap_blocks_commit` witness |

### CON-P3-S4–S6 — Designer audit B-half (**DONE** on master)

| Item | Detail |
|:---|:---|
| **Files** | `src/construction/scaling_audit.rs`, `witness_collectors.rs` |
| **Evidence** | `con_p3_scaling_audit_b_half_green` lib test |
| **Exit** | `construction_scaling_audit_001_b_witness_green()` — **do not re-pick** unless regression |

### CON-P3-WIT — Live JSON rollup (Coder A)

| Item | Detail |
|:---|:---|
| **Files** | `witness_collectors.rs`, `construction_stage_witness.rs` / sim writer |
| **Blocked by** | CON-P3-S1..S3 green |
| **Exit** | `construction_scaling_audit_001.green: true` on disk; `construction_parametric_placement_001.partial_alpha: true`; refresh via `simulation_writes_construction_stage_live_json` |

---

## 6. Witness schema

**Primary block:** `construction_scaling_audit_001`

| Pointer | Type | Pass when |
|:---|:---|:---|
| `/construction_scaling_audit_001/gate` | string | `CONSTRUCTION-SCALING-AUDIT-001` |
| `/construction_scaling_audit_001/green` | bool | all S1–S6 true |
| `/construction_scaling_audit_001/s1_preset_matrix_match` | bool | `true` |
| `/construction_scaling_audit_001/s2_occupied_tiles_wired` | bool | `true` |
| `/construction_scaling_audit_001/s3_blocked_disables_commit` | bool | `true` |
| `/construction_scaling_audit_001/s4_terrain_mod_legend` | bool | `true` |
| `/construction_scaling_audit_001/s5_scale_persists_on_site` | bool | `true` |
| `/construction_scaling_audit_001/s6_tray_independent_of_building_scale` | bool | `true` |
| `/construction_scaling_audit_001/partial_alpha_wired` | bool | `true` |
| `/construction_scaling_audit_001/overlap_badge_wired` | bool | `true` (HUD — may trail sim) |

**Rollup into existing block:** extend `construction_parametric_placement_001`:

| Field | Pass when |
|:---|:---|
| `partial_alpha` | `true` |
| `green` | `true` (full PARAM + scaling rollup) |

---

## 7. Designer ↔ coder pairing

| Surface | Designer doc § | Coder witness |
|:---|:---|:---|
| Overlap badge | § Overlap / partial-alpha badge | `overlap_badge_wired` |
| Partial-alpha ghost | § Partial-alpha ghost rule | `partial_alpha_wired` + `visual_authority` |
| Staged panel confirm | § When messaging shows | `staging_validity_badges_wired` |
| Post-commit | no overlap HUD | S5 only |

---

## 8. Unblocks

| Downstream | Slice |
|:---|:---|
| **Phase 4** module kit + PG-1 | Scale audit green before art attach |
| **PROC-PG-2-001** | Greybox assembly uses audited footprints |
| **SET-P5-003** | Independent — may parallel S1–S3 |

---

## 9. Anti-patterns

- Widget resize changing building footprint without `scale_factor`
- Commit with `allows_commit == false`
- Witness green without lib refresh test
- Reopening CON-P3-S4–S6 unless regression

---

## 10. Regression

```powershell
cargo test -p proc_A_dine01 --lib construction scaling_audit con_p3
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
```

---

## 11. Coder handoff

| Field | Value |
|:---|:---|
| **Pull now** | **A:** CON-P3-S1 → S2 → S3 → CON-P3-WIT |
| **B half** | **CLOSED** — S4–S6 |
| **Parallel OK** | CON-P3-S1 with INFRA-E0-001 (disjoint files) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | S1–S6 matrix; B-half done; A-half + witness refresh open |
