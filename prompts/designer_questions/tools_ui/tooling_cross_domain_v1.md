# Cross-domain tooling patterns `v1`

**For:** engineers building **egui** tools that touch multiple sim areas.

---

## Principle

One **plugins + resources** shape; **domain-specific** tabs bind to documented contracts. Avoid duplicate egui bootstrap per feature.

---

## Shared building blocks

| Block | Role |
|:---|:---|
| `ToolUiState` (Resource) | Which tab / panel focused; dock state optional |
| `ToolSelection` | Selected `Entity` or `ChunkId` or `BlueprintId` — typed enum |
| `ApplyCommand` events | Tools emit intents; **apply systems** mutate ECS (repo boundary) |

---

## Domain hooks (technical)

- **Generation:** read-only `WorldData` / params; writes only via explicit “apply to world” command path.
- **Build:** placement ghost uses same grid as pathfinding / power graph stub.
- **Vehicles / ships:** inspector reads `RoadVehicle` / future hull components; no special-case UI crate splits.
- **Munitions / EW:** timeline + LOD tier column; links `strategic_platforms/implementation_questions_v1.md`.

---

## Cross-links

- `debug_perf_ui_split_v1.md` — stack choice  
- `prompts/designer_questions/strategic_platforms/platforms_ew_munitions_v1.md` — what to inspect  
- `prompts/matrix/repo/repo_boundary_matrix_v1.md` — never mutate Serializable from UI thread without apply system
