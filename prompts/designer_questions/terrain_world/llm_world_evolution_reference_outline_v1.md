# LLM world-evolution reference outline `v1`

**Status:** **Non-authoritative.** Brainstorm / ingested external prompt fragment about an autonomous rule-edit loop over worldgen (memory tiers, metrics, decision engine, regeneration strategy). The **binding** specs are:

- [`material_tag_rule_system_v1.md`](material_tag_rule_system_v1.md) (designer narrative; §§7–8 cover hot-reload + agent overlay)
- [`../../matrix/terrain_biome/material_unification_matrix_v1.md`](../../matrix/terrain_biome/material_unification_matrix_v1.md) (especially §§7, 13, 16, 17 — hot-reload, invalidation, performance, LLM/agent policy)
- [`implementation_questions_v1.md`](implementation_questions_v1.md) §§47, 71–78 (agent authority, determinism, audit log, packs)

Use this file to spawn **new matrix rows** and **new implementation questions**, not as runtime truth.

---

## Naming alignment (outline phrase ↔ canonical repo)

| Outline phrase | Canonical in this repo | Notes |
|:---|:---|:---|
| "rules" / "ruleset" | `RuleSet` (RON) | Per matrix §§1, 2 |
| "tags" | `TagSet` / `TagRegistry` (JSON) | Per matrix §1 |
| "thresholds" | `BiomeTuning` + future `tag_tuning` | Impl Q §42 |
| "partial regeneration" / "mark dirty" | `ChunkDirty` + `ChunkDependency` | Matrix §§13, 16 |
| "rule evolution log" | Append-only JSONL audit | Matrix §17, impl Q §72 |
| "world snapshot" | Aggregated metrics over loaded chunks | New table proposed below |

---

## Memory tiers (mapped, not adopted)

The outline proposes Tiers 0–4. Map to canonical repo concepts before any implementation:

| Outline tier | Repo mapping | Status |
|:---|:---|:---:|
| Tier 0 — frame memory (ephemeral) | Per-frame `Local<T>` system state; no new structure needed. | n/a |
| Tier 1 — chunk analysis memory | New `ChunkMetrics` component (proposed). Lives next to `MaterializedChunk` per matrix §5. | Pending — propose as **U7+** matrix row |
| Tier 2 — world snapshot memory | Aggregated `WorldMetricsSnapshot` resource recomputed on configurable cadence (every N chunk completions or every N seconds, **`ASK:`**). | Pending |
| Tier 3 — historical trend memory | Ring buffer of snapshots; size **`ASK:`** (default 64). | Pending |
| Tier 4 — rule evolution log | Append-only JSONL under `assets/config/terrain/audit/rule_evolution.jsonl`; **dev_tools-gated** by default. | Pending — pair impl Q §72 |

> **Constraint:** any of these that persist across runs **must** be name-keyed (rule names, tag names, material names) — never raw `MaterialId(u16)` — to survive registry reloads.

---

## Metric system (data-only — no engine logic in this outline)

**Core metrics** the outline proposes. The repo treats these as **candidate** signals; pick which become real engine measurements via impl Q §73.

| Metric | Definition | Real today? |
|:---|:---|:---:|
| `traversable_ratio` | Cells whose `MaterialDef` has `traversable: true` ÷ total cells. | Possible once `MaterialDef.properties` parsing lands (U3+). |
| `buildable_ratio` | Same with `buildable`. | Possible. |
| `wetness_coverage` | Cells whose `TagSet` contains `wet` tag ÷ total. | Possible after U4 pass 2. |
| `biome_fragmentation` | Connected-component analysis on `MaterialFamily` per chunk. | Expensive — pair impl Q §70 (LOD), defer. |
| `path_connectivity` | Cross-chunk reachability of `traversable` cells. | Requires nav graph (separate system). |
| `resource_distribution_uniformity` / `_variance` | Per-resource coefficient of variation across chunks. | Possible once resource rules exist (impl Q §62). |
| `edge_density` | Fraction of cells where neighbor's `MaterialFamily` differs. | Possible after U4 pass 2. |
| `tag_entropy` | Shannon entropy over `TagSet` distribution. | Possible. |

**Derived:** `overconcentration_index`, `dead_zone_ratio`, `spawn_safety_score` — defer until consumers are real (gameplay spawn / scenario authoring).

> **Determinism:** all metrics must be **pure functions of loaded asset state** (registry, rules, tags, tuning, seed). Random sampling for metrics is **not allowed**.

