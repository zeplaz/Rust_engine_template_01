# Scenario editor — scripted engine tools & campaign authoring `v1`

> **STATUS:** Decisions locked · **§4b satisfied** · **Waves 1–3 landed** — schema **v2** objectives + validation + discovery (`toggle_scenario_script_panel`, default F10); **Wave 4** golden operational scenario + camera focus. Use **§4c–§4e** before deep UX changes.

Version: `v1.0.6`  
Audience: agents wiring **scenario / campaign** authoring **inside** editor tooling (alongside map edit, faction tools, transport bake).

**Parents:** [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md) (editor shell UX), [`gui_runbook_v1.md`](gui_runbook_v1.md) (authority + `TEMP-EGUI`), [`map_editor_runbook_v1.md`](map_editor_runbook_v1.md) (world tiles + **M5** snapshot).  
**Data policy:** scenario vs save vs assets — [`../designer_questions/factions/faction_editor/03_persistence.md`](../designer_questions/factions/faction_editor/03_persistence.md).

---

## 1. Problem

Campaign and scenario authoring need **repeatable, testable** steps that the **engine** executes (not hand-clicked QA): load a map, apply faction roster, set objectives, step the sim, export a bundle. Today map edit and transport dev saves exist; there is **no unified script host** for editor-time **or** headless runs.

---

## 2. Scripted engine tools (definition)

**Scripted engine tools** = a **versioned command stream** (steps) interpreted by a single **`EngineScriptHost`** (or equivalent) inside the Bevy app:

- **Deterministic** for fixed `scenario_version` + seed + committed assets (same invariant as diplomacy / scenario specs elsewhere).
- **Authoritative mutations** go through the same paths as manual tools **messages**, **`Commands`**, **orchestrators**, **existing sim APIs** — never hidden ECS “god writes.”
- **Persistence:** **RON = authoring source of truth** (comments, hand edit, diffs, nested structs, debugging). **JSON = optional** export / runtime interchange (CI validation, external tools, mods, telemetry/replay export). Align exports with [`serialization_hybrid_migration_matrix_v1.md`](../matrix/serialization/serialization_hybrid_migration_matrix_v1.md).

**Filename convention:** `*.scenario.ron` (authoritative), `*.scenario.json` (optional export). Examples: `ukraine_frontline_2028.scenario.ron`, `power_grid_failure_test.scenario.ron`.

Minimal **step kinds** (v1 — extend with new optional fields without bumping `schema_version`):

| Kind | Intent |
|:---|:---|
| `LoadWorldProfile` | Point at `WorldProfile` / tuning bundle (existing terrain U7 path). |
| `LoadMapSnapshot` | Hydrate from **M5** path or `MapSnapshotV1` (see `terrain::editor::map_snapshot`). |
| `ApplyFactionBlueprints` | Merge roster / stance from scenario slice (align **G3A** data). |
| `SimAdvance` | `N` ticks **only** via [`SimControlState`](../../src/systems/sim_control.rs) — pause, step, fast-forward; **no** ad-hoc time loops. |
| `Expect` / `Assert` | Lightweight checks for CI — optional in v1. |
| `ExportScenarioBundle` | Optional JSON interchange + header paths per matrix policy. |

Editor panels **enqueue** or **edit** this stream; **Run** drains it in **Editor** and **headless** tests; **runtime campaign** replay is **limited** in v1 (expand later).

---

## 3. Scenario editor surface (scope)

**Scenario editor tools** = editor mode(s) under [`BaseState::Editor`](../../src/engine/states.rs) that bundle:

1. **World** — M5 snapshot / world gen params (map editor patterns).
2. **Factions** — G3A resource model; no duplicate roster store.
3. **Campaign** — objectives / triggers (DTOs + ECS markers); not linear RTS mission chains — see **vision** below.
4. **Script** — list + reorder steps; validate; **run** / **stop** with documented authority.

**UI:** **`TEMP-EGUI` for v1** (tooling-only, rapid iteration) — integrated with map editor / scenario / overlay debug per [`ui_boundary_guide_v1.md`](ui_boundary_guide_v1.md). Player-facing campaign shell → **Bevy UI** later.

