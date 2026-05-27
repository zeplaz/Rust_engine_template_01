# Minimap unit aggregation markers — visual spec `v1` (DESIGN-M3-UNITS-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-M3-UNITS-001** |
| **Coder queue** | **UI-P3-M3-UNITS-001** (Coder B wave 3 **#5**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Parent** | [`minimap_m3_operational_overlay_spec_v1.md`](../prompts/guides/ui/minimap_m3_operational_overlay_spec_v1.md) § M3-03 |
| **Design gate** | [`minimap_d_m3_signoff_v1.md`](minimap_d_m3_signoff_v1.md) |
| **Data** | [`MinimapOperationalSnapshot::unit_markers`](../render/visual_domain_snapshots.rs) |
| **Compositor** | [`composite.rs`](../render/minimap_compositor/composite.rs) `paint_unit_markers` |
| **Witness** | `debug_runs/minimap_compositor_live.json` → `unit_marker_rows`, `ui_p3_m3_units_001_green` |

---

## Purpose

Define how **friendly force aggregation** reads on the minimap at **strategic / operational** zoom — intel ticks, not tactical unit sprites.

**Not:** per-entity icons, faction portraits, or main-map unit meshes.

---

## Visual intent

| Principle | Spec |
|:---|:---|
| **Read** | “Where is mass?” — cluster centroids, not individual soldiers |
| **Contrast** | Visible on logistics amber + ecology green + FoW gray; must not mimic EW denial |
| **Density** | Sparse — overcrowding defeats minimap purpose |
| **Toggle** | `MinimapOverlayMask.units` — default **on** in sim ([`simulation_minimap_overlay_defaults`](../gui/minimap_shell.rs)) |

---

## Glyph spec

| Token | Value | Notes |
|:---|:---|:---|
| **Shape** | 2×2 px **square** (v1) or 3×2 **chevron** (v2 polish) | Center on chunk coord mapped to texel |
| **Color** | Palette `label_muted` **#a8b0c0** @ **80%** | Do not use `accent` or alarm red |
| **Channel** | Written to **EW heat buffer** green channel bump | Keeps single compositor pass; distinct from EW amber (`ew` uses different weight curve) |
| **Alpha equivalent** | `+200` saturating add on `ew_out[base+1]` (current impl) | Designer accepts until dedicated units pass exists |

### Density tiers (zoom / extent)

| Tier | Visible minimap extent | Max markers | Aggregation rule |
|:---|:---|:---:|:---|
| **Strategic** | Full minimap | **8** | One marker per occupied chunk; merge same-chunk stacks |
| **Operational** | Full minimap | **8** | Same cap — prefer highest-strength cluster |
| **Tactical** | N/A on minimap | **0** | Units read on main map / ops strip, not minimap clutter |

**Hard cap (code):** `M3_UNIT_MARKER_CAP = 8` — do not raise without performance review.

---

## Layer interaction

```text
… → fog-of-war veil → EW stress (corridors) → unit markers (M3-03) → replay scrub (M3-04)
```

| Underlay | Interaction |
|:---|:---|
| **Logistics heat** | Markers sit **above** corridor heat |
| **EW denial** | Markers must remain distinguishable — use cool gray glyph, not amber |
| **FoW unexplored** | Markers **only** on explored chunks (sim supplies coords; coder filters veil==0) |
| **Fire heat** | Off by default in sim — if on, markers unchanged |

---

## Data contract (coder)

| Field | Type | Source (target) |
|:---|:---|:---|
| `unit_markers` | `Vec<(u32, u32)>` | Chunk coords (texel space mod w/h) |
| Witness | `unit_marker_rows > 0` | `ui_p3_m3_units_001_green` when `units_heat_enabled` |

**Seed (witness / lib):** `seed_minimap_m3_units_replay_witness` — 6 sample coords; acceptable for green, not product-final aggregation.

**Product (future):** logistics / strategic unit snapshot reader — must respect cap and explored mask.

---

## Acceptance (playtest)

| # | Pass | Fail |
|:---:|:---|:---|
| 1 | ≤8 ticks visible at default sim zoom | Full grid of dots |
| 2 | Ticks readable on logistics + FoW | Lost in EW amber wash |
| 3 | Toggle **Units** off → markers gone | Ghost markers remain |
| 4 | `ui_p3_m3_units_001_green: true` after witness refresh | JSON green with `unit_marker_rows: 0` |

```powershell
cargo test -p proc_A_dine01 --lib ui_p3_m3_units
cargo test -p proc_A_dine01 --lib minimap_compositor
```

---

## Coder handoff — UI-P3-M3-UNITS-001

```
Lane: UI-P3-M3-UNITS-001
Read: src/dev/minimap_unit_marker_visual_spec_v1.md
Touch: composite.rs, visual_domain_snapshots.rs (≤3 files)
Do: paint_unit_markers per spec; real reader when available
Do NOT: new ECS extract; tactical sprites on minimap
Exit: unit_marker_rows > 0 · ui_p3_m3_units_001_green
```

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **SIGNED** |
| Coder | — | Witness green on disk — polish / real reader optional |

**On-disk (2026-05-26):** `unit_marker_rows: 6`, `ui_p3_m3_units_001_green: true` in [`debug_runs/minimap_compositor_live.json`](../../debug_runs/minimap_compositor_live.json).

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-M3-UNITS-001** SIGNED |
