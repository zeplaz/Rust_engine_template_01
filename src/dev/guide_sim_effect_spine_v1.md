# GUIDE-SIM-EFFECT-SPINE-001 — CAUSE→EFFECT→OBSERVE→LEARN↺ `v1`

```text
⟦SYMLANG⟧⟐v1  ◈GUIDE  ◈NORMATIVE
⟨ID⟩ GUIDE-SIM-EFFECT-SPINE-001  ·  ARCH-REVIEW◈SIM-EFFECT-SPINE
Q↑↑  Ct▮▮  Cx▮▮▮▮▮▮▮▮  EV/Cx=HIGH  Au:🏛🟨🟨🟨🟨
Parent: $ref:src/dev/plan_sim_effect_spine_exec_001_v1.md
Lang: $ref:src/dev/agent_lang_v1.md · $ref:prompts/SYMBOLIC_LANGUAGE.meta.md
Ops: $ref:src/dev/ops_truth_memory_split_v1.md · $ref:src/dev/plan_agent_operations_intelligence_v1.md
Hydro ref: $ref:docs/archive/2026-06-src-dev/plans/plan_construction_hydro_coupling_001_v1.md
Fire inlet: $sym:EmberSpotIgnitionEvent@src/systems/fire/ember_spot_ignition.rs
Status: **ACTIVE** · Date: 2026-06-11
```

**Rule:** Fire is the **first consumer**, not the spine. The asset is the loop **CAUSE→EFFECT→OBSERVE→LEARN↺** riding one backbone for all domains.

---

## STORAGE-THREE-WORLDS◈ (read first)

Three storage classes — **do not merge**:

```text
① DEV POSTGRES 🧠     workstation · agent/dev progress · NOT in game binary
② GAME HOT PATH ⚡    ECS + in-memory rings · default for sim tick authority
③ GAME PERSISTENCE 💾 FS/RON saves today · optional embedded DB when warranted
```

```text
┌─ ② GAME RUNTIME — proc_A_dine01 ─────────────────────────────────────────┐
│  DEFAULT: ECS resources · Bevy Messages · capped in-memory EventLog rings │
│  SimEffectQueue · dispatch · observation — hot path, no I/O per tick      │
│  Saves: Wave S FS/RON ($ref:src/io/save/) · chunk cache spill ($ref:chunk_cache) │
│  Witness (dev runs): debug_runs/*.json · debug_runs/sim_effects/*.jsonl    │
│  ⛔ NO dev Postgres · ⛔ NO network DB · ⛔ NO DB on tick-critical path unless gated │
└───────────────────────────────────────────────────────────────────────────┘

┌─ ① DEV WORKSTATION — agents · MCP · orchestrator ────────────────────────┐
│  Optional local Postgres → development progress & cross-run analytics       │
│  Indexes COPIES of JSONL witnesses — never sim authority                    │
└───────────────────────────────────────────────────────────────────────────┘
```

| Class | Where | Purpose | In `proc_A_dine01`? |
|:---|:---|:---|:---:|
| **Hot sim** | ECS / RAM | tick authority, SimEffect drain, combat/fire this frame | ✅ always |
| **Game persist** | FS (today) · embedded DB **if gated** | saves, replay, long event history, spill index | ✅ when needed |
| **Dev Postgres** | Developer machine | agent runs, KPIs, causal analytics across dev sessions | ❌ never |

**Principle:** use a **game DB only where it helps gameplay or performance** — not by default, not for dev telemetry. When in doubt: ECS → FS → embedded store (last, gated).

---

## VERDICT◈

```text
◉Proposal: MOSTLY-CORRECT — reframe primary deliverable

  NOT  FireIgnitionSpine
  IS   SimEffectSpine + TelemetryGraph + QueryableCausalHistory

  Fire🔥 = P2 consumer (proves waist works)
  Quest🎯 NPC🧍 Grid⚙ Weather⚡ … = later consumers (same bus)
```

