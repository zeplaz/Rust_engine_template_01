# Construction invariants

Hard rules for `src/construction/`. New work **must** preserve these — see [`construction_round3_plan.md`](construction_round3_plan.md).

**North star:** every feature = **tool → intent → preview → validation → queue → execute**.

## Authority

1. **Preview never mutates gameplay** — ghosts, egui overlays, and cursor picks do not spawn sites, roads, zones, or transport topology.
2. **Only execute paths commit** — `execute_construction_plans_system`, `commit_construction_site_system` (via `CommitConstructionSiteEvent`), zone confirm, demolish execute after pending approval.
3. **All roads and rail** enter through [`ConstructionPlanQueue`](../construction/construction_pipeline.rs); no direct tile-road injection.
4. **`ActiveBuildTool`** is the sole active-tool source for construction input; UI sets tool, systems read tool.
5. **Zone paint** queues `PendingEntryKind::ZonePaint` or spawns `Zone` on confirm — never `CivilHousing` site rows for district paint.
6. **Demolish** requires pending pick → confirm; no instant despawn on LMB alone.
7. **Validation before commit** — `allows_commit` / plan `Validated` status before world mutation.
8. **No construction logic outside `src/construction/`** — except dev boards, engine plugin registration, and strategic commit handlers.

## Ghosts & visuals

9. **Ghost visuals are disposable** — egui painters / preview entities; not authoritative world state.
10. **Visual requests** (Round 3) must not own viewport camera or hole-latch state — route through representation/viewport authority when integrated.

## Tactical map tile occupation (PLAY-BUILD)

15. **Sim owns tile set + phase** — `FootprintMatrix` 0/1 cells, `SiteConstructionPhase`, road path tiles; gameplay never inferred from egui scale.
16. **Map paints occupation** — `ConstructionVisualRequests` → egui + `TileDebugInstanceMap`; zoom/pan via `ConstructionMapProjection` / `world_to_sim_map_egui` only (no widget resize as placement).
17. **Roads are tile strips** — committed/preview paths rasterize to world cells; transport graph IDs unchanged underneath.
18. **Textures later** — phase colors/labels today; baked tiles belong in `RepresentationResult`, not parallel extractors.

## Data & catalog

11. **Building placement** (Round 3 target) loads operational fields from `assets/configs/buildings/` — not land value, housing value, or abstract market grades.
12. **Zoning ≠ building** — `ZoneTool` paints districts; catalog `BuildingDefinition` places structures.

## Topology

13. **Road segments** are not a substitute for **intersection graph** — crossing commits must register [`IntersectionRegistry`](../construction/roads/intersections.rs) nodes (Round 3).

## Observability

14. **Proof JSON** — `debug_runs/construction_stage_live.json` updated when boards/witness change; operational gate uses same artifact family.

## Enforcement

- Code review + `rg` audits (`CONSTRUCTION-R3-H03`).
- Phase 2 closure ([`construction_phase2_todos.rs`](construction_phase2_todos.rs)) removes legacy bypass paths.
- Operational green ([`construction_operational_todos.rs`](construction_operational_todos.rs)) before scaling Round 3 breadth in the running app.
