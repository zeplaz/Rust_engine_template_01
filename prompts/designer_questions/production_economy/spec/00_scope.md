# Production & economy — scope `00`

## In scope

- **Manufacturing domains** in matrix: concrete, power, aluminum, core — Serializable configs + ECS runtime per `repo_boundary_matrix`.
- **Damage & repair** integration with production facilities and UI alerts.
- **Player vs tools UI:** Bevy HUD charts vs egui dev panels (`power_damage_ui_persistence_v1.md`).

## Cross-subsystem

- **Logistics:** output feeds `terrain_world/spec/05_logistics_settlement_coupling.md` (cargo, stockpiles 📎).
- **Strategic:** magazines / munition manufacturing link `strategic_platforms/spec/05_integration_phases.md`.

## Out of scope / defer

- Real-world **energy market** pricing; keep tunables in data.
