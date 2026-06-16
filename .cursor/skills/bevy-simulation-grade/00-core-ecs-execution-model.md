# ECS Execution Model (Conceptual + Repo)

## Purpose

Deterministic spine: parallel simulation, authority boundaries, view-driven rendering, cleanup-safe transitions.

**Implementation detail:** [07-repo-authority-map.md](07-repo-authority-map.md).

## Core principle

> No system writes without declaring ownership; no resource without a single authority.

## Conceptual phases (`CoreSystemSet` vocabulary)

Use these names in **plans and reviews**. Wire systems using **repo sets** (07), not a global `CoreSystemSet` enum.

```rust
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum CoreSystemSet {
    Input,
    ViewSync,
    Simulation,
    AuthorityResolve,
    Extraction,
    RenderPrep,
    Cleanup,
}
```

### Mapping to this repo (approximate)

| Conceptual | Repo anchor |
|------------|-------------|
| Input | `MapCameraSystemSet::ApplyInput`, UI collect |
| ViewSync | `ViewportPipelineSet::Resolve`, minimap intent |
| AuthorityResolve | `ViewAuthoritySystemSet::SyncViewManager` |
| Extraction | `FireVisualFrameSet::*`, `build_view_representation_snapshot` |
| RenderPrep | `ViewRepresentationSystemSet::RenderTargets` / `WorldRender` |
| Simulation | Domain sets (logistics, fire sim, construction tick) — own planners |
| Cleanup | Despawn / lifetime systems in domain modules |

## Example resources (domain pattern)

```rust
#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);

#[derive(Resource, Default)]
pub struct FrameDelta(pub f32);
```

## Parallel safe pattern

```rust
fn parallel_simulation_system(mut q: Query<&mut SimCellState>) {
    q.par_iter_mut().for_each(|mut cell| {
        cell.tick();
    });
}
```

**Forbidden inside workers:** `ResMut<ViewManager>`, `ResMut<ResolvedViewports>`, `ResMut<ViewProjectionAuthority>`, render extract resources.

## Cleanup

Run despawn / lifetime decay in explicit **Cleanup** phase or domain set **after** sim writers finish — never interleave with extraction readers on the same entities without ordering.

## Schedule layout (illustrative only)

```rust
// Conceptual — NOT copy-paste into App without mapping to 07
app.configure_sets(Update, (
    CoreSystemSet::Input,
    CoreSystemSet::ViewSync,
    CoreSystemSet::Simulation,
    CoreSystemSet::AuthorityResolve,
    CoreSystemSet::Extraction,
    CoreSystemSet::RenderPrep,
    CoreSystemSet::Cleanup,
).chain());
```
