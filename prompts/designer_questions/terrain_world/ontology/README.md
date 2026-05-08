# Terrain ontology — facts vs interpretations

**Core rule:** terrain stores **FACTS**; systems derive **INTERPRETATIONS**. Tags and materials must not encode universal gameplay conclusions (e.g. “traversable for everyone forever”).

| Doc | Role |
|:---|:---|
| [`fact_vocabulary_rulebook_v1.md`](fact_vocabulary_rulebook_v1.md) | Authoritative **fact** tag ontology (categories, naming, forbidden patterns) |
| [`mobility_profile_matrix_v1.md`](mobility_profile_matrix_v1.md) | **Mobility profiles** and rule tables: facts + profile → movement result |
| [`derived_metric_pipeline_v1.md`](derived_metric_pipeline_v1.md) | **Derived metrics** (continuous / computed): inputs, stages, consumers |
| [`refactor_execution_plan_v1.md`](refactor_execution_plan_v1.md) | **Phases 1–6** documented; **Phase 6** = matrix U5/§6, impl Q §73, LLM outline + **AI constraints** |
| [`terrain_family_extensibility_and_overlay_sim_prospectus_v1.md`](../terrain_family_extensibility_and_overlay_sim_prospectus_v1.md) | **Roadmap:** data-driven terrain families (escape hardcoded `TerrainClass` JSON bridge) + overlay sim (rules, FixedUpdate/dirty chunks, persistence) |
| [`project_status_and_questions_v1.md`](project_status_and_questions_v1.md) | **Rolling snapshot:** what shipped, what’s next, cross-track gaps, open questions |

**Paired design narrative:** [`../material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md) · **matrix:** `prompts/matrix/terrain_biome/material_unification_matrix_v1.md`

**Example assets:** Phase 1–2 + **material registry schema v2** (namespaced `properties`). **Phase 3–5:** [`ChunkDerivedMetrics`](../../../src/terrain/generation/derived.rs) with stitched `slope_grade`; mobility RON + preview modes — see [`project_status_and_questions_v1.md`](project_status_and_questions_v1.md). **Continuity + implementation tranche:** [`refactor_execution_plan_v1.md`](refactor_execution_plan_v1.md) (cross-chunk table, interpretation stack, post–Phase 5 backlog).
