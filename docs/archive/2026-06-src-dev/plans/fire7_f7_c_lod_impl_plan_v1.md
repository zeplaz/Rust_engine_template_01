# PLAN-F7-C-LOD-001 — F7-C LOD caps signoff `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-F7-C-LOD-001** |
| **Coder lane** | **FIRE7-F7-C-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — impl **CLOSED** @coder wave 3 |
| **Prereq** | **F7-A-EXIT** + **F7-B** streaming |
| **Design** | [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md) |
| **Architecture** | [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) § F7-C |
| **Code** | [`src/render/fire_view_extract.rs`](../render/fire_view_extract.rs) |

**No Rust in this deliverable.**

---

## Executive summary

| Verdict | Meaning |
|:---|:---|
| **CLOSED** | Strategic < operational < tactical caps wired; `fire7_f7_c_001_green()` true |
| **P2 optional** | Per-band instance count diff under tactical visual run |

---

## Exit criteria (C1–C3)

| # | Criterion | Pass when | Evidence |
|:---:|:---|:---|:---|
| **C1** | Designer table monotonic | `FIRE_LOD_CAP_STRATEGIC < OPERATIONAL < TACTICAL` | `fire_lod_designer_table_wired()` |
| **C2** | Extract clamps by band | `clamp_fire_lod_for_world_band` / `fire_cap_for_world_band` used | `fire_view_extract.rs` |
| **C3** | Minimap heat-only | F7-A A3 still true | `fire7_f7_a_exit_001.minimap_fire_overlay_only` |

**Rollup:** `fire7_f7_c_001_green()`.

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib fire_view_extract fire_visual_extract
```

Wave 3 bundle: `coder_a_wave3_closure_v1.rs` asserts `fire7_f7_c_001_green()`.

---

## Anti-patterns

| Forbidden | Why |
|:---|:---|
| LOD greens in JSON only | Not F7-C |
| Minimap tactical fire extract | F7-A / compositor boundary |
| Flat caps across bands | Violates designer table |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Post-impl signoff **PLAN-F7-C-LOD-001** |
