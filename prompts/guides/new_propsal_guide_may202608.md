# Simulation & config expansion — proposal index (May 2026)

> **STATUS:** Draft **v1**. This file is the **human-facing index**. Authoritative execution detail lives in the linked runbooks. Implementation work is **Pending** until owner matrices and step packs are anchored (see [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md)).

Version: `v1.0.2`  
Audience: leads, designers, agents planning large simulation layers.

---

## 1. Why this document exists

The engine has strong **ontology**, **ECS direction**, and **asset taxonomy**, but **weak binding** between assets and **simulation loops** (weather, logistics, power, damage, ecology, petroleum, AI). This program formalizes **simulation ownership**, **layering**, **execution ordering**, and **runbook-driven** delivery.

**Core rule (non-negotiable):** *Assets define capabilities and semantics. Simulation systems interpret and execute them.* Buildings and data must not hardcode gameplay outcomes.

---

## 2. Config authoring: beyond `*.example.json`

Today, paths like `assets/config/terrain/*example*.json` serve **three roles at once**: demo bootstrap, test fixtures, and implicit “default registry.” The filename suggests *optional sample* while dev code often treats them as *canonical*. That mismatch blocks clear **authoring** and **product** separation.

**Target shape:**

| Layer | Role |
|:---|:---|
| **Schema / template** | Versioned shapes, validation rules, empty stubs for new projects. |
| **Project / mod package** | Selects active registries (`terrain/`, profiles, tuning). This is what players/mods ship. |
| **Tooling** | CLI + GUI: validate, diff, preview (chunk matrix / materials / mobility), and guided edits (“add tag”, “add family”, “add rule”) with **invariants** (every rule `result_name` exists; tags referenced exist; `family` resolves in `TerrainFamilyRegistry`). |

**`biome_bucket` (terrain families)** is not “random string → string”; it is an explicit **aggregate gameplay worldview** (water, plains, forest, …) used for higher-level sim and UI. It stays useful when **tools document which systems consume it** and when **stable ids** (`TerrainFamilyId` / names in saves) are distinguished from **recomputed** fields.

**Preview loop:** authoring changes should optionally run deterministic **sample chunks** / world tiles (same spirit as chunk-first preview) so designers see consequences before commit.

---

## 3. Unified Simulation Expansion — orchestrator

