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

- Inspectors for each phase: `tools_ui/debug_perf_ui_split_v1.md`, `tooling_cross_domain_v1.md`
- Implementation gates: `implementation_questions_v1.md`
