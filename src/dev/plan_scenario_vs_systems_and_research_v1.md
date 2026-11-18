# PLAN-SCENARIO-VS-SYSTEMS + RESEARCH-CAPABILITY-v1

| Field | Value |
|:---|:---|
| **Id** | **PLAN-SCENARIO-SYSTEMS-RESEARCH-v1** |
| **Status** | **ADOPTED** — P0–P3 closed 2026-11-16; callers below |
| **Date** | 2026-11-16 |
| **Trigger** | Operator: DamFlood under `systems/` and `DroneResearchState` violate “clear systems, not individual ones.” |
| **Owners** | `@planner` (law) → `@coder` / `@sim-steward` (moves) → `@planner` (research kernel charter) |

---

## 0. Two smells, one law

| Smell | What went wrong | Correct shape |
|:---|:---|:---|
| **DamFlood as “system”** | Lived in `src/systems/dam_flood/` (re-exported water types) | **`src/scenario/dam_flood/`** harness only — seed fixture, timeline, dump counters. Truth stays in general water / construction / logistics / ecology / power |
| **Drone-only research** | `DroneResearchState` is a private tech bag (fiber, autonomy, seeker, yield) read only by drone missions | **One research/capability book**; drones, sat, comms, manuf, plants are **consumers** of capability ids |

Smell tests (hard):

1. Delete `TestScene::DamFlood` → water solver, dam placement, flood routes, cooling intakes still exist and work for a mid-game player dam.
2. Delete `DroneResearchState` → research still exists; drones lose only their *adapter* to capability ids, not the unlock ledger.
3. Adding “satellite research” must **not** create `SatelliteResearchState` — it adds ids + consumers on the same book.

Forbidden: `*FloodSolver` only under `--test dam`; `if DamFlood { … }` gameplay truth; new `FooResearchState` per product line.

---

## 1. P0 — Law docs (no production behavior change)

**Do:**

- This file is the charter (status OPEN → SIGNED when P0 Accept).
- Cross-link from [`plan_dam_flood_benchmark_v1.md`](plan_dam_flood_benchmark_v1.md) §0 and [`plan_drone_subsystem_fit_v1.md`](plan_drone_subsystem_fit_v1.md) D5: “D5 was a stub; migrate to general book.”
- Point at archived intent: [`research_capability_ecosystem_runbook_v1.md`](../../docs/archive/2026-06-prompts-guides/runbooks/guides/research_capability_ecosystem_runbook_v1.md) (capability emergence, not linear trees) — **thin kernel first**, not full institutions v1.

**Accept:** Status → SIGNED on this plan; both linked plans cite the smell tests.

---

## 2. P1 — DamFlood → scenario (not a system)

```text
src/systems/dam_flood/*  →  src/scenario/dam_flood/*   ✅ LANDED 2026-11-16
```

| Slice | Do | Do not |
|:---|:---|:---|
| **P1a move** | Relocate module; `DamFloodPlugin` registers only when scene/harness needs it (or always as no-op unless seeded) | Keep under `systems/` |
| **P1b strip** | Remove `pub use crate::systems::water::…` re-exports; callers import water directly | New water APIs inside scenario |
| **P1c aliases** | Call sites use `SlowDomainTimeSkip` (drop `DamFloodTimeSkip` alias after grep-clean); cooler marker → general `CoolingIntake` + scenario spawn only | Scene-private water writers |
| **P1d dump** | `sim_run_dump_dam_flood` stays counts-only; book = scenario score ledger, mirror of general field | Second depth buffer |
| **P1e docs** | Benchmark plan Status: “scenario harness”; ADOPTED only with production `file:line` on general path | Claiming DF-* ADOPTED from harness stubs |

**Files likely:** `src/systems/dam_flood/**`, `src/engine/engine_with_worldgen.rs`, `src/dev/sim_run_dump_dam_flood.rs`, `src/dev/plan_dam_flood_benchmark_v1.md`, grep `systems::dam_flood`.

**Accept:** `rg "systems::dam_flood"` empty (or thin re-export shim deprecated one release); `cargo test` scenario + water Accepts; playcheck still scores `dam.*` rows from same dump paths.

---

## 3. P2 — General research / capability book

### Kernel (minimal, shippable)

```text
ResearchCapabilityBook  (sole writer: research domain / future UI + scenario cheats)
  unlocked: HashSet<CapabilityId>   // or progress 0..1 per id
  multipliers: HashMap<CapabilityId, f32>  // optional yield/resist knobs
```

RON catalog e.g. `assets/configs/research/capabilities_v1.ron`:

