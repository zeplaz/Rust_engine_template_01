# GUI remediation runbook `v1`

> **STATUS:** Active orchestrator for **G3** GUI remediation. This runbook sequences faction editor, production HUD, diagnostics, and runtime in-game UI sub-packs. Rust steps remain gated per surface until data source + authority are recorded.

Version: `v1.0.0`
Audience: agents and humans replacing GUI TODOs with explicit, bounded UI behavior.

---

## 1. Invariants

1. **Bevy in-game UI is the goal** for runtime menus and HUDs.
2. **egui is allowed as `TEMP-EGUI`** for diagnostics, tuning, or short-lived placeholders; any runtime egui placeholder must be labelled and queued for replacement.
3. **Desktop asset tools never render in-game UI.** They edit files; the engine reload path owns runtime display.
4. **Read-only by default.** UI mutations require explicit authority: event, command, ECS write, or file write.
5. **Data source before widget.** Each sub-pack must name the resource/query/file/event it reads before Rust steps.
6. **Thread safety and performance first.** Expensive state should be cached / sampled; do not add per-frame global scans without an owning cadence.

---

## 2. Anchor set

Every GUI step reads:

1. This runbook §§1-5.
2. [`implementation_gap_hunt_runbook_v1.md`](implementation_gap_hunt_runbook_v1.md) §§1-4.
3. [`gap_remediation_runbook_v1.md`](gap_remediation_runbook_v1.md) §§1, 4, 10.
4. The active G3 sub-pack.
5. The single `src/gui/...rs` file being edited.

If a step needs more than this, split or surface `ASK:`.

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
2. If data source / authority is missing, add or update a doc-only `ASK:` / TODO and stop before Rust.
3. Plan one atomic step touching 1-3 files.
4. Run `cargo check -p proc_A_dine01`.
5. Run the named test if the step adds behavior.
6. Update the sub-pack and cross-link status.

---

## 6. Halt rules

- Missing data source or mutation authority.
- UI boundary crossed (desktop tool trying to render runtime UI, or runtime egui not labelled `TEMP-EGUI`).
- Step needs more than one GUI surface.
- Per-frame expensive query without cache / sampling plan.

---

## 7. Prompt fragment

> Read [`prompts/guides/gui_runbook_v1.md`](gui_runbook_v1.md) §§1-6, then open the active G3 sub-pack under `prompts/matrix/gap_remediation/runbook/`. Do not write Rust until the sub-pack names data source and authority. Keep egui runtime placeholders marked `TEMP-EGUI`; desktop asset tools only edit files.
