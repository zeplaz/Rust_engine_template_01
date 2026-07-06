# Zoom band fire read + placement crosshair UX `v1` (DESIGN-ZOOM-FIRE-READ-001)

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-ZOOM-FIRE-READ-001** · parent **⟨PLAN-PRODUCT-POLISH-001⟩** |
| **Date** | 2026-06-11 |
| **Owner** | `@designer` (charter) · `@coder` (P1 pick / P2 fire wire) |
| **Verdict** | **PASS** (charter) |
| **Unblocks** | **⟨TRIAGE-FIRE-PRODUCT-001⟩** UX review · operator G-PLAY acceptance |
| **Prereq** | [`fire_lod_player_read_v1.md`](../docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md) (FIRE7-DESIGN-001 **SIGNED**) |
| **Witness** | [`debug_runs/design_zoom_fire_read_live.json`](../debug_runs/design_zoom_fire_read_live.json) |

**No Rust in this doc.** Player-read policy + placement debug UX contract only.

---

## Mission

Players must answer two questions at every zoom level without F3:

1. **Where is fire spreading?** — heat vs sparks vs smoke must match zoom intent, not engineering cull defaults.
2. **Where will this building land?** — construction ghost must track the cursor; when alignment probes are **yellow** (qualified but not green), engineers and operators need a shared read model.

**Acceptance test:** *At default operational play zoom (`zoom_alpha ≈ 0.42`), a player spots fire fronts with sparse sparks — not a flat heat sheet. In construction mode with probes green, white cursor and ghost footprint share one tile center.*

---

## Authority model (presentation)

```text
FireSimulationSnapshot        → sim truth (heat, fuel)
FireVisualFrameSet            → extract → FireVisualFramesByView
WorldFireParticleFrame        → GPU sparks / embers / smoke garnish
Chunk heat compositor bin     → always-on strategic tint (CPU)
ViewProjectionAuthority       → camera + viewport
SimMapProjectionFrame         → pick / ghost projection contract
ConstructionPlacementDebugProbe → engineer overlay only (not player HUD)
```

**Rules:**

- Fire presentation reads **extract + particle policy** — never infer sim correctness from draw counts alone ([`design_fire_overlay_debug_v1.md`](design_fire_overlay_debug_v1.md)).
- Placement pick uses **one** projection spine — camera authoritative when `SimMapProjectionFrame::camera_authoritative` ([`09-sim-map-projection-placement.md`](../.cursor/skills/bevy-simulation-grade/09-sim-map-projection-placement.md)).
- Minimap **never** upgrades to tactical sparks regardless of main-map zoom (FIRE7 per-view exception).

---

## 1. Fire — zoom_alpha bands (player read)

Normalized `zoom_alpha` from [`map_zoom_alpha`](../src/gui/map_camera.rs) on `MAP_ZOOM_CLAMP`. Policy constants in [`gpu_particles.rs`](../src/render/gpu_particles.rs).

| Band label | `zoom_alpha` range | `WorldLodBand` | **Player should see** | Heat (CPU chunk tint) | GPU sparks | Smoke / ember garnish |
|:---|:---|:---|:---|:---|:---|:---|
| **Far strategic** | `< 0.28` | `Macro` · `Strategic` | **Regions on fire** — where, not how | Full warm blobs | **None** (hard cull) | None |
| **Strategic edge** | `0.28 – 0.35` | `Strategic` | Same as far — blobs only | Full | **None** | Faint smoke column at hottest peaks only |
| **Operational entry** | `0.35 – 0.42` | `Operational` | **Fronts forming** — blob + first pinpricks | Full | **Sparse** (floor @ `0.42 × 0.35`) | Low ember wisps at cluster centroids |
| **Operational play** | `0.42 – 0.58` | `Operational` | **Readable fronts** — glow + sparse sparks | Full | **Sparse → medium** scatter ramp | Smoke columns at active cells |
| **Tactical** | `0.58 – 0.85` | `LocalTactical` | **Spread direction obvious** — wind-aligned streaks | Reduced dominance | **Medium → full** scatter | Ember + smoke above heat |
| **Tactical proof / cinematic** | `≥ 0.85` | `LocalTactical` / debug | **Pinpoint shower** on active front | Background only | **Full** scatter (`fire_spark_011_green`) | Full smoke stack |

**Designer rule:** If the player can only answer “something is orange” at operational play zoom, the band is **too abstract** — wire sparse sparks before raising heat saturation.

**Default play anchor:** `FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA = 0.42` — product must be **passable** here, not only at `--test visual` tactical lock (`0.85`).

