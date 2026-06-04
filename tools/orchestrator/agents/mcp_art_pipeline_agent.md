# MCP art pipeline agent runbook

**Owner:** `@orchestrator-mcp` sequences · `@coder-mcp` implements tools · `@designer-mcp` specs + sign-off

## Gates G0–G5

| Gate | Proof | Tool / artifact |
|:---|:---|:---|
| G0 | Rules pass | `mcp-production-rules` verdict in signoff YAML |
| G1 | AssetSpecs valid | `spec_validate` / `write-spec` → `assets/staging/specs/` |
| G2 | Geometry jobs run | `geometry_run_job` → `assets/staging/<job_id>/model.glb` |
| G3 | GLB valid | `validate_glb_asset` |
| G4 | Promoted | `promote_staging_module` → `assets/models/modules/<job_id>/` |
| G5 | Indexed | `library_register` → `assets/configs/buildings/_module_index.ron` |

Witness: `debug_runs/art_pipeline/<batch_id>_live.json` — refresh with `write-witness <batch_id>`.

Sign-off: `debug_runs/art_pipeline/<batch_id>_signoff.yaml` (designer-mcp).

Batch manifest: `tools/mcp/schemas/examples/batch_<batch_id>.manifest.json`.

Coder audit: `debug_runs/art_pipeline/<batch_id>_coder_audit.json` + `coder_critique.yaml`.

## Handoff paths

| From | To | When |
|:---|:---|:---|
| orchestrator-mcp | all MCP agents | [`mcp_orchestrator_snap_CURRENT.md`](../../tools/orchestrator/queues/mcp_orchestrator_snap_CURRENT.md) |
| designer-mcp | coder-mcp | New bpy op / G5 wiring / schema drift |
| coder-mcp | designer-mcp | Wave unblocked (`kit_greybox_002` ready) |
| coder-mcp | @coder | Bevy `BuildingDefinition` load from `_module_index.ron` |
| orchestrator-mcp | both | Batch sequencing + gate closure |

## Smoke

```powershell
.\tools\mcp\scripts\smoke_geometry.ps1
.\tools\mcp\scripts\verify_mcp_setup.ps1
cd tools\mcp\python
python -m pytest tests/ -q
```

## Development tier (PLAN-MODULE-KIT-PRODUCTION-001)

| Tier | Batch prefix | StylePack |
|:---|:---|:---:|
| smoke | `kit_greybox_*` (legacy, frozen) | No |
| lod0 | `kit_lod0_*` | Explicit |
| production | `kit_production_*` | Yes |

**Stop `kit_greybox_004+`.** Use `validate_asset_report` — not verts-only green.

G3 for non-smoke: tier + silhouette rules pass. G5: index includes `development_tier`.

## Do not

- Paste bpy in chat as implementation
- Promote without validate (use `--force` only with witness flag)
- Execute `tile.generate` (draft JSON only until shipped)
- Start new `kit_greybox_*` batches (smoke harness retired from production paths)
- Treat 24-vertex cubes as pitched roofs / arched windows for StylePack
