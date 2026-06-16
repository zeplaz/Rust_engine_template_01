# Project status & open questions (rolling snapshot)

**Purpose:** Single place to align **ontology refactor**, **terrain/chunk pipeline**, **preview**, and **parallel** transport/map-editor threads. Refresh when a phase lands or priorities shift.

**Canonical phase list:** [`refactor_execution_plan_v1.md`](refactor_execution_plan_v1.md)

Last reviewed: **2026-05** (repo state; adjust date when you edit this file).

---

## 1. Recently landed (terrain / ontology track)

| Item | Notes |
|:---|:---|
| Phase 1 — Example assets | Fact-oriented tags + materials; nav verdict tags removed from example registries. |
| Phase 2 — `properties` + **schema v2** | [`material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md) §4.1; **`material_registry.example.json` → `schema_version: 2`** with **`facts.*` / `sim.*` / `build.*` / `warfare.*`** keys; invalid tag `traversable` removed from `silt_marsh` → **`marsh`**. |
| Phase 3 — Derived metrics | [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs): `slope_grade`; **cross-chunk stitching** at borders ([`stitch_chunk_slope_grades`](../../../src/terrain/generation/derived.rs)); chunk boundaries = **storage partitions only**. |
| Phase 4 — Mobility | RON **`mobility_profiles.example.ron`**, [`MobilityProfileRegistry`](../../../src/terrain/mobility/mod.rs), [`evaluate_tile`](../../../src/terrain/mobility/mod.rs); aggregation: **multiply** costs, **max** risk, **any** `blocked` / grade veto. |
| Phase 5 — Preview | [`world_preview.rs`](../../../src/gui/editor/world_preview.rs): `DerivedSlope` + `Mobility` from chunk metrics; **registry-driven** profile combo; [`tilemap_adapter`](../../../src/render/tilemap_adapter.rs) overlay parity (feature `bevy_tilemap_adapter`). |
| Phase 6 — Docs / AI context | Matrix **U5** + §6 preview rows (`DerivedSlope`, `Mobility`, dual-authority note); **LLM outline** metrics without global traversable; **impl Q §73** expanded; **`ontology/README`** + refactor plan Phase 6 marked **Done**. |
| Hydrology (partial) | Moisture-weighted accumulation in [`hydrology/flow.rs`](../../../src/terrain/generation/hydrology/flow.rs). |

---

## 2. Next steps (ordered by ontology plan)

| Phase | Focus | Primary code / assets |
|:---:|:---|:---|
| **7** | Transport alignment | Compose R8 graph edge costs with off-tile mobility hints; map editor overlay |

**Phase 6 (docs / prompts):** **Complete** (2026-05) — `material_unification_matrix_v1.md` §6 + STATUS, `llm_world_evolution_reference_outline_v1.md` (metrics + AI constraints), `implementation_questions_v1.md` §73, `ontology/README.md` + [`refactor_execution_plan_v1.md`](refactor_execution_plan_v1.md) Phase 6 table.

**Policy:** update matrices **per phase**, not giant batch rewrites (avoids stale blockers and merge drift).

---

## 3. Resolved decisions (from design review 2026-05)

| Topic | Decision |
|:---|:---|
| Chunk stitching | **Yes** — interior = in-chunk neighbors; **border** cells sample **neighbor chunk edge rows/columns** when present. |
| Persist vs recompute | **Persist** slow/stable derived (slope_grade, drainage, erosion_index, soil_depth, rock_hardness, flood_basin_id, …). **Recompute** dynamic sim (mud, snow, damage, traffic, congestion, water accumulation, …). See [`derived_metric_pipeline_v1.md`](derived_metric_pipeline_v1.md) §6. |
| `MaterialDef.properties` | **Namespaces now** — v2 example uses `facts.*`, `sim.*`, `render.*`, `gen.*`, `mobility.*`, `build.*`, `warfare.*`. |
| Mobility asset format | **RON** runtime; Markdown/YAML fragments in docs only. |
| Rule aggregation | **Multiplicative** `cost_multiplier`, **`stuck_risk = max`**, **any** hard veto blocks. |
| `TerrainDerivedMetrics` | **Yes, stub/expand** — API stability via [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs) until more fields land. |
| Preview authority | **Chunk-derived first** for slope/mobility; tile ECS = visualization where both exist. |
| Mobility UI | Start with one profile in asset (`wheeled_logistics`); UI = **registry dropdown**, not hardcoded. |
| Core principle | **World data describes reality**; systems (mobility, build, warfare, economy, AI) **interpret** the same ontology. |
| Ontology “freeze” | Phase 1 examples = **provisional v1** until explicit **ontology freeze** + **schema lock version** + **migration notes** (gate: AI tooling, save compat, multiplayer sync). |

---

## 4. Parallel / cross-cutting tracks (not exclusive to ontology doc)

| Track | Status / gap |
|:---|:---|
| **Legacy `TileMarker` world gen** vs **chunk `MaterializedChunk`** | Preview modes **read chunk metrics** for slope/mobility; ECS tiles still used for height/moisture/temp/biome paths where spawned. |
| **Hydrology v2 (design)** | Glacier/basin sources, explicit rainfall field, ocean/lake sinks — beyond current D8 + moisture-weighted acc. |
| **Transport / lane graph** | [`prompts/designer_questions/transport/`](../transport/README.md) — R8 graph; tie-in at ontology Phase 7. |
| **Navigation** | Optional hook: stable tile cost API → mobility evaluator ([`systems/navigation`](../../../src/systems/navigation/)). |

---

## 5. Open questions (remaining)

1. **Loader strictness:** Should `MaterialRegistry` **reject** unknown `schema_version` in engine builds (today: numeric field only; migration doc vs enforce)?
2. **Phase 0 formal sign-off:** Calendar **ontology freeze** milestone (provisional examples → locked v1 + migration guide).
3. **Derived fields roadmap:** Order for `drainage`, `erosion_index`, `soil_depth`, `flood_basin_id` on [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs) vs separate components.

---

## 6. Quick verification commands

```bash
cargo test terrain::generation::derived
cargo test hydrology
cargo test terrain::material
cargo test material_plugin
cargo test --features bevy_tilemap_adapter
```

---

## 7. Related links

- [`composite_style_worldgen_v1.md`](../composite_style_worldgen_v1.md)
- [`basic_nav_outline.md`](../../../guides/basic_nav_outline.md)
- Transport hub: [`../transport/README.md`](../transport/README.md)
