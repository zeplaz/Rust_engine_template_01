# Minimap Ecology Legend `v1` — DES-MINIMAP-VEG-LEGEND-001

| Field | Value |
|:---|:---|
| **ID** | **DES-MINIMAP-VEG-LEGEND-001** |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Aligns** | [`design_ecology_preview_legend_v1.md`](design_ecology_preview_legend_v1.md) |
| **Territory** | `src/render/minimap_compositor/` · `assets/ui/minimap/` |
| **Verdict** | **PASS** |

```text
DES-MINIMAP-VEG-LEGEND-001 Q✓
Same words as world preview — smaller chips on minimap
```

---

## 1. Topology kind tokens (minimap)

| Glyph | Word | Hex | Minimap chip px |
|:---:|:---|:---|:---:|
| `N` | Network | `#4a6fa5` | 10 |
| `C` | Corridor | `#7a6a4a` | 10 |
| `P` | Patch | `#3d8b5f` | 10 |
| `R` | Ring | `#6a5a8a` | 10 |
| `K` | Cluster | `#2f7d4a` | 10 |
| `F` | Fringe | `#8a9a6a` | 10 |

**Rule:** glyph + word in legend strip; tint on swatch only — not color-alone.

---

## 2. Layout

```text
┌─ minimap (corner) ─────────┐
│ [map composite]            │
│ ┌ legend (collapsed ▶) ──┐ │
│ │ N Network  P Patch …   │ │  ← expanded: 2×3 grid max
│ └────────────────────────┘ │
└────────────────────────────┘
```

| Mode | Default |
|:---|:---|
| Simulation | legend **collapsed** (▶ chip count) |
| WorldGen / editor preview | legend **expanded** first session |

---

## 3. Off / empty states

| State | Copy |
|:---|:---|
| Ecology layer off | `○ Ecology off` |
| No topology data | `○ No landscape data in view` |
| Stale / virtualized | `◐ Map updating…` |

---

## 4. Acceptance

- Words match ecology preview legend exactly
- ≥3 kinds distinguishable at default minimap scale OR legend expanded
- No LG-5 / `topology_graph` in visible strings

---

## 5. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |
