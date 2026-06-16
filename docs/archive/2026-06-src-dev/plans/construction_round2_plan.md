# Construction Round 2 — execution plan

**Board:** [`construction_round2_todos.rs`](construction_round2_todos.rs) · **Spec:** [`recovery_construction.md`](recovery_construction.md) § Round 2

## Wave A — Tier 1 (this pass)

| ID | Deliverable | Status |
|----|-------------|--------|
| R2-01 | `ActiveToolSession` — Esc keeps tool; commit counter | **Done** |
| R2-02 | Road ghost every frame (`update_road_path_preview_system`) | **Done** |
| R2-03 | Continuous road — anchor last point after Shift+LMB commit | **Done** |
| R2-04 | Grid + road-node snap (`snap.rs`) | **Done** |
| R2-05 | `ghost_visual.rs` soft colors | **Done** |

## Wave B — Tier 2 (this pass)

| ID | Deliverable | Status |
|----|-------------|--------|
| R2-06 | Duplex/Quadplex + `BuildingIntentPreview` panel | **Done** |
| R2-09 | `tool_hints.rs` bottom-left | **Done** |
| R2-10 | `BuildConfidence` on footprint overlay | **Done** |
| R2-11 | Zone auto-queue on drag release | **Done** |
| R2-12 | Hierarchical toolbox (`egui` collapsing) | **Done** |
| R2-07 | Alt+drag building paint | **Done** (existing) |
| R2-08 | `IntersectionRegistry` stub | **Done** (registry only) |

## Wave C — Tier 3

| ID | Deliverable | Status |
|----|-------------|--------|
| R2-13 | `ConstructionHistory` undo (Ctrl+Z) | **Done** |
| R2-14 | Phase tick + map labels | **Done** |
| R2-15 | Rail spline authority (grade, curve, own pipeline) | **Done** |
