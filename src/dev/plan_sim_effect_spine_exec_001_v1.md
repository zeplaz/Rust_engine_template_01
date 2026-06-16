# PLAN-SIM-EFFECT-SPINE-001 — SimEffect queue · telemetry graph · fire as first consumer `v1`

```text
⟦SYMLANG⟧⟐v1  ◈EXEC
⟨ID⟩ PLAN-SIM-EFFECT-SPINE-001  ·  GUIDE-SIM-EFFECT-SPINE-001
Q↑↑  Ct▮▮  EV/Cx=HIGH  Status: **SIGNED** (@planner 2026-06-11)
Guide: $ref:src/dev/guide_sim_effect_spine_v1.md
Program: POST-DRAIN-PHASE-4-001 (gameplay spine lane)
Date: 2026-06-11
Owner: @orchestrator → @planner → @coder / @designer
Not in scope: Stage 5 reopen · MCP art · **dev Postgres in game** · full munitions sim
```

**Storage policy (three worlds):**
1. **Dev Postgres** — workstation dev progress only; never in `proc_A_dine01`.
2. **Game hot path** — ECS + in-memory (SimEffect queue, EventLog rings); default.
3. **Game persistence** — FS/RON saves today (`src/io/save/`); **embedded DB only when gameplay/perf gate passes** — $ref:src/dev/guide_sim_effect_spine_v1.md §GAME-STORE-GATE◈.

**Headline:** Build **CAUSE→EFFECT→OBSERVE→LEARN↺** — fire proves the waist; telemetry makes the spine learnable.

---

## Problem

| Gap | Sym |
|:---|:---|
| Domain buses fragmented (hydro queue, Bevy messages, fracture bus, narrative tail) | 🕸 |
| No unified **cause → effect** vocabulary for editor/script/AI | 🔴 |
| `EmberSpotIgnitionEvent` exists but almost no external producers | 🔴 |
| Observation without telemetry = another bus, not a learning loop | ⚠ |
| Harness fire (`--test fire\|vfx`) ≠ normal play ignition | 🟡 by design |

Render path: sim heat → overlay → sparks (**TRIAGE-FIRE-PLAY-VIS-001** 🟢). **Still open:** sim producers for lightning, grid blow, script inject.

---

## STORAGE-THREE-WORLDS◈

| Class | Runs in | Default | Embedded DB? |
|:---|:---|:---|:---|
| Sim tick authority | `proc_A_dine01` | ECS resources, Bevy messages | ❌ never on hot path |
| SimEffect + P1 telemetry | `proc_A_dine01` | in-memory + JSONL export | ❌ P0–P1 |
| Player event log (P3) | `proc_A_dine01` | capped RAM ring | 🟡 if GAME-STORE-GATE passes |
| World saves | `proc_A_dine01` | FS/RON Wave S | 🟡 future if indexed save/query needed |
| Dev analytics | Workstation / MCP | optional Postgres **index** | ✅ dev only |

**Hard rules:**
- No **dev Postgres** in game binary.
- No **embedded DB** on SimEffect tick drain without EV/Cx gate + witness.
- In-game DB is **allowed and encouraged** when it measurably helps gameplay (event log, replay, causal UI) or performance (spill index, cache) — not "never."

---

## Authority spine (do not invert)

```text
Trigger⚡ → SimEffectQueue (single enqueue writer surface)
              │ drain (1×/tick)
              ▼
         Dispatch adapters ─┬─▶ EmberSpotIgnitionEvent → ChunkFireOverlay
                           ├─▶ HydrologyEventQueue (adapter)
                           └─▶ (future) damages / logistics / faction
              │
              ▼
         TelemetryRecord (IDs + ParentEffectID) → JSONL on disk
              │                                    (optional dev PG export — not sim)
              ▼
         Observation rings (EventLog · Narrative · in-game AI snapshot)
```

| Resource | Single writer |
|:---|:---|
| `SimEffectQueue` | `sim/effects/drain.rs` drain system |
| `ChunkFireOverlay` heat | existing fire tick + `apply_ember_spot_ignitions` |
| `HydrologyEventQueue` | hydrology drain (unchanged) |
| `SimEffectTelemetryLedger` | telemetry append system (after drain) — **runtime only** |

⛔ second placement writer · ⛔ render-time ignition · ⛔ dev Postgres in game · ⛔ DB on SimEffect tick drain (P0–P1)