```text
                    Weather⚡ Grid⚙ Logistics🚚 NPC🧍 Economy💰
                    Combat💥 Script📜 Quest🎯 Narrative🗞
                           ╲    │    │    │    │    ╱
                             ◉SimEffectSpine◉
                                    │
              ╔═════════════════════╪═════════════════════╗
              ▼                     ▼                     ▼
         WorldState            EventLog              AI Memory
              ▼                     ▼                     ▼
          Runtime               Player            Agent/game AI

```


| Sym | Meaning                                        |
| --- | ---------------------------------------------- |
| ◉   | Spine node — single authority contract         |
| ↺   | Closed learning loop (supervisor / designer)   |
| ⚡   | Trigger ingress (human, script, sim threshold) |


---

## GAME-STORE-GATE◈ (when a DB inside the game *is* appropriate)

**SimEffect spine P0–P1:** ECS queue + in-memory ledger + JSONL witness export. **No embedded DB required.**

Add an **in-game embedded store** (e.g. SQLite/rocksdb — pick per slice) **only** when gameplay or performance needs it:

| Candidate use | Helps | Default until gate | Gate signal |
|:---|:---|:---|:---|
| **Player event log** (P3) scrollback + filter | gameplay readability | capped ring in RAM (~256–1k rows) | seek/filter at 10k+ events or save must preserve log |
| **Causal chain UI** ("why did this fire start?") | gameplay | in-memory graph for active session | chain history exceeds RAM or reload requirement |
| **Replay / timeline scrub** | gameplay + debug | FS chunk artifacts — $ref:src/io/save/ | indexed seek on FS over budget |
| **Chunk cache spill index** | performance | FS + HashMap — $ref:src/io/streaming/chunk_cache.rs | cold-load / spill dir over budget |
| **Mod / asset lookup cache** | load-time perf | Bevy asset server + FS | profiling fails load target |

```text
◆ need storage ?
 ├─═[tick authority]▶ ECS only — never DB on sim hot path
 ├─═[save/load blob]▶ FS/RON Wave S — existing spine
 ├─═[indexed query · long history · spill]▶ embedded DB — EV/Cx gate per slice
 └─═[cross-dev-session analytics]▶ dev Postgres — NOT in game binary
```

**EV/Cx gate (game embedded DB):** approve when **measured** need ≥ 1.0 · defer when speculative.

**Forbidden:** ECS replacement on tick · dev agent telemetry · network DB on gameplay critical path.

---

## SPINE-LAYERS◈ (form I — do not collapse)

```text
⊚Sim plane     Trigger⚡ → Condition◇ → SimEffect◉ → Dispatch → WorldMutation  (RUNTIME)
⊚Observe plane Observation → EventLog → Player/in-game AI read surfaces      (RUNTIME)
⊚Learn plane   Runtime telemetry → JSONL witness → (export) dev KPI↺          (RUNTIME → DEV)
```

| Layer           | Owns                                                 | Forbidden                                      |
| --------------- | ---------------------------------------------------- | ---------------------------------------------- |
| **SimEffect**   | authoritative enqueue + tick drain + domain dispatch | prose, HUD strings, **DB on tick hot path** |
| **Observation** | capped rings, structured rows, lazy NL               | direct world writes · dev Postgres          |
| **Telemetry**   | IDs, causal graph, runtime JSONL ledger              | world content · **dev PG in sim crate**     |


**L3 evidence:** lone ✅ banned — close with 🧪 lib witness · 📜 JSON path · ⊚ authority map.

---

## CRITICAL-MISSING-PIECE⚠ — TELEMETRY◉ (runtime first)

**Runtime telemetry** (in sim + on disk after run) is mandatory. **Postgres** is optional dev-ops export only — see DEV-OPS-POSTGRES-ROLE◈ below.

```text
◆ naive spine ?
  Producer ═▶ SimEffect ═▶ Dispatch ═▶ Observation
  verdict: 🔴 another bus — no learning loop

◆ complete spine ?
  Producer ═▶ SimEffect ═▶ Dispatch ═▶ Observation
                              │
                              ▼
                         TELEMETRY◉
                              │
              ╔═══════════════╪═══════════════╗
              ▼               ▼               ▼
           Metrics         Witness          Learning
              │               │               │
              └───────────────┴───────────────┘
                              ▼
                            KPI↺
```

