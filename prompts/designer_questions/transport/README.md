# Transport / road-rail — designer questions

> **Pair:** migration matrix [`../../matrix/transport/road_rail_migration_matrix_v1.md`](../../matrix/transport/road_rail_migration_matrix_v1.md).

**Run plan (todos + phases):** [`transport_sim_runplan_v1.md`](transport_sim_runplan_v1.md)

**Entry:** UX and product risk — [`transport_editor_ux_risk_v1.md`](transport_editor_ux_risk_v1.md).

**Related:** navigation pathfinding spec — [`../navigation/spec/README.md`](../navigation/spec/README.md).

---

## Brainstorming guides (non-authoritative)

These live under [`../../guides/`](../../guides/). They are **concept sketches**, not acceptance criteria. The matrix **§1b** indexes them; before locking behavior, reconcile every claim with **terrain**, **serialization**, **map editor**, and **G4/G5** step packs.

| File | Role for transport work |
|:---|:---|
| [`hybred_road_stats.md`](../../guides/hybred_road_stats.md) | Hybrid **field** (edge state) + sparse **agents**; cost from congestion/damage/danger; debug colorization |
| [`lan level traffic.idea.md`](../../guides/lan%20level%20traffic.idea.md) | **Lane graph**, discrete **reservations**, lookahead, deadlock — Factorio-style mental model |
| [`basic_nav_outline.md`](../../guides/basic_nav_outline.md) | **A\*** on lane edges, field-driven costs, junction topology, reservation penalty, re-path triggers |
| [`intersection_road.md`](../../guides/intersection_road.md) | **Mesh** extrusion, multi-lane offsets, intersection polygon, triangulation/UV/height/markings pitfalls |
| [`ref_buildout_roads.md`](../../guides/ref_buildout_roads.md) | Broad slice draft (spline, bake, tags, tilemap, save, nav export); use to **cross-check** matrix rows only |

---

## Orchestrator + rulebooks (draft)

| File | Role |
|:---|:---|
| [`rulebook_drafts.md`](rulebook_drafts.md) | **P0–P3** phase map, **logical simulation schedule** (engine-agnostic), Rulebooks **A–C**, **P3** outline (trains, streaming/LOD **budget** framing) |
| [`lane_graph_model_idea.md`](lane_graph_model_idea.md) | Module layers; **LaneGraph first** in sim pipe; **authoring vs runtime** ghost; stubs until **T-LANE-001** |
| [`sysem_desitions.md`](sysem_desitions.md) | Hybrid tension spec (draft v1); reservations, collision `ASK:`, LOD as **budget bands** *(filename preserved until rename)* |

---

## Design tensions to resolve (product / matrix §9)

Unvalidated contrasts lifted from the guides — pick one coherent stack via [`reference_post_foundation_track_v1.md`](../../matrix/transport/reference_post_foundation_track_v1.md).

| Tension | Guide hints | Impacted matrix rows |
|:---|:---|:---:|
| **Centerline network** vs **lane-level graph** early | `lan level traffic`, `basic_nav_outline` | R2, R3, R7 |
| **Tile overlay** vs **extruded mesh** visual truth | `intersection_road`, `ref_buildout_roads` | R10, Phase II |
| **Static nav export** vs **dynamic field costs** | `hybred_road_stats`, `basic_nav_outline` | R7, G5 |
| **Discrete reservations** vs **field-only flow** | `lan level traffic` vs `hybred_road_stats` | Post-foundation A vs hybrid |
| **Ghost “cost preview”** before bake | `ref_buildout_roads` sketch | R9 (`ASK:`) |

When an option is locked, update matrix **§4**, **§10** JSON for **R7/R9/R10**, and this README if new designer specs split out.

---

## Tracked implementation todos (summary)

| ID | Summary |
|:---|:---|
| **T-SCHED-001** | Map logical schedule → real engine scheduler |
| **T-LANE-001** | Replace lane-graph stubs; real Phase II junction topology |
| **T-GHOST-001** | Authoring ghost vs runtime preview; bake boundary |
| **T-LOD-001** | LOD tiers = perf/streaming **budget bands** |

Full detail: [`transport_sim_runplan_v1.md`](transport_sim_runplan_v1.md).
