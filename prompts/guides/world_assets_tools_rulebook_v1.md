# World assets — tooling production, parity, and testing `v1`

**Purpose:** Keep **desktop world-gen**, **in-game F8 tools**, and **material / tag / rule assets** aligned so iteration on configs (noise, biome tuning, registries) behaves the same whether you run the full game or the `world_generator` binary.

**Pairs with:** [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) (U3–U7) · [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) · [`u5_steps_v1.md`](../matrix/terrain_biome/runbook/u5_steps_v1.md).

---

## 1. Canonical stacks (do not drift)

| Entry point | Plugins that define **world tools + registries** | Notes |
|:---|:---|:---|
| **`proc_A_dine01` main** (`EnginePlugin`) | `MaterialUnificationPlugin` → `WorldGenToolsPlugin` (+ **`TilemapAdapterPlugin`** if `--features bevy_tilemap_adapter`) | Full game; feature off = no `bevy_ecs_tilemap` types linked. |
| **`world_generator` binary** | Same **pair**: `MaterialUnificationPlugin` → `WorldGenToolsPlugin` | `src/bin/world_generator.rs` — must stay in lockstep with main for preview / F8. |

**Order invariant:** `MaterialUnificationPlugin` **before** `WorldGenToolsPlugin` so `TerrainRegistriesHandles` and `Assets<*Registry>` exist before `WorldGenUiPlugin` / `WorldPreviewPlugin` read them on first frames.

**Out of scope for parity:** `WorldGeneratorSubenginePlugin` (`src/bevysubengines/world_generator_plugin.rs`) uses a **separate** `WorldGenParams` and UI. It is **not** wired into the `world_generator` binary anymore; keep it only for isolated experiments or legacy save plumbing, and do not treat it as the designer-facing path.

---

## 2. Tool production checklist (new or changed tooling)

When adding or changing **world-asset** or **preview** behavior:

1. **Data path:** Confirm on-disk files match [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §2 (JSON vs RON, extensions).
2. **Loaders:** New `Asset` types get `AssetLoader` + `init_asset` + `register_asset_loader(...)` in **`MaterialUnificationPlugin`** (or one successor plugin) — **single registration site**; see matrix §7.
3. **Resources:** Shared handles (`TerrainRegistriesHandles`, etc.) are inserted in **one** startup path; UI systems use `Res<>` — no duplicate embed paths unless documented as dev-only.
4. **Both entry points:** If the feature touches F8 or preview, update **`world_generator` binary** plugin list when you update **`EnginePlugin`**, or extract a shared `Plugin` bundle in Rust to avoid copy-paste drift.
5. **No second mutation path:** Tools edit files; engine reloads via `Assets` / explicit reload buttons — same rule as terrain runbook §1.

---

## 3. Testing protocol

**Automated (CI-friendly):**

```text
cargo test -p proc_A_dine01 material_ -- --nocapture
cargo test -p proc_A_dine01 preview_ -- --nocapture
cargo test -p proc_A_dine01 --features bevy_tilemap_adapter tilemap_adapter_writes_terrain_layer -- --nocapture
cargo check -p proc_A_dine01
cargo check -p proc_A_dine01 --features bevy_tilemap_adapter
```

Add a **narrow test** for any new loader (`*_loader_extensions`) or boot (`*_app_boot`) following existing patterns in `src/terrain/material/` and `src/systems/terrain/material_plugin.rs`.

**Smoke (human, before declaring a phase done):**

1. `cargo run -p proc_A_dine01` — F8 → generate → Biome preview shows registry-driven colors when example registries load.
2. `cargo run -p proc_A_dine01 --bin world_generator` — same F8 / preview behavior as main for world-gen scope.

If smoke differs between the two commands, treat it as a **parity defect** and fix plugin lists before closing the phase.

---

## 4. Iteration loop (designers / implementers)

1. Edit committed examples under `assets/config/terrain/*.example.*` or the live paths in matrix §2.
2. For **Bevy-loaded** registries: rely on asset pipeline or restart app after file replace during development; tune overlay JSON via F8 “Reload tuning” where wired.
3. **Chunk pipeline:** Spawning `Chunk` + `ChunkCellMatrix` triggers `materialize_chunks`; preview consumes `MaterializedChunk` + `ChunkCellMatrix` for material / tag overlay — pair tile coords with chunk grid per `passes/mod.rs` comment.
4. Record schema / breaking changes in matrix migration rows, not only in code.

---

## 5. Gate before **U7**

- Matrix [`material_unification_matrix_v1.md`](../matrix/terrain_biome/material_unification_matrix_v1.md) §10: **U5** row **Applied**; **U6** may stay **Partial** until multi-layer (U7) or you promote after explicit review.
- This doc §1: **`world_generator` binary matches main** for material + world-gen tools.
- §3 tests green for `material_` + `preview_` filters; optional `bevy_tilemap_adapter` checks above when touching tilemaps.

---

## 6. Cross-links

| Doc | Role |
|:---|:---|
| [`terrain_unification_runbook_v1.md`](terrain_unification_runbook_v1.md) | Phase orchestration U3–U7 |
| [`bevy_asset_terrain_runbook_v1.md`](bevy_asset_terrain_runbook_v1.md) | Asset / loader policy |
| [`system_runbook_authoring_meta_v1.md`](system_runbook_authoring_meta_v1.md) | Template for sibling rulebooks |
| [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md) | UI/plugin boundaries (update when subengine vs canonical split changes) |

---

**Version:** `v1.0.1` — U6 optional tilemap adapter test: `cargo test -p proc_A_dine01 --features bevy_tilemap_adapter tilemap_adapter_writes_terrain_layer`.
