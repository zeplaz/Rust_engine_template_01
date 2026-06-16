# Build placement UX redesign `v1` (DESIGN-BUILD-UX-REDESIGN-001)

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-BUILD-UX-REDESIGN-001** · parent G-PLAY-01 playtest |
| **Date** | 2026-06-12 |
| **Owner** | `@designer` (charter) · `@coder` **TRIAGE-BUILD-CLICK-PLACE-001** |
| **Verdict** | **PASS** |
| **Source** | [`operator_playtest_report_20260612_v1.md`](operator_playtest_report_20260612_v1.md) |
| **Prereq** | [`design_sim_hud_build_v1.md`](../docs/archive/2026-06-src-dev/plans/design_sim_hud_build_v1.md) (rail) · [`construction_invariants.md`](construction_invariants.md) |
| **Witness** | [`debug_runs/design_build_ux_redesign_live.json`](../debug_runs/design_build_ux_redesign_live.json) |
| **Unblocks** | **TRIAGE-BUILD-CLICK-PLACE-001** · **TRIAGE-CURSOR-UNIFY-001** (cursor policy §5) |

**No Rust in this doc.** Interaction charter + HUD copy only — preview ≠ commit invariant preserved.

---

## Mission

Players must **place a building with two obvious clicks** — no hidden **Enter** path. Modifier keys while adjusting must match RTS muscle memory: **Ctrl = rotate**, **Shift = size**.

**Acceptance test:** *Pick building from rail → LMB map locks ghost → Ctrl+scroll rotates → Shift+scroll resizes → second LMB places — never pressed Enter.*

---

## Authority model

```text
ActiveBuildTool          → sole tool source (invariant 4)
BuildGhostState          → preview only — origin, rotation, scale, mirror
BuildPlacementPreview    → allows_commit gate before any commit
build_confirm_site_system → single execute funnel (Enter OR second LMB after wire)
```

**Rules:** Two-click flow is **presentation + input routing** only. No second commit writer. Invalid tiles never bypass `allows_commit` ([`construction_invariants.md`](construction_invariants.md) §7).

---

## 1. Two-click placement (buildings)

| Step | Input | Result |
|:---:|:---|:---|
| **1** | **First LMB** on valid map tile | **Lock ghost** at tile → enter **Adjust** mode |
| **Adjust** | Hold **Ctrl** + scroll or horizontal drag | **Rotate** footprint (`rotation_quarter_turns`) |
| **Adjust** | Hold **Shift** + scroll or vertical drag | **Change size** (`scale_factor`, clamped) |
| **Adjust** | **X** (tap) | **Mirror** footprint (`mirror_x`) — power-user, works in Preview or Adjust |
| **2** | **Second LMB** on map or ghost | **Place** — commit if `allows_commit` |
| **Cancel** | **RMB** or **Esc** | Drop lock → **Preview** mode (ghost follows cursor) |

### Mode state machine

```text
Preview   — ghost follows cursor · LMB on tile → Adjust (lock)
Adjust    — ghost fixed · Ctrl=rotate · Shift=size · LMB → Place · RMB/Esc → Preview
Place     — validate → CommitConstructionSiteEvent → Preview (tool stays active)
```

| Mode | Ghost motion | LMB | Invalid tile |
|:---|:---|:---|:---|
| **Preview** | Follows cursor | Lock at tile (or no-op off-map) | Red hatch; lock allowed but Place blocked |
| **Adjust** | Fixed at lock | Place (if valid) | Red hatch; second LMB blocked + toast |
| **Place** | — (one frame) | — | Toast only |

---

## 2. Resolved conflicts

| Conflict | Decision | Rationale |
|:---|:---|:---|
| Shift+LMB blueprint queue (buildings) | **Removed** — already **PARAM-002** (`shift_lmb_queues_building_blueprint` → `false`) | Frees **Shift** for size modifier in Adjust |
| Batch blueprint queue | **Alt+LMB drag paint** (existing `build_drag_paint_queue_system`) | Power-user batch without stealing Shift |
| Zone / road / rail Shift behaviors | **Unchanged** — Shift+LMB still applies per `shift_lmb_applies_to_active_tool` | Only **Building** tool uses two-click FSM |
| Enter commit | **Retained as optional** power-user shortcut | Do not remove; demote in HUD copy |
| Mirror | **X** key retained; no Ctrl+mirror combo | Avoid modifier overload |
| Construction floating catalog | **Gated off in Simulation** (PLAY-01) | Rail + submenu is product path — doc unchanged |
| Cursor misalignment | **TRIAGE-CURSOR-UNIFY-001** separate slice | Pick uses unified cursor after hide OS cursor |

---

## 3. HUD + copy (locked strings)

### 3a. Developmental context strip (`contextual_tip.rs`)

Replace building-tool strings that mention Enter-first or Shift+click queue:

| Mode | Template |
|:---|:---|
| **Preview** (building selected, no lock) | `CONTEXT — BUILD · {strip} · {archetype} · click map to lock · [{cycle}] change category` |
| **Adjust** (locked) | `CONTEXT — BUILD · {strip} · locked {x},{z} · Ctrl rotate · Shift size · click to place · Esc cancel` |
| **Adjust** (invalid) | `CONTEXT — BUILD · {strip} · locked {x},{z} · blocked: {reason} · Esc cancel` |
| **Idle** | unchanged |

