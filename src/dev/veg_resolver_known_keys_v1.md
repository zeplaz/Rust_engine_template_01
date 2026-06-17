# veg_resolver_known_keys_v1 — VegetationExtractFrame authority

| Field | Value |
|:---|:---|
| **Slice** | `CDR-B-VEG-RESOLVER-PARITY-001` |
| **Engine** | `variant_key_for_burn_row` · `topology_kind_to_variant_key` |
| **Catalog** | `assets/configs/landscape/_vegetation_variant_catalog.ron` |
| **Parity** | PASS |

## Veg resolver keys (`veg_*`)

Emitted by `variant_key_for_burn_row` in `src/systems/ecology/landscape_grammar_burn.rs`:

- `veg_clean_day`
- `veg_old_growth`
- `veg_damaged`
- `veg_regrowth_nuclei`
- `veg_regrowth_front`
- `veg_burn_00`
- `veg_burn_01`
- `veg_burn_02`
- `veg_burn_03`
- `veg_burn_04`
- `veg_burn_05`
- `veg_burn_06`
- `veg_burn_07`

## Topology stamp keys (`topology_*`)

Emitted by `topology_kind_to_variant_key` in `src/systems/ecology/landscape_atlas_registry.rs`:

- `topology_patch`
- `topology_corridor`
- `topology_ring`

## Expanded atlas topology rows

LG-5 expanded cells (`topology_*_scar`, `topology_*_burn_*`, regrowth suffixes) are catalog + tile_batch authority — not burn resolver output.

Catalog topology row count: **22**.

## Parity rule

Authored `veg_*` catalog keys must match engine resolver keys **byte-for-byte** (no extras, no omissions).

