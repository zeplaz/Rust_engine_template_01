# DESIGN-POWER-LINE-CONSTRUCTION-001 — Power grid build & map read `v1`

| Field | Value |
|:---|:---|
| **Program** | **PLAN-POWER-GRID-CONSTRUCTION-UX-001** |
| **ID** | **DESIGN-POWER-LINE-CONSTRUCTION-001** |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (charter)** |
| **Parent infra** | [`plan_infrastructure_world_layers_exec_001_v1.md`](plan_infrastructure_world_layers_exec_001_v1.md) INFRA-E4 · [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) |
| **Power sim** | [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) · `src/entities/production/power/` |
| **Build UX pattern** | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · road/rail path (`road_path`, `Curved preview`) |
| **Strategic read** | [`power_damage_ui_persistence_v1.md`](../../docs/reference/designer_questions/production_economy/power_damage_ui_persistence_v1.md) |
| **HUD shell** | [`plan_sim_hud_professional_polish_v1.md`](plan_sim_hud_professional_polish_v1.md) — tool sheet framework |

**Headline:** Power is **core strategy** — cut a line, island a smelter, knock out a transformer, repair your yard. Today the engine has **grid topology + overload + utility graph types**, but **no player-facing line draw tool** (`UtilityAuthoringTool` is a stub; `BuildTool` has Road/Rail only). Lines must be **fun to draw**, **obvious on the map**, and **readable under combat damage**.

**North star:** Drawing power feels as satisfying as drawing roads — with a **one-tap curved ↔ 90°** switch, **clear voltage read**, and **instant feedback** when a segment will connect, overload, or island.

**Rejected:** invisible graph edges · Enter-to-commit lines · AI-drawn routes · mixing water + power in one tool without type picker · tile `has_power` bool.

---

## 0. What exists (honest)

| Layer | State | Location |
|:---|:---|:---|
| **Power sim** | Medium | `PowerRuntimePlugin`, `ElectricalGrid`, `TransformerComponent`, `GridOverloadEvent` |
| **Utility graph** | Snapshot + hydrate | `UtilityNetworkSnapshot`, `PowerLine { voltage: VoltageClass }`, `UtilityGraph` |
| **Authoring tool** | **Stub** | `UtilityAuthoringTool` — mode enum only, no draw loop |
| **Build rail** | Buildings + road + rail | `BuildTool` — **no PowerLine variant** |
| **Utilities submenu** | Place transformer/plant | `utilities_menu.rs` — **not lines** |
| **Road/rail draw UX** | **Shipped pattern** | Control points, curved preview, grid snap, popup sheet |
| **Map overlay** | Read-only stroke | Power `#e8c040` 2px — [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) |
| **Overload UX** | Toast only | `grid_overload_ux.rs` — ops strip flash |

**Gap:** construction funnel stops at **nodes** (plants, transformers); **edges** (lines) are not a first-class build tool.

---

## 1. Gameplay pillars (design intent)

| Pillar | Player fantasy | Sim hook |
|:---|:---|:---|
| **Build** | “I’m wiring my industrial empire” | `UtilityGraph` edge commit |
| **Route** | Curved scenic HV vs tight 90° yard feeds | Spline vs orthogonal router |
| **Read** | See load, voltage, weak links at a glance | Overlay + hover card |
| **Attack** | Cut lines, blast transformers, island enemy smelters | Graph cut → activation fail |
| **Defend** | Redundant paths, spare transformers, repair queue | Island detection + repair jobs |
| **Knockout** | One transformer kill darkens a district | `rebuild_electrical_grid_topology` membership |

**Emergent loop (target):** enemy cuts your MV feed → smelter stalls → fabrication backlog → you reroute 90° through substation yard → repair crew restores redundant HV.

---

## 2. Power line tool — interaction model

### 2.1 Tool entry (build rail)

| Rail slot | Label | Opens |
|:---|:---|:---|
| **Ut** (Utilities) | long-press or submenu tab | **Nodes** (transformer, plant) · **Lines** (power draw) |

**Default sim path:** Utilities → **Draw power line** (not buried in editor-only menu).

Align with [`design_sim_hud_build_picker_v1.md`](design_sim_hud_build_picker_v1.md) when signed — Lines tab under Utilities.

### 2.2 Draw modes (routing style)

