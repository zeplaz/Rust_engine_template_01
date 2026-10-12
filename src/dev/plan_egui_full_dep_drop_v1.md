# PLAN-EGUI-FULL-DEP-DROP-v1 — remove `bevy_egui` from Cargo

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-EGUI-FULL-DEP-DROP-v1** |
| **Queue** | `EGR-FULL-0` … `EGR-FULL-8` |
| **Date** | 2026-10-10 |
| **Owner** | `@coder` (impl) · `@planner` (charter) |
| **Status** | **CLOSED** (2026-10-10) · witness `debug_runs/egui_full_dep_drop_live.json` |
| **Supersedes** | “editor keeps egui” in [`plan_egui_game_retirement_v1.md`](plan_egui_game_retirement_v1.md) |
| **Boundary** | [`prompts/guides/ui_boundary_guide_v1.md`](../../prompts/guides/ui_boundary_guide_v1.md) — Bevy-only |

## Goal

No `bevy_egui` in `Cargo.toml` until every panel (Simulation, Editor, map editor, WorldGen, diagnostics) runs on native Bevy UI.

**Exit:** `cargo check -p proc_A_dine01` green; zero `bevy_egui` / `EguiContexts` / `EguiPlugin` in `src/` + `src/bin/`; witness `debug_runs/egui_full_dep_drop_live.json`.

## Phases

| ID | Slice | Done when |
|:---|:---|:---|
| EGR-FULL-0 | Charter + boundary guide + queue seed | this file + guide Bevy-only |
| EGR-FULL-1 | Type decoupling (`UiRgba` / logical rect) | sim-hot paths import no `bevy_egui::egui` |
| EGR-FULL-2 | Zero sim egui acquires | shell diagnostics assert + mesh-only world paint in sim |
| EGR-FULL-3 | Editor product shell Bevy | `hud_product_shell_egui_root` retired |
| EGR-FULL-4 | RTT excision | no `EguiTextureHandle` |
| EGR-FULL-5 | Construction + map editor Bevy | no construction `EguiPrimaryContextPass` |
| EGR-FULL-6 | Logistics + WorldGen Bevy | panels + `world_generator` bin without egui |
| EGR-FULL-7 | Dev/diagnostics Bevy | F-key panels Bevy or removed from schedule |
| EGR-FULL-8 | Cargo drop | `bevy_egui` gone; witness green |

## Authority

- Pointer: `SimulationMapPointerGate` + `TacticalMapPickFrame`
- World paint: `Mesh2d` / `MinimapCompositor` — never egui strokes
- Panels: Bevy `Node` trees

## Rules

- Do not add new egui panels.
- Prefer delete orphan egui painters over re-enable in sim.
- Pathspec commits `[ui]` / `[egui-retire]` per phase.
