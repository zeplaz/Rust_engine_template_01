# U3–U7 terrain unification — applied maintenance `v1`

> **STATUS:** **Applied / maintenance** — execution trajectory for **U3–U7** is closed for mainline per [`../../matrix/terrain_biome/material_unification_matrix_v1.md`](../../matrix/terrain_biome/material_unification_matrix_v1.md) §10 and [`../../guides/terrain_unification_runbook_v1.md`](../../guides/terrain_unification_runbook_v1.md) §4. Use this doc for **audits, onboarding, and extensions**; do not treat U-packs as an open execution gap.

**Archive:** Full historical atomic steps remain in [`../../matrix/terrain_biome/runbook/u3_steps_v1.md`](../../matrix/terrain_biome/runbook/u3_steps_v1.md) … [`u7_steps_v1.md`](../../matrix/terrain_biome/runbook/u7_steps_v1.md). **Zero-loss index:** [`../README.md`](../README.md).

---

## When to open a U-pack again

| Situation | Open |
|:---|:---|
| Regression or determinism audit | Relevant `uN_steps_v1.md` § verify commands only |
| New material / tag / rule schema (extends U3) | `u3_steps_v1.md` + matrix §3 |
| New chunk pass (extends U4 pipeline) | `u4_steps_v1.md` + matrix §3 |
| Preview / plugin wiring (U5) | `u5_steps_v1.md` |
| Feature **`bevy_tilemap_adapter`** (U6) | `u6_steps_v1.md` (optional build) |
| Invalidation / profiles / dev trace (U7) | `u7_steps_v1.md` |

---

## Canonical code / config anchors

- Chunk pipeline: `src/terrain/generation/passes/`, `ChunkCellMatrix`, `p4_hydrology`
- Material / tags / rules: `src/terrain/material/`, `assets/config/terrain/`
- Plugin: `MaterialUnificationPlugin`, world gen binary parity per [`../../guides/world_assets_tools_rulebook_v1.md`](../../guides/world_assets_tools_rulebook_v1.md) §1

---

## Verification snapshot (post-Applied)

Run when touching terrain generation or configs:

- `cargo check -p proc_A_dine01`
- `cargo test -p proc_A_dine01 -- --nocapture` (scope to hydrology / material tests when iterating)

Hydrology gap phase **G1** is **separate** and **Applied**; see [`../../matrix/gap_remediation/runbook/g1_hydrology_steps_v1.md`](../../matrix/gap_remediation/runbook/g1_hydrology_steps_v1.md) for the capsule + [`../gap_remediation/g1_hydrology_steps_v1_FULL.md`](../gap_remediation/g1_hydrology_steps_v1_FULL.md) for full historical steps.
