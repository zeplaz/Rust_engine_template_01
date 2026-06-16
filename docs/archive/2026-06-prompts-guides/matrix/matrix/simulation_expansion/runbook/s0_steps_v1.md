# S0 — Asset simulation ownership — step pack `v1`

> **Parent:** [`../../guides/asset_system_audit_runbook_v1.md`](../../guides/asset_system_audit_runbook_v1.md)  
> **Matrix:** [`../asset_sim_ownership_matrix_v1.md`](../asset_sim_ownership_matrix_v1.md)

Version: `v1.0.1`  
**STATUS:** **Verified** (automated pass 2026-05-06) — all four steps **done**; reopen when matrix rows materially change.

---

## S0-S01 — Freeze matrix column layout + terrain block

| Field | Content |
|:---|:---|
| **Goal** | Terrain registry ownership rows are **Applied** with stable anchor paths. |
| **Anchor reads** | [`asset_sim_ownership_matrix_v1.md`](../asset_sim_ownership_matrix_v1.md); `src/systems/terrain/material_plugin.rs` |
| **Touch** | Matrix markdown only (no Rust required for this step). |
| **Verify** | Table renders; each terrain row has a non-empty anchor. |
| **DoD** | § Matrix terrain rows match grep of `register_asset_loader` in `material_plugin.rs`. |

**Result (2026-05-06):** Six loaders in `material_plugin.rs` match terrain registry rows; parity sentence in matrix **Appendix A**.

---

## S0-S02 — Production stack rows (concrete, power, manufacturing)

| Field | Content |
|:---|:---|
| **Goal** | `ConcreteRuntimePlugin`, `PowerRuntimePlugin`, `ManufacturingCorePlugin` reflected with **Partial** / **Applied** and file anchors. |
| **Anchor reads** | `src/systems/production/runtime.rs`; `src/entities/production/concrete/systems.rs`; `src/entities/production/power/systems.rs`; `src/entities/production/core/manufacturing_plugin.rs` |
| **Touch** | `asset_sim_ownership_matrix_v1.md` |
| **Verify** | `rg "ConcreteRuntimePlugin|PowerRuntimePlugin|ManufacturingCorePlugin" src -g'*.rs'` lists owners consistent with matrix. |
| **DoD** | Matrix rows exist for concrete / plant defs / manufacturing scaffold; status honest (**Partial** where TODO in code). |

**Result (2026-05-06):** Rows for `ConcreteRuntimePlugin`, `PowerRuntimePlugin`, `ManufacturingCorePlugin`, **`AluminumRuntimePlugin`** in matrix v1.2.0; `ProductionRuntimePlugin` composition verified in `runtime.rs`.

---

## S0-S03 — Logistics + resource taxonomy drift note

| Field | Content |
|:---|:---|
| **Goal** | Document **Rust** `ResourceType` vs **Python** `RESOURCE_TYPES` and vehicle cargo; single paragraph + matrix **Drift warning** link. |
| **Anchor reads** | `src/entities/types/p_enumz.rs`; `src/utils/asset_tools/src/config/asset_config.py` (`RESOURCE_TYPES`); `src/entities/vehicles/runtime.rs` |
| **Touch** | `asset_sim_ownership_matrix_v1.md` (already has drift row); optional one-line in [`python_asset_tools_a0_inventory_v1.md`](../python_asset_tools_a0_inventory_v1.md) |
| **Verify** | Spot-check: every `ResourceType` variant appears or is explicitly excluded in Python doc note. |
| **DoD** | Designers know engine truth is **Rust enum** until S8 expands or generates Python. |

---

## S0-S04 — Legacy I/O vs Bevy assets (exit criteria for “audit complete enough for S2+”)

| Field | Content |
|:---|:---|
| **Goal** | Legacy `deserializers.rs` paths flagged **Legacy**; no new gameplay systems should depend on them without matrix row. |
| **Anchor reads** | `src/io/serialization/deserializers.rs`; [`implementation_gap_hunt_runbook_v1.md`](../../guides/implementation_gap_hunt_runbook_v1.md) §2 if triaging |
| **Touch** | `asset_sim_ownership_matrix_v1.md` |
| **Verify** | `cargo check -p proc_A_dine01` |
| **DoD** | Matrix has **Legacy** row; proposal index S3+ work references registries not `.dat` where possible. |

**Result (2026-05-06):** Legacy row present for `deserializers.rs`; `cargo check -p proc_A_dine01` **passes** (warnings only). New features should use registries / components per matrix.

---

## Document history

- **2026-05-06:** Initial S0 step pack (four atomic steps).
- **2026-05-06:** `v1.0.1` — all steps executed against repo; matrix `v1.2.0`.
