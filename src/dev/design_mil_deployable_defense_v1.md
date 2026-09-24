# Military deployable defense — catalog IA `v1` (DES-MIL-DEPLOYABLE-CATALOG)

| Field | Value |
|:---|:---|
| **ID** | **DES-MIL-DEPLOYABLE-CATALOG** |
| **Priority** | **P2** |
| **Owner** | `@designer` (charter) · `@coder` **COD-WALL-CONCRETE-GATE-001** ∥ **COD-DEPLOYABLE-RECIPE-001** |
| **Verdict** | **PASS** |
| **Date** | 2026-09-24 |
| **Prereq** | **DES-MIL-DEFENSE-PLACE** · **COD-MIL-DEFENSE-PLACE-001** · **COD-WALL-ARCHETYPE-001** · **COD-P3-PLACE-ANIM-001** |
| **Parent UX** | [`design_mil_defense_place_v1.md`](design_mil_defense_place_v1.md) |
| **Plan** | [`plan_mil_deployable_defense_v1.md`](plan_mil_deployable_defense_v1.md) |
| **Planner packet** | [`debug_runs/des_mil_deployable_defense_planner_packet.json`](../debug_runs/des_mil_deployable_defense_planner_packet.json) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |
| **Witness** | [`debug_runs/des_mil_deployable_defense_live.json`](../debug_runs/des_mil_deployable_defense_live.json) |
| **Unblocks** | **COD-WALL-CONCRETE-GATE-001** · **COD-DEPLOYABLE-RECIPE-001** |

**No ECS in this charter** beyond locked HUD copy constants. Preview ≠ commit preserved. No Phase-9 chrome.

---

## Mission

Military Defense picker must distinguish **in-situ poured walls** from **manufactured deployables** that cannot free-stamp.

**Acceptance test:** *Open Military → Defense picker → Walls row still “Defensive wall” (concrete on commit) → Deployables section shows Dragon’s teeth (heavy) and Minefield (light) → same two-click FSM → second LMB blocked with clear copy when staged stock / concrete insufficient — Demolish footer unchanged.*

---

## 1. UX goals

1. **Class readability** — player instantly knows wall = pour concrete; teeth/mines = haul then deploy.
2. **Reject stamp UX** — deployables never look like free palette stamps; blocked state is first-class.
3. **Reuse chrome** — Military rail + picker **320×480**; no new sheet, panel, or Phase-9 industry UI.
4. **One funnel** — same two-click + `CommitConstructionSiteEvent` as live wall/trench/bunker.

---

## 2. Interaction problems (solved)

| Problem | Design answer |
|:---|:---|
| Walls vs prefabs look identical in catalog | Separate **Deployables** section + row captions (poured vs heavy/light staged) |
| Operator expects free-stamp place | Commit gated; strip + toast name **insufficient stock** / **in transit** |
| Planner split Obstacles + Emplacement | **Consolidate** into one **Deployables** section (fit 320×480; same authority chain) |
| Staging status chrome sprawl | **One context-strip line** on ghost — not a sheet section (Q1 closed) |

---

## 3. Catalog IA (Military picker — additive)

**Rail / sheet:** unchanged — `ToolContext::Military` · title **Defense** · `BUILD_PICKER_SHEET_W_PX` 320 · `BUILD_PICKER_MAX_H_PX` 480 · gap 8 · Military rail anchor.

### Sections (order top → bottom)

| Section | Player label | Rows | DeployClass | Place FSM |
|:---|:---|:---|:---|:---|
| **Walls** | Walls | Defensive wall | `InSituMaterial(concrete)` | Two-click · commit needs concrete |
| **Trenches** | Trenches | Trench line | *(unchanged — material deferred)* | Two-click |
| **Fortification** | Fortification | Bunker | *(unchanged — deferred)* | Two-click |
| **Deployables** *(new)* | Deployables | Dragon’s teeth · Minefield | `ManufacturedPrefab` heavy / light | Two-click · commit needs **staged** stock |
| **Editing** (footer) | — | Demolish | — | Pending → confirm *(unchanged)* |

**Default on Military open:** picker open, no tool armed — first catalog click arms (parity with live place).

### Row → runtime identity (coder)

| Row label | `DefenseKind` | `SiteArchetype` | Cargo / stock |
|:---|:---|:---|:---|
| Defensive wall | `DefensiveWall` | `DefensiveWall` | Concrete buffer · consume on commit |
| Dragon’s teeth | `DragonTeeth` *(new)* | `DragonTeeth` *(new)* | `dragon_teeth_unit` · heavy · staging gate |
| Minefield | `Minefield` *(new)* | `Minefield` *(new)* | `mine_unit` · light · staging gate |

Planner names **Obstacles** / **Emplacement** are product synonyms only — player chrome uses **Deployables**.

### Footprints (designer intent)