| id (example) | Domains that may read |
|:---|:---|
| `comms.fiber_ops` | drones, C2, sat downlink |
| `uas.autonomy_l2` / `l3` | drones |
| `sensor.seeker_ir` | drones, missiles |
| `manuf.uas_assembly` | plants / blueprints |
| `manuf.cruise_line` | plants |
| `wx.resist_link` / `wx.resist_sensor` | any link/sensor consumer |

### Migrate D5

| Today (`DroneResearchState`) | Tomorrow |
|:---|:---|
| `fiber_ops` bool | `book.unlocked("comms.fiber_ops")` |
| `autonomy_l2/l3` | capability ids |
| `wx_resist_*` | multipliers on ids |
| `uas_assembly_yield` | manuf multiplier |
| `seeker_ir_unlock` / `cruise_line_unlock` | ids |

`drone_missions` takes `Res<ResearchCapabilityBook>` (or a thin read adapter). Delete `DroneResearchState` when no callers remain.

### Wire other consumers (same book)

1. **Plants** — honor existing `required_research_ids` against the book (activation gate).
2. **Stub sat / comms** — one read each (even if no-op gameplay) so the API is proven shared.
3. **Manufacturing** — UAS blueprint / yield reads manuf ids (replace drone-only flags).

**Do not** implement full knowledge-domain / institution sim in this plan — that stays DEFER (archived runbook). This wave = **one ledger + shared ids**.

**Accept:**

- `research_capability_shared_*` — drone + plant (or stub) both gate on same id.
- `no_drone_research_state_*` — type gone or `#[deprecated]` with zero production readers.
- Unit: unlock `comms.fiber_ops` → drone modality + (stub) sat/comms flag both true.

---

## 4. P3 — Audit

- Grep: `ResearchState`, `*FloodPlugin` under systems, `if.*DamFlood` gameplay branches.
- Cleanup-intelligence packet if leftover scene-private resources look dead.
- Update [`codebase_index_v1.md`](codebase_index_v1.md): scenario row vs water/research rows.

**Accept:** audit note in this plan Status → **ADOPTED** for P1+P2 kernels with production callers listed.

---

## 5. Order and effort

```text
P0 law (S)
  → P1 DamFlood → scenarios/ (M)     // naming + import hygiene; low behavior risk
  → P2 research kernel + drone migrate (M–L)
  → P2 plant + stub sat/comms (S)
  → P3 audit (S)
```

Parallel OK after P0: P1 and P2 are different owners/files. Do **not** invent a third “DamResearch” or “FloodResearch”.

---

## 6. Explicit non-goals

- Rewriting DamFlood stages S0–S9 narrative.
- Full capability-ecosystem institutions / discovery graph.
- New research UI.
- Baseline dam v1 lock (still blocked on matrix honesty from DF-10).

---

## 7. Todo checklist

| Id | Item | Status |
|:---|:---|:---|
| p0-law | SIGN this charter + cross-links | ✅ |
| p1-dam-move | `systems/dam_flood` → `scenario/dam_flood` | ✅ |
| p1-dam-alias | Drop time-skip alias / scene-private water smells | ✅ |
| p1-dam-doc | Benchmark plan wording = scenario only | ✅ |
| p2-research-kernel | `ResearchCapabilityBook` + RON ids | ✅ |
| p2-migrate-drone | Replace `DroneResearchState` | ✅ deleted; drones read book |
| p2-wire-consumers | Plants + stub sat/comms/manuf | ✅ |
| p3-audit | Grep + index + ADOPTED callers | ✅ |

## 8. ADOPTED callers (honest exits)

| Kernel | Production caller |
|:---|:---|
| `ResearchCapabilityBook` | `src/research/mod.rs` plugin insert; writer surface for unlocks |
| Drone consumer | `src/combat/drone_missions.rs` — `effective_modality` / `effective_sensor` / pulse |
| Plant / building gate | `src/economy/activation/bridge.rs` — `required_research_ids_for_catalog` |
| Manuf yield | `src/entities/production/core/manufacturing_plugin.rs` — `tick_manufacturing_nodes` |
| Sat/comms/manuf stubs | `sync_research_consumer_stubs_system` → `ResearchConsumerStubs` |
| Building JSON | `uas_final_assembly` → `manuf.uas_assembly`; `missile_body_plant` → `manuf.cruise_line` |
| DamFlood scenario | `src/scenario/dam_flood/` only; water truth in `src/systems/water/` |

### P3 audit (2026-11-16)

- Grep `ResearchState`: **no** `*ResearchState` types remain in `src/` (orphan `drone_research.rs` removed).
- Grep `FloodSystem` / `systems/dam_flood`: **absent**; only `DamFloodPlugin` under `scenario/`.
- Smell tests Accept via unit tests in `src/research/mod.rs`.