**Without Telemetry◉:** agents cannot answer operational questions at scale.

**With runtime Telemetry◉ (+ optional dev PG export):** agents answer across dev sessions:


| Q                               | Sym | Telemetry slice                          |
| ------------------------------- | --- | ---------------------------------------- |
| Which triggers create gameplay? | 📊  | `trigger_fired` × `world_delta` join     |
| Which events spam?              | 📊  | rate / dedupe reject count               |
| Which effects are ignored?      | 📊  | dispatch ok ∧ no downstream delta        |
| What burns CPU?                 | 📊  | drain_ms · batch_size · chunk_activation |
| What creates player engagement? | 📊  | observation_impression × session (later) |


```text
LightningStrike#22
      │ ▷
IgniteCells#71
      │ ▷
FireSpread#93
      │ ▷
Observation
      │ ▷
Telemetry◉
      │ ▷
KPI↺
```

---

## ID-GRAPH◈ (causal chain — queryable)

Every drained effect carries:


| Field            | Sym       | Role                               |
| ---------------- | --------- | ---------------------------------- |
| `SimEffectID`    | `#nnn`    | unique row this drain              |
| `RunID`          | `RUN-*`   | session / cargo run / scenario run |
| `ScenarioID`     | `SCN-*`   | optional scenario file id          |
| `CauseID`        | `CAUSE-*` | producer class + instance          |
| `ParentEffectID` | `#parent` | chain link                         |


```text
Lightning#22
      │ parent=
      ▼
Ignition#71
      │
      ▼
StructureFire#93
      │
      ▼
PowerLoss#141
      │
      ▼
FactionStress#202
```

**Storage policy (two worlds — do not merge):**

| Tier | Where | Content | In game? |
| --- | --- | --- | :---: |
| **T1 truth** | Git + FS | specs, RON triggers, witness JSON bodies, saves | ✅ sim reads FS |
| **T2 working** | FS | HANDOFF, queues, `debug_runs/agent_ops/*.jsonl` | ❌ tooling only |
| **T3 runtime tel** | FS `debug_runs/sim_effects/*.jsonl` | effect rows, causal edges per run | written at dev/test exit · **not PG** |
| **T4 dev analytics** | Optional local Postgres | **copies/index** of T2/T3 for cross-run KPI | ❌ **never in game binary** |

⛔ **Never** in Postgres: world content, saves, assemblies, sim authority — $ref:src/dev/ops_truth_memory_split_v1.md.

✅ **Dev workstation Postgres only** (when gated): development progress — agent runs, effect-telemetry **index**, witness **index**, trigger stats, causal-chain **analytics** for humans/agents building the game — $ref:src/dev/ops_sql_workstation_arch_v1.md.

---

## DEV-OPS-POSTGRES-ROLE◈ (NOT in game — workstation backend only)

```text
⛔ NOT USED IN proc_A_dine01 · NOT a gameplay system · NOT player-facing

FILESYSTEM 🏛  (game + repo truth)     DEV POSTGRES 🧠  (dev progress tracking)
  ├─ assets                              ├─ agent_runs
  ├─ source                              ├─ effect_telemetry (index of JSONL copies)
  ├─ saves                               ├─ observations (dev rollup)
  └─ handoffs                            ├─ KPIs
                                         ├─ witness_records (path/green index)
                                         ├─ trigger_statistics
                                         └─ causal_chains (cross-run analytics)
```

| Question | Answer |
|:---|:---|
| Does the game connect to Postgres? | **No.** |
| Where does sim telemetry go at runtime? | In-memory ledger + `debug_runs/sim_effects/*.jsonl` |
| When does Postgres appear? | Optional **after** run — MCP/orchestrator **ingest** on dev machine |
| Who reads Postgres? | Agents, ops scans, supervisor — **not** Bevy systems |

**Phase 1 (now):** runtime JSONL + lib witness — **zero Postgres**.

