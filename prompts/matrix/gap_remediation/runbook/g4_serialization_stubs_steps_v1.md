# G4 — Serialization stub remediation `v1`

> **STATUS:** **Hybrid save direction is locked** in [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md) §**Locked direction** (header + **binary** bulk body; small text header). Remaining open items there (header format, endianness/compression, per-chunk vs monolithic) are **matrix-level `ASK:` / 📎**, not G4 blockers. Author `G4-SNN` steps when **wave S** + this pack agree on touchpoints; **terrain pair S** only when the step actually persists **terrain/world** slice — otherwise **no forced link**.

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · **source of truth** [`../../serialization/serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md) · terrain pairing [`../../../guides/terrain_paired_runbooks_queue_v1.md`](../../../guides/terrain_paired_runbooks_queue_v1.md) (when relevant).

**Execution gate:** [`../../../guides/rulebook_backlog_designer_brief_v1.md`](../../../guides/rulebook_backlog_designer_brief_v1.md) §4 **BQ-110** — first domain row + code/test alignment with wave **S** before authoring `G4-SNN` steps.

---

## Scope

G4 owns serialization placeholders surfaced by gap hunt, especially:

- [`../../../../src/systems/production/serialization.rs`](../../../../src/systems/production/serialization.rs) — `ConcreteSerializationPlugin`, `AluminumSerializationPlugin`, `PowerSerializationPlugin` registration bodies.
- Additional production / economy DTO rows as they appear in the hybrid matrix — **concrete, aluminum, and power are examples**, not an exhaustive closed list.

---

## Recorded answers (replaces promotion table)

| Topic | Decision |
|:---|:---|
| **Save format** | **Hybrid** — per [`serialization_hybrid_migration_matrix_v1.md`](../../serialization/serialization_hybrid_migration_matrix_v1.md): textual **header** (metadata / version) + **binary** body for bulk snapshot; building/vehicle configs remain **RON**-oriented per asset migration matrix. |
| **Runtime boundary** | **Truth in code:** registration and loaders live beside the plugins that own each domain (`serialization.rs` + `deserializers.rs`); **mutating ECS** stays in simulation plugins — deserialization **hydrates**, it does not silently invent gameplay rules. Adjust per-step comments in the touched module. |
| **Terrain coupling** | Link **terrain §8b / wave S** only when this step reads or writes **terrain/world/chunk** save artifacts. **Production-only** steps do not need a terrain dependency for paperwork’s sake. |
| **Matrix row “which one?”** | **Not a single row.** Concrete, aluminum, power, and **future** domains each get **their own** hybrid-matrix / migration rows as implementation expands — **add rows** when new DTOs appear; **flip status** per domain in that matrix. G4 tracks **code touchpoints + matrix** together, not “pick one row.” |

---

## When to add an atomic step

- The hybrid matrix names the DTO and version policy for the domain.
- `src/io/serialization/` + owning plugin agree on register/load path.
- A **test** proves load → ECS boundary (or save round-trip) for that DTO slice.

---

## Placeholder sequencing

When promoted, split by **serialized domain** (extend matrix as needed):

1. Concrete production config persistence (+ matrix row).
2. Aluminum chain persistence (+ matrix row).
3. Power topology / plant spec DTOs (+ matrix row).
4. Additional domains as they appear **aluminum/concrete/power are not the cap**.
5. Aggregator plugin tests and matrix close for touched rows.

No `G4-SNN` atomic steps exist until the **first** domain row in §Placeholder is executed with code + test alignment.
