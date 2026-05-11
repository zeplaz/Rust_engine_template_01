# P2 — Territorial site execution order `v1`

> **STATUS:** Execution index for **operational sites** (not instant building spawns).  
> **Parent:** [`territorial_infrastructure_orchestration_v1.md`](territorial_infrastructure_orchestration_v1.md) · [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) §10–11 · **Code:** `src/strategic/site/`

Version: `v1.0.1`

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

Tracked in-session as implementation todos; execute in roughly this order:

| ID | Work |
|:---|:---|
| **np-1** | Map pick → `GhostBuildCursor` (origin tile + footprint) driven by camera / pointer |
| **np-2** | Ghost pipeline — `evaluate_site_placement_stubs` / full validator → scores + warnings on HUD (or compact toast) |
| **np-3** | Confirm action — `queue_commit_construction_site` (`MessageWriter`) with owner `Entity` + `SiteArchetype` mapped from `ToolContext` |
| **np-4** | **P2-H** — extend preview invalidation (`InvalidationReason` or equivalent) so site commits can mark strategic / U7 dirty where product requires terrain refresh |
| **np-5** | **P2-B+** — replace stubs: slope/hydro/occupancy + transport graph sample in `SitePlacementValidation` |
| **np-6** | **P2-G** — AI candidate sites (sparse grid or graph probes) → same validation → optional `CommitConstructionSiteEvent` |

---

## Document history

- `v1.0.1` — Next-phase backlog + status of immediate sprint vs code.  
- `v1.0.0` — P2-A–J index + sprint list; aligns with user P2 master execution order.
