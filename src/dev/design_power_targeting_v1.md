# Power targeting UX — cut line & transformer KO `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-TARGETING-001** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track C |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Charter** | [`design_power_line_construction_ux_v1.md`](design_power_line_construction_ux_v1.md) §4.4 · §5 |
| **Overlay** | [`design_power_map_overlay_v1.md`](design_power_map_overlay_v1.md) |
| **Handoff** | COD-POWER-DAMAGE-SEGMENT-001 |
| **Verdict** | **PASS** |

```text
DES-POWER-TARGETING-001 Q✓
Preview matches UtilityGraph cut — never fake island counts
```

---

## 0. Scope

Player (or weapon system) **targets** power infrastructure — UX must preview **graph truth** before confirm: how many consumers island, which district darkens.

**Not:** new damage sim — presentation + confirm copy only.

---

## 1. Target modes

| Mode | Entry | Target |
|:---|:---|:---|
| **Cut segment** | Military / sabotage tool on line | Single `PowerLine` edge |
| **Knockout transformer** | Same tool on transformer node | `TransformerComponent` entity |
| **Knockout substation** | Heavy weapon / scripted | Substation node |

---

## 2. Segment cut — preview

| Element | Spec |
|:---|:---|
| Hover | Segment thickens + `danger` outline |
| HP bar | Above midpoint — `{current}/{max}` |
| Preview card | `Cut line → islands {n} consumers` |
| Map | Island overlay §4 pre-visualize (dim subgraph) |
| Confirm | Hold LMB 0.5s or explicit **Confirm cut** button |

### Copy

| Key | Template |
|:---|:---|
| `power.target.cut.preview` | `Cut line → islands {n} consumers` |
| `power.target.cut.confirm` | `✓ Line cut — {n} offline` |
| `power.target.cut.none` | `Cut line → no consumers lost` |

**n** = count from `UtilityGraph` split simulation — **not** estimated.

---

## 3. Transformer knockout — preview

| Element | Spec |
|:---|:---|
| Hover | Transformer pad `danger` ring |
| Preview card | `Knockout → darkens {district_label}` |
| Subtitle | `{n} buildings · {total_load} load` |
| Map | Upstream/downstream subgraph dim per graph direction |

### Copy

| Key | Template |
|:---|:---|
| `power.target.xfmr.preview` | `Knockout transformer → darkens {district}` |
| `power.target.xfmr.detail` | `{n} buildings · {load}% load` |
| `power.target.xfmr.confirm` | `✓ Transformer destroyed — {n} offline` |

**district_label:** human region from site zone or nearest factory cluster name — not entity id.

---

## 4. Post-event feedback

| Event | Channels |
|:---|:---|
| Cut / KO | Toast + ops strip + island overlay |
| Partial damage (not destroyed) | `◐ Line damaged — {hp}%` on segment |
| Friendly fire | Same read — no special case hide |

Sync with [`design_power_map_overlay_v1.md`](design_power_map_overlay_v1.md) damaged/destroyed states.

---

## 5. Cancel & Esc

| Action | Result |
|:---|:---|
| RMB | Clear target hover |
| Esc | Exit targeting mode |
| Invalid target | `○ No power target here` |

---

## 6. Accessibility

| Rule | Spec |
|:---|:---|
| A1 | Island count always in **text** on preview card |
| A2 | HP + state not color-only — numeric + word `damaged` / `destroyed` |
| A3 | Confirm requires explicit action — no misclick single LMB on map |

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