### 1a. Channel priority by band (what wins visually)

| Band | Dominant read | Secondary | Tertiary | Forbidden |
|:---|:---|:---|:---|:---|
| Strategic | Chunk **heat tint** | — | — | Spark soup on minimap |
| Operational | Heat **blob shape** | **Sparse sparks** at centroids | Smoke wisps | Screen-filling particles |
| Tactical | **Spark motion** + ember | Smoke column | Heat underlay | Flat heat sheet with zero depth |
| Cinematic | Per-cell flame/smoke mix | Spark shower | Debug overlay optional | Identical to tactical with no added detail |

### 1a. Heat vs sparks vs smoke — decision tree

```text
zoom_alpha < 0.28 ?
  yes → heat blob only
  no → WorldLodBand Strategic ?
    yes → heat + optional faint smoke peak garnish
    no → zoom_alpha < 0.42 ?
      yes → heat + sparse sparks (operational floor)
      no → zoom_alpha < 0.58 ?
        yes → heat + ramping sparks + smoke columns
        no → full tactical stack (sparks lead, smoke above, heat underlay)
```

### 1b. Relation to FIRE7 table

This doc **narrows** [`fire_lod_player_read_v1.md`](../docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md) with **numeric zoom_alpha gates** already in code. Coder wires `fire_spark_hard_zoom_culled` / `fire_spark_scatter_zoom_alpha` per **⟨TRIAGE-FIRE-PRODUCT-001⟩** — do not invent parallel LOD tables.

| FIRE7 row | Code anchor | This doc emphasis |
|:---|:---|:---|
| Strategic heat blobs | `FIRE_SPARK_MIN_ZOOM_ALPHA` (0.28) | No sparks below 0.28 |
| Operational sparse sparks | `FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA` (0.42) | **Default play** must pass here |
| Tactical instances + sparks | `FIRE_SPARK_FULL_SCATTER_ZOOM_ALPHA` (0.58) | Scatter ramp completes |
| Cinematic / proof | `FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA` (0.85) | Harness witness only — not required for G-PLAY |

### 1c. Minimap + ops strip (player surfaces)

| Surface | Fire read | Rule |
|:---|:---|:---|
| **Minimap** | Heat blobs only | `fire_heat` default **off** in sim — operator enables in tray |
| **Ops strip ALERTS** | Fire channel text | Never duplicate WX line ([`design_weather_player_read_v1.md`](../docs/archive/2026-06-src-dev/plans/design_weather_player_read_v1.md)) |
| **WorldMain** | Band table §1 | Zoom drives channel mix |
| **F3 fire rows** | Engineer witness | Not player-critical path |

### 1d. Fire vs weather smoke (hue separation)

| Phenomenon | Hue | Motion | When visible |
|:---|:---|:---|:---|
| **Fire smoke** | Warm amber / orange | Directional plume, localized | Operational+ when `FireLodBand ≥ SmokeOnly` |
| **Weather fog** | Cool gray-blue | Uniform veil | WX overlay — not fire channel |
| **Chunk heat** | Orange-red flat tint | Static | All bands — reduce alpha at tactical so sparks lead |

---

## 2. Construction — crosshair / ghost UX

### 2a. Debug crosshair vocabulary (engineer / `--test vfx`)

Canonical colors from [`placement_debug.rs`](../src/construction/placement_debug.rs):

| Color | Meaning | Trust when… |
|:---|:---|:---|
| **White** | OS cursor logical position | Always — ground truth for pointer |
| **Magenta** | Ghost center via **live camera** unproject | `camera_authoritative=true` + probes green |
| **Green** | Ghost center via **egui manual** projection | Fallback path; must match magenta when green |
| **Cyan** | Pick world reprojected to screen | Camera path self-consistency check |
| **Orange dots** | Footprint tiles via camera | Per-tile camera path |
| **Blue dots** | Footprint tiles via egui fallback | Per-tile manual path |

**Legend string (locked):** `White=cursor · Magenta=ghost via live camera · Green=ghost via egui math · Cyan=pick world reprojected`

### 2b. Probe tiers — green / yellow / red

Metrics from `ConstructionPlacementDebugProbe` ([`09-sim-map-projection-placement.md`](../.cursor/skills/bevy-simulation-grade/09-sim-map-projection-placement.md)):