---

## Decision engine (action types — proposed shape)

Outline proposes four action types. Map to **file edits** (the only allowed mutation path per matrix §7):

| Action | Allowed surface | Constraint |
|:---|:---|:---|
| Modify rule (priority, predicates) | Edit `material_rules.ron` in place | Single name-keyed rule per atomic edit; no rename |
| Adjust threshold | Edit `world_gen_tuning.json` (or future `tag_tuning.json`) | Bounded range per field; **`ASK:`** for bounds |
| Inject tag rule | Append to `material_rules.ron` | New rule must declare a name; never anonymous |
| Create rule | Append to `material_rules.ron` | Same as inject; priority must fall in a reserved band per impl Q §77 |

**Hard rule:** the agent does **not** mutate runtime state directly. All edits go through the file system so the existing hot-reload pipeline (matrix §7) remains the single mutation path.

---

## Trigger conditions (proposed thresholds)

Outline lists illustrative thresholds (`traversable_ratio < 0.6`, etc.). The repo treats these as **`ASK:`** until paired with real metric implementations (item §73). Do **not** hardcode them in Rust — they belong in a designer-edited `assets/config/terrain/world_evolution_policy.json` (proposed; **`ASK:`**).

---

## Constraint system (lifted with edits)

| Constraint | Repo position |
|:---|:---|
| Do not break determinism | Already invariant — orchestrator §1, matrix §8 |
| Do not remove valid material mappings | Authoring tool / asset editor must validate before save (production_economy §§1–2 patterns); also impl Q §47 |
| Do not create cyclic rule dependencies | Cycle detection on rule load (`ASK:` algorithm) |
| Tag count per cell ≤ 16 | Conflicts with default `TagSet([u64; 4])` cap of 256; reconcile via `ASK:` impl Q §43 |

---

## Learning / impact scoring (deferred)

Impact scoring (`impact_score = (new_metric - old_metric) * weight`) and exploration/exploitation ratio (80/20) are **gameplay-side** decisions. They don't bind the engine. Park here as ideas; promote to implementation questions only after impl Q §73 lands real metrics.

---

## Regeneration strategy (already covered)

Outline's "partial invalidation" and "priority regen" map cleanly onto:

- Matrix §13 invalidation table.
- Matrix §16 `ChunkDirty` bitmask + partial rebuild dispatcher.
- Impl Q §51 partial-rebuild rule.

No new rows needed; this outline reinforces those existing rows.

---

## Debug output (mandatory in outline; recommended here)

The outline insists on JSON debug emissions per agent decision. Map to:

- `RuleTrace` component (impl Q §75) — already proposed for U7-S05.
- Append-only audit log (impl Q §72) — already proposed for U7-S05.

If the outline's "expected effect" field is desired, add it to `RuleTrace.expected_effect: Option<String>` later, **after** real metrics exist.

---

## What to feed back into authoritative docs

When the human is ready, lift these as new **implementation questions** (continuing the §§49–78 numbering convention) and **matrix rows** (continuing matrix §§13–18 layout):

- **Q79** Memory tiers — which become real (Tier 1 component, Tier 2 resource) vs deferred (Tier 3 ring buffer)?
- **Q80** Metric set — confirm initial subset (likely `traversable_ratio`, `wetness_coverage`, `tag_entropy`) before others.
- **Q81** `world_evolution_policy.json` schema — JSON or extension to existing `world_gen_tuning.json`?
- **Q82** Audit log location and rotation policy.
- **Q83** Tag-count-per-cell cap (16 outline vs 256 repo) — pick one.

📎 Do **not** invent question numbers without explicit `ASK:`.

---

## Cross-links

- [`material_tag_rule_system_v1.md`](material_tag_rule_system_v1.md) — designer narrative
- [`../../matrix/terrain_biome/material_unification_matrix_v1.md`](../../matrix/terrain_biome/material_unification_matrix_v1.md) — especially §§7, 13, 16, 17
- [`procedural_world_pipeline_reference_outline_v1.md`](procedural_world_pipeline_reference_outline_v1.md) — sister non-authoritative outline (pipeline)
- [`implementation_questions_v1.md`](implementation_questions_v1.md) §§49–78
- [`../factions/diplomacy_bargaining_reference_outline_v1.md`](../factions/diplomacy_bargaining_reference_outline_v1.md) — sister outline for game-theory side of agent decisions
- Meta-runbook: [`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) §12 (reference-outline policy)
