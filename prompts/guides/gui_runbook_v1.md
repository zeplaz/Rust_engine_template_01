# GUI remediation runbook `v1`

> **STATUS:** Active orchestrator for **G3** GUI remediation. This runbook sequences faction editor, production HUD, diagnostics, and runtime in-game UI sub-packs. Rust steps remain gated per surface until data source + authority are recorded.

Version: `v1.0.1`
Audience: agents and humans replacing GUI TODOs with explicit, bounded UI behavior.

**Closed / maintenance step history:** [`../legacy_runbooks/README.md`](../legacy_runbooks/README.md). **Active G3 work:** this runbook + [`../matrix/gap_remediation/runbook/`](../matrix/gap_remediation/runbook/README.md).

---

1. **Owning spec is per surface, not one cross-cutting UI mega-spec.** Each G3 sub-pack ties to its **own** spec and links outward to the deployment domain it reflects (faction, production/buildings, utilities, terrain, other subsystems). Shared wording in this runbook is policy only; behavior lives in domain specs + code.
2. **Bevy in-game UI is the goal** for runtime menus and HUDs — including, where available, simple graphs, lines, and styled numeric readouts. **Prefer Bevy UI** when widgets exist or can be built with basic tooling; use **`TEMP-EGUI`** when Bevy coverage is not there yet, and schedule replacement.
3. **egui is allowed as `TEMP-EGUI`** for diagnostics, tuning, or short-lived placeholders; any runtime egui placeholder must be labelled and queued for replacement.
4. **Desktop asset tools never render in-game UI.** They edit files; the engine reload path owns runtime display.
5. **Read-only by default.** **Authority is decided per usage case** (each button, command, or file path): event, command, ECS write, or file write. **No shortcuts** — each mutation path should have a **test** that proves the intended authority boundary. If unclear, stop and add **BQ-###** / `ASK:` before Rust.
6. **Data source before widget.** Data comes from **ECS queries/resources** and **assets** for the *targeted* UI; the map **evolves** as production states, resource flows, and configs land — refresh the sub-pack row when promoting new features.
7. **Performance follows data churn.** Values that change dramatically, go out of sync easily, or are safety-critical should refresh on a tighter cadence; low-priority aggregates (e.g. “exact amount collected this tick”) may update on **larger, configurable intervals** for tuning. **Do not** assume every read needs per-frame polling.
8. **Thread safety** — cache or sample heavy reads; do not block gameplay threads on unbounded scans.

---

## 2. Anchor set

Every GUI step reads:

1. This runbook §§1-8.
2. [`implementation_gap_hunt_runbook_v1.md`](implementation_gap_hunt_runbook_v1.md) §§1-4.
3. [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md) §§1, 4, 10.
4. The active G3 sub-pack.
5. The single `src/gui/...rs` file being edited.

If a step needs more than this, split or surface `ASK:` with **BQ-###** in the backlog brief §4.

---

## 3. Sub-pack index

| Sub-pack | Surface | Anchor |
|:---|:---|:---|
| [`g3a_faction_editor_steps_v1.md`](../matrix/gap_remediation/runbook/g3a_faction_editor_steps_v1.md) | Faction editor | `src/gui/faction_tools_ui.rs` |
| [`g3b_production_hud_steps_v1.md`](../matrix/gap_remediation/runbook/g3b_production_hud_steps_v1.md) | Production HUD | `src/gui/in_game_hud.rs` |
| [`g3c_diagnostics_steps_v1.md`](../matrix/gap_remediation/runbook/g3c_diagnostics_steps_v1.md) | Diagnostics | `src/gui/diagnostics_ui.rs` |
| [`g3d_runtime_ui_steps_v1.md`](../matrix/gap_remediation/runbook/g3d_runtime_ui_steps_v1.md) | Runtime in-game UI | `src/gui/in_game_ui.rs` |

---

## 4. Active sequence

1. **G3A** Faction editor.
2. **G3B** Production HUD.
3. **G3C** Diagnostics.
4. **G3D** Runtime in-game UI.
5. Matrix/spec close.

In-game menus are higher priority than options / secondary menus unless those are trivial.

---

## 5. Loop protocol

1. Open the active sub-pack and confirm blockers.
2. If the **§3 slice** lacks data source / authority rows, add **`ASK:`** + **BQ-###** in [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4 — stop before Rust.
3. Plan one atomic step touching 1-3 files.
4. Run `cargo check -p proc_A_dine01`.
5. Run the named test if the step adds behavior.
6. Update the sub-pack and cross-link status.

---

## 6. Halt rules

- Missing data source map for the widgets in this step, or mutation authority **for this action** (with no plan for a proving test).
- UI boundary crossed (desktop tool trying to render runtime UI, or runtime egui not labelled `TEMP-EGUI`).
- Step needs more than one GUI surface.
- Per-frame expensive query without cache / sampling plan.

---

## 7. Prompt fragment

> Read [`prompts/guides/gui_runbook_v1.md`](gui_runbook_v1.md) §§1-8, then open the active G3 sub-pack under `prompts/matrix/gap_remediation/runbook/`. Do not write Rust until the **slice** has a **§3 living map** row (or `ASK:` + **BQ-###**). Keep egui runtime placeholders marked `TEMP-EGUI`; desktop asset tools only edit files.

---

## 8. G3 execution cycle + results discipline

1. Follow [`../matrix/gap_remediation/runbook/g3_execution_cycle_v1.md`](../matrix/gap_remediation/runbook/g3_execution_cycle_v1.md) §§2–3: one step, then hand off to the backlog orchestrator row, then the next GUI row.
2. When a sub-pack (**G3A–G3D**) is closed for the cycle, write **one** results file from [`../matrix/gap_remediation/runbook/g3_cycle_results_TEMPLATE.md`](../matrix/gap_remediation/runbook/g3_cycle_results_TEMPLATE.md) under `prompts/matrix/gap_remediation/runbook/results/`.
3. In results files, route open work only via **BQ-###** in [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4 — not informal open-work shorthand.
