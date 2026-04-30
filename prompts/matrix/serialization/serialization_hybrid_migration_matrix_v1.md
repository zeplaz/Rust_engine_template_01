# Serialization hybrid — migration matrix `v1`

**Paired designer folders:** `prompts/designer_questions/terrain_world/` (chunks, world saves) + `production_economy/` (production snapshots) + **`factions/`** (faction/diplomacy DTOs in saves).

**STATUS:** ⏳ **Spec active — implementation partial** (saves still TBD; configs migrating).

| Axis | State |
|:---|:---:|
| Code | ⏳ binary snapshot + header not fully wired |
| Docs | ✅ this matrix |

> **Prompt use:** Pair `terrain_world/` + `production_economy/` + **`factions/`** READMEs for save-affecting features · don’t assume snapshot format live until `rg`/`cargo check` confirms · cite · `ASK:` for versioning policy.

---

## Production snapshot stubs (code truth)

Active placeholders in `src/systems/production/serialization.rs`:

- `ConcreteSerializationPlugin` — TODO register `ConcreteProductionConfig` persistence.
- `AluminumSerializationPlugin` — TODO `AluminumProductionConfig`.
- `PowerSerializationPlugin` — TODO substation/plant **DTO** graph.

Until these register real loaders, **production matrix “serialization plugin” rows are API boundaries only**, not save compatibility.

## Locked direction

1. **Move away from pure JSON** for heavy / hand-edited data.
2. **Hybrid save/load**:
   - **Small text header** (JSON *or* RON) — version, checksum, mod list, world id.
   - **Bulk body** — **binary** snapshot (efficiency) 📎 format: `bincode` / `postcard` / other — **ASK** before locking.
3. **Optional deltas / replay** — separate track; not blocking first snapshot.

---

## Migration table

| Artifact | Old | New target | Notes |
|:---|:---|:---|:---|
| Save file | ad-hoc / JSON | header + binary blob | Version field mandatory |
| Building/vehicle configs | JSON in `assets/configs/` | **RON** (+ migration) | See `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md` |
| Hot reload | ⏳ | dev-only watcher | Feature-gate |

---

## Sub-questions (📎 user)

1. Header format: JSON only, RON only, or **JSON metadata + RON patch**?
2. Endianness / compression (zstd layer?) for binary body?
3. Per-chunk save files vs monolithic — ties `prompts/designer_questions/terrain_world/chunks_streaming_v1.md`

---

## Repo touchpoints

- Deserializers: `src/io/serialization/deserializers.rs`
- Production serialization plugins: `src/systems/production/serialization.rs`

## Paired designer docs

- Chunk save policy / queues: `prompts/designer_questions/terrain_world/chunks_streaming_v1.md`
- Production state & repair: `prompts/designer_questions/production_economy/`
- **Implementation:** `terrain_world/implementation_questions_v1.md` (I/O §), `production_economy/implementation_questions_v1.md`
- **Terrain materials:** persist `MaterialDef.name` (not raw `MaterialId`) — [`terrain_world/material_tag_rule_system_v1.md`](../../designer_questions/terrain_world/material_tag_rule_system_v1.md) · [`terrain_biome/material_unification_matrix_v1.md`](../terrain_biome/material_unification_matrix_v1.md)
