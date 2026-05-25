# UI + construction playtest guide `v1`

**Audience:** operator / designer / coder smoke tests  
**Related:** [`post_stage6_design_decisions.md`](post_stage6_design_decisions.md) DQ-POST-06/07 · [`ui_gates.rs`](../gui/ui_gates.rs) PLAY-01 · [`build_toolbox.rs`](../construction/build_toolbox.rs)

---

## 1. Minimap — movable (remember)

**Product rule (DQ-POST-06):** minimap stays a **floating, draggable** panel in simulation — not hard-pinned to the bootstrap top-right rect only.

| What works today | Where |
|:---|:---|
| egui **Minimap** window — drag title bar, resize edges | `draw_simulation_minimap_egui` · `std_floating` (`constrain(false)`, `resizable(true)`) |
| Layout persistence | `HudLayoutStore` + `capture_shell_layout` on drag |
| Bevy **chrome** stroke follows egui rect | `sync_minimap_chrome_root_system` ← `MinimapShellState::last_window_rect` |
| Detached / size | `MinimapShellState.detached`, viewport sliders (when not dragging) |

**How to move:** In sim, find the **Minimap** floating window → drag its **title bar**. Position saves for the session (Wave S shell restore can reload layout).

**Known gaps (not “forget movable”):**

- Bootstrap rect seeds once if no prior layout (`bootstrap_simulation_layout_rect`).
- GPU compositor path (`MINIMAP_GPU_COMPOSITOR=1`) — chrome sync still tied to egui rect; movable policy unchanged.
- Future UX-A: detachable native window is stub (`hud_native_minimap_window` feature).

---

## 2. Where did the building submenus go?

**Short answer:** They did not disappear — they are **gated off in Simulation** on purpose (PLAY-01 / Phase 2B).

| Session | Construction UI | Host |
|:---|:---|:---|
| **Simulation** (`BaseState::Simulation`) | Bevy **build rail** only — Rd / Rl / Ut / In / Cv / Mi / Ec | `BuildRailRoot` · `BuildStripState` |
| **Editor** (`BaseState::Editor`, not WorldGen) | Full **Construction** floating window + submenus | egui `draw_build_toolbox_egui` |

Submenus live under **Construction** window:

- **Buildings → Industrial…** — aluminum chain (`aluminum_bauxite_mine`, `aluminum_alumina_refinery`, `aluminum_smelter1`, `aluminum_fabrication_plant`) from catalog
- **Buildings → Utilities…** — power plants + JSON **`BuildingFamily::Power`** (includes `grid_distribution_transformer`)
- **Infrastructure → Roads / Rail** — spline placement tools

Witness: `build_toolbox_egui_gated: true` in sim means toolbox is **intentionally** closed (`simulation_session.rs`).

---

## 3. How to test aluminum, transformer, roads

### A — Full catalog (recommended for buildings)

1. Boot with world (auto or menu):
   ```powershell
   cargo run -p proc_A_dine01 -- --test frame
   ```
2. **Stay in editor session** — use **map editor** path, **do not** press “enter simulation” / play if you need the Construction window.
   - Main menu → **New map in editor** (or complete world-gen and remain in editor chrome).
   - `AppState` must not be `WorldGen` (world-gen UI blocks product shell).
3. Open **Construction** floating window (egui). If missing: it is a product-shell widget — should appear in editor; dock slot `BuildToolbox`.
4. Pick tool:
   - **Industrial…** → e.g. `aluminum_smelter1`, `aluminum_alumina_refinery`
   - **Utilities…** → e.g. `Distribution transformer` (`grid_distribution_transformer.json`)
   - **Infrastructure → Roads** → click map to place road spline
5. **LMB** on map to place building ghost → confirm key to commit (see keybindings / toolbox header).

### B — Roads / rail from Simulation (build rail only)

1. Enter **Simulation** (after world-gen).
2. Left **build rail** → **Rd** (roads) or **Rl** (rail) — sets `BuildStripState` / `ActiveBuildTool`.
3. Click map to draft corridor (spline); confirm per construction hints.
4. No aluminum/transformer catalog here until sim-side catalog UX is designed.

### C — Quick dev: re-open toolbox in sim (violates PLAY-01)

Only for local debugging — do not ship without designer sign-off:

- Temporarily disable `enforce_simulation_product_egui_gates` suppression of `BuildToolbox`, **or**
- Run with `BaseState::Editor` while viewing the map (editor product shell).

---

## 4. Catalog IDs (smoke list)

| Asset | JSON | Menu path |
|:---|:---|:---|
| Aluminum smelter | `assets/configs/buildings/aluminum_smelter1.json` | Construction → Industrial… |
| Alumina refinery | `aluminum_alumina_refinery.json` | Industrial… (supply chain group) |
| Distribution transformer | `grid_distribution_transformer.json` | Construction → Utilities… |
| Concrete plant | `concrete_basic_production_plant.json` | Industrial… |
| Roads | — | Construction → Infrastructure → Roads **or** sim rail **Rd** |

Index: `assets/configs/buildings/_building_types_index.json` · chains: `assets/configs/industrial_supply_chains.json`

---

## 5. Backlog hooks

| ID | Track | Note |
|:---|:---|:---|
| UX-E01 | GPU minimap | Movable + compositor |
| UX-C | Construction in sim | Catalog tray without full egui shell |
| IND-E01 | Industrial E2E | Place chain in **sim** with activation pipeline |
