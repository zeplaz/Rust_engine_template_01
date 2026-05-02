# Transport / simulation — review run plan `v1`

> **STATUS:** Living checklist. **Does not** override [`../../matrix/transport/road_rail_migration_matrix_v1.md`](../../matrix/transport/road_rail_migration_matrix_v1.md). Use after doc cleanup to steer implementation and runbook authoring.

Version: `v1.0.0`

---

## 1. What we have today

| Artifact | Role |
|:---|:---|
| Matrix **R1–R10** | Authoritative migration gates (G4/G5, R8 halt for R9, Phase II fenced) |
| [`rulebook_drafts.md`](rulebook_drafts.md) | Phased **orchestrator** + rulebooks **A–C** + **P3** outlines (field, cost cache, junctions; trains/streaming deferred) |
| [`lane_graph_model_idea.md`](lane_graph_model_idea.md) | Module layering + **logical schedule** (engine-agnostic); stubs until **T-LANE-001** |
| [`sysem_desitions.md`](sysem_desitions.md) | Hybrid tension spec (draft): movement, field, reservations, ghost classes, LOD as **budget bands** |
| [`transport_editor_ux_risk_v1.md`](transport_editor_ux_risk_v1.md) | Authoring UX; **authoring ghost vs runtime preview** |
| `prompts/guides/*.md` | Non-authoritative brainstorm (matrix §1b) |

---

## 2. Phased delivery (recommended)

| Phase | Scope | Exit signal |
|:---:|:---|:---|
| **P0** | Matrix **R1–R10**: graph, profiles, **R8**, **R9** authoring + bake, **R10**, minimal **R7** for G5 | Rows move **Partial/Applied** per matrix + tests |
| **P1** | Rulebooks **A–C** as *contracts* (field lifecycle, read-only cost cache, junction block *spec*) aligned with P0 graph | Written tests / step packs when runbooks exist |
| **P2** | Matrix **Phase II**: junction topology, lane connectivity, **replace all lane-graph stubs** (**T-LANE-001**) | No hand-waved `update_junction_topology`; explicit components + algorithms |
| **P3** | Trains, economy coupling, chunk streaming, sim LOD — **budget-driven** | Separate orchestrator + perf runbook; not blocked on P0 |

---

## 3. Open todos (tracked)

| ID | Item | Owner |
|:---|:---|:---|
| **T-SCHED-001** | Map **logical schedule** in [`rulebook_drafts.md`](rulebook_drafts.md) §0.2 to the real engine scheduler when transport plugins land | transport + ecs-core |
| **T-LANE-001** | Replace `LaneEdge` / `LaneNavGraph` **stubs** and “junction topology” hand-waves with Phase II **data model + build rules** (matrix §8) | transport + designer |
| **T-GHOST-001** | Keep **authoring ghost** (editor, pre-bake) separate from any **runtime preview**; document in [`transport_editor_ux_risk_v1.md`](transport_editor_ux_risk_v1.md) + [`sysem_desitions.md`](sysem_desitions.md) | editor |
| **T-LOD-001** | Define sim LOD tiers as **ms/CPU/memory/streaming-radius bands** in a perf or world runbook — not fixed entity counts | perf + world sim |

---

## 4. How to move forward

1. **Complete P0** per matrix and gap remediation G4/G5 — no Phase II lane combinatorics required for first playable road authoring + save + coarse nav export.
2. **Freeze rulebooks A–C** text as “behavior contracts”; implementation step packs follow [`../../guides/system_runbook_authoring_meta_v1.md`](../../guides/system_runbook_authoring_meta_v1.md) when `transport_runbook` exists.
3. **Execute T-LANE-001** before claiming lane-level reservation or lane A* “shipped” — graph shape is the main structural risk.
4. **Revisit P3** after reference lock ([`../../matrix/transport/reference_post_foundation_track_v1.md`](../../matrix/transport/reference_post_foundation_track_v1.md)) for Option A/B/C priorities.

---

## 5. Review cadence

- After each milestone: check matrix §4/§10, this run plan §3, and Phase II paragraph in matrix §8 for drift.
- When **`sysem_desitions.md`** is renamed (e.g. `system_decisions_v1.md`), update links in README + this file.
