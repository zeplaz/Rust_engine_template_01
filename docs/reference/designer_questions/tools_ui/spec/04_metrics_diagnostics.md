# Tools UI — metrics & diagnostics `04`

**Impl questions:** §1–2, §5–6 in `implementation_questions_v1.md`.

## Resource inventory (target)

- `FrameTimeStats`, `ChunkStreamerStats`, `EcsEntityCounts` — define fields when implementing; update in **PostUpdate** or dedicated schedule.

## Optional profilers

- Tracy or similar — **feature-gated** dependency.

## Panels

- egui table rows for latency, drop counters (serialization queue — `terrain_world/implementation_questions_v1.md` §5).