### 3.1 Vision — operational sandbox authoring

Scenarios should chiefly **initialize conditions**, **set pressures**, **define actors/objectives**, and **inject events** — then let the **simulation** evolve. Avoid hardcoded linear missions, cutscene chains, and brittle script-only outcomes as the default product shape.

---

## 4. Prerequisite gate (before implementation)

### 4a. Decision record — **signed (author, 2026-05)**

| Topic | Decision |
|:---|:---|
| **A. Serialization** | RON authoritative; JSON optional export/interchange. |
| **A. Layout** | Authored content under **`assets/scenarios/`** — e.g. `templates/`, `campaigns/`, **`tests/`**, `generated/`. **Do not** put scenarios under `saves/` or mix with runtime save blobs. |
| **A. Extensions** | `*.scenario.ron`, `*.scenario.json`. |
| **A. `schema_version`** | `pub schema_version: u32`, start **1**. Bump only on **structural** / **incompatible execution** changes — **not** for balance, tuning, or new **optional** fields. |
| **B. Contexts** | **Editor:** yes. **Headless:** yes (CI / replay / verification). **Runtime campaign:** **limited** v1. |
| **B. Mutations** | **Only** through messages, commands, orchestrators, existing sim APIs. |
| **B. Sim stepping** | **Must** use `SimControlState` (pause / step / fast-forward). |
| **C. Golden fixture** | **Not** empty or single-tile — use **`assets/scenarios/tests/golden_operational_minimal_v1.scenario.ron`** once built: tiny operational map + **one** logistics chain + **one** power chain + **one** settlement + **one** faction + **one** road + **one** utility line — enough for serialization, load, overlays, infra references, scripting hooks. |
| **D. Panel** | **`BaseState::Editor`**, integrated with map / scenario / overlay / script validation — not main-menu popup flow. |
| **E. Objectives (wave 3)** | **DTO + ECS stub markers** (e.g. region capture, destroy infra, maintain supply) — map-linked / trigger anchors; **not** resources-only. **Win/lose** logic **out of scope** wave 3 (DTOs, hooks, serialization, triggers only). |
| **F. Program order** | **Investment order:** infra + logistics + corridors + operational fields **first** — scenario tooling is **much more valuable** once those exist (scripts stay non-brittle). **Rollout phases:** **P1** infra + logistics core → **P2** overlays + op fields → **P3** scenario scripting → **P4** campaigns → **P5** replay / analytics. Scenario waves (§5) track **P3**; design can proceed in parallel, **implementation weight** follows P1–P2 maturity. |

### 4b. Governance (backlog tracking)

