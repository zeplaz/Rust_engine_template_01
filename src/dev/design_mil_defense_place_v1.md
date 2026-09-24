# Military defense place — catalog IA `v1` (DES-MIL-DEFENSE-PLACE)

| Field | Value |
|:---|:---|
| **ID** | **DES-MIL-DEFENSE-PLACE** |
| **Priority** | **P2** |
| **Owner** | `@designer` (charter) · `@coder` **COD-MIL-DEFENSE-PLACE-001** |
| **Verdict** | **PASS** |
| **Date** | 2026-09-24 |
| **Prereq** | **TRIAGE-BUILD-CLICK-PLACE-001** · **TRIAGE-CURSOR-UNIFY-001** |
| **Parent UX** | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · [`design_pointer_hud_regions_v1.md`](design_pointer_hud_regions_v1.md) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |
| **Witness** | [`debug_runs/des_mil_defense_place_live.json`](../debug_runs/des_mil_defense_place_live.json) |
| **Unblocks** | **COD-MIL-DEFENSE-PLACE-001** |

**No Rust in this doc.** Catalog IA + locked copy + sheet/pointer notes only. Preview ≠ commit preserved.

---

## Mission

Military rail must place **defensive structures** (walls / trenches / fortification) — not silently arm **Demolish**.

**Acceptance test:** *Select Military on rail → picker opens → pick Wall or Trench or Bunker → two-click place on map (same FSM as buildings) → site queues via existing commit funnel — Demolish remains a sibling action, never the default Military tool.*

---

## Problem (current)

| Surface | Today | Defect |
|:---|:---|:---|
| `BuildTool::from_tool_context(Military)` | → `Demolish` | Military slot = destroy |
| `SimBuildPickerState::open_for_slot(Military)` | forces `open = false` | No catalog |
| Toolbox / hints | Demolish-only under Military | No place copy |
| `tool_context_to_picker_category(Military)` | falls through to Zone | Wrong IA |

---

## 1. Catalog IA (Military picker)

**Rail:** keep `ToolContext::Military` slot (label **Mi** / Military).

**On select:** open Defense picker sheet (same chrome as Industry — do **not** auto-arm Demolish).

### Sections (order top → bottom)

| Section | Player label | Catalog rows (P2) | `SiteArchetype` | Place FSM |
|:---|:---|:---|:---|:---|
| **Walls** | Walls | Defensive wall | `MilitaryBase` *(place stub — see §1a)* | **Two-click** (building) |
| **Trenches** | Trenches | Trench line | `TrenchLine` | **Two-click** (building) |
| **Fortification** | Fortification | Bunker | `BunkerComplex` | **Two-click** (building) |
| **Editing** (footer) | — | Demolish | — | Pending pick → confirm *(unchanged)* |

**Default armed tool after opening Military with no prior pick:** none / picker open only — first catalog click arms the tool.

### 1a. Wall archetype note (honest stub)

No dedicated wall `SiteArchetype` yet. P2 maps **Defensive wall** → `MilitaryBase` for commit identity only; player-facing label is **Defensive wall** (never “Military base”). Residual: dedicated wall archetype / corridor plan kind — **out of P2**.

### 1b. Explicit non-goals

| Out | Why |
|:---|:---|
| Phase 9 military industry / production | Separate program |
| Logistics / convoy | Residual packet |
| Building Look / module kits / APS | Residual packet |
| Place animation (P3) | Residual |
| G-PLAY rubric | Residual |
| New execute writers / `allows_commit` bypass | Invariants 2·7 |
| Road-paint FSM for trenches/walls | Prefer reuse of green two-click |

---

## 2. Interaction model

### 2a. Defense place (Wall / Trench / Bunker)

**Reuse** [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) §1 two-click FSM verbatim:

```text
Preview → (LMB) Adjust → (LMB) Place → CommitConstructionSiteEvent → Preview
Cancel: RMB / Esc → Preview
Adjust: Ctrl+scroll rotate · Shift+scroll size · X mirror
```

| Kind | Footprint default (designer intent) | Rotate / size |
|:---|:---|:---|
| Defensive wall | Strip **4×1** (or 1×4 after rotate) | Yes |
| Trench line | Strip **6×1** | Yes |
| Bunker | Compact **2×2** (or catalog matrix if present) | Yes |

Invalid tiles: red hatch + toast — second LMB blocked (`allows_commit`).

### 2b. Demolish (sibling)

Unchanged: LMB pick → pending → confirm. Lives in Military picker **Editing** footer **and** toolbox Editing. Selecting Demolish closes defense place tool; selecting a defense row clears demolish pending.

### 2c. Authority

```text
ActiveBuildTool          → sole tool source (invariant 4)
BuildGhostState          → preview only
BuildPlacementPreview    → allows_commit gate
CommitConstructionSiteEvent → single site commit funnel
ConstructionPlanQueue    → unchanged (roads/rail only); defense sites do NOT invent a parallel plan writer
```

**Coder option (either is PASS-valid):**

1. `BuildTool::Defense(DefenseKind)` sharing building two-click systems, **or**
2. `BuildTool::Building(...)` + Military catalog ids that resolve to the archetypes above.

Prefer (1) if Military must not collide with Industry building menus; (2) if existing catalog registry already covers the three ids.

---

## 3. Locked HUD copy

### 3a. Picker (`sim_hud_copy` — add constants)

