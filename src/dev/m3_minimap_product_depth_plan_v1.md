# PLAN-M3-PRODUCT-DEPTH-001 — minimap M3 product depth `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-M3-PRODUCT-DEPTH-001** |
| **Prior** | [`minimap_m3_units_replay_impl_plan_v1.md`](minimap_m3_units_replay_impl_plan_v1.md) — witness **CLOSED** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **P2 optional** |

**No Rust.**

---

## Scope

| Slice | Witness | Product forward |
|:---|:---:|:---|
| **UI-P3-M3-UNITS-001** | ☑ | Strategic unit aggregation reader (not seed coords) |
| **UI-P3-M3-REPLAY-001** | ☑ | Live `CommittedSimReplayRing` in Simulation |

**Do not** reopen **UI-P3-M4-001** (FoW/EW).

---

## Exit (product)

| ID | Task | Files (≤3 each) |
|:---|:---|:---|
| U-P1 | Logistics/strategic snapshot → unit markers | `visual_domain_snapshots.rs`, reader module |
| R-P1 | Ring commits on sim tick | `sim_frame_delta.rs`, sim schedule hook |

**Verify:** `cargo test -p proc_A_dine01 --lib minimap_compositor`

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Optional polish after witness close |