| Kind | Default | Rotate / size |
|:---|:---|:---|
| Defensive wall | Strip **4×1** *(live)* | Yes |
| Dragon’s teeth | Strip **3×1** | Yes |
| Minefield | Field **3×3** | Yes (size optional) |

---

## 4. Place UX (two-click + stock gate)

Reuse [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) §1 verbatim:

```text
Preview → (LMB) Adjust → (LMB) Place → CommitConstructionSiteEvent → Preview
Cancel: RMB / Esc → Preview
Adjust: Ctrl+scroll rotate · Shift+scroll size · X mirror
```

### Commit gates (player-visible)

| Kind | `allows_commit` false when | Player reason (locked) |
|:---|:---|:---|
| Defensive wall | Concrete stock &lt; footprint need | `insufficient concrete` |
| Dragon’s teeth / Minefield | Staged units &lt; footprint need | `insufficient staged stock` |
| Deployable + freight only | Staging 0 but in-transit &gt; 0 | `stock still in transit` |
| Any | Tile/terrain invalid | existing invalid reasons |

**Ghost may show** while blocked — second LMB does **not** commit. Invalid: red hatch + strip/toast (shape + text, not hue-only).

### Staging UX (Q1 — closed)

**One context-strip / ghost caption line** while a deployable tool is armed:

```text
Staged {n} · in transit {m}
```

When `n ≥ need`: append ` — ready`. When `n = 0` and `m = 0`: `No staged stock — manufacture and haul first`.

⛔ New picker sheet section for staging · Phase-9 plant browser · stamp-without-gate mode.

---

## 5. Locked HUD copy

Authority: [`sim_hud_copy_registry_v1.md`](sim_hud_copy_registry_v1.md) · consts in `src/gui/hud/sim_hud_copy.rs`.

### 5a. Picker

| Const / key | String |
|:---|:---|
| `PICKER_TITLE_DEFENSE` | `Defense` *(unchanged)* |
| `PICKER_DEFENSE_LEAD` | `Poured walls or staged deployables — demolish is separate.` |
| `PICKER_SECTION_WALLS` | `Walls` |
| `PICKER_SECTION_TRENCHES` | `Trenches` |
| `PICKER_SECTION_FORTIFICATION` | `Fortification` |
| `PICKER_SECTION_DEPLOYABLES` | `Deployables` |
| `PICKER_ROW_DEFENSIVE_WALL` | `Defensive wall` |
| `PICKER_ROW_CAPTION_DEFENSIVE_WALL` | `Poured concrete` |
| `PICKER_ROW_DRAGON_TEETH` | `Dragon's teeth` |
| `PICKER_ROW_CAPTION_DRAGON_TEETH` | `Heavy · staged stock required` |
| `PICKER_ROW_MINEFIELD` | `Minefield` |
| `PICKER_ROW_CAPTION_MINEFIELD` | `Light · staged stock required` |
| `PICKER_ROW_DEMOLISH` | `Demolish` |
| `PICKER_DEFENSE_FOOTER_HINT` | `Two clicks to place · stock gates commit · Ctrl rotate · Shift size` |

### 5b. Context strip / tray

| Mode | Copy |
|:---|:---|
| Defense Preview | `Click map to lock {row label}` |
| Defense Adjust + valid + stock OK | `Place {row label} — click map again` |
| Defense Adjust + invalid tile | `Cannot place — {short reason}` |
| Defense Adjust + stock blocked | `Cannot place — {stock reason}` |
| Deployable stock line | `Staged {n} · in transit {m}` (+ ` — ready` when enough) |
| Deployable empty | `No staged stock — manufacture and haul first` |
| Demolish armed | `LMB: pick target · Confirm: demolish` *(unchanged)* |

### 5c. Stock reason tokens (validation / toast)

| Const | String |
|:---|:---|
| `REASON_INSUFFICIENT_CONCRETE` | `insufficient concrete` |
| `REASON_INSUFFICIENT_STAGED` | `insufficient staged stock` |
| `REASON_STOCK_IN_TRANSIT` | `stock still in transit` |

### 5d. Tool hints

| Tool | Lead |
|:---|:---|
| Defensive wall | `Defensive wall — poured concrete on place` |
| Dragon’s teeth | `Dragon's teeth — needs staged heavy units` |
| Minefield | `Minefield — needs staged mine units` |
| Demolish | existing *(unchanged)* |

---

## 6. Visual hierarchy Δ

```text
Defense sheet
  lead (class contrast: poured vs staged)
  Walls → Trenches → Fortification   (live place rows)
  Deployables                        (new — captions carry heavy/light)
  Editing / Demolish                 (footer separation preserved)
  footer hint                        (stock gates commit)
```

Overlay priority unchanged: Selection/focus ▷ Construction preview ▷ Sim state. Stock line rides **preview caption**, not a new overlay layer.

---

## 7. Accessibility

