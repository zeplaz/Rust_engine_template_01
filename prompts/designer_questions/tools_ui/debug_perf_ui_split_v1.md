# Debug, performance, and UI technology split `v1`

**Round:** 2026-04-27 (rev. 2 — dev-technical only) · **For:** LLM + engineers  
**Canonical UI rule:** `prompts/guides/ui_boundary_guide_v1.md`

This file covers **how** diagnostics and player-facing UI are implemented — not product positioning or distribution.

---

## Locked stack split

| Concern | Stack | Examples |
|:---|:---|:---|
| **Frame time, memory, ECS counts, chunk queues, path LOD, net RTT** | **egui** | Profiler, archetype/entity counts, interest-orbs, chunk load graph |
| **Dense grids** (10k+ rows, sort/filter) | **egui** first | Save diff viewers, bulk entity tables |
| **World gen / production / faction / strategic inspectors** | **egui** | Parameter panels, manifests, replay of gen seeds |
| **In-game HUD** — resources, build palette, **tactical status** | **Bevy native UI** | `Node`, `Text`, minimal charts; world-attached overlays |
| **Cross-domain tools** (same patterns for terrain, build, platforms) | **egui** + shared **Resource** contracts | See `tooling_cross_domain_v1.md` |

**Rationale:** egui = fast iteration and heavy tables; Bevy UI = runtime HUD tied to simulation state.

---

## Cross-cutting tool domains (engine work)

Inspectors and debug views should reuse one **tooling backbone** (schedules, state, egui contexts), but **bind** to different `Resource` / query surfaces:

| Domain | Typical egui panels | Metrics / state sources |
|:---|:---|:---|
| **Terrain / generation** | Noise layers, hydrology preview, brush undo | `WorldGenParams`, chunk dirty sets, `terrain_world/*` |
| **Building / modifiable structures** | Blueprint stats, footprint, power stub | ECS queries on structure components; `production_economy/*` |
| **Road / rail / logistics** | Graph edge list, capacity, block state | Navigation + production graphs |
| **Surface vehicles** | Config hot-reload, damage, queue | `entities/vehicles`, `damages` |
| **Naval / shipping** | Route, berthing, draft (when present) | Same patterns as vehicles + water graph 📎 |
| **Automation / orders** | Order queue, LOD tier of controller | Server interest + AI stubs |
| **Strategic platforms** | Missile track, seeker mode, jam STNR; radar/snifter; UAS battery | `strategic_platforms/*` + sim LOD pockets |

**Improvement rule:** add a **new panel** only after the corresponding **Resource or event contract** exists — avoid UI-led invented APIs.

---

## Implementation phases (technical)

### Phase 0 — Wiring

- Single `DiagnosticsUiPlugin` (or split by feature module) registered from dev / optional feature; **`devtools` cargo feature** and/or **`Res<DiagnosticsConfig>`** with hotkey — engineering choice only.

### Phase 1 — Metric resources (no UI in sim cores)

- `FrameTimeStats`, `ChunkStreamerStats`, `SimBudget`, `PathfindingLodStats`, optional `NetRTTStats` — updated in existing schedules; **read-only** from egui.

### Phase 2 — egui shell

- One window, tabs: **Perf | Memory | Chunks | Net | Hydrology | Platforms (stub)** — tabs appear as subsystems gain metrics.

### Phase 3 — Bevy HUD

- Sparklines / alerts: implementation 📎 `Image` vs mesh vs deferred egui — see `implementation_questions_v1.md`.

---

## Open technical questions (📎 size in implementation)

1. Graph sample cadence: every sim tick vs fixed **Δt** — trade memory vs accuracy.
2. Memory instrumentation: **RSS** only v0 vs **tracy**/custom allocator hooks — when to add dependency.
3. **Network** metrics: server tick queue depth vs client RTT — which first for MP stub.
4. **Platform debug:** missile state machine breakpoint UI — inspect `Entity` components vs aggregated “corridor” DTO.

---

## Cross-links

- Shared inspector patterns: `tooling_cross_domain_v1.md`
- Repair / production UI data: `prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md`
- Chunks / load shedding: `prompts/designer_questions/terrain_world/chunks_streaming_v1.md`
- Strategic sim design: `prompts/designer_questions/strategic_platforms/README.md`
- Boundary rule: `prompts/guides/ui_boundary_guide_v1.md`
