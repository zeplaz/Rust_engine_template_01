# Greybox production tile freeze (TILE-FIX-001)

**Do not promote or register** these atlases in `_tile_atlas_index.ron`.

| Content | Location |
|:---|:---|
| Packed atlases | `atlases/*_production_v1_atlas.png` |
| Batch meta + status | `staging/tile_*_production_v1/` |
| Keyframe still folders | `keyframe_stills/` (if moved) |
| Move log | `MOVED_LOG.json` |

**Replacement:** atlas schema v2 (`variant × facing × frame`) — see `tools/mcp/schemas/atlas_meta_v2.schema.json`.

Restore for debug only: copy row from `_tile_atlas_index_archive.ron` into active index **and** fix art pipeline — not for ship.
