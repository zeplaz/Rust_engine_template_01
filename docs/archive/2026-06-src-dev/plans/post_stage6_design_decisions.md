# Post–Stage 6 design decisions

Log product/architecture choices for the [`post_stage6_active_todos.md`](post_stage6_active_todos.md) program. Amend via designer + planner before large Rust diffs.

| ID | Decision | Status |
|----|----------|--------|
| DQ-POST-01 | Shell hydrate: **user-triggered restore first**; autoload on bundle open behind `WAVE_S_AUTOLOAD_SHELL=1` | **Accepted** (implemented) |
| DQ-POST-02 | Wave P exit: lib tests + `wave_p_live.json` in sim; visual regression on release train | **Accepted** (writer shipped; visual ops pending) |
| DQ-POST-03 | `RepresentationResult` stays global; per-view policy via caps/hints until VM-11 audit | **Accepted** (matches S6-24) |
| DQ-POST-04 | Construction ghosts remain `SimulationMapViewport`-scoped until VM-09 + Wave P green | **Proposed** |
| DQ-POST-05 | Industrial first chain: **concrete** (aggregate → kiln → plant) using new building JSON | **Proposed** |
| DQ-POST-06 | **Minimap must stay user-movable** in sim — drag title bar, resize, persist layout (`HudLayoutStore` / Wave S shell); do not lock to bootstrap rect only | **Accepted** (2026-05-23) |
| DQ-POST-07 | **Construction catalog submenus** (`BuildToolbox` egui: Industrial/Utilities/Infrastructure) are **editor-session** tools; sim uses Bevy **build rail** (`ToolContext`) only until UX-C ships catalog in sim | **Accepted** (2026-05-23) |
