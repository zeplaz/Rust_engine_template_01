# Power grid art — downstream witness & exit gates `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-POWER-GRID-ART-ASSETS-001 · downstream wave |
| **Queue** | [`power_grid_art_downstream_queue.json`](../tools/orchestrator/queues/power_grid_art_downstream_queue.json) |

---

## Anti–fake-green rules

1. **DMCP spec PASS ≠ ship** — `DMCP-SPEC-*-001 done` means JSON spec only; batch/promote rows still required.
2. **Promote row needs promoted GLB path** in `assets/models/modules/` — staging alone fails.
3. **validate_asset_report pass** on production GLB before Q✓ on batch rows.
4. **Catalog authority** — `grid_substation.json` / `grid_distribution_transformer.json` must reference production `model.glb` after promote.
5. **HUD atlas** — `icon_count >= 13` from design_hud_power_icons_v1 §1; lib test + texture on disk.
6. **Nuclear** — DMCP-SPEC-NUCLEAR-PWR-001 is **spec-only** in this wave; no bpy until spec witness green.

---

## Row witnesses (summary)

| Row | Witness | Must prove |
|:---|:---|:---|
| MCP-PWR-UTILITY-MANIFEST-001 | `mcp_pwr_utility_manifest_live.json` | manifest ≥8 modules · both specs valid |
| MCP-PWR-SUBSTATION-BATCH-001 | `mcp_pwr_substation_batch_live.json` | staging GLB · asset_glb pass · ≥4 modules baked |
| MCP-PWR-TRANSFORMER-BATCH-001 | `mcp_pwr_transformer_batch_live.json` | staging GLB · 2×2 grid |
| MCP-PWR-PROMOTE-SUBSTATION-001 | `mcp_pwr_substation_promote_live.json` | promoted · registry grid_substation |
| MCP-PWR-PROMOTE-TRANSFORMER-001 | `mcp_pwr_transformer_promote_live.json` | promoted · supersedes lod0 |
| DMCP-SPEC-NUCLEAR-PWR-001 | `dmcp_nuclear_pwr_spec_live.json` | kit_nuclear_pwr 6×6 spec file |
| COD-ART-HUD-ICON-ATLAS-001 | `sim_hud_power_icons_live.json` | atlas PNG · 13+ icons registered |
| PWR-ART-DOWNSTREAM-CLOSE-001 | `power_grid_art_downstream_close_live.json` | WIT-HON rollup |

---

## Verification commands

```powershell
# MCP specs (manifest row)
cd tools/mcp/python
python -m rust_engine_mcp.cli validate-report mcp_spec ../../assets/staging/specs/kit_substation_yard_production_001.json --compress 3
python -m rust_engine_mcp.cli validate-report mcp_spec ../../assets/staging/specs/prop_transformer_production_run001.json --compress 3

# After bpy
python -m rust_engine_mcp.cli validate-report asset_glb ../../assets/staging/.../model.glb --compress 3

# HUD
cargo test -p proc_A_dine01 --lib icon_atlas power_hud_icons

# Close
python -m rust_engine_mcp.cli validate-report witness_honesty debug_runs/art_pipeline/power_grid_art_downstream_close_live.json --compress 3
```

---

## Sim coupling (Power P1 minimum)

When promote rows close, refresh:

`debug_runs/industrial_activation_live.json` → utility GLB paths non-null for substation + transformer.

Do **not** mark Power P1 art green on catalog JSON alone.
