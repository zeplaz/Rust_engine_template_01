# APS-VALIDATOR-PLAIN-001 — P0 validator codes → artist sentences `v1`

| Field | Value |
|:---|:---|
| **ID** | **APS-VALIDATOR-PLAIN-001** |
| **Owner** | @planner-mcp |
| **Consumer** | APS Assembly tab · `assembly_panel.py` · future `aps_validator_plain.py` |
| **Sign-off** | [`aps_validator_plain_signoff_v1.md`](aps_validator_plain_signoff_v1.md) **SIGNED** |
| **Status** | **SIGNED** — implement via MCP-P0-PLAIN-001 |
| **Date** | 2026-06-03 |

---

## Purpose

Map machine `signature` / `kind` from P0 gate validators to **one sentence** artists understand — no JSON, no field paths in the primary message.

**APS rule:** Show **plain sentence first**, technical `hint` second (expandable).

---

## P0 gate composition

`validate_assembly_p0_gate` = production + grammar + materials ([`assembly_grammar_verify.py`](../../tools/mcp/python/rust_engine_mcp/validators/assembly_grammar_verify.py)).

| Sub-validator | When it runs |
|:---|:---|
| `assembly_production` | Ship tier, GLB paths, module count |
| `assembly_grammar` | Footprint, shell, grammar chain |
| `material_profiles` | ARCH-MAT-001 / APS-MAT-008 |

---

## Production (`assembly_production`)

| signature | kind | Artist sentence | Fix hint (secondary) |
|:---|:---|:---|:---|
| `assembly_production_tier` | TierMismatch | This snapshot is not marked **production** — bake/ship needs production modules. | Set tier to **production** when generating, or regenerate from Assembly. |
| `assembly_production_min_placements` | ModuleCount | The building has too few pieces — it won't read as a structure. | Generate again or widen footprint until you have at least 4 placements. |
| `assembly_production_unique_modules` | ModuleCount | Not enough different module types — looks like a repeated strip. | Check style pack / grammar; need more wall/roof/door variety. |
| `assembly_production_missing_glb` | MissingField | A placement is missing its **3D file path** — the worker can't load it. | Regenerate snapshot or fix module index / promote GLB. |
| `assembly_production_glb_path` | NonProductionGlb | A wall/roof module is still a **greybox or lod0** file — not production art. | Promote module in Catalog (production_run GLB). |
| `assembly_production_lod0_rejected` | Lod0Module | Greybox modules are in the building shell — swap to production GLBs. | Run module promotion batch; refresh Catalog. |
| `assembly_production_glb_missing` | MissingFile | One or more **GLB files are missing on disk**. | Promote or reindex modules; verify path in Catalog. |
| `assembly_graph_material_profile` | MissingField | Some placements have **no material** — assign in APS Material library. | Select each cell → pick profile → Save snapshot. |
| `assembly_production_snapshot_missing` | MissingFile | Snapshot file not found. | Load or Save a valid assembly JSON. |

---

## Grammar (`assembly_grammar` / GRAMMAR-GEN-VERIFY-001)

| signature | kind | Artist sentence | Fix hint |
|:---|:---|:---|:---|
| `grammar_verify_footprint_min` | FootprintTooSmall | Footprint is **too small** to read as a building. | Increase W×D (minimum 3×3). |
| `grammar_verify_warehouse_footprint` | WarehouseFootprintThin | Warehouse footprint is **too narrow** — looks like a fence, not a hall. | Use at least **4×3** for Industrial Warehouse. |
| `grammar_verify_perimeter_count` | PerimeterIncomplete | Building **shell is incomplete** — missing wall or roof ring cells. | Regenerate with grammar on; check massing strategy. |
| `grammar_verify_missing_wall` | MissingWallModule | No **wall modules** in this assembly. | Check grammar / style pack wall slot. |
| `grammar_verify_missing_roof` | MissingRoofModule | No **roof modules** — open-top stack. | Check roof slot in grammar / district. |
| `grammar_verify_grammar_chain` | GrammarChainMissing | Snapshot is missing **grammar history** (how it was generated). | Regenerate with **Use building grammar** checked. |
| `grammar_verify_style_pack_drift` | StylePackDrift | Some modules belong to a **different style pack** than the snapshot. | Regenerate or fix style pack / module index rows. |
| `grammar_verify_snapshot_missing` | MissingFile | Snapshot file not found. | Load valid JSON. |

---

## Materials (`material_profiles` / APS-MAT-008)

| signature | kind | Artist sentence | Fix hint |
|:---|:---|:---|:---|
| `material_profiles_placement_missing` | MissingMaterialProfile | A placement has **no material profile**. | Assembly tab → select cell → Materials library → Save. |
| `material_profiles_unknown_id` | UnknownMaterialProfile | Material **not in registry** — worker may fall back to grey. | Materials tab → Add/Generate profile → Register. |
| `material_profiles_missing_albedo` | MissingTexture | Material is missing **color texture (albedo)**. | Materials tab → Generate or drop `albedo.png` in profile folder. |
| `material_profiles_missing_normal_roughness` | MissingTexture | Material is missing **normal or roughness** maps (ship warning). | Add maps or accept pilot albedo-only for preview. |
| `material_profiles_snapshot_missing` | MissingFile | Snapshot file not found. | Load valid JSON. |

---

## APS display contract (coder-mcp)

```text
P0 gate failed

● Footprint is too small to read as a building.
  → Increase W×D (minimum 3×3).

● 3 placements have no material profile.
  → Assembly tab → select cell → Materials library → Save.

[Show technical details ▼]
```

| Field | Source |
|:---|:---|
| Primary line | This doc `Artist sentence` by `signature` |
| Arrow line | `Fix hint` |
| Technical block | Raw `ValidationIssue.hint` + `field` |

**Implementation ID:** `APS-VALIDATOR-PLAIN-002` (@coder-mcp) — `rust_engine_mcp/aps_validator_plain.py` loader from this markdown table or JSON mirror.

---

## JSON mirror (optional codegen)

Path: `assets/configs/aps/validator_plain_language_v1.json` — future; markdown is authoritative for v1.

---

## References

- [`grammar_iter_001_spec_v1.md`](grammar_iter_001_spec_v1.md)
- [`arch_mat_001_material_authority_v1.md`](arch_mat_001_material_authority_v1.md)
- P0 gate: `assembly_panel.on_validate_p0`
