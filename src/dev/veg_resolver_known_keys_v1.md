# veg_resolver_known_keys_v1 — VegetationExtractFrame authority

| Field | Value |
|:---|:---|
| **Slice** | `CDR-B-VEG-RESOLVER-PARITY-001` |
| **Charter** | `src/dev/plan_veg_variant_key_naming_v1.md` |
| **Engine (burn)** | `variant_key_for_burn_row` · `src/systems/ecology/landscape_grammar_burn.rs` |
| **Engine (stamp)** | `topology_kind_to_variant_key` · `src/systems/ecology/landscape_atlas_registry.rs` |
| **Engine (extract)** | `variant_key_for_extract_row` · `src/render/extraction/vegetation_visual_extract.rs` |
| **Catalog** | `assets/configs/landscape/_vegetation_variant_catalog.ron` |
| **Parity** | PASS |
| **Witness** | `debug_runs/art_pipeline/veg_resolver_parity_live.json` |

## Veg resolver keys (`veg_*`)

Byte-parity set — emitted by `variant_key_for_burn_row`:

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

LG-5 stamp resolver — emitted by `topology_kind_to_variant_key`:

- `topology_patch`
- `topology_corridor`
- `topology_ring`

## Extract topology keys (`veg_topo_*`)

Non-catalog dynamic keys from program topology when `ActiveBurn.heat <= ε`:

- Pattern: `veg_topo_{topology_kind_ascii_lower}` (non-alphanumeric stripped)
- Examples: `veg_topo_patch`, `veg_topo_corridor`, `veg_topo_barrier`
- Fallback (no program): `veg_topo_patch`
- **Not** in byte-parity catalog set

Witness: `debug_runs/landscape_grammar_extract_live.json` (`sprite_variant_from_program: true`).

## Expanded atlas topology rows

LG-5 expanded cells (`topology_*_scar`, `topology_*_burn_*`, regrowth suffixes) are catalog + tile_batch authority — not burn resolver output.

Catalog topology row count: **22**.

## Parity rule

Authored catalog `veg_*` keys must match engine resolver keys **byte-for-byte** (no extras, no omissions). Stamp resolver base keys (`topology_patch`, `topology_corridor`, `topology_ring`) must exist in catalog.

## Q4 sign-off (@coder B)

- **Q4a:** Burn `veg_burn_*` wins over topology tint when `ActiveBurn.heat > ε`.
- **Q4b:** Empty/missing UV allowed pre-LG5 ship; parity scope is resolver-known `veg_*` only.
- **Q4c:** Ship catalog path is `assets/configs/landscape/_vegetation_variant_catalog.ron`.