---

## Phases (revised priority — guide is normative)

### P0 — `SIM-EFFECT-QUEUE-001` (@coder)

**Goal:** Tick-drained queue + dispatch adapters (hydro + fire waist).

| Task | Exit |
|:---|:---|
| `src/sim/effects/` module | `SimEffectEvent` enum · `SimEffectQueue` (dedupe like `$ref:substrate/hydrology/event_bus.rs`) |
| Drain in `Update` after `SimControlSystemSet::AdvanceSimTick` | one drain/frame |
| Fire adapter | maps `IgniteCells` / `LightningStrike` → `EmberSpotIgnitionEvent` |
| Hydro adapter | maps `HydroDirty` → existing `HydrologyEventQueue.push` |
| Lib test | drain order · dedupe · no double apply |

**Witness:** `debug_runs/sim_effect_spine_live.json` → `queue_drain_ok: true`

### P1 — `SIM-EFFECT-TEL-001` (@coder — same wave if thin)

**Goal:** Runtime telemetry graph (JSONL) — without this P0 is "another bus." **Not** a Postgres milestone.

| Field | Required |
|:---|:---:|
| `sim_effect_id` | ✅ |
| `run_id` | ✅ |
| `scenario_id` | optional |
| `cause_id` | ✅ |
| `parent_effect_id` | ✅ |
| `source` enum | ✅ |
| `tick` | ✅ |

| Task | Exit |
|:---|:---|
| Append JSONL | `debug_runs/sim_effects/effects.jsonl` |
| Causal chain in witness | `causal_chain_depth_max >= 1` in lib fixture |
| Rollup brief | MCP/CLI `sim_effect_brief` (compress=4) — optional stub ok |

**Witness keys:** `effect_rows`, `dedupe_rejected`, `drain_us` (sample)

**Dev Postgres:** 🧊 defer — optional workstation ingest only — $ref:src/dev/plan_agent_operations_intelligence_v1.md gate. **Never** linked from `proc_A_dine01`.

### P2 — `FIRE-IGNITION-P0-001` (@coder)

**Goal:** First **external** producers — proves spine in operator play.

| Producer | Condition | Effect |
|:---|:---|:---|
| Lightning | `ChunkWeather.lightning_risk` edge + strike roll | `SimEffect::LightningStrike` → ember batch |
| Transformer | `GridOverloadEvent` + `thermal >= max` | `SimEffect::StructureCatastrophe(Transformer)` → ember/heat |

⛔ harness seed in normal Simulation — $ref:src/systems/fire/play_fire_visibility.rs

**Witness:** `fire_ecology_live.json` → `ember_events_emitted` from non-ecology `cause_id`

**Parallel (render):** $ref:src/dev/plan_product_polish_exec_001_v1.md P2 fire product finish

### P3 — `EVENT-LOG-UI-001` (@designer + @coder)

**Goal:** RTS-style structured event log — not prose-first.

| Surface | Content | Storage |
|:---|:---|:---|
| Player event log panel | category · severity · target ref · tick | **RAM ring first** |
| Minimap ping hook | optional P3.1 | in-memory |
| AI snapshot | last N structured rows | in-memory |
| Long scrollback / filter / save-load log | if gate passes | embedded DB per GAME-STORE-GATE◈ |

Derive from runtime telemetry ledger — **do not** subscribe sim systems to UI. DB is a **P3 optional** perf/gameplay layer, not P0 spine.

### P4 — `SCENARIO-TRIGGER-001` (@coder)

**Goal:** Editor/script inject via authoritative host.

| Task | Exit |
|:---|:---|
| `ScenarioStep::EmitSimEffect { .. }` | `$ref:src/scenario/scenario_steps.rs` |
| RON `TriggerSpec` sketch | `assets/scenarios/triggers/` |
| `EngineScriptHost` drain | enqueue only — no god writes |

Runbook: $ref:docs/archive/2026-06-prompts-guides/runbooks/guides/scenario_campaign_scripted_tools_runbook_v1.md §3.1

### P5 — `FACTION-REACT-001` (@coder + @planner slice)

**Goal:** Behavioral hooks read telemetry — not direct fire ECS.

| Hook | Input |
|:---|:---|
| Faction stress edge | `StructureFire` / `PowerLoss` telemetry rows |
| `FractureEventBus` | adapter from structured effects (not new parallel fire path) |

