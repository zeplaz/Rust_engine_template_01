# World Map Preview — designer layout decision `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.2.0` |
| **Date** | 2026-05-24 |
| **Owner** | `@designer` (layout + motion authority) |
| **Status** | **SIGNED** (2026-05-24) — **UI-WP-LAYOUT-001 done** (D-01 shell) |
| **Designer workflow** | [`world_preview_layout_designer_next_steps_v1.md`](world_preview_layout_designer_next_steps_v1.md) |
| **Decision worksheet** | [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md) *(pre-filled recommended column)* |
| **D-01 sign-off** | [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) — shell only |
| **D-02 sign-off** | [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) — **optional** impl |
| **D-WP track** | [`world_preview_d_wp_track_signoff_v1.md`](world_preview_d_wp_track_signoff_v1.md) — D-01…D-12 rollup |
| **Philosophy parent** | [`design_theme.md`](design_theme.md) — post-industrial ecological command archive |
| **Code today** | [`src/gui/editor/world_preview/window.rs`](../../../src/gui/editor/world_preview/window.rs), [`world_gen_ui.rs`](../../../src/gui/editor/world_gen_ui.rs) |
| **Coder queue** | [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) |
| **Mock** | [`assets/ui/world_preview/layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png) (1920×1080) |
| **Visual references** | [`world_preview_visual_references_v1.md`](world_preview_visual_references_v1.md) · [`assets/ui/world_preview/references/capturez/`](../../../assets/ui/world_preview/references/capturez/) |
| **Playbook** | [`tools/orchestrator/agents/ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md) |

---

## 0. Gate (read first)

**Chrome refactor** follows signed **§5** below. **UI-WP-LAYOUT-001** (D-01 shell) is **done** — see [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md); motion (WP-L3) and paper assets (WP-L1) remain deferred.

| Role | May do now | Still deferred |
|:---|:---|:---|
| **@coder** | **UI-WP-LAYOUT-D07** · optional D-02; motion / WP-L4 | D-01 + D-04 **done** ([`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md)) |
| **@designer** | WP-L1 paper assets, terrain color key (WP-L4) | — |
| **@planner** | Queue slices 002–004 | — |

**Exit (D-01):** [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md) · mock matches [`layout_mock_v1.png`](../../../assets/ui/world_preview/layout_mock_v1.png).

---

## 1. North star — art direction prompt

The World Map Preview is **not** a videogame minimap or cyberpunk HUD. It is a **living archival operations table** — exhausted planners, ecologists, and logisticians administrating an evolving ecological-industrial world under pressure.

### Visual language

| Use | Avoid |
|:---|:---|
| Layered tracing paper, faded Soviet cartography | Glowing gamer HUD, esports readability |
| Ecological field notebook, industrial systems planning board | Hard sci-fi neon, generic RTS chrome |
| Tarkovsky atmosphere, analog plotting systems | Shiny gradients, mobile polish |
| Hand-annotated operational archive | Perfect rectangles, rigid symmetry |
| Translucent paper textures, soft registration offsets | Crisp vector “software windows” |

### Palette (preview chrome — not terrain art)

| Token | Hex / role |
|:---|:---|
| Archive void | `#000000` deep black background |
| Oxidized teal | terrain / hydrology hints |
| Faded cyan | survey grid, cold data |
| Muted chlorophyll | ecology overlays |
| Archival beige | paper sheets |
| Dirty amber | aged ink, warnings |
| Drafting magenta | `#D946EF` — registration marks, wire (**pigment**, not neon glow) |
| Hot pink | **active authority only** — selected route, live alert |
| Warm industrial orange | infrastructure emphasis |
| Faded rust red | stress, damage, fire memory |

Map **terrain** carries its own memory (sediment, flood marks, ash, drought, vegetation ghosting, old fire traces). **Infrastructure** (roads, rails, power) reads **accumulated over decades** — varying opacity, ink density, imperfect alignment.

**Visual direction stills:** [`world_preview_visual_references_v1.md`](world_preview_visual_references_v1.md) — six `capturez` PNGs (wireframe terrain, scanline sun, sketch rain, industrial/organic collision). Guides **WP-L4 map look**, not D-01 shell.

### Emotional target

Without ecological fragility and human touch, the UI reads as **software**. With it, the world reads as **inhabited**. Flower/collage references = memory, decay, ecology — traces entering infrastructure, not literal decoration everywhere.

---

## 2. Current implementation (code snapshot)

Today WorldGen mode runs **two independent floating egui windows**:

```text
┌─ World Generator (520×640) ─┐   ┌─ World Preview (640×480) ─────────────┐
│ params, sliders, generate   │   │ TOP: toolbar (layers, zoom, GPU)      │
│ tuning I/O                  │   ├────────┬──────────────────────────────┤
│ checkbox: show preview        │   │ LEFT   │ CENTRAL map viewport         │
└─────────────────────────────┘   │ 180px  │                              │
                                  │ sidebar│                              │
                                  ├────────┴──────────────────────────────┤
                                  │ BOTTOM: status bar                    │
                                  └───────────────────────────────────────┘
```

| Trait | Current | Conflicts with north star |
|:---|:---|:---|
| Shell | Dual floating `std_floating` windows | Clean floating windows, not pinned archive |
| Frame | `egui::Frame::NONE`, sharp panels | Hard rectangular HUD framing |
| Symmetry | Toolbar top / status bottom / sidebar left — rigid grid | Rigid panel symmetry |
| Accent | Generic CMD / terminal tokens from `UiPalette` | Can read cyberpunk if magenta glows |
| Map inset | `MAP_PANEL_INSET_PX` crisp shrink | Overly crisp vector edges |
| Motion | Instant egui open/resize; scroll snap | No analog drift or layered reveal |

**Presentation-only invariant unchanged:** preview raster must not mutate gameplay state ([`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md)).

---

## 3. Critical design changes (REMOVE / ADD)

### REMOVE (from preview chrome)

- Hard neon boxes and 1px high-contrast esports borders on map frame
- Pure rectangular HUD framing around the map (single perfect rect)
- Rigid panel symmetry (equal gutters, centered toolbars)
- Overly crisp vector panel edges with no material offset
- “Floating clean window” metaphor for primary workspace

### ADD

| Layer | Qualities |
|:---|:---|
| **Terrain memory** | Aged, surveyed, archived, politically shaped, environmentally stressed |
| **Ecological materiality** | Sediment, flood marks, ash bloom, drought discoloration, vegetation ghosting, fire spread traces |
| **Infrastructure layering** | Decades of accumulation — opacity ramps, ink density, misregistration |
| **Organic panel integration** | Pinned, projected, overlaid, archival — not OS windows |
| **Operational negative space** | Atmospheric haze around map; dead zones; imbalance |

---

## 4. Target layout — archival operations table

Designer **must** validate or amend this wireframe before coding. Numbers are **proposals** pending §5.

```text
┌──────────────────────────────────────────────────────────────────────────┐
│ ARCHIVAL HEADER — operational index · epoch · layer legend (asym offset) │
│         faint registration tick (magenta)              torn edge overlay  │
├──────────────────┬───────────────────────────────────────────────────────┤
│ FIELD INDEX      │                                                       │
│ (translucent     │              WORLD MAP — dominant                     │
│  paper stack)    │              layered terrain + infrastructure threads   │
│                  │              ecological scars · survey grid · haze      │
│ · ecology stress │              hand circles / graphite notes (sparse)     │
│ · logistics flow │                                                       │
│ · weather        │         ┌─ optional corner overview (not esports) ─┐   │
│ · population     │         └──────────────────────────────────────────┘   │
│ · construction   │                                                       │
│   queue (peek)   │                                                       │
│                  │                                                       │
├──────────────────┴───────────────────────────────┬───────────────────────┤
│ ANNOTATIONS STRIP — atmospheric reports · queue    │ FIELD NOTES (narrow)  │
│ construction status · operational drift            │ hand margin · drift   │
└────────────────────────────────────────────────────┴───────────────────────┘
```

### Proposed proportions (1080p reference)

| Region | Width / height | Notes |
|:---|:---|:---|
| Header | 100% × 48–64px | Index title + active layer chip; **not** a dense toolbar |
| Field index (left) | 220–280px | Scroll; panels **offset** 4–12px from each other |
| Central map | **≥62%** width | Hero; breathing room top/bottom |
| Field notes (right) | 160–200px | Optional v1; may collapse to bottom strip |
| Annotations strip | 100% × 56–72px | Status + queue; torn top edge |

**Generator params** (sliders, seed, generate): **not** in this wireframe by default — see **D-04**.

---

## 5. Layout decisions — signed (2026-05-24)

**@coder implements only this column.** Worksheet: [`world_preview_layout_decision_worksheet_v1.md`](world_preview_layout_decision_worksheet_v1.md).

| ID | Final | Summary |
|:---|:---:|:---|
| **D-01** | **A** | Single workspace; generator slide sheet · [`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md) |
| **D-02** | **A** | Map ≥ **65%** area · [`world_preview_d02_map_dominance_signoff_v1.md`](world_preview_d02_map_dominance_signoff_v1.md) (impl optional) |
| **D-03** | **A** | Left field index stack |
| **D-04** | **A** | Left slide-over params (dimmed map) |
| **D-05** | **B** | Layer strip on map top edge |
| **D-06** | **A** | Zoom/pan/GPU icons in header margin |
| **D-07** | **A** | Corner overview inset 120–160px |
| **D-08** | **A** | egui custom `Frame` + textures |
| **D-09** | **A** | Fixed offsets: index +8px, notes −6px, header +4px |
| **D-10** | **A** | Registration ticks + selected wire only |
| **D-11** | **B** | Min **12%** map margin |
| **D-12** | **A** | Chrome dissolve **400ms** on enter simulation |

<details>
<summary>Full option matrix (reference)</summary>

| ID | Question | Option A | Option B | Option C |
|:---|:---|:---|:---|:---|
| **D-01** | **Shell model** | **Single workspace** — one window/frame; generator as slide sheet | **Dual window** (keep today) with visual reskin only | **Docked pair** — generator tab attached to preview edge |
| **D-02** | **Map dominance** | Map ≥ **65%** area always | Map ≥ **55%**; more index visible | Full-bleed map; index overlay on hover |
| **D-03** | **Field index placement** | Left stack (§4 wireframe) | Top tabs (ecology / logistics / weather) | Right inspector (inverse asymmetry) |
| **D-04** | **Generator params** | **Left slide-over** sheet over map (dimmed) | Stay separate window (today) | Bottom drawer (engineering tray) |
| **D-05** | **Layer / overlay controls** | Sparse chips in **header** | Thin strip **on map** top edge (tracing overlay) | Stay in sidebar under index |
| **D-06** | **Zoom / pan / GPU** | Icon glyphs in header margin | Foot pedal strip on map corner | Hidden in dev menu only |
| **D-07** | **Overview minimap** | **Corner inset** on map (120–160px) | Keep sidebar thumb | Remove v1; pan only |
| **D-08** | **Panel chrome tech** | **egui** custom `Frame` + textures (fastest) | **Bevy UI** paper sheets + egui map texture | Hybrid: Bevy frame, egui content |
| **D-09** | **Asymmetry rule** | Fixed offsets: index +8px, notes −6px, header +4px | Per-panel random seed within 4–12px | Designer mock defines per-asset offsets |
| **D-10** | **Magenta usage** | Registration ticks + selected wire only | + active layer chip outline | + map survey grid (very faint) |
| **D-11** | **Negative space** | Min **8%** margin around map content | Min **12%** (more archive silence) | Dynamic: grows when no overlay active |
| **D-12** | **Enter simulation** | Chrome **dissolve** 400ms then dismiss | Hard cut (today) | Map zooms to theater then fade |

</details>

---

## 6. Motion & smoothness — functional requirements

Preview must feel **clear and naturally flowing**, not snappy or gamified. Motion supports readability; it never obscures data.

### Principles

1. **Continuity** — panel open/close preserves map camera; no jump on layout reflow.
2. **Layer crossfade** — overlay toggles blend **120–200ms**; no hard frame swap.
3. **Analog pacing** — fades **180–320ms** ease-out; avoid elastic/bounce easing.
4. **Input lag budget** — pan/zoom response **≤1 frame**; decorative motion may lag **2–4 frames**.
5. **Rest state** — when idle, subtle **drift ≤0.5px** optional (projector instability); disable if motion-sickness concern.

### Spec table (implement after SIGNED)

| Interaction | Target | Easing | Notes |
|:---|:---|:---|:---|
| Open workspace | 280ms fade + 8px vertical settle | ease-out cubic | Map already visible underneath if reskin |
| Slide generator sheet (D-04 A) | 240ms horizontal; map dim 40% | ease-out | Focus trap in sheet |
| Layer toggle | 160ms opacity crossfade | linear → ease-out | Raster may lag; UI state immediate |
| Panel scroll | smooth scroll (egui) | default | No snap paging |
| Zoom wheel | incremental; optional inertia off v1 | — | Match [`camera_map_navigation_runbook_v1.md`](../camera_map_navigation_runbook_v1.md) |
| Select index row | 120ms highlight wash | ease-out | Magenta at 25% alpha, not glow |
| Enter simulation (D-12 A) | 400ms dissolve chrome | ease-in-out | `dismiss_world_gen_preview_chrome` after fade |

### Forbidden

- Esports snap tweens, overshoot bounce, neon pulse loops on idle chrome
- Full-screen flash on layer change
- Symmetric slide animations that feel like mobile drawer apps

---

## 7. Panel materiality (presentation tokens)

Extend [`palette_v2_tokens.md`](palette_v2_tokens.md) with **preview-only** tokens when signed:

| Token | Role | Implementation hint |
|:---|:---|:---|
| `paper_sheet_fill` | Panel background | beige @ 12–18% alpha over `#000` |
| `paper_edge_wear` | Torn edge overlay | 9-slice or static alpha strip |
| `registration_magenta` | Corner ticks | `#D946EF` @ 60% alpha, **no bloom** |
| `graphite_annotation` | Hand notes | `#888` @ 40% |
| `infra_thread` | Logistics lines | dirty amber @ 50%, 1px jitter |
| `ecology_stain` | Panel edge intrusion | chlorophyll noise @ 8% |

**Do not** add glow shaders to egui panels v1 — materiality from **texture + alpha + offset**.

---

## 8. Map presentation (designer → coder handoff)

Separate from **chrome layout** but must align aesthetically:

| Topic | Designer delivers | Coder wires |
|:---|:---|:---|
| Base terrain read | Default layer + color key mock · refs **04**, **01** | `color_presets.rs` / registry colors |
| Infrastructure threads | Line weight + opacity scale · refs **01**, **05** | overlay pass (future U7 compositing) |
| Ecological scars | Reference stills · refs **02**, **06** | raster tint masks (phase 2) |
| Survey grid | Spacing + fade curve · ref **01** | optional shader/overlay |
| Hand annotations | Sketch / rain texture · refs **02**, **06** | decorative egui painter layer |
| Analog sun / bloom | Stipple + scanlines · refs **03**, **04** | map backdrop presentation-only |

**Blocked:** swapping programmer tints for full art without signed color key ([`composite_style_worldgen_v1.md`](../../designer_questions/terrain_world/composite_style_worldgen_v1.md)).

---

## 9. Information flow (operator journey)

Natural flow the layout must support:

```text
Open WorldGen
  → see map context immediately (even if low-res)
  → adjust params in sheet (D-04) without losing map framing
  → toggle ecology / logistics overlays in header (D-05)
  → read stress in field index without covering map center
  → generate → preview refines in place (no window jump)
  → commit to simulation → chrome dissolves (D-12)
```

**Friction to eliminate:**

- Hunting between two windows for layer vs params
- Map resize reflow that resets zoom/center
- Status bar duplication (generator progress vs preview status)

---

## 10. Migration phases (after sign-off)

| Phase | Owner | Deliverable |
|:---|:---|:---|
| **WP-L0** | Designer | Signed §5 + reference mock PNG (1920×1080) |
| **WP-L1** | Designer | Paper frame assets + registration marks |
| **WP-L2** | Coder | Shell refactor per D-01/D-08 (`window.rs`, optional `world_gen_ui.rs`) |
| **WP-L3** | Coder | Motion table §6 |
| **WP-L4** | Designer | Terrain / overlay color key |
| **WP-L5** | Coder | Map presentation passes §8 |

---

## 11. Designer sign-off checklist

| # | Item | Done |
|:---|:---|:---:|
| 1 | §5 decisions **D-01…D-12** recorded with chosen column | ☑ |
| 2 | Wireframe mock committed **`assets/ui/world_preview/layout_mock_v1.png`** (1920×1080) | ☑ |
| 3 | 1080p annotation: offsets (+8/−6/+4), **12%** map margin (D-11 B) | ☑ |
| 4 | Motion §6 accepted (no amend table) | ☑ |
| 5 | Magenta / hot-pink usage boundary confirmed (§1 palette) | ☑ |
| 6 | Generator ↔ preview flow walkthrough approved (§9) | ☑ |
| 7 | Explicit **REMOVE** list acknowledged for current chrome (§3) | ☑ |

**Verdict:** ☑ **SIGNED** · ☐ **CONDITIONAL** · ☐ **DRAFT**

| Role | Date | Verdict | Notes |
|:---|:---|:---|:---|
| Designer | 2026-05-24 | **SIGNED** | Recommended defaults; mock committed |
| Coder | 2026-05-24 | **Done** | **UI-WP-LAYOUT-001** — D-01 shell ([`world_preview_d01_shell_signoff_v1.md`](world_preview_d01_shell_signoff_v1.md)) |

---

## 12. `@coder` handoff — **UI-WP-LAYOUT-001 active**

**Queue:** [`ui_world_preview_coder_queue_v1.md`](ui_world_preview_coder_queue_v1.md)

```
Lane: UI-WP-LAYOUT-001
Read: world_map_preview_layout_decision_v1.md (signed §5) + layout_mock_v1.png
Playbook: ui_layout_agent.md + world_preview_runbook_v1.md
First: D-01 single workspace shell only (max 3 files)
Verify: F8 WorldGen → one workspace; camera stable on resize
Do NOT: motion §6, D-09 offsets, paper textures, raster graph
```

Likely touch (after sign-off):

| File | Change |
|:---|:---|
| `src/gui/editor/world_preview/window.rs` | Layout panels per D-03, D-07, D-09 |
| `src/gui/editor/world_gen_ui.rs` | D-04 sheet vs window |
| `src/gui/std_floating.rs` / new `archive_frame.rs` | Paper frame helpers |
| `src/gui/style/palette.rs` | Preview tokens §7 |

---

## 13. Cross-links

| Doc | Role |
|:---|:---|
| [`design_theme.md`](design_theme.md) | Global UI philosophy + motion language |
| [`world_preview_runbook_v1.md`](../world_preview_runbook_v1.md) | Raster/GPU pipeline (orthogonal to chrome) |
| [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md) | egui vs Bevy during `AppState::WorldGen` |
| [`composite_style_worldgen_v1.md`](../../designer_questions/terrain_world/composite_style_worldgen_v1.md) | Layer semantics |
| [`world_gen_chrome_contract.rs`](../../../src/gui/world_gen_chrome_contract.rs) | Exclusive chrome during WorldGen |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.2.0 | 2026-05-24 | **SIGNED**; mock committed; UI-WP-LAYOUT-001 unblocked |
| v1.1.0 | 2026-05-24 | Designer next steps + one-page worksheet; D-06/D-10 in recommended set |
| v1.0.0 | 2026-05-24 | Initial layout gate; art direction + §5 decisions + motion spec |
