# WSS readability impact `v1` (WSS-DESIGN-GATE-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **WSS-DESIGN-GATE-001** |
| **Deliverable** | 2 of 4 — player-facing readability |
| **Parent brief** | [`wssr_design_gate_brief_v1.md`](wssr_design_gate_brief_v1.md) |
| **Theme anchor** | [`design_theme.md`](../prompts/guides/ui/design_theme.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |

---

## Summary

WSS changes **what is authoritative** under the map, not the player’s need for **legible command surfaces**. Readability policy: tactical band shows full material detail; strategic band shows **compressed state** (heat ribbons, water bands, haze envelopes); minimap shows **operational glyphs** only; construction and world preview must not inherit substrate debug density.

**Global rule:** L3 embellishment may simplify at zoom; **L1 sim clipmaps still advance** — cull affects draw, not truth.

---

## Surface matrix

| Surface | WSS touch | Readability risk | Mitigation spec |
|:---|:---|:---|:---|
| **Tactical map** | Fire/smoke/dust/contamination overlays from substrate snapshot + existing extract | Clutter from overlapping alpha fields; contamination confused with fog; smoke obscures logistics routes | **Z-order:** terrain → hydrology tint → contamination (pattern) → smoke haze → fire sparks → construction ghosts → UI. **Alpha budget:** smoke Layer B max 0.45 ground haze tactical; sparks unchanged (D-F07). **Contamination:** never color-only — see migration contract § Contamination. **Logistics:** preserve routing congestion overlay legibility — contamination alpha capped when congestion active. |
| **Minimap** | Heat-only fire; hydrology dim ribbon; L2/L3 atmosphere sample | Fire sparks on minimap (forbidden); ocean shimmer noise; toxic plume unreadable at 64px | **Fire:** heat channel only — no ECS fire query, no Hanabi (charter + WSS-PLAN-004 § edge cases). **Water:** single desaturated ribbon from `HydrologyVisualExtract.strategic_ribbon` — no W2 particle draw. **Atmosphere:** visibility + toxic hazard as **two-pixel-wide** edge tint max. **D-F09 parity:** same strategic cull thresholds as tactical sparks. |
| **World preview** | Must not pull full L0 clipmap into preview raster | Preview becomes weather debug; performance hit; breaks D-01 single workspace calm | **No substrate bleed:** world preview pipeline reads **representation summary** only — biome/height/hydrology tint, not live advecting smoke. **Explicit gate:** `world_preview_pipeline_enabled` excludes `AtmosphereRenderClipmap` upload. Preview may show **static** coast/river from gen hydrate, not runtime flood animation. |
| **Construction ghosts** | Unchanged authority — parametric placement separate lane | Accidental overlay of contamination/smoke under ghost footprint; weighted occupation confused with ghost alpha | **Authority unchanged:** [`construction_invariants.md`](construction_invariants.md) — ghosts disposable, sim owns weighted occupation when parametric lands. **WSS rule:** construction footprint validation reads **tile occupation book**, not GPU field. **Visual:** ghost outline **above** smoke haze; contamination pattern **below** ghost fill. No WSS writer in `src/construction/`. |
| **Strategic zoom** | D-F09 / D-W09 cull policy; L3 particle draw suppressed | Witness greens forced by disabling cull; strategic map looks empty OR tactical noise at zoom-out | **Preserve cull:** sparks fade `zoom_alpha < 0.35`; water particles reduce per D-W09 A; **field haze** may remain as low-frequency color shift (not particles). **Witness policy:** zero spark rows at strategic zoom is **PASS**, not failure. **Sim:** clipmap L2/L3 continues; render uploads throttled. |

---

## Tactical map — detail spec

### Fire

| Element | Readability rule |
|:---|:---|
| Sparks | Pinpoint legacy vocabulary; density cap D-F07; fade D-F09 |
| Heat | Ground tint from thermal slab sample — subtle, behind sparks |
| Smoke | Layer B haze — **partial alpha**; column readability from Layer A density sample, not particle spam |

### Smoke / dust

| Layer | Tactical read |
|:---|:---|
| Layer A (sim) | Drives hazard badges + AI visibility sample — may be invisible directly |
| Layer B (GPU) | Ground haze + column billboards; **dust** uses same ash_density channel with warmer desaturate |

**Dust storms:** strategic overlay haze OK; tactical must not reduce terrain material read below 60% luminance contrast.

### Contamination

| Type | Tactical read |
|:---|:---|
| Airborne plume | Amber-olive haze + **crosshatch** edge (not green/red alone) |
| Soil | Stipple pattern at tile corner — persistent stain |
| Waterborne | Teal-darkening + flow-aligned streak (ties hydrology ribbon) |
| Radiation | Magenta drafting ink **dashed** contour — accessibility pair with pattern |

### Hydrology (runtime)

| State | Tactical read |
|:---|:---|
| River | Directional shader motion (W1 signed) — narrower strip |
| Flood | Depth tint pulse + optional foam hints — must not mirror contamination color |
| Ocean | Deep band + coast foam line — slab-backed |

---

## Minimap — detail spec

```text
Compositor input priority (bottom → top):
  terrain base (muted)
  hydrology ribbon (dim, 40% alpha max)
  fire heat (R channel, no particles)
  routing congestion (existing M3)
  unit markers (M3)
  fog of war / EW (M4)
```

**Forbidden on minimap:** Hanabi draws, spark particles, L0 smoke upload, construction ghost tiles (use tray badge only).

---

## World preview — detail spec

| Allowed | Forbidden |
|:---|:---|
| Static height/biome | Live smoke advection |
| Gen-time river/coast | Runtime flood depth animation |
| Archive paper chrome ([`design_theme.md`](../prompts/guides/ui/design_theme.md)) | Full contamination fields |
| Scenario summary labels | Debug clipmap heatmaps |

---

## Construction — detail spec

WSS substrate work **must not** change:

- Tool → intent → preview → validate → execute funnel
- Weighted/parametric footprint authority (future) in `src/construction/`
- Ghost partial-alpha for scaled placement (future) — distinct from contamination alpha

**Coordination:** hydrology `HydrologyDirtyReason::ConstructionComplete` triggers deep solve **after** commit event — ghost preview never writes `water_depth`.

---

## Strategic zoom — D-F09 / D-W09 preservation

| Decision ID | Policy | WSS interaction |
|:---|:---|:---|
| **D-F09 A** | Sparks fade when zoomed out | Unchanged — extract cull before GPU |
| **D-W09 A** | Water particles reduce; color band remains | Hydrology slab drives band; particles L3 only |
| **Field haze** | May persist softly | Render clipmap decimated upload — not full L0 |

**Designer reject:** disabling cull globally to green `particle_routing.*_rows` witnesses.

---

## Accessibility

| Concern | Mitigation |
|:---|:---|
| Color-only hazard | Contamination uses pattern + icon glyph in tile info labels (F7 toggle) |
| Smoke vs fog | Smoke warmer + vertical bias; fog cooler + uniform |
| Red/green toxicity | Pair with hatch density and tooltip numeric hazard |
| Motion sensitivity | Strategic zoom reduces motion (particles off); shader ripple amplitude capped |

---

## Witness / playtest checklist

| Check | Pass criteria |
|:---|:---|
| Tactical spark rows | > 0 at default tactical zoom |
| Strategic spark rows | 0 by design |
| Water tactical | W1 motion visible on river fixture |
| Water strategic | Band visible, particles culled |
| Minimap | No spark draw call |
| Construction | Ghost valid/invalid unchanged with atmosphere running |
| Preview | No smoke column in world preview capture |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **SIGNED** | 2026-05-26 |
