# Cities, settlements & AI urban growth `04`

## Procedural **city gen** (worldgen)

- **Site selection:** slope, water access, distance to resources/trade 📎; deterministic from seed + constraints.
- **Macro layout:** **arterial graph** + blocks (grid or organic); reserve **port/berth**, **airfield**, **railyard** slots for `strategic_platforms/spec/01_platform_taxonomy.md`.
- **Zoning hints:** residential / industrial / port — data-driven palette; feeds production domain placement later.

## AI **plans** to build cities “correctly”

- **Planner layers:** (1) demand signals (population/job/production), (2) **legal constraints** (terrain, protected land), (3) **connectivity** (nav graph hooks), (4) **services** (power, water 📎).
- Growth is **staged:** expansion rings or infill; avoid overlapping infrastructure — validation similar to `faction_editor/04_validation.md` (graph rules).
- **Server authority** for MP: city growth commands replicated like construction.

## Tooling

- City brush / “regenerate district” lives next to terrain tools (`terrain_tools_brushes_v1.md`) or dedicated **world tools** row in `tools_ui/tooling_cross_domain_v1.md`.

## Persistence

- City graph in save vs regenerated from seed 📎 — must be explicit to avoid MP desync.
