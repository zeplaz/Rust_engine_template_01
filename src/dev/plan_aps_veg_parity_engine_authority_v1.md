# PLAN-APS-E5-ENGINE-AUTHORITY-001 — veg extract / map-stamp engine authority `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-APS-E5-ENGINE-AUTHORITY-001
Date: 2026-06-16
Status: **SIGNED** (@planner-mcp)
Owner: @planner-mcp → @coder (authority) · @coder-mcp (parity tool)
Parent: APS-E5 · $ref:src/dev/plan_aps_evolution_veg_capability_20260616_v1.md G-E
Authority: $ref:.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md
```

**Goal:** Document **single-writer** contracts for vegetation parity — authored `variant_key` → resolver → map/chunk stamp — without duplicating `RepresentationResult` building paths.

---

## Authority chain (read-only extract)

```text
Sim Update
  landscape_grammar + landscape_grammar_burn
    → SuccessionState, ActiveBurn, VegetationPopulation

View / Extract (after FireVisualFrameSet::BuildProfiles)
  VegetationExtractFrameSet::BuildProfiles
    → VegetationExtractFrame { rows[].variant_key, glyphs, modifiers }

Render / stamp (LG-5)
  landscape_chunk_atlas_stamp
    → topology_kind_to_variant_key → LandscapeAtlasEntry.resolve_variant_uv
    → chunk UV stamp (orthogonal to burn tint until art-ship)
```

---

## Contracts

| Surface | Single writer | Readers | APS parity check |
|:---|:---|:---|:---|
| `SuccessionState` | succession advance systems | extract, fuel bridge | — |
| `ActiveBurn` | `LandscapeBurnSet` | extract only | — |
| `VegetationExtractFrame` | `BuildProfiles` | preview, minimap, witnesses | **authoritative resolver output** |
| `LandscapeAtlasRegistry` | MCP promote + index loader | stamp system | authored UV ⊆ index |
| `RepresentationResult` | building representation pipeline | building render | **not** veg sprite authority |

**Rule:** APS "Engine reads" panel cites `VegetationExtractFrame` + catalog — not `RepresentationResult` fire/building rows.

---

## Map-stamp contract (LG-5)

| Field | Source |
|:---|:---|
| `variant_key` | row from extract OR `topology_*` for heterogeneity stamp |
| `atlas_domain` | `landscape` |
| Index | `assets/configs/landscape/_landscape_atlas_index.ron` |
| `chunk_stamp_allowed` | entry flag — pilot may be `false` until G4 |

Burn frames may ship **tint-only** before atlas rows exist; stamp uses topology keys first.

---

## Parity witness (coder-mcp)

`debug_runs/aps_veg_extract_parity_live.json`:

```json
{
  "authored_keys": ["from catalog file"],
  "resolver_keys": ["from VegetationExtractFrame sample"],
  "subset_ok": true,
  "engine_authority": "vegetation_extract_frame"
}
```

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-16 | **SIGNED** |
| **@coder** | 2026-06-16 | **CONFIRMED** — `VegetationExtractFrame` + LG-5 stamp are veg authority; `RepresentationResult` is building-only |

```text
⟦/PLAN-APS-E5-ENGINE-AUTHORITY-001⟧  ΔWF→ APS-EVO-E5-EXTRACT-PARITY-001
```
