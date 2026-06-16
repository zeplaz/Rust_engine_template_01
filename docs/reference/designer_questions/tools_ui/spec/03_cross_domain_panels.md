# Tools UI — cross-domain panels `03`

**Source:** [`../tooling_cross_domain_v1.md`](../tooling_cross_domain_v1.md).

## Domains (tabs or dock)

- **World gen:** params, preview — tie `terrain_world/spec/01_world_generation_pipeline.md`.
- **Production:** manifest + domain picker — `production_economy/spec/02_tools_ui_production.md`.
- **Vehicles:** road vehicle tools if present (`RoadVehicleToolsUiPlugin`).
- **Platforms / EW:** placeholder until strategic sim lands — align `strategic_platforms/spec/`.
- **Navigation debug:** optional path overlay 📎.

## Consistency

- Shared **selection** model (picked `EntityId`) across tabs when feasible.
