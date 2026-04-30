# Designer Q — power, damage, UI, factions, persistence `v1`

---

## Power (electrical)

- Model includes **frequency**, **AC vs DC**, **1φ / 3φ** — reflected in **tech trees** and **costs** (designer intent; 📎 verify against `src/entities/production/power/*` when implementing).
- Full topology (transformers, substations, islanding, brownouts) — see legacy §C13.

### UI (in-game)

- **Charts / graphs** for flows over time, power, key resources.
- **Alerts**: neat, **sortable**, **context** panel; user can **hide / dismiss**; **tiered** severity (info → critical).

---

## Damage & maintenance

- **One uber damage model** — not separate “combat” vs “maintenance” tracks.
- **Gradients** + **component-level breakage** → affects function, **quality**, **heat**, **efficiency**, defects → **total breakdown**.
- Causes: **wear**, **overstress**, **explosions**, etc.
- **Repair paths**:
  - Facilities request **spare parts** via **supply chain**.
  - **Dedicated repair specialists** speed up.
  - **Surplus warehouses** of spares improve resilience.

---

## Factions & traits

- **Traits**: **placeholder** schema; expect **tag bag** (flexible) vs rigid enum — 📎 finalize data model.
- **Faction editor tool** (egui / tools track) — create/edit before or during scenario.
- **Persistence**:
  - Per **save** or **scenario** definition.
  - If no preset scenario — **build from rules** (procedural / default roster).

---

## Sub-questions (⏳) — partial answers in **Answers** below

1. Chart backend → see **In-game analytics vs tooling** + `prompts/designer_questions/tools_ui/debug_perf_ui_split_v1.md`.
2. Damage serialization — per-component bitmask vs float array? (see `src/entities/damages.rs` when extending)
3. Repair queue architecture (global vs per-building) 📎; UX + **priority 1–100** decided in **Repair queue** table.

---

## Repo touchpoints

- Power components: `src/entities/production/power/`
- Damage: `src/entities/damages.rs`
- Production manifest: `src/systems/production/manifest.rs`
- UI boundary: `prompts/guides/ui_boundary_guide_v1.md`

---

## Answers & implementation cues (append-only)

**Round:** 2026-04-27 · **For:** LLM + engineers

### In-game analytics vs tooling

| Surface | Tech | Content |
|:---|:---|:---|
| **Performance, memory, queues, sim budgets, chunk load metrics** | **egui** | Operator / engineer tooling; enable via **`devtools` feature** and/or runtime config — same patterns as `debug_perf_ui_split_v1.md`. |
| **Player-facing** charts (power, flows, resources over time) | Prefer **Bevy native UI** where it fits product polish; **egui** acceptable for heavy grids until Feathers/custom charts mature. |
| **Debug** | Default **egui**; keep **Bevy** for in-world overlays that must attach to ECS entities. |

Detail & branching Q’s: `prompts/designer_questions/tools_ui/debug_perf_ui_split_v1.md`.

### Repair queue (gameplay)

| Topic | Decision |
|:---|:---|
| UX | Queue is **arrangeable** (drag reorder), **filterable**, user-**modifiable** (cancel, split, merge jobs 📎). |
| Priority | Integer **1–100** (inclusive) per job or order; stable **tie-break** rule 📎 (time, distance, severity). |

### 📎 Sub-questions

1. Global repair queue resource vs per-facility queues surfaced in one UI?
2. Can AI / server inject jobs with priority bands players cannot override?

### Implementation hints

- Consider `RepairOrder` component + `RepairQueue` resource with sorting key `(priority: RepairPriority, enqueued_tick: u64)` where `RepairPriority` wraps **1..=100** (validate on ingest).
- Faction editor: `prompts/designer_questions/factions/faction_editor_tooling_matrix_v1.md`.
