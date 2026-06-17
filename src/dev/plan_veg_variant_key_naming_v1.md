# PLAN-VEG-RESOLVER-KEY-NAMING-001 — variant_key naming vs VegetationExtractFrame resolver `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-VEG-RESOLVER-KEY-NAMING-001
Date: 2026-06-16
Status: **SIGNED** (@planner-mcp 2026-06-16 · re-verified 2026-06-17)
Schema witness: `debug_runs/mcp_aps_e3_veg_catalog_schema_sign_live.json`
Owner: @planner-mcp → @coder B (parity) · @designer-mcp (catalog rows)
Parent: APS-EVO-E3 · $ref:src/dev/plan_veg_burn_extract_001_v1.md §6
Schema: $ref:tools/mcp/schemas/vegetation_variant_catalog_v1.schema.json
Engine: $ref:src/systems/ecology/landscape_grammar_burn.rs::variant_key_for_burn_row
```

**Goal:** Lock **canonical `variant_key` strings** so authored catalog rows, tile_batch variants, and `VegetationExtractFrame` resolver output are **byte-identical** at parity time.

---

## Q1 — Prefix policy (DECIDED)

| Prefix | Use | Owner | Example |
|:---|:---|:---|:---|
| `topology_*` | LG-5 **sprite slots** tied to topology kind (Patch/Corridor/Ring) | @designer-mcp atlas | `topology_patch` |
| `veg_*` | **Sim state** keys from burn/succession/regrowth resolver | @coder A extract | `veg_burn_03`, `veg_regrowth_nuclei` |

**Rule:** Never mix prefixes for the same semantic row. Atlas UV slots reference `topology_*` when stamping chunk heterogeneity; burn overlay rows reference `veg_*` even on the same chunk.

**Rejected:** single flat namespace without prefix — breaks atlas index grep and building `burning_*` parallel.

---

## Q2 — Burn frame formatting (DECIDED)

| Field | Value |
|:---|:---|
| Pattern | `veg_burn_{:02}` |
| Count | `8` (`VEG_BURN_FRAME_COUNT` in engine) |
| Range | `veg_burn_00` … `veg_burn_07` |
| Tick driver | `veg_burn_frame_index(tick, seed)` — same seed + tick → same index |

Catalog may list **sparse** entries (00, 01, 07) in examples; ship catalog must enumerate **all 8** before G4 art-ship.

---

## Q3 — Resolver precedence (DECIDED)

Match order in `variant_key_for_burn_row` (engine is authoritative):

```text
1. ActiveBurn.heat > ε           → veg_burn_{frame:02}
2. ActiveBurn.regrowth_macro     → veg_regrowth_* | veg_damaged | veg_clean_day
3. SuccessionTopologyStage       → veg_damaged (BurnScar) | veg_old_growth | veg_clean_day
4. default                       → veg_clean_day
```

Topology kind → `topology_*` is resolved **separately** in `topology_kind_to_variant_key` for LG-5 stamp path — not merged into burn resolver.

---

## Q4 — Open for @coder sign-off

| # | Question | Proposed default | Coder action |
|:---:|:---|:---|:---|
| **Q4a** | When both topology stamp and burn active, which wins for **minimap tint**? | Burn `veg_burn_*` overrides topology tint | Confirm in `VegetationExtractFrame` row merge |
| **Q4b** | Empty `variant_key` allowed pre-LG-5? | Yes until catalog file on disk | Parity test: authored ⊆ resolver-known |
| **Q4c** | RON on-disk path | `assets/configs/landscape/_vegetation_variant_catalog.ron` | Loader extension dispatch like other configs |

**@coder B sign-off (CDR-B-VEG-RESOLVER-PARITY-001):** **CONFIRMED** 2026-06-17
- **Q4a:** Burn `veg_burn_*` wins over topology tint when `ActiveBurn.heat > ε` on extract row.
- **Q4b:** Empty/missing UV allowed pre-LG5 ship; parity scope is resolver-known `veg_*` ⊆ catalog.
- **Q4c:** Ship catalog path is `assets/configs/landscape/_vegetation_variant_catalog.ron` (RON on disk, byte parity green).

Extract program rows emit `veg_topo_{kind}` (non-catalog dynamic keys) — documented in `veg_resolver_known_keys_v1.md`; not part of byte parity set.

---

## Acceptance

| # | Criterion | Pass |
|:---:|:---|:---:|
| A1 | Schema `variant_key` pattern matches table | `vegetation_variant_catalog_v1.schema.json` |
| A2 | Example catalog validates against schema | `vegetation_variant_catalog_pilot_v1.json` |
| A3 | Every engine-emitted key in §Q2–Q3 has catalog row or explicit `default` | designer-mcp DMCP-E3 |
| A4 | Parity witness `aps_veg_extract_parity_live.json` green | @coder-mcp E5 |

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-16 | **SIGNED** — prefix + precedence locked |
| **@planner-mcp** | 2026-06-17 | **Q✓** — schema + ship catalog (29 keys, 8 burn) validate; unblocks CDR-B parity |
| **@coder B** | 2026-06-17 | **SIGNED** Q4a–c — witness `debug_runs/art_pipeline/veg_resolver_parity_live.json` |

```text
⟦/PLAN-VEG-RESOLVER-KEY-NAMING-001⟧  ΔWF→ DMCP-E3-VARIANT-KEY-SET-001 · CDR-B-VEG-RESOLVER-PARITY-001
```
