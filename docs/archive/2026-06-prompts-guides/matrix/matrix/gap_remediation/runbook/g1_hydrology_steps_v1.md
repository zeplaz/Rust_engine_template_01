# G1 — Hydrology gap remediation `v1` (Applied — capsule)

> **STATUS: Applied (historical steps archived).** One canonical hydrology implementation (`terrain::generation::hydrology`, D8 + priority-flood) feeds **p4** chunk tags (`flooded`, `eroded`, `silted`) and **ECS** river/lake visuals via `world_generator_enhanced`, with `world_generator_plugin` documenting the legacy non-designer path. Routing matches orchestrator [**§4**](../../../guides/gap_remediation_runbook_v1.md#4-phase-index) **G1 Applied**, terrain matrix [**§3**](../../terrain_biome/material_unification_matrix_v1.md) pass **4**, and gap hunt [**§4.5**](../../../guides/implementation_gap_hunt_runbook_v1.md#45-routing-to-remediation) **G1 Applied**.

**Pair:** orchestrator [`../../../guides/gap_remediation_runbook_v1.md`](../../../guides/gap_remediation_runbook_v1.md) · hunt guide [`../../../guides/implementation_gap_hunt_runbook_v1.md`](../../../guides/implementation_gap_hunt_runbook_v1.md) · terrain matrix [`../../terrain_biome/material_unification_matrix_v1.md`](../../terrain_biome/material_unification_matrix_v1.md) §3 pass 4 · tools parity [`../../../guides/world_assets_tools_rulebook_v1.md`](../../../guides/world_assets_tools_rulebook_v1.md) §1.

**Full atomic history (G1-S01–S06):** [`../../../legacy_runbooks/gap_remediation/g1_hydrology_steps_v1_FULL.md`](../../../legacy_runbooks/gap_remediation/g1_hydrology_steps_v1_FULL.md) — audits and regression archaeology only; **do not** treat this pack as an open execution gap.

**Archive index:** [`../../../legacy_runbooks/README.md`](../../../legacy_runbooks/README.md).

---

## Outcome snapshot

- **`apply_hydrology` / `HydrologyParams`** in [`p4_hydrology.rs`](../../../../src/terrain/generation/passes/p4_hydrology.rs); hydrology module under [`hydrology/`](../../../../src/terrain/generation/hydrology/).
- **Deterministic** flow + accumulation; **tags** wired through `TagSet`; example tag registry lists hydrology tag names.
- **ECS** consumes the same hydrology result as the chunk pipeline — no parallel greedy river/lake model.

---

## Verification (maintenance)

When touching hydrology, p4, or world generation:

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 hydrology_ -- --nocapture`
- `cargo test -p proc_A_dine01 p4_hydrology_ -- --nocapture`

---

## Matrix / routing anchors

| Doc | Where |
|:---|:---|
| Terrain material matrix | §3 pipeline — hydrology / erosion row **Applied** |
| Gap remediation orchestrator | §4 phase index — **G1 Applied** |
| Implementation gap hunt | §4.5 routing — **G1** closed to canonical code paths |