| Const | String |
|:---|:---|
| `PICKER_TITLE_DEFENSE` | `Defense` |
| `PICKER_DEFENSE_LEAD` | `Place walls, trenches, or bunkers — demolish is separate.` |
| `PICKER_SECTION_WALLS` | `Walls` |
| `PICKER_SECTION_TRENCHES` | `Trenches` |
| `PICKER_SECTION_FORTIFICATION` | `Fortification` |
| `PICKER_ROW_DEFENSIVE_WALL` | `Defensive wall` |
| `PICKER_ROW_TRENCH_LINE` | `Trench line` |
| `PICKER_ROW_BUNKER` | `Bunker` |
| `PICKER_ROW_DEMOLISH` | `Demolish` |
| `PICKER_DEFENSE_FOOTER_HINT` | `Two clicks to place · Ctrl rotate · Shift size` |

### 3b. Context strip / tray peek

| Mode | Copy |
|:---|:---|
| Defense Preview | `Click map to lock {row label}` |
| Defense Adjust + valid | `Place {row label} — click map again` |
| Defense Adjust + invalid | `Cannot place — {short reason}` |
| Demolish armed | `LMB: pick target · Confirm: demolish` |

Reuse `TRAY_PEEK_MODIFIERS` (`Ctrl rotate · Shift scale`) for defense Adjust.

### 3c. Tool hints (editor floater / tray)

| Tool | Lines |
|:---|:---|
| Defense place | Same as Building two-click hints; lead with row label |
| Demolish | Existing demolish hints — unchanged |

### 3d. Validation feedback labels

Existing strings OK: `Trench line`, `Bunker complex`. Add/override display for wall stub: **`Defensive wall`** (do not show “Military base” in player HUD when source was defense catalog).

---

## 4. Sheet rect / pointer-gate notes (coder)

| Item | Spec |
|:---|:---|
| Sheet size | Reuse `BUILD_PICKER_SHEET_W_PX` (320) · `BUILD_PICKER_MAX_H_PX` (480) · gap `BUILD_PICKER_SHEET_GAP_PX` (8) |
| Anchor | `build_rail_slot_anchor_xy(ToolContext::Military, …)` — **no new rect API** |
| Open rule | **Remove** Military early-return that forces `open = false` |
| Category | Add `BuildPickerCategory::Defense` **or** map Military → Defense title without Zone fallback |
| Pointer gate | Same as Industry picker: sheet rect → `egui_blocks` / finalize path — **no new chrome region** in [`design_pointer_hud_regions_v1.md`](design_pointer_hud_regions_v1.md) |
| Cursor | Placement tools: build-scoped OS hide + unified crosshair ([`sim_hud_cursor_live.json`](../debug_runs/sim_hud_cursor_live.json)) — defense place inherits Building policy |
| Icon atlas | `tool_context_icon_id(Military)` may stay `None` for P2; text **Mi** / title **Defense** sufficient |

---

## 5. Accessibility

| # | Requirement |
|:---:|:---|
| A1 | Section headers + row labels in text — not icon-only |
| A2 | Demolish visually separated (footer / Editing) from place rows |
| A3 | Invalid place states use reason text (toast + strip) |
| A4 | Esc unlocks ghost; Esc again can clear tool (parity with Building) |
| A5 | Colorblind: reuse existing valid/invalid ghost hatch + lock ring — no hue-only defense state |

---

## 6. Coder handoff — COD-MIL-DEFENSE-PLACE-001

```text
Read:  src/dev/design_mil_defense_place_v1.md
       src/dev/design_build_ux_redesign_v1.md
       src/dev/construction_invariants.md
       src/dev/design_pointer_hud_regions_v1.md
Touch: build_tool_authority.rs (Military ≠ Demolish default; DefenseKind or catalog Building)
       sim_build_picker_sheet.rs (open Military; Defense category + sections)
       sim_hud_copy.rs (locked strings §3)
       tool_hints.rs · contextual_tip / context_tray_build_egui (strip copy)
       build_interaction.rs (reuse two-click FSM — do not fork)
       validation_feedback.rs (Defensive wall display label)
Do:    Military → picker → place via CommitConstructionSiteEvent
       Demolish as sibling only
Do NOT: Phase 9 industry · place anim · logistics · new commit writer · bypass allows_commit
Verify: validate-report cargo --cached --compress 4
        cargo test -p proc_A_dine01 --lib construction::
Witness: debug_runs/des_mil_defense_place_live.json (set impl_wired after code)
```

### joint: @coder

Does `BuildTool::Defense(_)` violate view authority or dual-write `ActiveBuildTool`? Prefer single writer + shared two-click path with Building.

---

## 7. Residuals (explicit)

| ID | Scope | Owner |
|:---|:---|:---|
| **P3 place anim** | Charter **PASS** → [`design_place_feedback_anim_v1.md`](design_place_feedback_anim_v1.md) · **COD-P3-PLACE-ANIM-001** | `@coder` next |
| Wall `SiteArchetype` | Replace MilitaryBase stub — **do not expand in place-anim** | `@planner` (packet only) |
| Phase 9 military industry | Production / activation | Do not start |
| Logistics / G-PLAY / APS / Building Look | Packet scope_out | Skip |

---

## 8. Review gate (designer)

| Check | Pass |
|:---|:---:|
| Military opens Defense catalog (not Demolish-only) | ✓ |
| Walls / Trenches / Fortification IA locked | ✓ |
| Two-click reuse — no novel place FSM | ✓ |
| Demolish sibling, not default | ✓ |
| Sheet/pointer reuse — no new chrome region | ✓ |
| Commit funnel = existing site event | ✓ |
| Copy strings locked | ✓ |
| Phase 9 / anim / logistics excluded | ✓ |
