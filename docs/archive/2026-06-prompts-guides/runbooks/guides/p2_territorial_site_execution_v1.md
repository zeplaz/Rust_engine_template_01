# P2 — Territorial site execution order `v1`

> **STATUS:** Execution index for **operational sites** (not instant building spawns).  
> **Parent:** [`territorial_infrastructure_orchestration_v1.md`](territorial_infrastructure_orchestration_v1.md) · [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) §10–11 · **Code:** `src/strategic/site/`

Version: `v1.0.2`

---

## Master phases

| ID | Focus |
|:---|:---|
| **P2-A** | Core site authority + planning |
| **P2-B** | Validation + terrain / network coupling |
| **P2-C** | Logistics-fed construction progression |
| **P2-D** | Network activation + provisioning |
| **P2-E** | Strategic overlays + operational zones |
| **P2-F** | Build UX + planning strip (Bevy) |
| **P2-G** | AI construction planners (same pipeline as player) |
| **P2-H** | GPU overlays + chunk invalidation |
| **P2-I** | Underground + layered infrastructure |
| **P2-J** | Campaign / mission integration (pressure, not forced layouts) |

---

## Critical design rules

1. **No instant spawn** — plan → validate → deliver → construct → activate → maintain.  
2. **Networks first-class** — sites consume / produce on graphs.  
3. **Territory primary** — regions, corridors, zones, supply systems.  
4. **AI = player** — same planner, validation, logistics; no cheat layer.  
5. **Fracture as nuance** — logistics destabilization, not constant civil-war spam.

---

## ECS schedule (target)

```rust
pub enum InfrastructureSiteSet {
    Planning,
    Validation,
    NetworkSolve,
    Logistics,
    Construction,
    Provisioning,
    OperationalZones,
    PreviewInvalidation,
}
```

**Order:** Planning → Validation → NetworkSolve → Logistics → Construction → Provisioning → OperationalZones → PreviewInvalidation  

*(Integrate with existing `StrategicFieldPipeline` / chunk scheduler as wiring matures.)*

---

## Immediate implementation sprint

1. `CommitConstructionSiteEvent` consumer + book rows  
2. `PlannedSite` + `ConstructionSite` ECS authority on same entity bundle  
3. `SiteConstructionBook` runtime integration (already a resource)  
4. Validation pipeline (`SitePlacementValidation`: scores + warnings, not only bool)  
5. Ghost placement UI (`src/gui/build/` — P2-F)  
6. Chunk invalidation hooks (`ChunkDirtyReason` / preview layers)  
7. Provisioning transition (`UnderConstruction` → `Provisioning` → `Operational`)  
8. Strategic zone emitters → `ChunkStrategicOverlay`  

**Status (engine):** Items 1–4, 7–8 have initial code in `src/strategic/site/`, `InfrastructureSiteSet` in `plugin.rs`, `src/gui/build/` (strip + `;` cycle + ops strip BUILD line), `src/ai/construction/`. Item 5–6 remain **partial** (no map-pick ghost, no U7 `InvalidationReason` bridge yet).

---

## Next phase backlog (sprint+1)

Execute **np-1 → np-6** in order. For each numbered round, use the same **3-step rhythm** (mirrors Cursor todos: `np-*-impl`, `np-*-verify`, `np-*-tests`):

1. **Implement** — land the feature on a focused branch.  
2. **Test round** — run **`cargo test`** (full suite) and fix regressions before merge.  
3. **Add tests** — new automated tests that lock the behavior introduced in that round (unit or integration; headless-safe).

| ID | Implement | Test round | New tests (examples) |
|:---|:---|:---|:---|
| **np-1** | Map pick → `GhostBuildCursor` (origin tile + footprint; camera / pointer) | Full `cargo test` | Pick → cursor ECS / resource state; tile math |
| **np-2** | HUD (or strip) shows `SitePlacementValidation` scores + warnings for ghost | Full `cargo test` | Message/resource contract; formatting snapshot or query |
| **np-3** | Confirm → `queue_commit_construction_site` (owner `Entity`, `ToolContext` → `SiteArchetype`) | Full `cargo test` | Event → `SiteConstructionBook` + bundle (existing patterns) |
| **np-4** | **P2-H** — preview / `InvalidationReason` or chunk dirty hook on site commit | Full `cargo test` | Dirty mask / epoch / queue assertions |
| **np-5** | **P2-B+** — real inputs in `SitePlacementValidation` (terrain, graph, occupancy) | Full `cargo test` | Per-gate tests with matrix / graph fixtures |
| **np-6** | **P2-G** — AI candidate sites → shared validator → optional commit | Full `cargo test` | Scoring order; “no cheat” (calls same evaluate API) |

---

## Document history

- `v1.0.2` — Backlog: **3-step rhythm** per round (implement → test round → new tests); table expanded.  
- `v1.0.1` — Next-phase backlog + status of immediate sprint vs code.  
- `v1.0.0` — P2-A–J index + sprint list; aligns with user P2 master execution order.
