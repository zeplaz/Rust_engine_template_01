# STRUCT-CASCADE-v1 — load-bearing cascade from StructuralRole

| Field | Value |
|:---|:---|
| **Id** | **STRUCT-CASCADE-v1** |
| **Program** | [`plan_destruction_presentation_v1.md`](plan_destruction_presentation_v1.md) |
| **Status** | **ADOPTED** — foundation→column mask cascade |
| **Date** | 2026-11-17 |
| **Owner** | `@coder` |
| **Depends** | STRUCT-ROLE |

## Rule

If any **Foundation** tile of a site is `MASK_DESTROYED`, mark all **load-bearing** tiles (Foundation + Column) destroyed on the presentation mask. Facade/Trim stay until site demolish.

Dam integrity aggregation already lives in [`dam_barrier.rs`](../construction/dam_barrier.rs); this slice is the **mask** cascade so PixelWorld does not leave floating columns.

## Production

[`structural_cascade.rs`](../construction/structural_cascade.rs) `cascade_mask_from_roles_system` after mask writers / before GPU upload.
