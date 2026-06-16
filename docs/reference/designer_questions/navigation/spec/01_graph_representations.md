# Navigation — graph & field representations `01`

## Questions resolved in checklist

- **RoadGraph / RailGraph vs `TransportGraph`:** document chosen split in `implementation_questions_v1.md` §1 after decision.
- **Flow field:** dense vs sparse; resolution ladder per LOD — tie to explicit table 📎 in impl Q §2.

## Serializable vs ECS

- **Static topology** (planned roads/rail): Serializable-friendly or bake-time asset 📎.
- **Runtime blockages:** ECS components or bitmask layers updated by sim; rebuild rules in `04_validation_perf.md`.

## Terrain coupling

- Graph nodes snap to **tile/chunk** coordinates; elevation and bridge/tunnel flags 📎 when terrain API stabilizes.