**Remove** from building path: `shift+click queues blueprint`, `ok to commit [Enter]` as primary affordance.

### 3b. Build toolbox button (`build_toolbox.rs`)

| State | Label |
|:---|:---|
| Preview | `Lock placement — click map` (enabled when catalog picked) |
| Adjust + valid | `Place building — click map again` |
| Adjust + invalid | `Cannot place — {short reason}` (disabled) |

**Remove:** `Place on map (Enter)` as primary label — move Enter to tooltip: `Shortcut: {Enter}`.

### 3c. Toast (invalid second click)

| Trigger | Copy |
|:---|:---|
| Second LMB while invalid | `Placement blocked — {primary_validation_message}` |
| Second LMB off-map | `Click on the map to place` |

Use existing `validation_feedback::primary_validation_message` — no new validation logic.

### 3d. Build rail (unchanged)

Per [`design_sim_hud_build_v1.md`](../docs/archive/2026-06-src-dev/plans/design_sim_hud_build_v1.md): 52px rail, gold selected slot, context tray collapsed on sim enter.

---

## 4. Ghost visual language (Adjust mode)

| State | Read |
|:---|:---|
| **Preview** | Standard valid/invalid ghost ([`ghost_visual.rs`](../construction/ghost_visual.rs)) |
| **Adjust — valid** | **Gold lock ring** (2px) on footprint + existing valid fill |
| **Adjust — invalid** | Red hatch + lock ring (placement still blocked on click 2) |
| **Scale/rotate live** | Footprint updates in place — no detached overlay |

**Rule:** Lock ring is the only new visual — no second ghost entity.

---

## 5. Cursor policy (handoff to TRIAGE-CURSOR-UNIFY-001)

| Requirement | Spec |
|:---|:---|
| Hide OS cursor | When `BuildStripState.active != None` over game window |
| Game cursor | Single sprite/crosshair for rail + map + ghost anchor |
| Pick alignment | `Window::cursor_position()` must match visible game cursor ([`design_zoom_fire_read_v1.md`](design_zoom_fire_read_v1.md) probe tiers) |

Designer **does not** wire cursor hide in this slice — charter only. Coder pairs with pick closure.

---

## 6. Accessibility

| # | Requirement |
|:---:|:---|
| A1 | Mode always in context strip text — not color-only |
| A2 | Invalid state names reason in text (toast + strip) |
| A3 | Enter shortcut documented in tooltip — not only discoverable via F3 |
| A4 | Esc always returns to Preview from Adjust |
| A5 | Scale/rotate work with keyboard modifiers + scroll (no drag-only gate) |

---

## 7. Acceptance (operator + lib)

| Check | Pass |
|:---|:---:|
| No Enter required for normal building place | ✓ |
| Click 1 locks; click 2 places | ✓ |
| Ctrl+scroll rotates while locked | ✓ |
| Shift+scroll changes size while locked | ✓ |
| Invalid tile: red ghost; second click blocked + toast | ✓ |
| RMB or Esc cancels lock | ✓ |
| Zone/road/rail tools unchanged | ✓ |
| `allows_commit` still gates commit | ✓ |

---

## 8. Coder handoff

### TRIAGE-BUILD-CLICK-PLACE-001

```text
Read:  src/dev/design_build_ux_redesign_v1.md
       src/dev/construction_invariants.md
Touch: build_state.rs (placement_mode enum)
       build_interaction.rs (two-click FSM; second LMB → commit path)
       build_toolbox.rs + contextual_tip.rs (§3 copy)
       ghost_visual.rs or visual_authority.rs (gold lock ring)
Do:    Preview | Adjust modes; wire Ctrl/Shift modifiers in Adjust only
Do NOT: second commit writer · bypass allows_commit · change zone/road input
Verify: cargo test -p proc_A_dine01 --lib construction::
Witness: debug_runs/design_build_ux_redesign_live.json (wire flags after impl)
```

### TRIAGE-CURSOR-UNIFY-001 (parallel after §5 charter)

```text
Read:  src/dev/design_build_ux_redesign_v1.md §5
Touch: gui input / cursor hide — pair with MAP-PICK if needed
```

---

## 9. Non-goals

- New building catalog UI in Simulation
- Construction execute funnel changes
- Staged placement panel rework
- MCP / module kit art

---

## Sign-off

| Role | Verdict | Date | Note |
|:---|:---|:---|:---|
| Operator | requirements captured | 2026-06-12 | playtest report |
| `@designer` | **PASS** | 2026-06-12 | charter + copy locked |
| `@coder` | pending | — | **ΔWF→@coder** TRIAGE-BUILD-CLICK-PLACE-001 |

```text
DESIGN-BUILD-UX-REDESIGN-001 complete
Verdict: PASS
Doc: src/dev/design_build_ux_redesign_v1.md
Unblocks: TRIAGE-BUILD-CLICK-PLACE-001, TRIAGE-CURSOR-UNIFY-001
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v0.1 | 2026-06-12 | DRAFT — operator requirements |
| v1.0.0 | 2026-06-12 | **PASS** — conflicts resolved, HUD copy locked |
