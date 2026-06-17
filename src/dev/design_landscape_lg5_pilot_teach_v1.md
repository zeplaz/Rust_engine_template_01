# DMCP-PILOT-TEACH-ANNOT-001 — 3-tile LG-5 pilot teach exception

| Field | Value |
|:---|:---|
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |

The pilot batch [`tile_batch_landscape_lg5_pilot_v1.json`](../../tools/mcp/schemas/examples/tile_batch_landscape_lg5_pilot_v1.json) is a **teach exception**:

```json
"_meta": {
  "teaches": ["landscape_lg5", "topology_sprite_extract"],
  "not_a_ship_target": true
}
```

**Rules:**
- `ship: false` · `development_tier: pilot` — never register as production atlas
- Procedural keyframes (seed 550005) prove MCP spine only — not G4 art-ship
- Expanded v1 batch (`tile_landscape_expanded_v1`) supersedes for burn/scar/regrowth matrix

**APS badge:** **Teach** per [`design_aps_preset_qc_criteria_v1.md`](design_aps_preset_qc_criteria_v1.md)
