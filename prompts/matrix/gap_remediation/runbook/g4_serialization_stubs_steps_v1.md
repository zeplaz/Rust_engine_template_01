# G4 — Serialization stub remediation `v1`

> **STATUS:** **Placeholder pack only.** Serialization gaps must stay aligned with paired serialization work. Do not author Rust steps until the serialization terrain runbook **S** or a production serialization owner exists.

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · serialization matrix [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md) · paired queue [`../../../guides/terrain_paired_runbooks_queue_v1.md`](../../../guides/terrain_paired_runbooks_queue_v1.md).

---

## Scope

G4 owns serialization placeholders surfaced by gap hunt, especially:

- [`../../../../src/systems/production/serialization.rs`](../../../../src/systems/production/serialization.rs) — `ConcreteSerializationPlugin`, `AluminumSerializationPlugin`, `PowerSerializationPlugin` TODO registration bodies.
- Save / DTO registration rows named in [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md).

---

## Promotion blockers

Before writing atomic steps, answer gap-hunt §5:

| Question | Required answer |
|:---|:---|
| Save format | JSON, RON, binary snapshot, or hybrid? Which schema/version owns each DTO? |
| Runtime boundary | Which plugin registers DTOs vs which plugin mutates ECS? |
| Terrain coupling | Does the step depend on terrain §8b serialization pair **S**? |
| Matrix row | Which serialization matrix row flips for concrete, aluminum, or power? |

---

## Placeholder sequencing

When promoted, split by serialized domain:

1. Concrete production config persistence.
2. Aluminum chain persistence.
3. Power topology / plant spec DTOs.
4. Aggregator plugin tests.
5. Serialization matrix close.

No `G4-SNN` atomic steps exist yet by design.
