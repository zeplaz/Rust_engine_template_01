# G2 — Power placeholder remediation `v1`

> **STATUS:** **Placeholder pack only** for **Rust**. **G2-S00** (doc-only traceability) may run immediately; do not execute behavioral Rust steps until designer / matrix answers identify exact player-visible behavior and save/schema impact.

**Queue note:** Parked behind the **G3 GUI** wave per [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) §10. G2-S00 remains valid traceability work.

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · power parity matrix [`../../production/power_legacy_functional_parity_v1.md`](../../production/power_legacy_functional_parity_v1.md) · production migration matrix [`../../production/production_migration_matrix_v1.md`](../../production/production_migration_matrix_v1.md).

---

## Scope

G2 owns power-related placeholders found by the gap hunt, especially:

- [`../../../../src/entities/production/power/failure_modes.rs`](../../../../src/entities/production/power/failure_modes.rs) — `steam_system_placeholder`, `nuclear_containment_placeholder`, `variable_renewable_placeholder`.
- [`../../../../src/systems/production/power_systems.rs`](../../../../src/systems/production/power_systems.rs) — comment-only pointer to placeholders / legacy behavior.

---

## Promotion blockers

Before writing atomic steps, answer gap-hunt §5:

| Question | Required answer |
|:---|:---|
| Player-visible behavior | What should steam leak / condenser, nuclear containment / decay heat, and renewable derates do to status, efficiency, alerts, and UI? |
| Save/schema impact | Are failure modes persisted as deterministic runtime state, derived from config, or replayed from events? |
| Matrix row | Which power parity rows flip when each failure mode lands? |
| Owner | Power runtime, damage, or production tools? |

---

## G2-S00 Parity matrix traceability (doc-only)

**Goal:** Map `failure_modes.rs` placeholder systems to accountable rows in the power parity matrix so the first Rust steps after promotion do not sprawl.

**Anchor reads:** orchestrator §§1-2 · gap hunt §4.5 · [`../../production/power_legacy_functional_parity_v1.md`](../../production/power_legacy_functional_parity_v1.md) · [`../../../../src/entities/production/power/failure_modes.rs`](../../../../src/entities/production/power/failure_modes.rs).

**Touch:** [`../../production/power_legacy_functional_parity_v1.md`](../../production/power_legacy_functional_parity_v1.md) — add §8 *G2 gap remediation traceability* (table provided there).

**Verify:** Doc-only — optional `cargo check -p proc_A_dine01` (expect unchanged).

**Matrix / routing update:** None; orchestrator G2 row stays **Pending** until a Rust step lands.

**Definition of done:**
- [ ] Power parity matrix §8 exists and lists all three placeholder fns + capability markers + §5 anchor.
- [ ] No gameplay numbers or new save fields invented.

---

## Placeholder sequencing

When promoted, split into one phase or sub-pack per capability marker:

1. Steam cycle derates.
2. Nuclear containment / scram / decay heat.
3. Variable renewable resource coupling.
4. Alerts / UI summaries.
5. Matrix close.

No `G2-SNN` atomic **Rust** steps exist yet by design. **G2-S00** above is doc-only and safe to run before promotion.