**Entry runbook:** [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

It defines **simulation layers**, **dependency ordering**, **chunk/overlay rules**, and links to every domain runbook below.

---

## 4. Runbook catalog

| Domain | Path | Notes |
|:---|:---|:---|
| **Meta / layers** | [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md) | Start here. |
| **Weather** | [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md) | Overlays → interpretation; never raw ontology edits. |
| **Flora / ecology** | [`flora_ecology_runbook_v1.md`](flora_ecology_runbook_v1.md) | Chunk fields + LOD detail entities. |
| **Chunk scheduler** | [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md) | FixedUpdate, dirty chunks, persistence. |
| **Infrastructure + environment** | [`infrastructure_environment_integration_v1.md`](infrastructure_environment_integration_v1.md) | Buildings, grid stress, weather coupling. |
| **Concrete industry** | [`concrete_industry_sim_runbook_v1.md`](concrete_industry_sim_runbook_v1.md) | Batch curing, weather, roads. |
| **Asset audit** | [`asset_system_audit_runbook_v1.md`](asset_system_audit_runbook_v1.md) | asset → owner → layer matrix. |
| **Petroleum** | [`petroleum_industry_simulation_runbook_v1.md`](petroleum_industry_simulation_runbook_v1.md) | Strategic industrial + logistics. |
| **Petroleum UI** | [`ui/petroleum_industry_ui_snippet_v1.md`](ui/petroleum_industry_ui_snippet_v1.md) | Policy resources, not graph surgery. |
| **Python asset tools** | [`python_asset_tools_alignment_runbook_v1.md`](python_asset_tools_alignment_runbook_v1.md) | **S8** — prefer **after S0** for engine-faithful vocabulary rewrites; **parallel with S2** for incremental alignment. |

**Step packs (umbrella):** [`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md) — placeholder index until per-domain matrices exist.

---

## 5. Suggested implementation order (dependencies)

**Parallelism:** **S0** (asset audit) and **S1** (chunk scheduler) may run **in parallel**. **S2** (weather) and **S8** (Python asset tools) may also run **in parallel** — weather is Rust/sim pipeline work; S8 is editor/taxonomy work. They touch different codebases unless you intentionally sync enums (`ResourceType`, fuel lists, etc.).

**Tooling gate:** **S8** (Python asset tools alignment) should follow **S0** for any taxonomy that must mirror the **live engine**: the editor should reflect **declared** owners and stable names from [`asset_sim_ownership_matrix_v1.md`](../matrix/simulation_expansion/asset_sim_ownership_matrix_v1.md), not ahead of them. Small A0-style inventories or `py_compile` fixes can land anytime; **broad** `asset_config.py` / page rewrites should track the matrix.

1. **Asset audit (S0)** — assign owners and extend the matrix before large dependent sim work **and** before locking editor vocabularies to engine reality.  
2. **Chunk scheduler (S1)** — foundation for scale (dirty regions, LOD, persistence policy); can proceed alongside S0.  
3. **Weather (S2)** — feeds overlays consumed by mobility, power, petroleum, ecology; can run **parallel** with **S8** (tools).  
4. **Python asset tools (S8)** — narrow tasks (aliases, `py_compile`, single-field parity) can run **while** S2 proceeds; **broad** `asset_config` rewrites still follow S0 matrix.  
5. **Infrastructure + environment** — consumers of weather + chunk fields.  
6. **Concrete** — ties to construction, roads, weather.  
7. **Flora / ecology** — builds on weather + hydrology/terrain interpretation.  
8. **Petroleum** + **Petroleum UI** — large cross-cutting; needs logistics + chunk fields + weather hooks.

(Adjust order if a domain matrix is already **Applied** in-repo.)

---

## 6. Master direction (summary)

Target architecture: **semantic world model** + **infrastructure simulation** + **field economics** + **hybrid agents** + **environmental systems** + **chunked LOD ECS** — not a traditional RTS-only unit simulator. That choice drives scalability, AI, modding, and determinism.

---

## 7. Related docs

- [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) — how to add step packs and phases.  
- [`asset_system_audit_runbook_v1.md`](asset_system_audit_runbook_v1.md) — start S0; matrix: [`matrix/simulation_expansion/asset_sim_ownership_matrix_v1.md`](matrix/simulation_expansion/asset_sim_ownership_matrix_v1.md).  
- [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) / [`bevy_asset_terrain_runbook_v1.md`](bevy_asset_terrain_runbook_v1.md) — terrain pipeline (note: terrain **families** are registry-driven; some older matrix lines may still mention legacy naming).  
- [`world_assets_tools_rulebook_v1.md`](world_assets_tools_rulebook_v1.md) — tools that edit assets on disk.  
- [`python_asset_tools_alignment_runbook_v1.md`](python_asset_tools_alignment_runbook_v1.md) — PyQt Asset Content Studio ↔ engine ontology.  
- [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md) — UI → policy resources, not simulation internals.

---

## 8. Document history

- **2026-05:** Proposal index created; domain specs split into dedicated runbooks under `prompts/guides/`. Original brainstorm consolidated from a single long paste into maintainable files.
- **2026-05-06:** Catalog + implementation order extended with **Python asset tools alignment** (`python_asset_tools_alignment_runbook_v1.md`); step-pack **S8** row.
- **2026-05-06:** `v1.0.2` — **S2 ∥ S8** parallel workstreams; renumbered §5 (S8 no longer only “step 8” at end).