`ScriptInfluence` stays pressure-only — $ref:src/strategic/behavior_script.rs

### P6 — `NARRATIVE-GEN-001` (@designer)

**Goal:** Lazy NL from `(source, severity, target)` → `NarrativeObservationBus` / Transmission

---

## Test / VFX harness contract (unchanged)

```powershell
cargo run -p proc_A_dine01 --release -- --test fire
cargo run -p proc_A_dine01 --release -- --test weather      # no auto fire seed
cargo run -p proc_A_dine01 --release -- --test atmosphere
cargo run -p proc_A_dine01 --release -- --test visual
cargo run -p proc_A_dine01 --release -- --test vfx
cargo run -p proc_A_dine01 --release -- --test renderdebug
```

Map: `$ref:src/engine/launch_args.rs` `TestScene` · seeds via `$ref:src/engine/test_harness.rs`

**Rule:** harness ≠ play producers. Play fire requires P2 sim effects.

---

## Acceptance (operator + lib)

| Probe | Threshold |
|:---|:---|
| P0 drain | lib green · no double-apply on dedupe |
| P1 chain | lib fixture: Lightning#n → Ignition#m parent link |
| P2 play | normal `cargo run --release` — fire appears after overload/lightning scenario (no `--test`) |
| CPU | drain_us p99 < budget TBD in witness (start logging only) |
| Regression | `cargo test -p proc_A_dine01 --lib fire_ecology` + sim_effect tests |

---

## Queue slices (orchestrator)

| ⟨ID⟩ | Agent | Priority | Depends |
|:---|:---|:---:|:---|
| PLAN-SIM-EFFECT-SPINE-001 | @planner | 0 | — (this doc) |
| SIM-EFFECT-QUEUE-001 | @coder | 0 | PLAN sign |
| SIM-EFFECT-TEL-001 | @coder | 0 | QUEUE-001 |
| FIRE-IGNITION-P0-001 | @coder | 1 | TEL-001 |
| EVENT-LOG-UI-001 | @designer **PASS** · @coder ready | 2 | TEL-001 |
| SCENARIO-TRIGGER-001 | @coder | 2 | QUEUE-001 |
| TRIAGE-FIRE-PLAY-VIS-001 | @coder | 0 | render lane (parallel) |

---

## Risk matrix

| Risk | Heat | Mitigation |
|:---|:---:|:---|
| Spine scope creep | ███████ | P0/P1 only first sprint |
| Telemetry schema drift | ██████ | lock ID fields in P1 before P2 |
| Dev Postgres in game | █████████ | Forbidden — workstation only |
| Embedded DB without gate | ██████ | RAM/FS first; GAME-STORE-GATE EV/Cx |
| Scenario editor creep | ███████ | `EmitSimEffect` only — no win/lose FSM |
| Dual fire path | ███ | single waist: `EmberSpotIgnitionEvent` |

---

## Planner sign-off

| Field | Value |
|:---|:---|
| Verdict | **SIGNED** — spine + telemetry before fire producers |
| EV/Cx | **≥ 1.0** — reuses hydro pattern; fire is consumer not product |
| Date | 2026-06-11 |
| Reframe | ◉SimEffect + TelemetryGraph > fire ignition |

### ΔWF

```text
ΔWF→@coder ⟨SIM-EFFECT-QUEUE-001⟩ ⚡P0
     then ⟨SIM-EFFECT-TEL-001⟩ (same wave)
     then ⟨FIRE-IGNITION-P0-001⟩
⏸ Dev-workstation PG · ⏸ P5/P6 until P1 witness green
```

### Orchestrator paste

```text
PLAN-SIM-EFFECT-SPINE-001 🟢 SIGNED 2026-06-11
Loop: Trigger⚡→SimEffect◉→Dispatch→Observe→RuntimeTelemetry◉→KPI↺
Fire=first consumer · DevPG=🧊 workstation only · NOT in game · harness=parallel
ΔWF→@coder SIM-EFFECT-QUEUE-001 + SIM-EFFECT-TEL-001
```

---

## Regression

```text
@coder:  cargo test -p proc_A_dine01 --lib sim_effects fire_ecology
BLANG:CARGO --cached --compress 4
Witness: debug_runs/sim_effect_spine_live.json
```
