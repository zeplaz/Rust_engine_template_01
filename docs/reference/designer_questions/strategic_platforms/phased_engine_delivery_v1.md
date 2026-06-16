# Phased engine delivery (technical) `v1`

**Purpose:** Order **implementation work** for the simulation engine — **not** product milestones or marketing phases.

Phases are **reorderable** if dependencies change; each phase exits with **measurable criteria** (compiles, test, or deterministic replay artifact).

---

## Phase 0 — Foundation (current track)

- `cargo check` clean, Bevy API per `matrix/engine_bevy/`.
- Repo boundaries per `matrix/repo/`.
- Canonical world data + chunk policy docs (`terrain_world/`).

**Exit:** CI builds; no new gameplay required.

---

## Phase 1 — World + logistics spine

- Deterministic **worldgen → `WorldData` → preview** (existing tools extended).
- **Chunk interest** stub (server-side set, client render decoupled) per `chunks_streaming_v1.md`.
- **Road graph** + one vehicle class + **production** chain slice (`production_economy/`).

**Exit:** One resource flows end-to-end in sim + HUD counter.

---

## Phase 1a — Strategic field shell (continuous operational map)

**Purpose:** Wire **dynamic operational fields** (not Voronoi polygons) per [`ChunkStrategicOverlay`](../../src/strategic/mod.rs), aligned with [`ChunkCellMatrix`](../../src/terrain/generation/cell_matrix.rs).

**Done (integration spine):**

- [`StrategicFieldsPlugin`](../../src/strategic/plugin.rs) after [`MaterialUnificationPlugin`](../../src/systems/terrain/material_plugin.rs): every chunk with `ChunkCellMatrix` gets zero-filled SOA buffers (control, threat, recon, logistics, mobility, fire/smoke, civilian stability).
- Types and architecture: [`crate::strategic`](../../src/strategic/mod.rs) (`MAX_STRATEGIC_FACTION_SLOTS`, `StrategicFieldCell`, [`LogisticsGraph`](../../src/strategic/mod.rs)).
- Engine + `world_generator` bin load the plugin; unit test `strategic_overlay_spawns_with_chunk_matrix`.
- **Nets → fields (v0):** [`LogisticsGraph`](../../src/strategic/mod.rs) is a **Bevy `Resource`** (default empty). [`LogisticsNode::anchor`](../../src/strategic/mod.rs) is an optional [`ChunkCellKey`](../../src/terrain/dynamic_overlay.rs). [`logistics_net_inject_into_overlays`](../../src/strategic/logistics_net.rs) runs after overlays exist: each edge contributes effective flow `capacity × (1 − disruption)` split across its two anchors into `logistics_throughput`; `logistics_strength[cell][0]` mirrors throughput clamped to **0..1** (aggregate / debug channel until per-faction routing). Unit test: `logistics_edge_injects_throughput_at_anchors`.

**Next (ordered follow-ups — pick by dependency):**

1. **Baseline seed** — optional one-shot: copy terrain priors into `mobility_cost` / `fire_risk` from chunk fields or [`MacroTerrainSemantics`](../../src/terrain/generation/polygon_world_semantics.rs) (affordance → modifier, not ownership).
2. **Graph authoring** — build `LogisticsGraph` from road/rail/pipeline entities (or import); multi-hop flow / min-cut later.
3. **Unit / intel sources** — emit into `threat`, `recon_confidence`, `faction_control` (discrete injectors; no full EW).
4. **Diffusion / decay** — cheap CPU pass (later GPU): blur or PDE-like step on selected layers; couple smoke ↔ recon.
5. **Derived blobs for AI/UI** — flood-fill or level-set on thresholds (contested belt = opposing control gradients); **never** replace with province reassignment.
6. **Persistence** — save/load overlay slices + graph for Phase 6+ MP.

**Exit:** Same as spine above for “done”; each follow-up adds a test or deterministic replay hook.

---

## Phase 2 — Damage + power topology slice

- **Damage uber-model** drives output derating (`production_economy/`).
- **Power graph** islanding + one tactical consequence (brownout → production stall).

**Exit:** Save/load round-trip includes damaged state + grid segment 📎.

---

## Phase 3 — Navigation breadth

- **Rail** distinct from road (signals / blocks) — minimal.
- **Naval** stub: port connectivity + **missile magazine** as static for first fire test.

**Exit:** Two transport modes + shared path-debug UI row (`tools_ui/`).

---

## Phase 4 — Strategic / kinetic v0

- **Munition entity** + **track** resource; **terminal effect** applies damage to structure.
- **Radar track list** (abstract); no full EW yet.
- **LOD pocket** promotes corridor on launch (per `simulation_lod_v1.md`).

**Exit:** Scripted scenario: launch → hit → damage on building; replay in egui panel.

---

## Phase 5 — EW + UAS expansion

- **Jamming** module affects seeker or uplink quality.
- **UAS** loiter + link budget drains; returns on command.

**Exit:** Countermeasure toggles change **Pd** or track aging in inspector.

---

## Phase 6+ — Scale & MP hardening

- **Interest management** + authoritative **fire-control** validation.
- **Serialization** of magazines, tracks, EW state (`matrix/serialization/`).

---

## Cross-links

- Strategic spatial model (static vs fields vs graphs): [`src/strategic/mod.rs`](../../src/strategic/mod.rs)
- Inspectors for each phase: `tools_ui/debug_perf_ui_split_v1.md`, `tooling_cross_domain_v1.md`
- Implementation gates: `implementation_questions_v1.md`
