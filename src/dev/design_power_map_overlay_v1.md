# Power map overlay states `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-MAP-OVERLAY-002** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track B |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Extends** | [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) |
| **Voltage** | [`design_power_voltage_picker_v1.md`](design_power_voltage_picker_v1.md) |
| **Handoff** | COD-POWER-OVERLAY-RENDER-001 · COD-POWER-ISLAND-HIGHLIGHT-001 |
| **Verdict** | **PASS** |

```text
DES-POWER-MAP-OVERLAY-002 Q✓
Six line states + island dim — auto-on while power tool active
```

---

## 1. When overlay shows

| Session | Power lines | Power nodes |
|:---|:---|:---|
| Sim default | **off** | off |
| **Power line tool active** | **on** (auto) | **on** |
| User toggle Overlays → Power | on | on |
| Minimap | **off** (alert blink only) | off |

---

## 2. Line states

| State | Stroke | Pattern | α | Notes |
|:---|:---|:---|:---:|:---|
| **Live** | class color §voltage | solid | 100% | committed graph edge |
| **Preview** | class color | dash 4/4 | 60% | draw-in-progress |
| **Damaged** | `warn` `#e9c46a` | dash 3/3 | 90% | spark glyph ◆ at break |
| **Destroyed** | `danger` `#ff4444` | gap | 80% | **×** at break node |
| **Enemy-owned** | class color | dash 6/2 | 100% | faction tint on outer 1px |
| **Island (unpowered)** | `#4a7878` muted | solid | 40% | consumers in subgraph dim |

**Class colors/weights:** voltage picker §3 — do not invent new gold hues per mode.

### 2.1 Preview invalid segment

Red hatch overlay on tile strip + stroke `danger` @ 80% — paired with strip `blocked: {reason}`.

---

## 3. Node glyphs

| Node | Map | Hover card (P1) |
|:---|:---|:---|
| Transformer | ▣ + coil | `{class} · {load}% · {n} consumers` |
| Substation | ▣ yard ring | `Feeds {n} consumers` |
| Power plant | building + ⚡ | `{MW} out` |
| Junction tee | ● | `{n} edges` |

**Snap highlight:** valid anchor `accent_gold` ring 2px while drawing.

---

## 4. Island highlight

When `UtilityGraph` split detected:

| Channel | Read |
|:---|:---|
| **Map** | Unpowered subgraph → **island** stroke style · **gold boundary** on cut edges |
| **Consumers** | `offline` badge on industrial buildings in island |
| **Toast** | `Power island — {n} buildings offline` |
| **Ops strip PWR** | `⚠ Island · {n} offline` |

**Auto-show:** island highlight **on** when alert active (even if user had power overlay off).

---

## 5. Load heat (P1 optional — layout reserved)

| Load % | Visual |
|:---|:---|
| <70% | base weight |
| 70–90% | +1px weight |
| >90% | `warn` pulse 1Hz on segment |

Not P0 — witness field `load_heat_enabled: false` default.

---

## 6. Legend (Overlays menu)

Extend infra legend:

```text
Infrastructure
  ─── Road    ╍╍╍ Rail
  ··· Distribution   ─── Medium   ═══ Transmission
  ╍╍╍ Damaged   × Destroyed
```

---

## 7. Witness

```json
{
  "power_overlay_auto_on_tool": true,
  "line_state_live": true,
  "line_state_preview_dashed": true,
  "island_highlight_active": false,
  "minimap_power_strokes": false
}
```

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
