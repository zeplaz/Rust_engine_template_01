# Sim HUD minimap chrome `v2` — legend dock + veg/fire tokens

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-MINIMAP-002** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 3 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_minimap_veg_legend_wire_v1.md`](design_minimap_veg_legend_wire_v1.md) |
| **Handoff** | CDR-B-VEG-MINIMAP-LEGEND-UI-001 |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-MINIMAP-002 Q✓
Minimap legend dock — not tint-only · burn scar tokens
```

---

## 1. Chrome layout

```text
┌ Minimap ──────────────┐
│ [raster]              │
│ ─────────────────     │
│ Legend ▸ (peek 48px)  │
│ ● Canopy  ▲ Scar      │
└───────────────────────┘
```

| State | Height |
|:---|:---|
| Collapsed | raster only |
| Peek | +48px legend strip |
| Pinned | legend scroll max 96px |

---

## 2. Tokens (extends wire v1)

| Token | Glyph | Label |
|:---|:---:|:---|
| Clean canopy | `●` | `Canopy` |
| Burn scar | `▲` | `Burn scar` |
| Recovery | `◐` | `Recovery` |
| Logistics heat | `░` | `Logistics` (optional mask) |

---

## 3. Interaction

| Input | Behaviour |
|:---|:---|
| Click legend header | toggle peek/pinned |
| Hover scar on map | highlight matching legend row |
| Sim enter | legend **peek** if ecology overlay on |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
