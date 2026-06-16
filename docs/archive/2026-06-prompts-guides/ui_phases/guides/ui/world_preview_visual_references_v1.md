# World Map Preview — visual references `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` |
| **Authority** | [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) (SIGNED) |
| **Source archive** | `C:\dev\razerz-master\Images_captures_&old_data\capturez` (authoritative on disk; **repo copies** below for team) |

These captures are **direction**, not pixel-perfect targets. They define **map read**, **atmosphere**, and **material language** for World Map Preview — complementary to [`layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) (chrome layout).

---

## Repo copies (committed)

| File | Original capture | Role |
|:---|:---|:---|
| [`ref_01_terrain_wireframe_grid.png`](../../../assets/ui/world_preview/references/capturez/ref_01_terrain_wireframe_grid.png) | `wisave_…1105` | **Map hero** — gridded terrain wireframe, rainbow infrastructure read, central asset |
| [`ref_02_sketch_rain_glitch.png`](../../../assets/ui/world_preview/references/capturez/ref_02_sketch_rain_glitch.png) | `v92_…3208` | **Ecological fragility** — hand sketch + rain hatching + CRT glitch band |
| [`ref_03_sun_chromatic_wireframe.png`](../../../assets/ui/world_preview/references/capturez/ref_03_sun_chromatic_wireframe.png) | `imagterz_…3620` | **Analog light source** — stippled sun, scanline bloom, chromatic wire peaks |
| [`ref_04_lowpoly_terrain_scanlines.png`](../../../assets/ui/world_preview/references/capturez/ref_04_lowpoly_terrain_scanlines.png) | `savtrrinz_…2333` | **Terrain memory** — low-poly height field, olive void, horizontal scan sun |
| [`ref_05_industrial_branch_atmosphere.png`](../../../assets/ui/world_preview/references/capturez/ref_05_industrial_branch_atmosphere.png) | `sssss…3737` | **Infrastructure + ecology collision** — ship/hull wireframe + branch + debris |
| [`ref_06_organic_tree_glitch_void.png`](../../../assets/ui/world_preview/references/capturez/ref_06_organic_tree_glitch_void.png) | `esaaaa…3158` | **Void + organic signal** — tree-in-glitch oval, diagonal rain shards, layered fold |

**Do not commit:** `.tga` / `.avi` from source folder (too large; PNG extracts sufficient).

---

## What to borrow (design → code)

### Map presentation (WP-L4 / raster aesthetic)

| From captures | Apply to preview map | Avoid |
|:---|:---|:---|
| Low-poly / wireframe terrain read (01, 04) | Height + biome as **faceted** or thin-line overlay; not smooth gradient RTS | Glossy satellite photo |
| Rainbow / chromatic line accents (01, 03, 05) | Infrastructure threads, logistics edges — **muted** cyan/magenta/green lines | Full-spectrum neon HUD |
| Olive / charcoal void (03, 04, 05) | `#000` archive void + oxidized olive **map surround** | Pure grey editor background |
| Stippled / scanline sun (03, 04) | Weather / pressure bloom, **not** UI button glow | Lens flare, esports bloom |
| Grid + tile rhythm (01) | Survey grid, chunk boundaries, faded registration | Bright CS:GO grid |

### Chrome & panels (WP-L1 / layout mock)

| From captures | Apply to UI chrome | Avoid |
|:---|:---|:---|
| Hand hatching + rain (02, 06) | Field notes margin, annotation texture | Literal weather particles on every panel |
| Diagonal fold / layer cut (02, 06) | Paper stack offset (D-09 A) | Perfect aligned rectangles |
| Sparse white line sketch (02, 05, 06) | Graphite annotation strokes | Icon font glyphs |
| Glitch band horizontal (02, 06) | Optional **idle** projector flicker (§6 motion) | Constant full-screen glitch |

### Emotional north star (all)

> Exhausted planners reading a **damaged but legible** archive — technology failing gracefully, ecology interrupting geometry.

Captures **05** and **06** are the strongest “inhabited world” references (industrial mass vs organic branch/tree).

---

## Capture → layout mock mapping

| Layout region ([`layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png)) | Primary reference | Secondary |
|:---|:---|:---|
| Central **WORLD MAP** | **01**, **04** | 03 (distant peaks as overlay) |
| **Field notes** (right) | **02** | 06 (hand + rain texture) |
| **Ecology stress** index | **06** | 05 (branch intrusion) |
| **Infrastructure** layer strip | **01**, **05** | 03 (chromatic lines) |
| **Atmospheric** header / strip | **03**, **04** | 02 (glitch band sparingly) |
| Map **negative space** / void | **04**, **06** | — |

---

## Coder guidance (when implementing map look)

**UI-WP-LAYOUT-001 (D-01 shell):** use **layout mock only** — do not port capture shaders yet.

**WP-L4 (map presentation):**

1. Start from **04** for default terrain faceting + scanline sun behind map (presentation-only).
2. Add infrastructure overlay language from **01** (thin colored lines, accumulated opacity).
3. Field-note painter decals from **02** / **06** at panel edges only.
4. Cap glitch effects: **≤2s** idle cycle, **≤15%** opacity (§6 motion — no esports pulse).

**Palette bridge:**

| Capture read | Token ([`palette_v2_tokens.md`](palette_v2_tokens.md) / §1 authority) |
|:---|:---|
| Olive void | extend: `archive_olive_void` ~ `#3a3a28` |
| Stippled sun | `dirty_amber` + `warm industrial orange` |
| Chromatic lines | `faded cyan`, restrained `ink_magenta`, muted chlorophyll |
| White sketch | `graphite_annotation` @ 40% |

---

## Designer checklist (references)

- [x] Six PNG direction stills copied to repo
- [ ] Optional: annotate refs with arrows (map vs chrome vs motion) in Figma
- [ ] WP-L4 color key sheet citing ref IDs 01–06
- [ ] Confirm glitch/motion budget with §6 before coder WP-L4

---

## Cross-links

| Doc | Role |
|:---|:---|
| [`design_theme.md`](design_theme.md) | Global philosophy (flower/collage, paper light table) |
| [`world_map_preview_layout_decision_v1.md`](world_map_preview_layout_decision_v1.md) | Signed layout + motion |
| [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) | Slice order |
| [`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md) | Raster pipeline |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Imported `capturez` PNG set; mapped to layout mock + WP-L4 |