| Mode | Icon | Behaviour | Best for |
|:---|:---|:---|:---|
| **Curved** | ~ arc | Catmull-Rom / gentle spline between control points (reuse road spline spine) | Long HV transmission, terrain following |
| **Orthogonal (90°)** | ⊞ corner | Manhattan routing: only axis-aligned segments, auto corner nodes | Substation yards, factory feeds, city blocks |

**Toggle:** tool sheet chip **`Curved`** | **`90°`** — same mental model as road **`Curved preview`** checkbox; **one key** `[` / `]` or **`O`** cycles mode.

**Rule:** mode applies to **preview + commit** — no “curved preview but orthogonal commit”.

### 2.3 Control point flow (match road muscle memory)

| Input | Result |
|:---|:---|
| **LMB** | Add control point / extend from last node |
| **RMB** | Undo last point |
| **Shift+LMB** | Commit path (alternative to Build button — power users) |
| **Esc** | Cancel path |
| **Snap hover** | Highlight valid anchor: transformer, substation, line junction, optional transport corridor |

**Minimum commit:** 2 nodes + valid voltage profile + both endpoints snapped to **connectors** (transformer bus, substation pad, line tee).

### 2.4 Line types (voltage class)

Map to existing `VoltageClass` + designer-facing labels:

| Class | Player label | Stroke (map) | Weight | Typical use |
|:---|:---|:---|:---:|:---|
| **Low** | Distribution | `#e8c040` | 2px | Last mile to factory/mine |
| **Medium** | Medium voltage | `#f0d050` | 3px | Substation → industrial cluster |
| **High** | Transmission | `#ffd878` + subtle glow | 4px | Plant → substation long haul |

**Picker:** tool sheet dropdown **before** first point — default remembers last; invalid combo (HV into residential stub) shows **blocked: voltage mismatch** on context strip.

Optional future: `PowerDistributionType` (1φ/3φ) as **advanced** collapsed row — not P0.

### 2.5 Snap & assist (not janky)

| Assist | Default | Why |
|:---|:---:|:---|
| Snap to transformer/substation | **on** | Lines must terminate on nodes |
| Snap to existing line junction | **on** | Tee splits readable |
| Snap to transport corridor | off | Optional — industrial aesthetic |
| Grid snap (orthogonal mode) | **on** | Clean 90° yards |
| Auto-bridge gap ≤ N tiles | off | Avoid magic autowire — player places tee |

**Invalid preview:** red hatch segment + strip reason (`no anchor`, `voltage mismatch`, `through blocked tile`).

---

## 3. Tool sheet UI (power line)

Use **Tool Sheet** tier from [`design_sim_hud_popup_tiers_v1.md`](design_sim_hud_popup_tiers_v1.md) (when signed) — same chrome as road popup, not ad-hoc window.

```text
┌ Power line — Medium voltage ─────────────┐
│ Mode:  [ Curved ] [ 90° ]                │
│ Type:  ( ) Distribution  (•) Medium  ( ) Transmission │
│ Snaps: [x] Transformers  [x] Junctions  [ ] Corridors │
│ Points: 4   Valid: 3   Est. cost: 120    │
│ ─────────────────────────────────────── │
│ LMB add · RMB undo · Shift+LMB commit    │
│ [ Build line ]  [ Cancel ]               │
└──────────────────────────────────────────┘
```

**Context strip (always):**

| State | Template |
|:---|:---|
| Drawing | `POWER · {class} · {mode} · LMB add point · RMB undo · Shift commit` |
| Invalid segment | `POWER · blocked: {reason}` |
| Committed | `POWER · line queued · {n} segments` |

---

## 4. Map read & overlay (clarity)

### 4.1 Line states (visual)

| State | Stroke | Pattern |
|:---|:---|:---|
| **Live** | class color | solid |
| **Preview** | class color @ 60% α | dashed |
| **Damaged** | `warn` amber | dash + spark glyph at break |
| **Destroyed** | `danger` | gap + X node |
| **Enemy-owned** | class color | alternate dash (faction tint edge) |
| **Unpowered island** | muted gray | dim + pulsing consumer icons optional P2 |

### 4.2 Node glyphs (transformers / substations)

| Node | Map glyph | Hover card |
|:---|:---|:---|
| Transformer | ▣ pad + coil icon | In/out voltage, load %, member count |
| Substation | ▣ larger yard | Feeds N consumers |
| Power plant | existing building read | Output MW, fuel |
| Line junction | ● tee | Connected edges |

**Hover card (minimal P0):** `MV · 72% load · 3 factories` — not engineer IDs.