| # | Requirement |
|:---:|:---|
| A1 | Section + row + caption text — not icon-only class cues |
| A2 | Demolish remains footer-separated from place rows |
| A3 | Stock-blocked states use **reason text** (strip + toast), not color alone |
| A4 | Esc unlocks ghost; Esc again clears tool (Building parity) |
| A5 | Valid/invalid hatch + lock ring reused; stock block may share invalid hatch + distinct reason string |

---

## 8. Viewport / multiview

No change. Defense place inherits Building cursor / pointer-gate / sheet rect. ⛔ New chrome region in pointer HUD map.

---

## 9. Authority (presentation only)

```text
ActiveBuildTool            → sole tool source
BuildGhostState            → preview only (may show while stock-blocked)
BuildPlacementPreview      → allows_commit ◂⊳[snapshot] concrete | SiteStagingStock
CommitConstructionSiteEvent → sole defense commit funnel
SiteStagingStock           → logistics arrivals (coder/sim-steward) — UI reads only
UI / HUD                   → ◂⊳[snapshot] staging + allows_commit — never writes stock
```

⛔ Parallel place event · ConstructionPlanQueue for defense · free-stamp Deployables · Phase-9 industry chrome.

---

## 10. Required engine hooks (coder)

| Slice | Hook |
|:---|:---|
| **COD-WALL-CONCRETE-GATE-001** | `allows_commit` for `DefensiveWall` requires concrete; debit on successful commit; trench/bunker unchanged; wire `REASON_INSUFFICIENT_CONCRETE` |
| **COD-DEPLOYABLE-RECIPE-001** | `mfg_dragon_teeth_v1` · `mfg_mine_unit_v1` under `ManufacturingDomain::Custom` |
| **COD-DEPLOYABLE-LOGISTICS-001** *(after recipes)* | buffer_tags → `InTransitLedger` → single-writer `SiteStagingStock` |
| **COD-DEPLOYABLE-PLACE-001** *(after logistics + wall gate)* | `DefenseKind`/`SiteArchetype` + picker Deployables section + staging gate + strip line |

### joint: @coder

Does staging readout on the ghost caption violate view authority if it reads `SiteStagingStock` from UI systems? Prefer snapshot/resource read in HUD only — never write staging from picker.

### joint: @sim-steward

Confirm single writer for `SiteStagingStock` before COD-DEPLOYABLE-LOGISTICS-001 (plan Q4).

---

## 11. Diagnostics required

| Artifact | Proves |
|:---|:---|
| `debug_runs/des_mil_deployable_defense_live.json` | This DES PASS · catalog sections · copy locked |
| `debug_runs/cod_wall_concrete_gate_live.json` | Wall concrete gate |
| `debug_runs/cod_deployable_pipeline_live.json` | Recipe → transit → stage → place |

After coder: `place_blocked_no_stock` · `place_ok` · `ai_commit_gated` counters non-zero where applicable.

---

## 12. Explicit non-goals

| Out | Deferral |
|:---|:---|
| Mine combat / detection / clearing | **DR-MIL-MINE-COMBAT** |
| Munitions industry wholesale / Phase 9 | **DR-MIL-MUNITIONS-INDUSTRY** |
| Deployable module kits / APS art | **DR-MIL-DEPLOYABLE-ART** |
| Trench / bunker material gates | **DR-MIL-TRENCH-MATERIAL** |
| Corridor continuous wall topology | separate unblock_when |
| New picker chrome / staging sheet section | rejected |

---

## 13. Coder handoff packets

| Packet | Path |
|:---|:---|
| Wall concrete gate | [`debug_runs/cod_wall_concrete_gate_001_delta_wf.json`](../debug_runs/cod_wall_concrete_gate_001_delta_wf.json) |
| Deployable recipes | [`debug_runs/cod_deployable_recipe_001_delta_wf.json`](../debug_runs/cod_deployable_recipe_001_delta_wf.json) |

```text
ΔWF: @designer DES-MIL-DEPLOYABLE-CATALOG PASS
  → @coder COD-WALL-CONCRETE-GATE-001 ∥ COD-DEPLOYABLE-RECIPE-001
  → COD-DEPLOYABLE-LOGISTICS-001 (@sim-steward dual-writer)
  → COD-DEPLOYABLE-PLACE-001
  → WIT-HON
```

---

## 14. Review gate (designer)

| Check | Pass |
|:---|:---:|
| Walls remain Walls · InSituMaterial(concrete) | ✓ |
| Deployables section · Dragon’s teeth + Minefield | ✓ |
| Two-click reuse — no stamp FSM | ✓ |
| Stock-blocked state copy locked | ✓ |
| Demolish footer unchanged | ✓ |
| Sheet 320×480 · no Phase-9 chrome | ✓ |
| Q1 = one strip line (not sheet section) | ✓ |
| Combat / munitions art / trench material deferred | ✓ |
| ΔWF packets for wall gate ∥ recipes | ✓ |
