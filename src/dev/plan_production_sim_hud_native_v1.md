# PLAN-PRODUCTION-SIM-HUD-NATIVE-001 — Kill egui jank · Bevy-native product GUI

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-PRODUCTION-SIM-HUD-NATIVE-001** |
| **Machine queue** | [`tools/orchestrator/queues/production_sim_hud_native_queue.json`](../../tools/orchestrator/queues/production_sim_hud_native_queue.json) |
| **Date** | 2026-09-23 |
| **Owner** | `@designer` (charter) → `@coder` (Bevy ports) · human (G-PLAY) |
| **Status** | **ACTIVE** — plan + drain seed |
| **Boundary** | [`prompts/guides/ui_boundary_guide_v1.md`](../../prompts/guides/ui_boundary_guide_v1.md) |
| **Aesthetic** | [`src/gui/style/palette.rs`](../gui/style/palette.rs) · keep hues; tighten craft |
| **Parent audits** | [`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md) · [`debug_runs/agent_ops/gui_jank_audit.yaml`](../../debug_runs/agent_ops/gui_jank_audit.yaml) |
| **Particle drain** | PARTICLES-GUI code **done** — human OPERATOR-VERIFY still open |

---

## 0. Verdict (designer + steward)

**Machine Phase 2/2B is closed.** Human play still reads as **two apps**: Bevy shell + floating **bevy_egui** satellites (this is the “imgui jank” — **not Dear ImGui**; we do **not** introduce Dear ImGui).

**North star:** *One instrument* — always-on product chrome is **Bevy `Node` UI**; egui is **editor/dev only** (plus short interim satellites while Bevy ports land).

**Rejected:**
- New colour system / APS Tk patterns in sim
- Dear ImGui / second immediate-mode stack
- Re-opening Stage 5 / overlay writers
- Re-picking closed SIM-HUD-PHASE2 / power-grid UX / APS-UIUX rows

---

## 1. Presentation contract (sizes + clarity)

Keep [`UiPalette`](../gui/style/palette.rs). Tighten craft:

| Surface | Target | Rule |
|:---|:---|:---|
| Ops strip | 28–32 px row | One data density; cyan labels / green values |
| Build rail icons | 48–52 px hit | Gold selected border ≥2 px; no tiny 16 px icons as primary |
| Context tray peek | ≥36 px body when tool armed | Modifiers always visible mid-place (PLAY-01 idle collapse OK) |
| Anchored sheet | ≤320 px wide · rail-adjacent | **≤1** open at a time |
| Map hover chip | ≤180×72 px | Map-attached only; no free `Window` |
| Type | Caption / title / data roles | No engineer IDs as primary labels (`concrete_portland` → human name) |
| Focus | Visible focus ring | Keyboard Esc cascade: sheet → tray → pause |
| Motion | 120–180 ms | Panel open/close only — no bounce |

Density: [`HudDensityProfile`](../gui/style/density.rs) stays; product Bevy chrome must **not** depend on egui `pixels_per_point` for hit targets.

---

## 2. Architecture (native vs satellite)

```text
Simulation session
├── Bevy product chrome (ALWAYS)     ← expand this
│   ops strip · build rail · tray · tip · cause · minimap · pause
└── bevy_egui
    ├── INTERIM product satellites   ← migrate away (Phase B–C)
    │   build picker · road/power sheets · tray build body · hover cards
    └── ALLOWED forever (editor/dev)
        WorldGen · map editor · F3/F4 · placement debug · production tools
```

**Decision rule** ([`ui_boundary_guide_v1.md`](../../prompts/guides/ui_boundary_guide_v1.md)):
player-facing → Bevy · editor/dev → egui · **never mix in one plugin**.

---

## 3. Fix inventory (throw out jank)

### Kill / migrate (product path)

| ID | Surface | Today | Target |
|:---|:---|:---|:---|
| HUD-NAT-001 | Build picker sheet | `sim_build_picker_sheet.rs` egui Area | Bevy rail-anchored sheet |
| HUD-NAT-002 | Road / power tool sheets | `sim_road_tool_sheet` / `sim_power_tool_sheet` | Bevy sheets |
| HUD-NAT-003 | Tray Build body | `context_tray_build_egui.rs` | Bevy tray children |
| HUD-NAT-004 | Staged parametric | `staged_ghost_panel.rs` RIGHT_BOTTOM | Tray P1 only |
| HUD-NAT-005 | Tool hints | `tool_hints.rs` LEFT_BOTTOM | Fold into `contextual_tip` |
| HUD-NAT-006 | Category menus | `industrial_menu` / commercial / utilities / mock | Cards inside Bevy picker |
| HUD-NAT-007 | Road popup Window | `roads/popup.rs` | Delete float; use HUD-NAT-002 |
| HUD-NAT-008 | Side status overlap | `hud_side_status_panel.rs` | Bevy status or retire |
| HUD-NAT-009 | Plant / power hover | plant_focus + power_node_hover | Bevy map chip or keep **one** themed egui chip |

### Keep egui (allowed)

WorldGen, world preview, map editor, agent perms, faction tools, production tools UI, F3/F4, `placement_debug` (must not steal rail — already gated).

### Already shipped (do not re-pick)

GUI-ESC-001 · GUI-EGUI-BUDGET · GUI-CRAFT-001 peek · BUILD-READ-WORLD-002 (1.65) · SIM-HUD Phase2B · popup-tiers witness · power-grid UX drain.

---

## 4. Phased execution

### Phase 0 — Hygiene + human gate (serial first)

| Slice | Owner | Exit |
|:---|:---|:---|
| HUD-OPS-001 | human | OPERATOR-VERIFY + G-PLAY-OPERATOR-01 ([checklist](../../debug_runs/agent_ops/particles_gui_operator_checklist.md)) |
| HUD-OPS-002 | `@sim-steward` | Unblock VSS-T3-003/004 in queue (T3-001 done); scrub stale HANDOFF “ACTIVE Track B / pick T3-001” |
| HUD-OPS-003 | `@ops` | Fix `multi_parallel_tracks_dispatch_v1.json` parse if still broken |

### Phase A — Presentation polish on Bevy shell (no big ports)

| Slice | Owner | Exit |
|:---|:---|:---|
| HUD-A-001 | `@designer`→`@coder` | Ops strip + rail spacing/type roles match §1 table; witness screenshot or lib layout asserts |
| HUD-A-002 | `@coder` | Tray mid-place always Peek+Build tab when building armed (strengthen GUI-CRAFT) |
| HUD-A-003 | `@coder` | Ban new `egui::Window` on sim place path (lint / test) |
| HUD-A-004 | `@coder` | Human labels on remaining egui menus until HUD-NAT-006 |

### Phase B — Native Bevy build sheets (critical path)

| Slice | Owner | Exit |
|:---|:---|:---|
| HUD-NAT-001 | `@coder` | Build picker is Bevy; egui picker **off** in Simulation |
| HUD-NAT-002 | `@coder` | Road + power sheets Bevy |
| HUD-NAT-006 | `@coder` | Category cards live inside Bevy picker; delete floating `*_menu` egui paths in sim |
| HUD-NAT-007 | `@coder` | Road popup Window gone |

**SERIAL:** one sheet owner file set per session; pointer-gate tests green.

### Phase C — Tray + hints + hover

| Slice | Owner | Exit |
|:---|:---|:---|
| HUD-NAT-003 | `@coder` | Tray Build body Bevy |
| HUD-NAT-004 | `@coder` | Staged book only in tray |
| HUD-NAT-005 | `@coder` | tool_hints retired |
| HUD-NAT-008/009 | `@coder` | Side status + hover chips Bevy or single themed satellite |

### Phase D — Subsystem drains (parallel after Phase B start)

| Slice | Domain | Owner | Note |
|:---|:---|:---|:---|
| VSS-T3-003 | Fire save hash | `@coder` | After queue unblock |
| VSS-T3-004 | Effects DEBT stubs | `@cleanup-intelligence` | After T3-001 |
| VSS-T4-005 | APS EffectSpec panel | `@designer` | Artist UI — not sim HUD |
| DR-RTT-VR16 / RTT-C-* | Tactical RTT | human | `--test vfx` |
| ES-7-004 | Hanabi L3 | deferred | `hanabi_l3` + eyes — never world weather |

### Phase E — Accept

| Slice | Owner | Exit |
|:---|:---|:---|
| HUD-GPLAY-002 | human | 10‑min professional feel checklist (reflection § professional) |
| HUD-WIT-001 | `@coder` | `ui_shell_migration_live.json` + new `production_sim_hud_native_live.json` green |

---

## 5. Remaining drain ladder (all open product work)

Ordered — agents must `get que` against this, not stale hub:

1. **Human:** OPERATOR-VERIFY + G-PLAY-OPERATOR-01  
2. **Human:** G-PLAY-01 formal close  
3. **Hygiene:** unblock VSS-T3-003/004 · scrub HANDOFF stale ACTIVE  
4. **HUD-A-*** presentation polish  
5. **HUD-NAT-001 → 002 → 006 → 007** Bevy build sheets  
6. **HUD-NAT-003 → 005 → 004 → 008/009** tray/hints  
7. **VSS-T3-003** fire save · **VSS-T4-005** APS panel · **VSS-T3-004** DEBT  
8. **Operator:** RTT/VFX display · PERF-SHELL · VT-5  
9. **Deferred:** Hanabi L3 · city grammar P2 · BQ screenshots (not GUI primary)

### Do not pick

MIG-* · PARTICLES-GUI P-A/B/G-A..D · VSS-T3-001 / T4-005b · RGR-M1 · power-grid UX drain · COD-SIM-HUD-* false-blocked · APS-UIUX closed · BUILD-READ-WORLD-002 redo.

---

## 6. Validation

| Gate | Proof |
|:---|:---|
| Lib | Esc cascade · pointer rail · tray peek · no sim `egui::Window` on place path |
| Cargo | `validate_cargo_report` compress=4 |
| Human | checklist + professional 10‑min |
| Boundary | `product_egui_shell_in_simulation == false` for dock shells |

**Forbidden:** harness-only green · new particle draw forks · Dear ImGui · default gray egui chrome on product path.

---

## 7. First picks after plan approve

```text
1. HUD-OPS-001 (human) — parallel OK with HUD-OPS-002 steward hygiene
2. HUD-A-001 + HUD-A-003 (designer/coder) — polish + Window ban
3. HUD-NAT-001 (coder) — Bevy build picker
```