### 4.3 Overlay toggles (sim)

Extend [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md):

| Toggle | Default sim | Note |
|:---|:---|:---|
| Power lines | **on** when power tool active | auto-on while drawing |
| Power nodes | on with lines | transformers visible |
| Load heat | off | optional P1 — line thickness ∝ load |
| Island highlight | on when alert | dim unpowered subgraph |

**Minimap:** power off default; **blink** overload/island alert only (no 2px tactical clone).

### 4.4 Strategic targeting read

When military/damage tool active:

| Target | Feedback |
|:---|:---|
| Line segment | HP bar + “Cut → islands N consumers” preview |
| Transformer | “Knockout → darkens {district}” |
| Repair | “Restore segment · parts: {n} · ETA” |

Links repair queue to [`power_damage_ui_persistence_v1.md`](../../docs/reference/designer_questions/production_economy/power_damage_ui_persistence_v1.md).

---

## 5. Damage, islanding, repair (UX charter)

### 5.1 Island detection (player-facing)

When graph split detected:

| Channel | Copy |
|:---|:---|
| Toast | `Power island — {n} buildings offline` |
| Ops strip PWR | `⚠ Island · {n} offline` |
| Map | Dim island + gold boundary on cut edge |

### 5.2 Knockout logic (readable, not hidden)

| Event | Player sees |
|:---|:---|
| Transformer destroyed | Subgraph loses upstream → consumers **offline** badge |
| Line cut | Same — **instant** preview before confirm attack |
| Overload | existing toast + line **heat** optional P1 |
| Repair complete | Segment solid + brief green flash |

**Sim authority:** `UtilityGraph` cut + `ElectricalGrid.members` — UX mirrors graph truth, never fake.

---

## 6. Relationship to roads/rails

| Aspect | Road/Rail (today) | Power line (target) |
|:---|:---|:---|
| Tool enum | `BuildTool::Road/Rail` | **`BuildTool::PowerLine`** |
| Path resource | `ActiveRoadPlacement` | `ActivePowerLinePlacement` (parallel) |
| Curved | `use_curved_preview` | **`RoutingMode::Curved`** |
| Orthogonal | — | **`RoutingMode::Orthogonal90`** |
| Popup | `road_tool_popup` | **`power_line_tool_sheet`** |
| Commit | construction queue | **`UtilityNetworkSnapshot` edge** |
| Overlay | road gray | gold family |

**Reuse:** spline subdivide from `infrastructure/transport/spline.rs`; input rhythm from `road_path_input_system`.

---

## 7. Designer / coder split

| Phase | @designer | @coder |
|:---|:---|:---|
| **P0 Charter** | This doc + wire sheet + copy | — |
| **P1 Tool** | Voltage picker IA, snap rules | `BuildTool::PowerLine`, path input, commit to graph |
| **P2 Map read** | Overlay states, hover cards | Compositor strokes, damage visual |
| **P3 Combat** | Targeting previews, repair panel | Graph cut, island toasts, repair queue UI |
| **P4 Polish** | Load heat, minimap alert | Performance + witness |

**@designer-mcp (later):** pole/tower modules for HV — not blocking P1.

---

## 8. Acceptance tests (operator)

| # | Test | Pass |
|:---:|:---|:---:|
| T1 | Pick MV line → curved → 4 points → commit → gold stroke on map | |
| T2 | Switch to 90° → rectangular yard feed → corners clean | |
| T3 | Snap transformer → factory — activation succeeds when line live | |
| T4 | Cut line — island toast + dim consumers | |
| T5 | Destroy transformer — knockout preview matched result | |
| T6 | Repair segment — line solid, power restored | |
| T7 | Invalid voltage — red preview + strip reason, no commit | |
| T8 | 1080p — stroke readable over grass + industrial tiles | |

---

## 9. Dependencies

| Upstream | Need |
|:---|:---|
| INFRA-E4-002 | `UtilityGraph` hydrate |
| INFRA-E4-003 | Building `UtilityConnection` |
| DES-SIM-HUD-BUILD-PICKER-001 | Utilities tab houses Lines tool |
| IND-E03 | Overload toast pattern extend for island |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (charter)** | 2026-06-18 |

```text
DESIGN-POWER-LINE-CONSTRUCTION-001 → DES-POWER-LINE-TOOL-SHEET-001 · COD-POWER-LINE-DRAW-001
```