**Phase 4 (dev gate):** optional ingest → **local workstation** Postgres — $ref:src/dev/plan_agent_operations_intelligence_v1.md (>500 structured events OR cross-run analytics need).

---

## LONG-TERM-LOOP◈ (form H — supervisor cycle)

```text
Trigger⚡
   │ ▷
Condition◇
   │ ▷
SimEffect◉
   │ ▷
Dispatch
   │ ▷
WorldMutation
   │ ▷
Observation
   │ ▷
Telemetry◉
   │ ▷
Analytics
   │ ▷
Supervisor↺
   │ ▷
ToolFeedback
   │ ▷
Designer
   │ ▷
Trigger⚡   …↺
```

**This loop > fire system.** Fire proves the waist; the loop enables quests, NPCs, logistics, warfare, narrative, LLM supervisors, debugging.

---

## DOMAIN-FAN-IN◈ (consumers — one spine)


| Domain      | Producer (today)          | Target dispatch                 | Status     |
| ----------- | ------------------------- | ------------------------------- | ---------- |
| Fire🔥      | ecology ember only        | `EmberSpotIgnitionEvent`        | 🟡 partial |
| Weather⚡    | `lightning_risk` field    | `SimEffect::LightningStrike`    | 🔴         |
| Grid⚙       | `GridOverloadEvent`       | thermal → structure catastrophe | 🔴         |
| Hydro💧     | construction execute      | `HydrologyEventQueue`           | 🟢 pattern |
| Script📜    | `EngineScriptHost`        | `SimEffect::ScriptInject`       | 🔴         |
| Scenario🎯  | objectives stub           | trigger → effect                | 🔴         |
| NPC🧍       | `ScriptInfluence`         | **not** effects — pressure only | 🟡         |
| Narrative🗞 | `NarrativeObservationBus` | derived from **runtime** telemetry rows | 🟡         |


**Adapter rule:** existing domain queues (`HydrologyEventQueue`, Bevy `Message<T>`) stay — `SimEffectQueue` **drains into** them, not around them.

---

## CPU-DISCIPLINE◈


| Mechanism              | Ref                                  | Sym |
| ---------------------- | ------------------------------------ | --- |
| Tick dedupe            | `HydrologyEventQueue.push`           | 💰  |
| Batch drain once/frame | `ChunkEnvironmentSet` ordering       | 🏛  |
| Chunk LOD activation   | `chunk_sim_lod.rs`                   | 💰  |
| Observation cap        | `NarrativeObservationBus` BUS_CAP=48 | 💰  |
| Telemetry sample       | ring + aggregate per tick            | 💰  |
| Edge-trigger only      | overload ratio, lightning spike      | 🎯  |


⛔ per-frame global Query for "any event" · ⛔ NL on hot path · ⛔ unbounded `Vec` without cap.

---

## PRIORITY-ORDER◈ (canonical — exec plan follows)


| P      | Slice                  | Deliverable                                  | Unblocks           |
| ------ | ---------------------- | -------------------------------------------- | ------------------ |
| **P0** | `SIM-EFFECT-QUEUE-001` | `SimEffectQueue` + drain + dispatch adapters | all consumers      |
| **P1** | `SIM-EFFECT-TEL-001`   | IDs + causal graph + **runtime JSONL** (no PG) | agent dev analytics |
| **P2** | `FIRE-IGNITION-P0-001` | Lightning + transformer → ember waist        | operator play fire |
| **P3** | `EVENT-LOG-UI-001`     | structured player event log (RAM first; DB if gated) | human read         |
| **P4** | `SCENARIO-TRIGGER-001` | `ScenarioStep::EmitSimEffect` + RON triggers | editor/script      |
| **P5** | `FACTION-REACT-001`    | NPC/faction hooks on telemetry rows          | behavioral sim     |
| **P6** | `NARRATIVE-GEN-001`    | lazy NL from structured rows                 | flavor             |


Fire render/VFX harness: **parallel** — $ref:src/dev/plan_product_polish_exec_001_v1.md · `--test fire|vfx|visual` unchanged.

---

## RISK-HEAT◈

