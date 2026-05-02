# Transport editor — user-input and UX risk `v1`

> **Pair:** matrix [`../../matrix/transport/road_rail_migration_matrix_v1.md`](../../matrix/transport/road_rail_migration_matrix_v1.md) §7.  
> **Brainstorm inputs:** [`../../guides/`](../../guides/) — see folder [`README.md`](README.md); ideas below are **not** spec until matrix/reference lock them.

Version: `v1.0.2`

---

## 0. Authoring ghost vs runtime preview (**T-GHOST-001**)

| Class | Lives in | Touches save / sim? | UX rules |
|:---|:---|:---|:---|
| **Authoring ghost** | Map editor (**matrix R9**) | **No** until user **bake** + **R8** round-trip includes network | Preview only; invalid states **reject + highlight**; confirm is explicit |
| **Runtime preview** | Optional debug / client overlay | **No** — non-authoritative | Gated behind debug flags; never confused with placement commit |

**Best practice:** do not reuse authoring ghost components for in-world vehicles; keep **bake** as the single gate from editor state → durable ECS + snapshot.

---

## 1. Placement and commit model

| Pattern | Risk | Default mitigation |
|:---|:---|:---|
| Click-to-add control point | Double-clicks, mis-clicks | Debounce or require **modifier** for add vs select (product `ASK:`) |
| Drag tangent handle | Accidental curvature change | Hit-test priority; **Escape** cancels in-flight drag |
| **Bake** | Premature graph mutation | **Confirm** (Enter) or explicit button; ghost spline has **no** `NetworkEdge` until bake |

---

## 2. Curvature and validation

When sample curvature violates profile (`max_turn_radius`, grade for rail):

| Strategy | When to use |
|:---|:---|
| **Reject + highlight** | Default until reference doc chooses otherwise |
| **Auto-smooth** | Only if product mandates; document in matrix **R2** blockers resolution |
| **Partial accept** | `ASK:` — rarely safe without designer UX |

---

## 3. Layer and mode confusion

- **Road** vs **rail** vs **terrain** edit modes must be obvious (toolbar label, cursor hint).
- Align with map editor tool taxonomy ([`map_editor_matrix_v1.md`](../../matrix/map_editor/map_editor_matrix_v1.md)); transport spline tool may extend or sit beside **R9**.

---

## 4. Undo / redo

- **Blocker** for marking **R9 Applied** until: stack depth, coalescing rules (per control point vs per session), and interaction with **bake** (undo pre-bake only vs post-bake graph edits).

---

## 5. Post-foundation complexity (product reference)

Lane reservation vs intersection mesh vs lane-level A* drives different **preview** and **validation** UX. See matrix §9 and [`../../matrix/transport/reference_post_foundation_track_v1.md`](../../matrix/transport/reference_post_foundation_track_v1.md).

| Track (matrix §9) | UX risks from brainstorming guides |
|:---|:---|
| **A** — Reservation (`lan level traffic.idea`) | Failed lookahead → user-visible **blocked** state vs silent retry; deadlock → **global pause** vs per-vehicle feedback; editor **confirm/cancel** must stay coherent with manufacturing |
| **B** — Mesh (`intersection_road`) | Long mesh builds → progress or async bake indicator; wireframe/debug modes must not ship as default clutter |
| **C** — Lane A* + field (`basic_nav_outline`, `hybred_road_stats`) | Rapid **re-route** can read as flicker — need thresholds, cooldowns, and optional “stick to committed path” |

---

## 6. Ghost and authoring overlays

All rows below apply to **authoring ghost** unless labeled runtime preview.

| Idea | Source | Risk | Default |
|:---|:---|:---|:---|
| Live **curvature / slope** ghost | `ref_buildout_roads` | Misread as final collision | Label as preview; bake still authoritative |
| **Cost / throughput** tint on ghost edges | `ref_buildout_roads`, `hybred_road_stats` | Misleading before economy + nav exist | **Off** until R7/R8 story; product `ASK:` |
| **Congestion / damage** debug ribbons | `hybred_road_stats` | Visual noise during level art | **Runtime preview** / debug tier only; off in shipping default |

---

## 7. Blocking questions (add as rows)

*(Surface via matrix §9 + backlog brief BQ when promoting G4/G5.)*
