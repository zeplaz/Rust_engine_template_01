# Refactor execution plan — terrain facts ontology → code & assets

**Goal:** Align the repo with [`README.md`](README.md) + the three contracts ([`fact_vocabulary_rulebook_v1.md`](fact_vocabulary_rulebook_v1.md), [`mobility_profile_matrix_v1.md`](mobility_profile_matrix_v1.md), [`derived_metric_pipeline_v1.md`](derived_metric_pipeline_v1.md)).

**Paired matrix rows:** `prompts/matrix/terrain_biome/material_unification_matrix_v1.md` (materialize, tags, preview).

---

## Cross-chunk continuity — systems and why

Chunk boundaries are **storage partitions**, not **simulation truth** boundaries. The same idea applies across layers: avoid chunk-shaped seams in anything that should read like a continuous field.

| System | Why it must respect cross-chunk / global consistency |
|:---|:---|
| **Slope derivation** | **Seam prevention** — edge cells use neighbor elevation rows/cols so grade does not “reset” at partitions. |
| **Hydrology** | **Basin continuity** — flow, accumulation, and basin ids must not treat chunk edges as ridges/sinks unless the terrain truly is. |
| **Field propagation** | **Pressure continuity** — moisture, temperature blends, wind/weather drivers, and similar passes should agree at boundaries (stitch, ghost cells, or global solve — engineering choice per pass). |
| **Mobility evaluation** | **Path stability** — costs and vetoes must use the same derived + fact inputs a path would see at chunk borders (no profile-specific per-chunk drift). |
| **Preview rendering** | **Avoid dual-authority drift** — editor and AI should sample the same buffers as sim intent (**chunk SoA / derived first**; ECS tiles as projection). |
| **Chunk streaming** | **Deterministic transfer** — load order must not change stable derived inputs; versioning + registry hashes must gate regen; overlays may need merge rules at handoff. |

**Canonical interpretation stack (design):**

```
WORLD FACTS (tags + materials + static fields)
        → DERIVED METRICS (expensive, mostly stable — persist-friendly)
        → DYNAMIC OVERLAYS (transient sim — optional persistence)
        → MOBILITY / BUILD / WARFARE / ECONOMY (interpretation — never stored as terrain verdict tags)
        → AI / gameplay / routing
```

**Reflection before coding (limits of example snippets):** Any Rust in prompts is **illustrative**. Before implementing:

- **Sparse `HashMap` overlays** — convenient for prototypes; hot paths may need **chunk-partitioned** slabs or `Vec` per chunk to match cache behavior and ECS queries. Define **`CellId`** (world tile vs `(chunk, local)`) once; avoid two incompatible coordinate stories.
- **`sim.traction_mod` / material hints** — belongs in **interpretation** (e.g. cost multiplier). **Do not** use as a universal **blocked** flag; **mobility profiles** own capability and veto rules.
- **Strict `schema_version`** — require an explicit **allowlist** and **migration notes** per bump; “reject unknown” prevents silent corruption but blocks partial mod stacks until migration path exists.
- **Preview refactor** — moving all modes chunk-first may require **non-spawned** preview paths (no `TileMarker` dependency) for downscaled or headless tools; schedule in slices.

---

## Phase 0 — Freeze vocabulary (short gate)

| Step | Action | Owner |
|:---|:---|:---|
| 0.1 | Lock **v1 fact tag list** + categories in the rulebook (minimal set; expand later). | Design |
| 0.2 | Decide fate of **`mineable`**, **`buildable`**, **`friction`** in `MaterialDef.properties` — move to derived/sim or keep as **authoring hints** with renamed semantics (document). | Design + eng |
| 0.3 | Add matrix row or checklist item: “ontology refactor in progress; do not reintroduce nav tags.” | Eng |

**Exit:** No code change; signed-off tag list.

---

## Phase 1 — Example assets (applied in repo)

| Step | Status |
|:---|:---|
| 1.1–1.3 `tag_registry` / `material_registry` / `material_rules` | **Done** — nav tags removed; facts + `highland` + pass3 biome tags + material symbol tags (`water`, `fluid`, …) |
| 1.6 tests | **Green** (`cargo test`) |

---

## Phase 2 — Material `properties` hygiene

| Step | Status |
|:---|:---|
| 2.1 | **Done** — §4.1 in [`material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md): physical hints vs forbidden gameplay keys. |
| 2.2 | **Done** — **`schema_version: 2`** + namespaced keys in [`material_registry.example.json`](../../../../assets/config/terrain/material_registry.example.json); §4.1 in [`material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md). |
| 2.3 | **Done** — example registry has no `mineable`; `mineral_rich` remains a **tag** on `basalt_dense`; `buildable` removed from `material_registry.example.json` properties. |

**Exit:** Example materials only emit facts + documented scalar hints.

---

## Phase 3 — Derived metrics (first slice)

| Step | Status |
|:---|:---|
| 3.1 | **Done** — [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs) on chunk entities; `slope_grade` from cardinal max \|Δelevation\| on [`ChunkCellMatrix::elevation`](../../../src/terrain/generation/cell_matrix.rs). |
| 3.2 | **Done** — computed after [`materialize`](../../../src/terrain/generation/passes/p6_materialize.rs) in [`material_plugin.rs`](../../../src/systems/terrain/material_plugin.rs) (`materialize_chunks`, `rebuild_dirty_chunks`). |
| 3.3 | **Done** — unit tests in `derived.rs` (flat → 0; ramp → interior nonzero). |

**Exit:** `slope_grade` + stitching on chunks; see Phase 4 for mobility.

---

## Phase 4 — Mobility interpretation (minimal vertical slice)

