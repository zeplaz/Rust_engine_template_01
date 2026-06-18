# Minimap ecology legend wire `v1` — UI + burn scar

| Field | Value |
|:---|:---|
| **ID** | **DES-MINIMAP-VEG-LEGEND-002** |
| **Program** | Product UI · Track D |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`design_minimap_veg_legend_v1.md`](design_minimap_veg_legend_v1.md) (001 tokens) · [`design_ecology_preview_legend_v1.md`](design_ecology_preview_legend_v1.md) |
| **Implementation** | [`minimap_topology_legend.rs`](../../src/gui/hud/minimap_topology_legend.rs) |
| **Handoff** | **CDR-B-VEG-MINIMAP-LEGEND-UI-001** |
| **Verdict** | **PASS** |

```text
DES-MINIMAP-VEG-LEGEND-002 Q✓
Collapsible legend UI wire — topology + burn scar rows (not tint-only)
```

---

## 0. Delta from 001

| 001 | 002 |
|:---|:---|
| Token table | **Pixel wire** + interaction states |
| Topology only | **+ burn scar** adjunct rows |
| Design intent | **Coder wire** for `legend_ui_wired: true` witness |

---

## 1. Chrome placement

```text
┌─ minimap frame (bottom-right dock) ─────┐
│ ┌──────────────────────────────────┐  │
│ │     [composited map]             │  │
│ │                                  │  │
│ └──────────────────────────────────┘  │
│ ┌ Legend ──────────────────────── ▼ ┐  │  ← inside minimap chrome, not floating HUD
│ │ Ecology kinds                    │  │
│ │ [swatch] N Network   [swatch] P …  │  │
│ │ ── Fire read ──                  │  │
│ │ [swatch] S Scar    [swatch] B Burn│  │
│ └──────────────────────────────────┘  │
└───────────────────────────────────────┘
```

| Field | Value |
|:---|:---|
| Width | match minimap inner width |
| Font | `egui` small · `UiPalette.fg_primary` |
| Background | `bg_elevated` @ 90% |
| Border | 1px `wire_magenta` top edge only |

---

## 2. Collapse interaction

| State | Header label | Content |
|:---|:---|:---|
| Collapsed | `▶ 6 kinds` | hidden |
| Expanded | `▼ Ecology kinds` | topology grid + fire section |

| Session | Default expanded |
|:---|:---|
| `BaseState::Simulation` | **collapsed** |
| WorldGen / editor | **expanded** first visit |

**Toggle:** full header row clickable · min height **22px** · `topology_legend_user_toggled` persists choice.

---

## 3. Topology grid (expanded)

**3 columns:** swatch 10×10 · glyph · word

| Glyph | Word | Hex |
|:---:|:---|:---|
| `N` | Network | `#4a6fa5` |
| `C` | Corridor | `#7a6a4a` |
| `P` | Patch | `#3d8b5f` |
| `R` | Ring | `#6a5a8a` |
| `K` | Cluster | `#2f7d4a` |
| `F` | Fringe | `#8a9a6a` |

**Layout:** 2×3 grid · row-major · spacing 8×4 px.

**Rule:** glyph + word always — swatch reinforces only ([`design_aps_color_a11y_audit_v1.md`](design_aps_color_a11y_audit_v1.md)).

---

## 4. Burn scar tokens (new section)

Shown when `overlays.ecology_heat == true` **and** `veg_burn_rows > 0`.

| Glyph | Word | Hex | Minimap merge |
|:---:|:---|:---|:---|
| `S` | Scar | `#3a3a3a` | `topology_*_scar` base |
| `B` | Active burn | `#e87830` | `veg_burn_*` overrides topology (Q4a) |
| `G` | Regrowth | `#6a9a48` | post-burn pioneer |

```text
── Fire read ──
[■] S Scar     [■] B Active burn     [■] G Regrowth
```

**When burn inactive:** hide Fire read section entirely — do not show greyed rows.

---

## 5. Status rows (replace legend body)

| Condition | Copy | Replaces grid |
|:---|:---|:---:|
| `!ecology_heat` | `○ Ecology off` | yes |
| `ecology_rows == 0` | `○ No landscape data in view` | yes |
| `map_updating` | `◐ Map updating…` | yes |

Matches [`minimap_topology_legend_status_copy`](../../src/gui/hud/minimap_topology_legend.rs).

---

## 6. Burn scar visual on map (paired)

When legend expanded and burn active:

| Map cue | Legend link |
|:---|:---|
| Topology tint washed | topology rows |
| Charcoal scar patch | `S Scar` |
| Orange ember override | `B Active burn` |

**Coherence:** same tick revision as [`minimap_fire_veg_coherence_live.json`](../../debug_runs/minimap_fire_veg_coherence_live.json).

---

## 7. Assets

| Path | Content |
|:---|:---|
| `assets/ui/minimap/legend_topology.ron` | optional layout constants |
| No new PNG | vector swatches drawn in egui |

---

## 8. Acceptance

| # | Check |
|:---:|:---|
| L1 | Collapsed default in Simulation |
| L2 | Words match world preview legend exactly |
| L3 | Burn section appears only when `veg_burn_rows >= 1` |
| L4 | `minimap_topology_legend_live.json` → `legend_ui_wired: true` |
| L5 | No `topology_graph` / `LG-5` in visible strings |

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
