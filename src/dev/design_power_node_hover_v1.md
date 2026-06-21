# Power node hover cards `v1` — transformer + substation

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-NODE-HOVER-001** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track B |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Depends** | [`design_power_map_overlay_v1.md`](design_power_map_overlay_v1.md) · [`design_power_voltage_picker_v1.md`](design_power_voltage_picker_v1.md) · [`design_sim_hud_popup_tiers_v1.md`](design_sim_hud_popup_tiers_v1.md) |
| **Copy** | [`design_power_grid_copy_v1.md`](design_power_grid_copy_v1.md) §hover |
| **Handoff** | COD-POWER-NODE-HOVER-001 |
| **Verdict** | **PASS** |

```text
DES-POWER-NODE-HOVER-001 Q✓
Map-attached hover — transformer + substation fields, human labels, state rows
```

---

## 0. Scope

**P1 map read** — lightweight card on hover over **distribution transformer** and **grid substation** nodes (building pick or overlay glyph hit).

**Out of scope v1:** power plants (use plant focus card) · line segment hover · minimap.

---

## 1. Popup tier

| Field | Value |
|:---|:---|
| Tier | **P0 `map_attached`** |
| Renderer | egui satellite · `palette.to_egui_visuals()` |
| Max open | **1** card — replaces on node change |
| Blocks map | **no** — pointer passes through after 150ms dwell |

**Placement:** offset **(+12, −8)** px from cursor · clamp inside map viewport · flip above if bottom clip.

---

## 2. Trigger & highlight

| Input | Behaviour |
|:---|:---|
| Hover dwell **150ms** | Show card |
| Move off node | Hide after **80ms** grace |
| Power line tool active | Card + **gold snap ring** on valid anchor |
| LMB while card open | Select node (focus tray Alerts if damaged) |

**Hit target:** building footprint **or** map node glyph bounds (+4px pad).

---

## 3. Transformer card

**Catalog:** `grid_distribution_transformer` · 2×2 · `utility_role: transformer`

```text
┌ Distribution transformer ──────────────┐
│ ● Online          Medium voltage       │
│ Load      ████████░░  62%              │
│ Capacity  24 / 40 MVA                  │
│ Feeds     8 consumers                │
│ Links     3 lines · 1 upstream       │
└──────────────────────────────────────┘
```

| Row | Label | Source | Format |
|:---|:---|:---|:---|
| Title | `Distribution transformer` | catalog `asset_name` | human — not id |
| Status | `Online` / `Offline` / `Damaged` / `Destroyed` | node sim state | glyph + word |
| Voltage | `Distribution` / `Medium voltage` | max `VoltageClass` on incident edges | voltage picker labels |
| Load | bar + `%` | edge throughput / capacity | `fg_data` mono |
| Capacity | `used / max MVA` | `transfer_capacity_mva` | 1 decimal |
| Feeds | `N consumers` | downstream activation count | integer |
| Links | `N lines · M upstream` | `UtilityGraph` degree | optional collapse if 0 |

**Load bar bands:** &lt;70% green · 70–90% amber · &gt;90% warn pulse.

---

## 4. Substation card

**Catalog:** `grid_substation` · 4×3 · `utility_role: substation`

```text
┌ Grid substation ───────────────────────┐
│ ● Online          Transmission in      │
│ Load      ██████████░  78%           │
│ Capacity  94 / 120 MVA                 │
│ Feeds     42 consumers               │
│ Yard      4×3 · bus + breakers         │
└──────────────────────────────────────┘
```

| Row | Label | Source | Format |
|:---|:---|:---|:---|
| Title | `Grid substation` | catalog | human |
| Status | same enum as §3 | node state | |
| Voltage | highest class **inbound** | `Transmission in` / `Medium in` / `Mixed` | |
| Load | bar + `%` | aggregated downstream | |
| Capacity | MVA used/max | `transfer_capacity_mva` | |
| Feeds | consumer count | island-aware subgraph | |
| Yard | footprint hint | `building_size_x×y` + static read | **not** engineer zone ids |

---

## 5. Status row (shared)

| State | Glyph | Word | Header tint |
|:---|:---:|:---|:---|
| Online | `●` | `Online` | none |
| Offline (island) | `○` | `Offline` | `warn` @ 10% |
| Damaged | `◆` | `Damaged` | `warn` @ 15% |
| Destroyed | `×` | `Destroyed` | `danger` @ 15% |
| Overload | `⟳` | `Overload` | `warn` pulse |

**Island:** append tray peek `⚠ Island — N offline` when subgraph disconnected from generation.

---

## 6. Chrome

| Token | Use |
|:---|:---|
| `bg_elevated` | card fill |
| `wire_magenta` | 1px border · radius 4px |
| `fg_primary` | title |
| `fg_muted` | captions |
| `fg_data` | numeric rows |
| `accent_gold` | selected / snap highlight |

**Width:** min **220px** · max **280px** · no shadow.

---

## 7. Overlay pairing

| Overlay state | Card behaviour |
|:---|:---|
| Power overlay off | Card still shows on building hover |
| Power overlay on | Glyph + card share same status color |
| Island highlight active | `Offline` + consumer count reflects island |
| Damaged edge incident | `Damaged` if node HP &lt; 100% |

Align glyphs: [`power_glyphs_spec_v1.md`](../assets/ui/infrastructure/power_glyphs_spec_v1.md).

---

## 8. Copy keys (`power_grid_copy.rs`)

| Key | Template |
|:---|:---|
| `power.hover.transformer.title` | `Distribution transformer` |
| `power.hover.substation.title` | `Grid substation` |
| `power.hover.status.online` | `● Online` |
| `power.hover.status.offline` | `○ Offline` |
| `power.hover.status.damaged` | `◆ Damaged` |
| `power.hover.status.destroyed` | `× Destroyed` |
| `power.hover.status.overload` | `⟳ Overload` |
| `power.hover.load` | `Load` |
| `power.hover.capacity` | `Capacity` |
| `power.hover.feeds` | `Feeds` |
| `power.hover.links` | `Links` |
| `power.hover.links.fmt` | `{lines} lines · {upstream} upstream` |
| `power.hover.feeds.fmt` | `{n} consumers` |
| `power.hover.capacity.fmt` | `{used} / {max} MVA` |
| `power.hover.voltage.mixed` | `Mixed voltage` |

---

## 9. Acceptance

| # | Check |
|:---:|:---|
| H1 | Hover dwell 150ms before show — no flicker on pan |
| H2 | Transformer vs substation titles distinct |
| H3 | No `Low`/`Medium`/`High` engineer strings in card body |
| H4 | Island state matches `UtilityGraph` membership — not fake count |
| H5 | At most one card · tier `map_attached` |
| H6 | Witness `debug_runs/power_node_hover_live.json` → `hover_card_wired: true` |

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
