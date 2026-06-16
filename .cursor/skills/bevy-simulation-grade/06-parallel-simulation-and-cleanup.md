# Parallel Simulation + Cleanup

**Repo:** domain modules (`src/systems/`, logistics, fire sim, etc.) · **Authority:** [07-repo-authority-map.md](07-repo-authority-map.md)

## Purpose

Throughput via `par_iter_mut` / chunks without breaking view/render authority.

## Parallel pattern

```rust
fn sim_step(mut q: Query<&mut SimCellState>) {
    q.par_iter_mut().for_each(|mut cell| {
        cell.integrate();
    });
}
```

## Chunked pattern

```rust
#[derive(Component)]
pub struct SimChunk { pub id: u32 }

fn chunked_sim(mut q: Query<(&SimChunk, &mut SimCellState)>) {
    q.par_iter_mut().for_each(|(_, mut cell)| { /* ... */ });
}
```

## Forbidden in parallel workers

- `ResMut<ViewManager>`
- `ResMut<ResolvedViewports>`
- `ResMut<ViewProjectionAuthority>`
- `ResMut` on `FireVisualFrame`, extract buffers, render prep
- `Commands` that spawn view/camera entities (queue on main thread / non-parallel system)

## Async / tasks

Async may compute snapshots; **apply** deltas in designated sync systems on the main schedule — see `coder` agent async rules.

## Cleanup

```rust
#[derive(Component)]
pub struct Lifetime(pub f32);
```

Despawn in explicit cleanup systems **after** sim writers for that domain complete.

## Determinism

Parallel sim must not introduce unordered writes to shared resources — per-entity components only, or reduce in a single-threaded merge system.
