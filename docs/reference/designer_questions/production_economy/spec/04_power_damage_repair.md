# Production — power, damage, repair `04`

**Canonical detail:** [`../power_damage_ui_persistence_v1.md`](../power_damage_ui_persistence_v1.md).

## Summary hooks

- **Frequency / phase / islanding** — formulas and tick costs in Serializable tunables when possible.
- **Repair queue:** global vs per-facility aggregation for UI (`implementation_questions_v1.md` §5).
- **Component damage:** bitmask / reflection for inspectors.

## Alerts

- ECS vs `AlertLog` resource — choose and document in impl Q §8.
