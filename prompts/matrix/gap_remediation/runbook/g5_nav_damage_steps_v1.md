# G5 — Navigation / damage / manufacturing remediation `v1`

> **STATUS:** **Placeholder pack only.** These gaps cross multiple owners. Do not author Rust steps until each finding has a matrix/spec owner and a release priority.

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · navigation implementation questions [`../../../designer_questions/navigation/implementation_questions_v1.md`](../../../designer_questions/navigation/implementation_questions_v1.md) · production specs [`../../../designer_questions/production_economy/spec/README.md`](../../../designer_questions/production_economy/spec/README.md).

---

## Scope

G5 owns non-GUI gameplay placeholders surfaced by gap hunt, especially:

- [`../../../../src/systems/navigation/potental_feild_nav.rs`](../../../../src/systems/navigation/potental_feild_nav.rs) — owner defaults until `AgentOwnable` is on vehicle entities.
- [`../../../../src/systems/damage/damage_system.rs`](../../../../src/systems/damage/damage_system.rs) — road damage accumulation TODO.
- [`../../../../src/entities/production/core/manufacturing_plugin.rs`](../../../../src/entities/production/core/manufacturing_plugin.rs) — throughput / decay / alert placeholder.

---

## Promotion blockers

Before writing atomic steps, answer gap-hunt §5:

| Question | Required answer |
|:---|:---|
| Owner | Navigation, damage, production runtime, or shared sim-control? |
| Player-visible behavior | What changes in movement, damage state, throughput, or alerts? |
| Data source | Which component/resource supplies owner, damage inputs, blueprints, or tick cadence? |
| Matrix row | Which navigation / damage / production matrix row flips? If none exists, write `ASK:` first. |

---

## Placeholder sequencing

When promoted, split by subsystem:

1. Potential-field owner derivation.
2. Road damage accumulation.
3. Manufacturing throughput / efficiency.
4. Alert integration.
5. Matrix/spec close.

No `G5-SNN` atomic steps exist yet by design.
