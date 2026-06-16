# SIM-HUD-PRODUCT-001 — Designer brief (player simulation chrome)

**Program:** [`plan_bevy_hud_grammar_parallel_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_bevy_hud_grammar_parallel_v1.md) · Phase E [`post_stage6_active_todos.md`](../../src/dev/post_stage6_active_todos.md)  
**Owners:** `@designer` (lead UX) · `@coder` (Bevy native UI)  
**Not:** MCP art pipeline · Tk APS · egui dev panels (except dock shells already on egui product path)

---

## Mission

Keep **player-facing** simulation HUD coherent under **PLAY-01** session rules — collapsed editor chrome, readable ops strip, dock/minimap/build rail, context tray.

**Wording fix:** “In-engine HUD / multiview — not APS” means **do not build simulation chrome in Tkinter**. Bevy product HUD is **this lane**.

---

## Surfaces (do not merge with APS-BEVY-QC-HUD-001)

| Surface | Files | PLAY-01 expectation |
|:---|:---|:---|
| Session defaults | `simulation_session.rs` | Enter Simulation → editor panels dismissed; floating shells collapsed |
| Ops strip | `in_game_hud.rs` | Time, power, weather, alerts readable at tactical zoom |
| Dock / command tray | `dock_shell.rs`, `hud_root_tick.rs` | Collapsed command tray default in sim |
| Minimap | `in_game_hud.rs` + compositor | M1–M3 overlays; no editor-only chrome bleeding |
| Build rail / context | `in_game_hud.rs` | Tool context + construction ghosts readable |
| Pause / transmission | `pause_menu_bevy.rs`, transmission shell | Qualified PASS records on file — polish only |

Entry hook: `apply_simulation_hud_defaults` on `OnEnter(Simulation)` — see AGENTS.md PLAY-01 table.

---

## In scope (designer)

1. **Readability pass** — one prioritized slice per cycle (ops strip · minimap · build rail · dock collapse)
2. **Regression checklist** — sim entry hides WorldGen/scenario script; restores on exit to editor
3. **Multiview** — no new scope unless `DESIGN-UI-P6-MULTIVIEW-001` follow-up filed
4. Witness note when slice closes — link to `ui_shell_migration_live.json` or slice-specific doc

---

## Out of scope

- Assembly snapshot QC table (Lane A′ **APS-BEVY-QC-HUD-001**)
- APS Tk previews / material studio
- Stage 5 FULL_APP spine changes
- Construction drain (`src/construction/`) — designer read-only review only

---

## Parallel rule

Runs **alongside** APS Track A and weather read doc — no preemption of construction coders.

---

## Deliverables

1. Slice brief per touch — **done:**
   - [`design_sim_hud_ops_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_sim_hud_ops_v1.md)
   - [`design_sim_hud_dock_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_sim_hud_dock_v1.md)
   - [`design_sim_hud_minimap_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_sim_hud_minimap_v1.md)
   - [`design_sim_hud_build_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_sim_hud_build_v1.md)
2. Before/after notes or capture path under `assets/ui/` when visual (@coder after impl)
3. Registry row **SIM-HUD-PRODUCT-001** when PLAY01 + ≥2 slice witnesses PASS (qualified)

---

## Paste back

```text
SIM-HUD-PRODUCT-001 slice: _
PLAY-01 regression: pass | fail (_)
Sign-off: PASS (qualified) | DEFER
```
