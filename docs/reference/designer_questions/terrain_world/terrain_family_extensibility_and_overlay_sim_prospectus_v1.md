# Terrain family extensibility + dynamic overlay sim — prospectus (v1)

**STATUS:** Design / roadmap — not yet fully implemented.  
**AUDIENCE:** Engine authors extending terrain without recompiling Rust for every new *family*.

---

## 1. Why `TerrainClass` is still a Rust enum (honest accounting)

Today, **`TerrainClass`** in [`src/terrain/biome.rs`](../../../src/terrain/biome.rs) is a **closed enum** used across:

- Pass 3 classification (`classify_biome` and thresholds in code),
- [`ChunkCellMatrix::family`](../../../src/terrain/generation/cell_matrix.rs),
- Tilemap / preview discriminants,
- Mobility and material defaults that branch on class.

**Material JSON** stores `family` as a string; [`terrain_class_from_pascalcase_str`](../../../src/terrain/material/registry.rs) maps that string **onto the same closed enum** at deserialize time. That is why it feels wrong for an “extensible” engine: **adding a new family name in JSON still fails** unless Rust adds a variant and updates every `match` that must stay exhaustive (or intentionally non-exhaustive).

**Historical reasons this was accepted:**

- **Type safety and refactors:** exhaustiveness catches forgotten call sites when semantics change.
- **Performance and layout:** `Copy` enum in hot grids (per-cell `family`) is compact and cache-friendly.
- **Incremental delivery:** materials/tags moved toward data; **full** terrain-family extensibility was deferred.

**Conclusion:** The current design is a **hybrid**. Materials/tags/rules are data-driven; **high-level terrain family for classification + grid storage** is still engine-defined. That is **technical debt** relative to the product goal of “add classes in config only.”

---

## 2. Target architecture: config-driven terrain families

**Goal:** Designers define **named families** in JSON (order or explicit ids), with optional **metadata** (display name, bucket for aggregation, default material name, migration alias). **Rust** holds generic machinery; **assets** define the taxonomy.

### 2.1 Core types (directional sketch)

| Concept | Today | Target |
|:--------|:------|:-------|
| Per-cell class | `TerrainClass` enum | `TerrainFamilyId` (dense `u16` or `u32`) or interned string key (trade speed vs simplicity) |
| Authoring | Enum variant names in JSON must match Rust | `terrain_family_registry.json`: list of `{ "name", "id"? , "tags"? }` |
| Classification output | `TerrainClass` | Writes `TerrainFamilyId` by rule table / expression keyed in data |
| Material `family` | Must parse to enum | Resolves **name → id** via registry; unknown name = load error with migration hint |

### 2.2 Phases (recommended)

| Phase | Scope | Outcome |
|:------|:------|:--------|
| **F0** | Documentation + load-time validation | Explicit error messages listing allowed family names from registry file (even if ids still map to enum internally). |
| **F1** | **TerrainFamilyRegistry** asset | Single JSON defines names → ids; `MaterialDef` stores resolved id; `terrain_class_from_pascalcase_str` replaced by registry lookup; **legacy enum** kept for `classify_biome` only with a fixed mapping table from id→enum **or** duplicate “legacy class” field until F2. |
| **F2** | **Data-driven classification** | Replace large parts of `classify_biome` with rules (RON/JSON): thresholds → output family id (or rule priority DAG). |
| **F3** | **Remove `TerrainClass` from chunk storage** | `ChunkCellMatrix.family` becomes `Vec<TerrainFamilyId>`; previews/tilemap use registry + optional palette from config. |
| **F4** | **Gameplay buckets without enum** | Replace `BiomeBucket`-style `match` on enum with **tags or family metadata** (“aggregates_as: water”) in the family registry. |

**Risk:** Large refactors touch saves, networking, and tools. **Version** chunk formats and document migration (same discipline as `schema_version` on material registry).

---

## 3. Dynamic overlay sim — beyond the stub

Today [`DynamicTerrainOverlay`](../../../src/terrain/dynamic_overlay.rs) and [`stub_accumulate_overlay_from_chunk_fields`](../../../src/terrain/dynamic_overlay.rs) are **prototypes**: they prove cell identity (`ChunkCellKey`) and writers/decay plumbing; they are **not** authoritative weather or a persistence contract.

### 3.1 Replace stub with **real sim rules**

| Concern | Stub today | Production-oriented direction |
|:--------|:-----------|:------------------------------|
| Inputs | Moisture + temperature only | Facts: hydrology tags, snow/ice tags, traffic events, combat events; **derived** layers optional |
| Outputs | mud/snow scalars | Named channels with documented units; optional caps; interaction with mobility (cost scale only, vetoes stay in profiles) |
| Determinism | Deterministic given state | Fixed seeds, fixed ordering, **no RNG in core path** unless explicitly documented |

Authoring options: small **RON rule sets** per channel (when_all tags → delta), or ECS systems with **shared eval helpers** (same as material/mobility style).

### 3.2 **FixedUpdate** and **dirty-chunk** scheduling

| Topic | Recommendation |
|:------|:---------------|
| Tick | Run overlay step on `FixedUpdate` at a **configurable** Hz (e.g. 1–10/s for weather; higher only if needed). |
| Scope | Do **not** scan all chunks every frame. Use **`ChunkDirty`** / pass masks or a **`OverlayChunkDirty`** bitset to mark chunks whose overlay may have changed. |
| Wake conditions | Wake neighbors only when rules cross chunk borders (stitch policy mirrors derived metrics). |
| Preview / editor | Editor may run stub at `Update` rate for responsiveness; game uses FixedUpdate — document the difference. |

### 3.3 **Persistence** policy

| Mode | Behavior |
|:-----|:---------|
| **Exclude by default** (current design intent) | Saves omit `DynamicTerrainOverlay`; regenerate or run catch-up sim on load. |
| **Selective persist** | Serialize only regions in play or only high-water-mark scalars. |
| **Full persist** | Chunk-partitioned blobs; schema versioned; optional compression for sparse maps |

**Saves must not** silently grow without bound from debug writers; cap or decay before persist, or strip overlay on save.

---

## 4. Links

- Cross-chunk / interpretation stack: [`ontology/refactor_execution_plan_v1.md`](ontology/refactor_execution_plan_v1.md)  
- Facts vs interpretations: [`ontology/README.md`](ontology/README.md)  
- Material registry schema: [`material_tag_rule_system_v1.md`](../material_tag_rule_system_v1.md)

---

## 5. Open decisions (ASK)

1. **Family id width:** `u16` vs `u32` vs string intern (max families, modded content).  
2. **Classification language:** Pure JSON thresholds vs embedded expression mini-DSL vs WASM modules (modding story).  
3. **Backward compatibility:** How long do we keep `TerrainClass` as a **legacy** view for old tools?  

When these are decided, update [`project_status_and_questions_v1.md`](ontology/project_status_and_questions_v1.md) with concrete milestones.