| Tier | `Pick Δ world` (cam vs manual) | `Ghost screen Δ` (cam vs egui) | `Pick roundtrip screen Δ` | Panel color | Player impact |
|:---|:---:|:---:|:---:|:---|:---|
| **🟢 Green** | `< 1.0` | `< 4 px` | `< 4 px` | White labels | Ghost aligned — **commit allowed** if validity passes |
| **🟡 Yellow** | `1.0 – 3.0` | `4 – 12 px` | `4 – 12 px` | **Yellow** label on Pick Δ | Ghost **may lag one tile** under zoom scroll — show invalid hatch if Δ blocks commit |
| **🔴 Red** | `> 3.0` | `> 12 px` | `> 12 px` | ⚠ MISALIGN | **No commit** — ghost is diagnostic fiction |

**Yellow UX charter (probe qualified, product degraded):**

1. **Player ghost** follows the **authoritative pick path** only (`cursor_world_xy_rendered`) — never average white+magenta+green.
2. **Do not** show debug crosshairs in player HUD — engineer overlay only (`CONSTRUCTION_PLACEMENT_DEBUG=1` / `--test vfx`).
3. When yellow: footprint uses **invalid hatch** ([`footprint_invalid_color`](../src/construction/ghost_visual.rs)) if `allows_commit=false`; valid terrain but misaligned projection → **pending** tint, not green valid.
4. **Scroll zoom during placement:** expect transient yellow for **≤ 1 frame** after band change (ortho commits immediate, pan may lerp). If yellow persists **> 3 frames**, treat as P1 bug — not acceptable steady state.
5. **GPU footprint path active** (red panel warning): player sees bottom-left blob artifacts — force **egui-only** footprint for product; GPU path is debug/diagnostic.

### 2c. Zoom scroll + placement (spatial trust)

| Event | Expected player read | Probe expectation |
|:---|:---|:---|
| Scroll zoom in (one notch) | Ghost stays on same **world tile** | Transient yellow ≤ 1 frame OK |
| Pan while zooming | Ghost tracks cursor tile | Green within 1 frame after pan stops |
| Viewport heal (hole → full window) | No tile jump | Pick Δ must return green within 1 frame |
| Minimap click recenter | Ghost re-anchors to cursor | Green after camera settle |

### 2d. Production ghost color language (no debug crosshairs)

| `allows_commit` | Validity | Ghost read |
|:---|:---|:---|
| `true` | Good | Valid green footprint + edge outline |
| `true` | Overlap / terrain | Invalid hatch per [`ghost_visual.rs`](../src/construction/ghost_visual.rs) |
| `false` | any | Pending / blocked — **no** faux-valid green |

**Rule:** Probe yellow with `allows_commit=true` is **engineer-only contradiction** — file triage; player must not see green valid ghost when magenta≠green.

---

## 3. Zoom band summary — fire + construction together

| `zoom_alpha` | Fire player read | Construction overlay density | Footprint line weight |
|:---|:---|:---|:---|
| `< 0.28` | Heat blobs | Hide lane markings | 1 px — strategic |
| `0.28 – 0.42` | Heat + sparse sparks | Show district footprint grid | 1.5 px |
| `0.42 – 0.58` | Fronts + sparks | Full footprint + connections | 2 px |
| `0.58 – 0.85` | Tactical sparks lead | Full precision + lane marks | 2.5 px |
| `≥ 0.85` | Cinematic shower | Max precision | 3 px (cap — no scale blow-up) |

**Overlay line weight** scales with **screen-stable** thickness — not world units that shrink to invisible at far zoom ([`designer.md`](../.cursor/agents/designer.md) §Camera + Zoom).

---

## 4. Accessibility

| # | Requirement |
|:---:|:---|
| A1 | Fire at operational zoom: **motion** (spark drift) + **warm hue** — not color-only heat blob |
| A2 | Fire vs fog vs contamination: distinct **words** in any player label (SMOKE alert ≠ WX `f`) |
| A3 | Construction validity: **hatch pattern** for invalid — not red-only |
| A4 | Minimap fire: optional heat wash — default off; legend when enabled |
| A5 | Debug crosshairs: **never** the only placement affordance in product builds |

---

## 5. Acceptance probes (operator + agent)

| Probe | Threshold | Command / witness |
|:---|:---|:---|
| Operational sparks | `fire_spark_rows > 0` @ `zoom_alpha ≈ 0.42` | `cargo run -p proc_A_dine01 --release -- --test vfx` |
| Tactical sparks | Visible front, not heat-only blob | Mid-tactical scroll · [`fire_lod_player_read_v1.md`](../docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md) Operational row |
| Pick Δ world | `< 1` | `CONSTRUCTION_PLACEMENT_DEBUG=1` |
| Ghost screen Δ | `< 4 px` | same |
| Zoom ghost | ≤ 1 frame double-world after scroll | 5× zoom scroll · target `map_zoom_coherence_live.json` |
| Crosshair alignment | White ∩ Magenta ∩ Green on same tile | Construction mode, probes green |