| Step | Status |
|:---|:---|
| 4.1 | **Done** — `assets/config/terrain/mobility_profiles.example.ron` (`wheeled_logistics` + rules). |
| 4.2 | **Done** — `MobilityProfileRegistry`, `evaluate_tile` → `MovementHint` in [`mobility/mod.rs`](../../../src/terrain/mobility/mod.rs). |
| 4.3 | **Done** — no writes to `TagSet`. |
| 4.4 | **Optional / not required** — navigation hook when cost API is stable. |

**Exit:** Same tiles, different profiles → different `MovementHint` in tests. **Locked:** multiplicative cost, max risk, any block.

---

## Phase 5 — Preview UX

| Step | Status |
|:---|:---|
| 5.1 | **Partial** — tag pool UI remains; category chips / `FactsPool` rename optional. |
| 5.2 | **Done** — `PreviewMode::DerivedSlope` from [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs). |
| 5.3 | **Done** — `PreviewMode::Mobility` + registry-driven profile combo in [`world_preview.rs`](../../../src/gui/editor/world_preview.rs). |
| 5.4 | **Done** — [`tilemap_adapter.rs`](../../../src/render/tilemap_adapter.rs) overlay (feature `bevy_tilemap_adapter`). |

**Exit:** Designer can see facts pool, stitched slope, and mobility interpretation overlay.

---

## Phase 6 — Documentation & prompts cleanup

| Step | Action | Status |
|:---|:---|:---:|
| 6.1 | Update `material_unification_matrix_v1.md` / U5 + §6 with **`DerivedSlope`**, **`Mobility`**, dual-authority / chunk-first note; hot-reload note for mobility asset | **Done** (2026-05) |
| 6.2 | Replace `traversable_ratio` language in `llm_world_evolution_reference_outline_v1.md` with **mobility-under-profile** metrics + thresholds | **Done** |
| 6.3 | `implementation_questions_v1.md` §73 — mobility + derived + **dynamic overlay** | **Done** |
| 6.4 | `ontology/README.md` — Phase 6 completion + links | **Done** |
| 6.5 | **AI / LLM prompt hygiene** in LLM outline + refactor plan | **Done** |

**Why 6.5 mattered:** LLMs default to binary passability; explicit constraints in [`llm_world_evolution_reference_outline_v1.md`](../llm_world_evolution_reference_outline_v1.md) protect ontology integrity.

---

## Phase 7 — Transport & logistics alignment (later)

| Step | Action |
|:---|:---|
| 7.1 | Compose **edge** costs (R8 transport graph) with **off-tile** mobility hint when routing convoys. |
| 7.2 | Map editor: optional overlay “wheeled cost” using same evaluator (no new terrain tags). |

---

## Risk & ordering

- **Do Phase 1 before Phase 3** so tag ids in registries match passes and tests.
- **Derived before mobility** so rules can use `max_grade` vs tags alone.
- Keep **backward compatibility** optional: one release, duplicate deprecated tags with `#[deprecated]` doc only in JSON comments is not possible — use changelog entry instead.

## Effort sketch (relative)

| Phase | Size |
|:---|:---|
| 0 | XS |
| 1 | S–M (touch many examples + tests) |
| 2 | S |
| 3 | M |
| 4 | M |
| 5 | M |
| 6 | S |
| 7 | L (gameplay integration) |

---

## Implementation tranche (post Phase 5 — engineering backlog)

**Purpose:** Stabilize APIs and separate **truth layers** before simulation sophistication. Order is a **suggestion**; adjust per dependency.

| Step | Recommendation | Notes / limits |
|:---|:---|:---|
| **A. `ChunkDerivedMetrics` stubs** | Add dense `Vec<f32>` fields aligned to chunk cells: `drainage`, `support_capacity`, `concealment`, `erosion_risk` (and later `soil_depth`, `rock_hardness`, `flood_basin_id` as design locks). **Only `slope_grade` needs real math initially**; others `vec![0.0; area]`. | Keeps ECS queries and save schema stable; avoids blocking every system on one pass. |
| **B. `DynamicTerrainOverlay`** | Resource or parallel store: mud, snow, danger, congestion, temporary flooding, etc. **Must not** mutate fact `TagSet` or static material defs for transient state. | **Save rule:** optional / partial / regenerate per game mode. Prototype `HashMap` is fine; production may need chunk slabs. Define cell identity once. |
| **C. Material property accessors** | `MaterialDef` helpers: `sim_f32("traction_mod")`, `facts_str`, `build_f32`, `warfare_f32`, etc., keyed by **namespace suffix** or full key — never ad-hoc string digs across the codebase. | Evolve to `enum`/typed keys later; accessors reduce typo and migration pain. |
| **D. Mobility × `sim.traction_mod`** | Apply as **cost multiplier** (e.g. `* sim_f32("traction_mod").unwrap_or(1.0)`), not as a **hard block** — **profiles** own veto. | Wrong pattern: `if traction_mod < x { blocked = true }` — encodes universal verdict in terrain. |
| **E. Preview single authority** | Route **all** preview tints through chunk SoA / derived when data exists; ECS tiles only **project** for gameplay view. | Large refactor; do incrementally; document interim dual path. |
| **F. Schema version enforcement** | `MaterialRegistry` (and siblings): **known version** → load; **older** → migrate or error with guide; **unknown newer** → reject with message (**never silent**). | Pair with migration doc + matrix bump every ontology change. |

---

## Immediate next command for implementer

1. Diff `tag_registry.example.json` against [`fact_vocabulary_rulebook_v1.md`](fact_vocabulary_rulebook_v1.md).
2. Run `cargo test` after Phase 1 asset edits.
3. Open PR titled e.g. “Terrain ontology Phase 1: remove nav tags from example assets + tests”.
