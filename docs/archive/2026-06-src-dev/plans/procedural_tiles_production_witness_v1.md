# Procedural tiles production — witness spec `v1`

| Field | Value |
|:---|:---|
| **Program** | [`plan_procedural_building_tiles_production_v1.md`](plan_procedural_building_tiles_production_v1.md) |
| **Envelope** | [`debug_run_envelope.rs`](../dev/debug_run_envelope.rs) — `_agent_meta` required |

---

## Witness files

| Witness | Writer | When |
|:---|:---|:---|
| `debug_runs/art_pipeline/procedural_tiles_production_bake_live.json` | `write-witness` / pytest | After PT-2 real production bake |
| `debug_runs/procedural_tiles_runtime_live.json` | sim harness / lib test export | After PT-4 resolver |
| `debug_runs/art_pipeline/procedural_tiles_production_program_green_live.json` | orchestrator-mcp rollup | TILE-PROD-001…006 all pass |

---

## Gate predicates

### TILE-PROD-001 (bake)

- `bake_source == "keyframe_pack"` in batch_status / witness
- `dry_run == false`
- Every entry `development_tier == "production"`
- PNGs from keyframe workflow (not smoke_ortho_headless-only)
- `variant_count >= 6` per atlas
- Includes at least one `clean_night_on` and one `burning_00` key
- PNG dimensions ≥ 128×128 per variant (not 1×1 stub)

### TILE-PROD-002 (registry)

- `TileAtlasRegistry::load_errors` empty
- `resolve_variant_uv` succeeds for all keys in archetype variant matrix YAML

### TILE-PROD-003 (resolver)

- Table cases in witness: `{ phase, power, night, damage, fire_heat } → variant_key`
- `smoke_fallback_used: false`
- `animation_frame` set when variant_key starts with `burning_`

### TILE-PROD-004 (designer)

- Per-atlas `*_production_signoff.yaml` with `pass: true`
- Rubric: night lights readable at iso scale

### TILE-PROD-005 (FULL_APP)

- `readiness.procedural_tiles_production: true`
- Sample site stamp visible in tactical map capture (G-PROOF-01)

### TILE-PROD-006 (play)

- G-PLAY scenario includes fire → `burning_*` frame advances over ≥2s sim time

---

## Program green

`procedural_tiles_production_program_green_live.json`:

```json
{
  "green": true,
  "program_id": "PLAN-PROC-TILE-PROD-001",
  "production_atlas_count": 4,
  "lod0_atlas_ship_allowed": false
}
```

`green` is true only when all six gates pass.
