# Operator playtest report `v1` (2026-06-12)

**Source:** human play session · normal `cargo run --release`  
**Gate:** G-PLAY-01 partial — blockers found

---

## Findings (operator words → engineering)

| # | Report | Likely cause | Owner |
|:---:|:---|:---|:---|
| 1 | **OS cursor ≠ game cursor** — misaligned; OS cursor needed for UI clicks but not for placement | Single `Window::cursor_position()` drives pick + Bevy UI + egui; no hidden OS cursor / unified game cursor | @designer + @coder |
| 2 | **Cannot place buildings** — clicking feels wrong | Hidden **Enter** commit; no two-click flow | @designer **PASS** [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) → **@coder** TRIAGE-BUILD-CLICK-PLACE-001 |
| 3 | **No fire or sparks** | Simulation defaults **fire heat overlay OFF**; default world may have **no active burns**; sparks need heat + zoom band | @designer read · @coder product path |

---

## Build flow today (hidden from player)

1. Left **build rail** → pick category (**In** = Industry, **Ut** = Utilities, **Rd** = Roads, …)
2. **Submenu** opens → pick specific building (e.g. concrete mine)
3. **Left-click map** → moves ghost
4. **Enter** → commits (default key — **not** second click)

Construction floating window with full catalog is **gated off in Simulation** (PLAY-01).

---

## Recommended product direction (operator)

- **Cursor:** hide OS cursor over game window; one visible in-game cursor for map + Bevy buttons + placement
- **Build (2026-06-12):** **Click 1** lock ghost → **Ctrl** rotate · **Shift** size → **Click 2** place (no Enter)
- **Fire:** visible when sim has fire OR starter scenario ignites demo fire

---

## Queue slices opened

| ⟨ID⟩ | Agent |
|:---|:---|
| DESIGN-BUILD-UX-REDESIGN-001 | @designer **PASS** 2026-06-12 |
| TRIAGE-BUILD-CLICK-PLACE-001 | @coder **ready** |
| TRIAGE-CURSOR-UNIFY-001 | @designer + @coder |
| TRIAGE-FIRE-PLAY-VIS-001 | @coder |
| TRIAGE-MAP-ZOOM-SMOOTH-001 | @coder A (prior report) |
