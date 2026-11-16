# Agent / authoring notes

## Single canonical worktree

**Only:** `<repo>` on branch **`master`**. All work merges into master.
Policy: [`src/dev/single_worktree_policy_v1.md`](src/dev/single_worktree_policy_v1.md).

## Project

Rust + Bevy **0.20** on master (PLAN-BEVY-020-MIG-v1 **PROGRAM CLOSE** — gate [`debug_runs/mig_bevy_020/mig_v2_gate.json`](debug_runs/mig_bevy_020/mig_v2_gate.json)). UI is Bevy UI (egui is gone). Guide: https://bevy.org/learn/migration-guides/0-19-to-0-20/
Closed **0.19** `MIG-*` rows stay closed. Residual lib-test `If`/harness polish: [`src/dev/mig_bevy_020_m0_test_defer.md`](src/dev/mig_bevy_020_m0_test_defer.md). Book-driven upgrade: [`src/dev/plan_ref_books_upgrade_v3.md`](src/dev/plan_ref_books_upgrade_v3.md) (**PROGRAM CLOSE**) · residual unlock program: [`plan_ref_books_defer_unlock_v1.md`](src/dev/plan_ref_books_defer_unlock_v1.md) · next product charters: [`plan_unit_member_v1.md`](src/dev/plan_unit_member_v1.md) · [`plan_town_growth_v1.md`](src/dev/plan_town_growth_v1.md) (incl. **TWN-PCG07**) · [`plan_voxel_occupation_v1.md`](src/dev/plan_voxel_occupation_v1.md) (**OCC-P4…P6**) · conditional [`plan_dense_heat_wgsl_v1.md`](src/dev/plan_dense_heat_wgsl_v1.md). Other open work: [`src/dev/coder_todos_v1.md`](src/dev/coder_todos_v1.md).

## Review path (how to check a run)

There are no witness gates, live-proof JSONs, readiness profiles, queues or `cargo orchestrate`.
One debug-only dump, one scored matrix, one dashboard — see [`tools/sim_review/README.md`](tools/sim_review/README.md):

```powershell
$env:RUST_ENGINE_SIM_DUMP="1"; cargo run -p proc_A_dine01    # debug only -> sim_run_dump.json (counts, no verdicts)
python tools/sim_review/score_dump.py                        # -> debug_runs/sim_review/matrix.json + dashboard.html
python tools/sim_review/playcheck.py                         # build + game + capture + score in one go
```

Unit tests: `cargo test -p proc_A_dine01 --lib` (filter by module, e.g. `construction::`).

## Rules that still apply

- **One writer per domain.** Simulation truth is authoritative; rendering and UI read snapshots and never write back.
- **Preview never mutates gameplay.** Construction logic lives in `src/construction/` ([`src/dev/construction_invariants.md`](src/dev/construction_invariants.md)): single execute funnel, no logic outside it.
- **No parallel extraction or duplicate LOD** in rendering; attach to the existing contracts (`RepresentationResult`, `FireVisualFrame`, `SharedOverlayFieldBuffers`, projection graph). Transitional scaffolds need a `ScaffoldContract` (`src/gui/representation_governance.rs`).
- **Play UI only.** Panels shipped in `src/gui/` are the ones a player uses: build picker and rail, logistics orders, plant info card, AI explain panel (`Insert`), front controls, world-gen panel, main/pause menu, options/keybindings, minimap, HUD tray. No dev/debug panels or toggles.
- **No test-shaped production code.** No `*_green` predicates, `*_LIVE_JSON` constants or witness resources; assert in tests, score in `tools/sim_review/`.
- **No protocol clone farms.** N systems with the same control flow and only type/tag/sink swapped → one shared kernel + typed adapters (not a god-system that mashes domains). Standing rule: [`src/dev/plan_anti_clone_kernel_v1.md`](src/dev/plan_anti_clone_kernel_v1.md). Combat TOE restock: [`plan_combat_sustainment_v1.md`](src/dev/plan_combat_sustainment_v1.md).

## Serialization

Engine-owned serde assets (terrain registries, transport R8, world-gen tuning, snapshots) prefer **RON**; loaders dispatch on extension. JSON stays where external tooling or interchange needs it (Python asset editor, HTTP APIs, some fixtures).

## Agents

Default: work directly (`@coder`, `@planner`, `@designer` in chat), then validate with `cargo check` / `cargo test`. Role definitions: [`.cursor/agents/`](.cursor/agents/). Skills: `.cursor/skills/` (authoring tree `.claude/skills/`). Escalate to orchestrator agents only for multi-phase work across owners.

| Role | Owns |
|------|------|
| `coder` | ECS, render, viewport, logistics, diagnostics in `src/` |
| `planner` | architecture plans, phases, authority map (read-only) |
| `designer` | HUD, overlays, multiview UX |
| `coder-mcp` / `designer-mcp` | `tools/mcp/` toolchain / MCP art pipeline |
| `sim-steward` | authority drift and cleanup |

## Where to look

- Construction spec: [`src/dev/recovery_construction.md`](src/dev/recovery_construction.md), code in [`src/construction/`](src/construction/).
- Transport R8 snapshots may carry a `construction` slice; load hydrates `CorridorConstructionBook`.
- HUD: simulation session chrome in [`src/gui/hud/simulation_session.rs`](src/gui/hud/simulation_session.rs); UI boundary guide [`prompts/guides/ui_boundary_guide_v1.md`](prompts/guides/ui_boundary_guide_v1.md).
- Index of modules/concepts: [`src/dev/codebase_index_v1.md`](src/dev/codebase_index_v1.md).
- Reference books (GEA 4e I/II, AI for Games 3e): [`src/dev/ref_books_index/systems.md`](src/dev/ref_books_index/systems.md) → `python src/dev/ref_books_index/lookup.py <words> | --code <path>` gives book section + PDF pages + matching code.