```text
Fire System           [███░░░░░░░]  🟡 known domain · wrong center of gravity
Event Bus             [█████░░░░░]  ⚠ fan-in complexity
Telemetry             [██████░░░░]  ⚠ schema drift if IDs skipped
Scenario Integration  [███████░░░]  ⚠ editor scope creep
Agent Analytics       [█████████░]  ⚠ dev PG temptation before JSONL proves value
```

---

## COMPLEXITY-TOPOLOGY◈

```text
Fire              ███
Grid              ████
Scenario          █████
Event Spine       ███████
Telemetry         ████████
Agent Analytics   ██████████◉  ← highest long-term leverage
```

**Headline:** ◉SimEffect Spine + Telemetry Graph + Queryable Causal History — not "fire ignition."

---

## WITNESS-KEYS◈ (runtime JSON — game/dev runs; not Postgres)


| Witness       | Path                                          | Green when                                          |
| ------------- | --------------------------------------------- | --------------------------------------------------- |
| Queue wired   | `debug_runs/sim_effect_spine_live.json`       | `queue_drain_ok` · `dedupe_ok`                      |
| Telemetry     | same                                          | `causal_chain_depth_max` ≥ 1 · `effect_rows` > 0    |
| Fire consumer | `debug_runs/fire_ecology_live.json`           | external `ember_events_emitted` > 0 in play fixture |
| Ops gate      | `debug_runs/agent_ops/ops_report_latest.json` | `sim_effect_telemetry` row present                  |


---

## ANTI-PATTERNS◈


| Don't                       | Do                                                     |
| --------------------------- | ------------------------------------------------------ |
| Fire-only parallel bus      | dispatch → `EmberSpotIgnitionEvent`                    |
| Embedded DB without EV/Cx gate | RAM ring → FS save → gated embedded store per GAME-STORE-GATE◈ |
| Confuse dev PG with game store | Three worlds: dev PG · game hot ECS · game persist (FS/DB gated)  |
| ScriptInfluence as trigger  | `SimEffect` for world mutations                        |
| Observation writes sim      | derive after drain                                     |
| Skip ParentEffectID         | causal graph useless without links                     |
| Harness fire in normal play | $ref:src/systems/fire/play_fire_visibility.rs contract |


---

## BLANG-ROUTING◈

```text
AUTH: SimEffectQueue○ → RuntimeTelemetry○ → Observation○ → DevPG🧊 (workstation only)
ΔWF→@planner sign $ref:src/dev/plan_sim_effect_spine_exec_001_v1.md
     → @coder P0 SIM-EFFECT-QUEUE-001
     → @coder P1 SIM-EFFECT-TEL-001 (same sprint if thin)
     → @coder P2 FIRE-IGNITION-P0-001
⏸ Dev PG ingest until JSONL > 500 rows OR cross-run KPI need — never in proc_A_dine01
```

---

## RELATED◈


| Doc                                                                                                                                                       | Role                             |
| --------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| `[plan_sim_effect_spine_exec_001_v1.md](plan_sim_effect_spine_exec_001_v1.md)`                                                                            | Phased exec + acceptance         |
| `[plan_construction_hydro_coupling_001_v1.md](../docs/archive/2026-06-src-dev/plans/plan_construction_hydro_coupling_001_v1.md)`                          | Queue dedupe pattern             |
| `[scenario_campaign_scripted_tools_runbook_v1.md](../docs/archive/2026-06-prompts-guides/runbooks/guides/scenario_campaign_scripted_tools_runbook_v1.md)` | Script host vision               |
| `[fire_ecology_f1_todos.md](fire_ecology_f1_todos.md)`                                                                                                    | Fire sim truth (not spine owner) |
| `[plan_product_polish_exec_001_v1.md](plan_product_polish_exec_001_v1.md)`                                                                                | Render/play polish (parallel)    |


```text
⟦/GUIDE-SIM-EFFECT-SPINE-001⟧  NEXT ⚑ $ref:plan_sim_effect_spine_exec_001_v1.md · P0 queue · P1 IDs
```

