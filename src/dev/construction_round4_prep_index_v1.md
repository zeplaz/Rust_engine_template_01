# CONSTRUCTION-R4-PREP-001 — Round 4 prep index `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **CONSTRUCTION-R4-PREP-001** |
| **Planner gate** | **PLAN-CONSTRUCTION-R4-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Status** | **PREP CLOSED** — product board **not open** |
| **Witness** | `debug_runs/construction_stage_live.json` → `construction_r4_prep_001` |

**No Round 4 Rust** until the product board opens Round 4.

---

## Prep checklist (coder)

| # | Item | Artifact | Status |
|:---:|:---|:---|:---:|
| 1 | Round 3 operational green | `debug_runs/construction_stage_live.json` → `operational_green` | ☑ |
| 2 | Multiview sim writer | [`construction_multiview_sim_spec_v1.md`](construction_multiview_sim_spec_v1.md) · `construction_mv_001.green` | ☑ |
| 3 | Ghost readability (designer) | [`construction_multiview_ghost_readability_v1.md`](construction_multiview_ghost_readability_v1.md) | ☑ |
| 4 | Recovery / catalog index | [`construction_recovery_todos.md`](construction_recovery_todos.md) | ☑ |
| 5 | Product gate (blocked) | [`construction_round4_product_gate_plan_v1.md`](construction_round4_product_gate_plan_v1.md) | ☑ signed |

---

## R4 specs (signed — impl blocked)

| Phase | Owner | Deliverable |
|:---|:---|:---|
| **R4-PLAN-001** | `@planner` | [`construction_round4_corridor_phase_spec_v1.md`](construction_round4_corridor_phase_spec_v1.md) ☑ |
| **R4-PLAN-002** | `@designer` | [`construction_round4_multiview_ghost_presets_v1.md`](construction_round4_multiview_ghost_presets_v1.md) ☑ |
| **R4-CORRIDOR-001** | `@coder` | Corridor book + R8 roundtrip + witness (board open) |
| **R4-MV-GHOST-001** | `@coder` | Corridor overlay + legend (board open) |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json
cargo test -p proc_A_dine01 --lib construction_mv_001
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Prep index + `construction_r4_prep_001` witness |