- [x] **Matrix or backlog:** **[BQ-113](rulebook_backlog_designer_brief_v1.md)** — scenario scripting + golden fixture ownership/tracking ([`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4).

**Ownership (recommended):**

| Field | Value |
|:---|:---|
| **Owner** | Scenario / editor tooling lead |
| **Execution phase** | P3 |
| **Blocking wave(s)** | Wave 4 golden fixture validation |
| **Related systems** | Map editor (**M5**), logistics, power, faction DTO hydration |

---

## 4c. Authoring Q&A — use before Wave 4+ / UX hardening

Answer in tickets or this doc; unset = keep current defaults.

| # | Question | Why it matters |
|:---:|:---|:---|
| **Q1** | Should **`region_key`** adopt a **single** convention (`tile:x,z` vs `chunk:ix,iz` vs named **region ids**)? | Unifies overlays, map pickers, and future **RegisterObjectives** tooling. |
| **Q2** | Do objectives need **faction / side** attribution in v1 (`faction_id` / tag string)? | Blocks fair filtering in HUD and strategic AI consumers. |
| **Q3** | **Clear policy:** should `RegisterObjectives { clear_existing: true }` also wipe **non-scenario** markers if we add other objective sources later? | Avoids accidental despawn when multiple authoring tracks exist. |
| **Q4** | Panel **discoverability:** dedicate a **toolbar strip** or **keyboard** open for “Scenario script”, or keep as floating window only? | Affects daily author workflow vs screen clutter. |
| **Q5** | **Validation on Load:** warn on duplicate `id`, empty `label`, or unknown `kind` mapping — block or log-only? | Prevents silent bad scenarios in CI. |
| **Q6** | Should **Save** round-trip **losslessly** through `active_script` (editor never drops steps), or allow “Save = export subset”? | Legal for modding vs internal round-trip tests. |
| **Q7** | **Inspector:** add Bevy/egui list of **`ScenarioObjectiveMarker`** entities (select + focus camera) in Wave 4? | Bridges script authors to spatial context. |

### 4d. UI / UX quality notes (Wave 2–3)

- **Strengths:** one window, clear **Load path → Run → log** loop; **next-step preview** reduces “black box” anxiety; **Help** foldout points to runbook and example paths; **tooltips** on primary actions.  
- **Gaps:** no **RON syntax assist** or template insert; no **dirty** indicator if disk file changed after Load; no **step index** / progress bar (only `pending` count); **Runbook** path is plain text (OK for devs, not discoverable for non-repo authors).  
- **Doc hygiene:** canonical spec remains **this file** + `src/scenario/*.rs` module docs; example scenarios under `assets/scenarios/tests/`.

---

## 4e. Authoring Q&A — **locked (product, 2026-05-10)**

| # | Decision |
|:---:|:---|
| **Q1** | **`ObjectiveTargetRef`:** `Region(String)` (prefer `region:<namespace>/<name>`), `Tile(IVec2)`, `Chunk(IVec2)`, `Corridor(String)`, `Site(String)`. Do not freeze objectives to tile coords only. |
| **Q2** | **Factions on DTO:** `owning_faction`, `opposing_faction` (`Option<String>`) plus **`tags: Vec<String>`**; stable **`objective_id`** (serde alias `id`), never entity id / label / order. |
| **Q3** | **`clear_existing`:** despawn **only** entities with `ScenarioObjectiveMarker` — never generic “objective-like” entities or runtime campaign/AI goals. |
| **Q4** | **Discoverability:** menu/entry window (**Editor — Scenario tools**), **hotkey** (`toggle_scenario_script_panel`, default **F10**), and **main script window**; logistics targets panel default **F6** to avoid clash. |
| **Q5** | **Load validation:** errors block load (duplicate objective ids, bad `schema_version`, missing required fields, etc.); warnings allow load (empty label, deprecated optional fields, etc.). See `ScenarioValidationReport` in `src/scenario/validation.rs`. |
| **Q6** | **Save** = full lossless authoritative RON; **Export JSON** = separate runtime subset (`export_runtime_json_subset`), not merged with Save. |
| **Q7** | **Inspector + camera focus:** yes — objective list stub in script panel; Wave 4/5 camera + overlay hooks. |
| **Golden (Wave 4)** | Fixture should exercise: settlement, power line, transformer/substation, logistics depot, road, bridge, rail spur, fuel storage, faction, objective set, maintain-supply + destroy-infrastructure objectives (interdependent ops). |

**`schema_version`:** **2** is current for new files; **1** still parses with a deprecation warning.

---

## 5. Four implementation waves (wire + implement)

Execute **in order** — **§4b** satisfied (**[BQ-113](rulebook_backlog_designer_brief_v1.md)**). Each wave ends with `cargo test -p proc_A_dine01` green and a short note under **Status**.

| Wave | Goal | Touch (typical) | Done when |
|:---:|:---|:---|:---|
| **1** | **Script host core (landed 2026-05)** — `EngineScriptHost` in `src/scenario/`, step enum + **RON** serde (`schema_version: u32`), **one step/frame** drain **`before`** [`SimControlSystemSet::AdvanceSimTick`](../../src/systems/sim_control.rs); tests use `MinimalPlugins` + `InputPlugin` + [`InputBindings`](../../src/gui/input_bindings.rs) for `SimControlPlugin`. | `src/scenario/` · `ScenarioScriptingPlugin` in [`engine_with_worldgen.rs`](../../src/engine/engine_with_worldgen.rs) | `cargo test -p proc_A_dine01 scenario_wave1` |
| **2** | **Editor wiring (landed 2026-05)** — [`scenario_script_panel`](../../src/gui/editor/scenario_script_panel.rs): `TEMP-EGUI` window on **`BaseState::Editor`** (via [`MapEditorPlugin`](../../src/gui/editor/map_editor/mod.rs)), path field (crate-relative or absolute), **Load / Save** RON, **Run/resume** + **Stop**, scrollable log; **`EngineScriptHost::resume` / `restart_from_active`**; **`ScenarioFileV1::to_ron_string_pretty`**. | `src/gui/editor/scenario_script_panel.rs` | `cargo test -p proc_A_dine01 scenario_` (includes `scenario_wave2_scenario_file_ron_roundtrip`, `scenario_script_host_resume_after_stop`) |
| **3** | **Campaign primitives (landed 2026-05)** — [`objectives.rs`](../../src/scenario/objectives.rs): **`ObjectiveTargetRef`**, `ScenarioObjectiveV1` (**`objective_id`**, factions, **`tags`**, `target` + legacy `region_key`), `ScenarioObjectiveKindV1`, ECS [`ScenarioObjectiveMarker`](../../src/scenario/objectives.rs); **`RegisterObjectives`**; **`validation.rs`** + `EngineScriptHost::last_validation`; fixture **`schema_version: 2`** [`wave3_objectives.scenario.ron`](../../assets/scenarios/tests/wave3_objectives.scenario.ron); tests `wave3_*`. Panel: validation lines, objective stub list, **Export JSON (subset)**. | `src/scenario/*.rs` · `scenario_script_panel.rs` | `cargo test -p proc_A_dine01 scenario::` |
| **4** | **Golden + CI** — build **`golden_operational_minimal_v1.scenario.ron`** (per §4a); in-process or headless replay test; optional **`golden_operational_minimal_v1.scenario.json`** export for CI tooling. | `assets/scenarios/tests/`, tests | Replay passes; **[BQ-113](rulebook_backlog_designer_brief_v1.md)** validation complete; document fixture in §6. |

**Status:**

- [x] Wave 1
- [x] Wave 2
- [x] Wave 3
- [ ] Wave 4

---

## 6. Cross-links

| Doc | Role |
|:---|:---|
| [`map_editor_runbook_v1.md`](map_editor_runbook_v1.md) | M5 — `LoadMapSnapshot` |
| [`gui_runbook_v1.md`](gui_runbook_v1.md) | TEMP-EGUI, authority |
| [`strategic_program_execution_plan_v1.md`](strategic_program_execution_plan_v1.md) | Macro P1–P8; **scenario scripting ≈ program P3** per §4a |
| [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md) | Scripts read fields; write **intents** through APIs only |
| [`mission_authoring_framework_v1.md`](mission_authoring_framework_v1.md) | Mission / objective vocabulary for scripted waves |
| [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) | **BQ-113** — scenario scripting + golden fixture governance |
| [`experience_layer_ux_hud_designer_brief_v1.md`](experience_layer_ux_hud_designer_brief_v1.md) | Mission-driven **transmissions** after **UX-3** shell — not before |

**Golden fixture (target path):** `assets/scenarios/tests/golden_operational_minimal_v1.scenario.ron`  
**Wave 1 fixture:** `assets/scenarios/tests/minimal_wave1.scenario.ron` — `EngineScriptHost`, `scenario_wave1_*` tests (2026-05).  
**Wave 3 fixture:** `assets/scenarios/tests/wave3_objectives.scenario.ron` — `RegisterObjectives` + `scenario_wave3_*` tests.  
**Wave 2–3 UI:** [`src/gui/editor/scenario_script_panel.rs`](../../src/gui/editor/scenario_script_panel.rs) — **§4d** UX notes.

---

## 7. Prompt fragment for executing agents

> Read **§4a–b**. Implement **one** §5 wave. Use **RON** on disk; **JSON** only where export is explicitly in scope. Never bypass mutation rules in §2. If `golden_operational_minimal_v1` does not exist yet, W1–W3 use smaller fixtures; **W4** creates the golden per §4a. Update **Status** checkboxes and name new tests in the PR.
