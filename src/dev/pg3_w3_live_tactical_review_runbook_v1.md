# PG-3 — W3 live tactical review runbook `v1`

| Field | Value |
|:---|:---|
| **ID** | **PG3-W3-RUNBOOK-001** |
| **Status** | **ACTIVE** |
| **Blocks** | PG-4 grammar · production tile swap at scale |
| **Sign-off YAML** | [`debug_runs/art_pipeline/procedural_assembly_pg2_signoff.yaml`](../../debug_runs/art_pipeline/procedural_assembly_pg2_signoff.yaml) |

---

## Prerequisite (code — not blocking)

| Gate | Witness | Status |
|:---|:---|:---|
| **PROC-PG-3-001** | `construction_procedural_build_001` in `debug_runs/construction_stage_live.json` | **Green** when `pg2_spawn_wired` + `commit_carries_spec` |
| PG-2 spawn at Operational | `procedural_pg2_spawn_witness_green()` | Lib + harness |

**Trigger text in sign-off YAML is stale.** Spawn is wired; W3 is a **15 min designer viewport pass**, not waiting on `@coder`.

---

## What to compare (side-by-side)

| Field | Value |
|:---|:---|
| Footprint | Same W×D×floors (use **4×3** rowhouse or **4×2** witness) |
| Pack A | `style_victorian` |
| Pack B | `style_industrial_west` |
| Viewport | Tactical zoom default |
| Brightness note | Keyframe `clean_night_on` uses boosted emissive (PG-3 tune); may read brighter than old lod0 ortho pilot |

---

## Steps (FULL_APP)

1. Start sim → commit two sites same footprint, different `ProceduralBuildingSpec.style` (or use growth inspector if seeded).
2. Advance both to **Operational** (site stage tick) — confirm module GLBs spawn (not smoke).
3. Side-by-side tactical view — rubric W2–W5 from [`design_procedural_assembly_read_v1.md`](design_procedural_assembly_read_v1.md).
4. Optional: compare **map iso stamp** if production atlas registered for that assembly.
5. Capture screenshots → `assets/vfx/reference/review_captures/` or `debug_runs/art_pipeline/w3_captures/`.
6. Update sign-off:

```yaml
w3_live_tactical_review:
  status: pass  # or fail
  reviewed_at: "2026-06-03"
  screenshot_paths:
    - debug_runs/art_pipeline/w3_captures/victorian_4x3_tactical.png
    - debug_runs/art_pipeline/w3_captures/industrial_west_4x3_tactical.png
  notes: "Night emissive readable; wall/roof families distinguishable."
```

7. Write witness: `debug_runs/pg3_w3_tactical_review_live.json`:

```json
{
  "gate": "PG3-W3-LIVE",
  "green": true,
  "pack_a": "style_victorian",
  "pack_b": "style_industrial_west",
  "footprint": "4x3",
  "proceed_player_visible_confirmed": true
}
```

---

## Fail criteria

- Smoke/greybox cubes visible
- Same module ids for wall/roof between packs on same footprint
- Tactical brightness so low night windows unreadable (file PG-3 emissive tune issue)

---

## Relation to tile G4

| Lane | Independent? |
|:---|:---|
| PG-3 W3 (3D modules) | Yes — tactical mesh swap |
| Tile G4 (iso map) | Parallel — keyframe stills per [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) |

W3 can **pass** while warehouse/shopfront/bunker tile G4 still pending — do not conflate.
