# Navigation — validation & performance `04`

## Correctness

- No path through **illegal** edges (wrong direction rail, closed nation) after diplomacy hooks land.
- **Dynamic rebuild** triggers: construction complete, chunk load, disaster — document ordering vs hydrology updates 📎.

## Performance

- **Path cache** TTL by entity class (`implementation_questions_v1.md` §10).
- **Region graph** granularity: chunk grid vs coarser macro regions (`implementation_questions_v1.md` §9).

## Telemetry

- Optional egui rows: path compute time, cache hit rate — align `tools_ui/spec/` when diagnostics exist.
