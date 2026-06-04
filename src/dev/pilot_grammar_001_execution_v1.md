# PILOT-GRAMMAR-001 — Execution plan (grammar E2E vs ship) `v1`

| Field | Value |
|:---|:---|
| **Todo ID** | **PILOT-GRAMMAR-001** |
| **Archetype** | `IndustrialWarehouse` · `style_industrial_west` |
| **Checklist** | [`pilot_grammar_001_g4_checklist_v1.md`](pilot_grammar_001_g4_checklist_v1.md) |
| **Not placement-only** | Snapshot must carry `archetype_id`, `district_style`, `grammar_rule_chain`, grammar-driven footprint + slot overrides |
| **Date** | 2026-06-03 |

---

## Two tracks (do not conflate)

| Track | What it proves | Owner | Status |
|:---|:---|:---|:---:|
| **A — Grammar E2E** | `generate(IndustrialWarehouse, industrial_west, seed)` → valid snapshot → APS preview → assembly build | @coder-mcp + @coder | **ready to close** |
| **B — Ship E2E** | Manual keyframe 24 PNGs → G4 → `proceed_ship: yes` → register | @designer-mcp + operator | **blocked** — [`mcp_pilot_grammar_001_rejected_live.json`](../../debug_runs/art_pipeline/mcp_pilot_grammar_001_rejected_live.json) |

**Placement-only pilot** = footprint W×D fill **without** `grammar_rule_chain` / archetype — **rejected** for this program.

---

## Track A — Grammar E2E (automated witness)

### Exit criteria

| # | Gate |
|:---|:---|
| A1 | `grammar_rule_chain` present (`massing`, `footprint_mode`, …) |
| A2 | `reference_tags` include `grammar:` / `chain:massing:` |
| A3 | `validate_assembly_grammar_verify` **passed** (silhouette, wall+roof modules) |
| A4 | Every placement has `material_profile` (PG-MATERIAL-GENERATION) |
| A5 | APS / Bevy preview `green` (`bevy_preview_worker` or browser fallback) |
| A6 | `assembly-build-run` produces **ASSEMBLY-only** blend (no rig in file) |

### Command chain (copy-paste)

```powershell
cd C:\dev\github\Rust_engine_template_01
$env:RUST_ENGINE_BEVY_PREVIEW = "1"   # optional; omit for browser-only preview

# 1) Grammar snapshot (NOT footprint-only)
cd tools\mcp\python
python -c "
from rust_engine_mcp import assembly, building_grammar
from rust_engine_mcp.validators.assembly_grammar_verify import validate_assembly_grammar_verify
from pathlib import Path
import json

r = building_grammar.generate('IndustrialWarehouse', 'industrial_west', 43)
snap = assembly.generate_assembly_snapshot(
    archetype_id='IndustrialWarehouse',
    district_style='industrial_west',
    seed=43,
    source_tier='production',
    write=True,
)
assert snap.get('grammar_rule_chain'), 'missing grammar_rule_chain'
assert snap.get('archetype_id') == 'IndustrialWarehouse'
rep = validate_assembly_grammar_verify(snap, ship=True)
print('grammar_verify', rep.status, rep.summary)
print('assembly_id', snap.get('assembly_id'))
print('written', snap.get('written_path'))
"

# 2) Grammar verify witness JSON
python -m rust_engine_mcp.cli validate-report assembly_grammar_verify assets/staging/assemblies/<assembly_id>.json

# 3) APS preview (Bevy worker)
python -m rust_engine_mcp.cli preview-assembly assets/staging/assemblies/<assembly_id>.json --no-browser

# 4) Assembly blend (materials from snapshot — BUILD-WORKER material apply when landed)
cd ..\..
powershell -File tools\mcp\scripts\designer_mcp_pilot_grammar_prep.ps1
```

### Witness (Track A)

Write **`debug_runs/pilot_grammar_001_grammar_e2e_live.json`**:

```json
{
  "gate_id": "PILOT-GRAMMAR-001-GRAMMAR-E2E",
  "green": true,
  "archetype_id": "IndustrialWarehouse",
  "district_style": "industrial_west",
  "grammar_rule_chain": { "massing": "...", "footprint_mode": "..." },
  "grammar_verify": "passed",
  "preview_mode": "bevy_worker",
  "not_placement_only": true
}
```

**Queue row:** `PILOT-GRAMMAR-E2E-001` (@coder-mcp) — close when witness green.

---

## Track B — Ship E2E (human + designer)

### Why blocked

Operator rejected headless / mislabeled stills ([`mcp_pilot_grammar_001_rejected_live.json`](../../debug_runs/art_pipeline/mcp_pilot_grammar_001_rejected_live.json)):

- Grey slabs, truck in frame, no variant read, identical facings  
- **`proceed_ship: no`** — staging atlas de-indexed

### Prerequisites (before Phase 4)

| Step | Requirement |
|:---|:---|
| B0 | Track A green |
| B1 | Materials via APS / snapshot ([`plan_material_studio_phase_v1.md`](plan_material_studio_phase_v1.md) Phase A minimum: profiles on all placements + preview sanity) |
| B2 | `cleanup_assembly_blends.py` — ASSEMBLY only |
| B3 | **Real** `keyframe_render.py` in Blender UI — **not** `tile_compile_minimum_bake.py` |

### Phases (from checklist)

| Phase | Action |
|:---|:---|
| 3 | Append `Tile_iso_rig_v1` at bake time only |
| 4 | Manual keyframe — **3 states × 8 facings = 24** PNGs |
| 5 | `tile-atlas-pack` from manual folder |
| 6 | Designer G4 — `proceed_ship: yes`, `art_quality: keyframe_manual` |
| 7 | @coder-mcp `--register` + map stamp |

**Queue row:** `MCP-PILOT-GRAMMAR-001` stays **blocked** until Track B Phase 6.

---

## Parallel work (does not replace PILOT)

| Program | Relation |
|:---|:---|
| [`plan_material_studio_phase_v1.md`](plan_material_studio_phase_v1.md) | Unblocks Track B materials (APS, not Blender UI) |
| [`arch_pbg_massing_placement_v1.md`](arch_pbg_massing_placement_v1.md) | Better silhouettes later; pilot stays perimeter grid |
| PG-MODULE-AUDIT-002 | More production modules — improves stills, not grammar proof |

---

## Agent order (recommended)

```text
1. @coder-mcp  PILOT-GRAMMAR-E2E-001     grammar witness + prep script refresh
2. @coder-mcp  Material Studio Phase A   APS materials (Track B prep)
3. @coder-mcp  BUILD-WORKER-001          snapshot materials in Blender bake
4. @designer-mcp MCP-PILOT-GRAMMAR-001  Phase 4–6 human keyframe + G4
5. @coder-mcp  register + @coder        Phase 7
```

---

## Doc / queue hygiene

| Artifact | Action |
|:---|:---|
| [`pilot_grammar_001_g4_checklist_v1.md`](pilot_grammar_001_g4_checklist_v1.md) | Update § Current status — split Track A/B |
| [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) | PILOT = Track A done + Track B blocked |
| [`HANDOFF.md`](../../tools/orchestrator/queues/HANDOFF.md) | Remove false-green pilot; point here |
| `mcp_pilot_grammar_001_live.json` | Keep `green: false` until Track B Phase 6 |
