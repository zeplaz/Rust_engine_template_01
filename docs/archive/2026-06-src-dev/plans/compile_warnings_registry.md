# Compile warnings registry

**Scan:** `cargo check -p proc_A_dine01` (2026-05-23)  
**Log:** [`debug_runs/compile_warnings.log`](../../debug_runs/compile_warnings.log)  
**Before fixes:** 32 warnings (2026-05-23 regression batch) → **After:** 0 warnings (lib)

### 2026-05-23 batch (all CLEANUP)

| Area | Warnings | Fix |
|------|----------|-----|
| `in_game_hud.rs` | stale imports from HUD refactor | removed unused `use` lines |
| `test_harness.rs` | unused imports + unnecessary `mut` on `Option` params | `#[cfg(test)]` local imports; drop outer `mut` |
| `world_gen_chrome_contract.rs` | unused gate imports + dead contract helpers in lib | `#![allow(dead_code)]`; gate imports in `mod tests` only |
| `full_render_diagnostic.rs` | unused `in_simulation_or_editor` | removed import |
| `view_fire_projection.rs` | unused `bevy::prelude` in lib | prelude in `#[cfg(test)]` only |
| `minimap_compositor/pass.rs` | unnecessary `mut registry` | `Res` instead of `ResMut` |
| `simulation_shell_phase2.rs` | `private_interfaces` on proof writer | `pub(crate)` on writer fn |

---

**Prior scan:** `cargo build -p proc_A_dine01` (2026-05-20) — 20 warnings → 0 after prior cleanup.

## Classification legend

| Disposition | Meaning |
|-------------|---------|
| **CLEANUP** | Removed unused re-export / wired symbol — done |
| **WIRE** | Hook into production path — done or todo |
| **CONTINUE** | Scaffold for active lane — keep, annotate |
| **DEFER** | Future phase — `#[allow(dead_code)]` + witness |

---

## Tree by subsystem

```text
proc_A_dine01 (lib)
├── GUI / VIEWPORT_AUTHORITY
│   └── frozen_exceeds_semantic_authority     [WIRE] → publish_simulation_map_viewport
├── GUI / WORLD_PREVIEW
│   └── preview_gpu_authoritative             [WIRE] → preview_gpu_authoritative_run_if
├── CONSTRUCTION / ROADS
│   └── mod.rs unused pub use (commit_road, pathing, IntersectionNode)  [CLEANUP]
├── CONSTRUCTION / RAIL
│   └── mod.rs unused pub use + junction scaffold types                    [CLEANUP + DEFER]
├── CONSTRUCTION / ZONES
│   └── mod.rs unused pub use (commit_painted, zone_tool_tag)            [CLEANUP]
├── CONSTRUCTION / BUILD
│   ├── shift_lmb_* helpers                   [WIRE] → build_interaction shift/alt paths
│   ├── build_interaction unused `id`         [CLEANUP] → `_id`
│   ├── BuildingDefinitionFile fields         [WIRE] → display_name + construction_time
│   ├── ConstructionValidation.required_actions [WIRE] → debug log on reject
│   └── SnapTarget enum                       [DEFER] → PHASE2-BUILD-18 node UI
└── ECONOMY / INDUSTRIAL
    └── live_proof unused building imports    [CLEANUP]
```

---

## Original warnings (20) — disposition

| # | Warning | Subsystem | Disposition | Action |
|---|---------|-----------|-------------|--------|
| 1–3 | `roads/mod.rs` unused `pub use` | Construction / roads | CLEANUP | Trimmed re-exports; internal `roads::commit::` still used |
| 4–7 | `rail/mod.rs` unused `pub use` | Construction / rail | CLEANUP | Trimmed re-exports |
| 8 | `zones/mod.rs` unused `pub use` | Construction / zones | CLEANUP | Trimmed re-exports |
| 9 | `economy/activation/live_proof.rs` imports | Economy | CLEANUP | Removed unused imports |
| 10 | `build_interaction` unused `id` | Construction | CLEANUP | `_id` |
| 11 | `frozen_exceeds_semantic_authority` dead | Viewport | WIRE | Re-linked in `authoritative_viewport.rs` |
| 12–13 | `BuildingDefinitionFile` fields unread | Construction / catalog | WIRE | `description` → display fallback; `building_height` → ticks |
| 14–15 | `shift_lmb_*` never used | Construction / tools | WIRE | Used in shift-click + alt-drag systems |
| 16 | `required_actions` never read | Construction / pipeline | WIRE | Logged on road segment reject |
| 17–20 | Rail junction + `SnapTarget` dead | Construction / rail / snap | DEFER | `#[allow(dead_code)]` + phase tags |

---

## Clippy (non-blocking)

Run: `cargo clippy -p proc_A_dine01 --all-targets`

| Class | Examples | Disposition |
|-------|----------|-------------|
| Style | empty line after attribute, precedence | CONTINUE — batch fix later |
| Complexity | too many arguments (7+) | CONTINUE — refactors per runbook |
| Manual patterns | `div_ceil`, `is_multiple_of` | CLEANUP — optional clippy --fix |

---

## Related docs

- Viewport recovery: [`recovery_viewport.md`](recovery_viewport.md)
- Construction status: [`construction_active_progress.md`](construction_active_progress.md)
- Action queue: [`COMPILE_WARNINGS_TODOS.md`](COMPILE_WARNINGS_TODOS.md)
- Orchestrator: [`../../tools/orchestrator/NEXT.md`](../../tools/orchestrator/NEXT.md)
