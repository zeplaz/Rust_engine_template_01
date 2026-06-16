# Transport — post-foundation engineering track `v1` (reference placeholder)

> **STATUS:** **Placeholder.** Replace contents when product locks priority among **Option A** (lane reservation), **Option B** (intersection mesh), **Option C** (lane-level A* + field costs). Matrix [`road_rail_migration_matrix_v1.md`](road_rail_migration_matrix_v1.md) §9 points here.  
> **Brainstorm anchors** under [`../../guides/`](../../guides/) are **not** truth — use this file to record what the product **actually** chose.

Version: `v1.0.2` (stub)

---

## Locked choice

| Field | Value |
|:---|:---|
| Selected option | `ASK:` — A / B / C |
| Rationale | *(one paragraph)* |
| Impacted rows | R7, R9, R10 (minimum); often R2/R3 under **C** |

---

## Guide mapping (ideas to validate or discard)

When locking an option, tick each row: **adopt** / **reject** / **defer** and point to the matrix/runbook step that implements it.

| Option | Primary brainstorm files | Engineering emphasis |
|:---:|:---|:---|
| **A** | [`lan level traffic.idea.md`](../../guides/lan%20level%20traffic.idea.md) (+ manufacturing hooks in matrix §9) | Reservation tables, lookahead, deadlock UX; may subsume parts of “confirm to bake” flow |
| **B** | [`intersection_road.md`](../../guides/intersection_road.md) | Extruded lanes, junction fill mesh, UV/height/markings pipeline; may compete or combine with tilemap **R10** |
| **C** | [`basic_nav_outline.md`](../../guides/basic_nav_outline.md), [`hybred_road_stats.md`](../../guides/hybred_road_stats.md) | Lane graph as nav truth; `EdgeFieldState`-style costs; agent deltas; re-route policy |
| **Cross-draft** | [`ref_buildout_roads.md`](../../guides/ref_buildout_roads.md) | Redundant with matrix **R1–R10** — use only for example snippets after reference lock |

---

## Notes

- Option **A** — tight coupling to manufacturing / agent reservation and editor confirm flows.
- Option **B** — drives render pipeline, asset baking, **R10** layering vs mesh; unlocks Phase II geometry work in matrix §8.
- Option **C** — forces **lane graph** representation early; highest impact on **R2/R3** data model; must define **anti-thrash** rules for dynamic costs.
- **Sim LOD / streaming:** treat post-foundation fidelity as **budget bands** (CPU/ms, memory, streaming radius) — **T-LOD-001** — not fixed global entity counts.

**After lock:** follow matrix §9 checklist (revise §4 and §10 JSON for **R7**, **R9**, **R10**).
