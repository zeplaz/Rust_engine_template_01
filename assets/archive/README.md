# Archive — protected quarantine (never delete)

Artifacts moved here are **retired from runtime indexes** but **kept on disk** for diff, witness replay, and artist reference.

## Rules

1. Every move is logged in `archive/<bundle>/MOVED_LOG.json`.
2. Run [`tools/mcp/scripts/organize_texture_assets.ps1`](../../tools/mcp/scripts/organize_texture_assets.ps1) — supports `-WhatIf`.
3. Do not hand-delete PNG/GLB without updating the manifest and git history.
4. Engine loads **production** paths from `_tile_atlas_index.ron` only; archive index is documentation + optional manual restore.

## Bundles

| Folder | Contents |
|:---|:---|
| [`lod0_tile_pilots_2026-06/`](lod0_tile_pilots_2026-06/MANIFEST.yaml) | APS headless pilot building atlases + staging mirrors |

To restore a pilot to active index: copy manifest paths back, re-add row to `_tile_atlas_index.ron`, and set `development_tier: lod0` (still not ship — use for debug only).
