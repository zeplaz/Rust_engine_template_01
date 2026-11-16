# PLAN-TERRAIN-ANALYSIS-v1 — TAC-05 chokepoint / defensibility precompute

| Field | Value |
|:---|:---|
| **Id** | **PLAN-TERRAIN-ANALYSIS-v1** |
| **Status** | **PROGRAM CLOSE** — TAC-05 ADOPTED (P0…P2) |
| **Date** | 2026-11-15 |
| **Parents** | [`plan_ref_books_defer_unlock_v1.md`](plan_ref_books_defer_unlock_v1.md) W3 · **TAC-05** |
| **Unlocks** | **TAC-05** |
| **Dual-cite** | aig3 6.2.3 |

## Named consumers (agreement)

| Consumer | Module | Read |
|:---|:---|:---|
| **Fort** | `combat/fortification.rs` · `plan_fortification_rows` | RO `TerrainAnalysisBook::defensibility_at` — bias bunker origins toward high-defensibility tiles |
| **Faction AI** | `ai/faction/decide.rs` · `score_goals` | RO via `FactionPerception.mean_front_defensibility` — DefendFront score term `"terrain defensibility"` |

⛔ Neither consumer writes the book. No second combat control grid.

## Authority

```text
⊚TerrainAnalysisBook ═▶ ⊨ chokepoint / defensibility / high_ground at world-gen (p5)
combat/fortification ◂⊳[RO]
ai/faction/decide ◂⊳[RO] (via perception digest)
```

## Slices

| Id | Goal | Files | Accept |
|:---|:---|:---|:---|
| **TAC-05-P0** | `TerrainAnalysisBook` schema + compute | `terrain/analysis/` · `passes/p5_agent_overlay.rs` | `--lib tac05` |
| **TAC-05-P1** | Gen merge into book from chunk elevation | p5 + material_plugin | deterministic same dem |
| **TAC-05-P2** | RO fort + decide | `fortification.rs` · `perception.rs` · `decide.rs` | RO tripwire · no `Query<&mut TerrainAnalysisBook>` in fort/decide |

## Reject

- Runtime rewrite of analysis as sim authority  
- Second combat grid / Zone writer  
- Fort or AI mutating elevation / analysis book  

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 0.1.0 | 2026-11-15 | Stub header so TAC-05 has an owner path. |
| 1.0.0 | 2026-11-15 | **SIGNED** — fort + decide consumers named; P0…P2. |
| 1.1.0 | 2026-11-15 | **PROGRAM CLOSE** — Accept `--lib tac05`; ledger ADOPTED. |
