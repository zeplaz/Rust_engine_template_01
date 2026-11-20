# STRUCT-ROLE-v1 — structural role tags on site footprints

| Field | Value |
|:---|:---|
| **Id** | **STRUCT-ROLE-v1** |
| **Program** | [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) |
| **Status** | **ADOPTED** — tags; dam span writer + D-2 default roles for every committed non-dam site (`ensure_default_structural_roles_system`: corner/end tile Foundation, 3-neighbour perimeter Column, interior Facade); cascade = STRUCT-CASCADE (2026-11-19) |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` (construction) |

## Objective

Tag footprint tiles as Foundation / Column / Facade / Trim so cascade and disaster proxies can treat glass ≠ load column.

## Design

| Piece | Path |
|:---|:---|
| Enum + component | [`structural_role.rs`](../construction/structural_role.rs) |
| Dam writer | [`dam_barrier.rs`](../construction/dam_barrier.rs) — columns = `Column`, abutments = `Foundation` |

## Accept

| Check | Pass |
|:---|:---|
| Lib | `--lib structural_role` / `struct_role` |
| Production | Dam `ensure_columns` fills parallel [`SiteStructuralRoles`](../construction/structural_role.rs) |
| Cascade | Deferred to **STRUCT-CASCADE** |