---

## 6. Coder handoff

### P2 — Fire product (`⟨TRIAGE-FIRE-PRODUCT-001⟩`)

```text
Read:  src/dev/design_zoom_fire_read_v1.md
       docs/archive/2026-06-src-dev/plans/fire_lod_player_read_v1.md
Touch: gpu_particles.rs (policy fns) · tile_world_fallback.rs (default play zoom)
Do:    sparse sparks readable @ FIRE_SPARK_OPERATIONAL_PLAY_ZOOM_ALPHA (0.42)
Do NOT: second fire extract · minimap tactical particles
Verify: cargo test -p proc_A_dine01 --lib stage5 fire_ecology
Witness: stage5_full_app_live.json fire_spark_rows @ operational zoom
```

### P1 — MAP-PICK (`⟨TRIAGE-MAP-PICK-CLOSURE-001⟩`)

```text
Read:  src/dev/design_zoom_fire_read_v1.md §2
       .cursor/skills/bevy-simulation-grade/09-sim-map-projection-placement.md
Touch: map_egui_projection.rs · placement_debug.rs (probe thresholds only if needed)
Do:    green probes per §2b; transient yellow ≤ 1 frame on zoom
Do NOT: player-facing debug crosshairs · second placement writer
Verify: --test vfx + CONSTRUCTION_PLACEMENT_DEBUG=1
```

---

## 7. Non-goals

- New particle sim in UI layer
- Player-visible debug crosshairs in release builds
- MCP art / tile bake changes
- Stage 5 gate reopen
- VM-06…11 full infrastructure hardening (triage backlog only)

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| `@designer` | 2026-06-11 | **PASS** — charter on disk |
| `@coder` | — | Pending P1 + P2 wire |
| Operator | — | Pending §5 probes |

```text
DESIGN-ZOOM-FIRE-READ-001 complete
Verdict: PASS
Doc: src/dev/design_zoom_fire_read_v1.md
Unblocks: TRIAGE-FIRE-PRODUCT-001 UX review
ΔWF→@coder ⟨TRIAGE-MAP-PICK-CLOSURE-001⟩ ⚡P0 then ⟨TRIAGE-FIRE-PRODUCT-001⟩
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-11 | Initial PASS — zoom fire read + crosshair probe tiers |

---

## 2026-07-06 correction (FIRE-VIS-001)

The bands in §1 were authored against `zoom_alpha` normalized on the **old fixed span**
`MAP_ZOOM_CLAMP ≈ (0.35, 4.5)`. Per-world zoom limits ([`map_zoom_limits_for_world`](../src/gui/tactical/map_camera.rs))
later made `zoom_alpha` world-size-relative (`hi` derived from `viewport / 8 tiles`), so on a
320-world `hi ≈ 90` — the old alpha thresholds (0.10/0.28/0.42/0.58/0.85) became unreachable at
normal play zoom (alpha ≈ 0.10 corresponded to zoom ≈ 9, never hit in practice). This silently
hard-culled sparks, embers, smoke, and heat-blob seeding.

**Fix:** fire visibility gates are re-keyed onto **px-per-tile** — the camera's raw scale
(`ExtractedCameraMetrics::zoom_level`, equal to px-per-world-unit; tiles are 1 world unit) — which
is stable regardless of world size or zoom-limit policy.

| Old (`zoom_alpha`, fixed-span authored) | New (`px-per-tile`) | Constant |
|:---|:---|:---|
| `0.10` (hard cull floor) | `1.5` | `FIRE_SPARK_MIN_PX_PER_TILE` |
| `0.42` (operational play anchor) | `2.5` | `FIRE_SPARK_OPERATIONAL_PLAY_PX_PER_TILE` |
| `0.58` (full scatter) | `4.0` | `FIRE_SPARK_FULL_SCATTER_PX_PER_TILE` |
| `0.85` (tactical proof / cinematic) | *unchanged* — stays on `zoom_alpha` axis (proof-lock harness only, not the cull path) | `FIRE_SPARK_TACTICAL_PROOF_ZOOM_ALPHA` |

At fit zoom (~2.07 px/tile for a 320-world @ 1280×720) sparks/embers/smoke are now visible when
fire burns; full scatter is reached at ~4 px/tile. Constants live in
[`src/render/fire_vfx/witness.rs`](../src/render/fire_vfx/witness.rs).
