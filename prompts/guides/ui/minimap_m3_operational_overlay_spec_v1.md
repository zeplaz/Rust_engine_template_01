# Minimap M3 — operational overlay spec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **MINIMAP-DESIGN-M3-001** / **D-MINIMAP-M3** |
| **Version** | `0.1.1` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** (2026-05-25) |
| **Sign-off** | [`minimap_d_m3_signoff_v1.md`](../../../src/dev/minimap_d_m3_signoff_v1.md) |
| **Parent** | [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) §7 M3 |

---

## Visual intent

Operational minimap reads as **intel picture**, not terrain wallpaper:

- **FoW:** unexplored = cool gray veil @ 50–60% over terrain
- **EW:** amber/red edge stress on corridors under denial (subtle, not alarm red)
- **Units:** 2–4px cluster ticks, max N per chunk at strategic zoom
- **Replay scrub:** vertical tick marks on minimap margin when timeline active

**Mock (v1):** extend simulation HUD minimap capture or commit `assets/ui/minimap/m3_operational_target_v1.png` when available.

---

## Layer stack (compositor order)

```text
terrain / fallback
  → fire heat (M1)
  → logistics heat (M2)
  → construction heat (M2)
  → ecology heat (M2)
  → fog-of-war veil (M3-01)
  → EW stress (M3-02)
  → unit aggregation markers (M3-03)
  → replay scrub ticks (M3-04)
```

---

## Per-channel spec

### M3-01 Fog-of-war

| Token | Value |
|:---|:---|
| Unexplored | `#1a1a1a` @ 55% over composite |
| Explored | no extra veil |
| Toggle | `MinimapOverlayMask.fow` |

### M3-02 EW

| Token | Value |
|:---|:---|
| Denial | dirty amber `#e8c03a` @ 25% along corridor centerline |
| Source | transport / overlay matrix EW bit |
| Toggle | `MinimapOverlayMask.ew` |

### M3-03 Unit aggregation

| Token | Value |
|:---|:---|
| Glyph | 2px square or chevron, `label_muted` @ 80% |
| Cap | 8 markers per visible minimap extent at strategic zoom |
| Toggle | `MinimapOverlayMask.units` |

**Coder spec (SIGNED):** [`minimap_unit_marker_visual_spec_v1.md`](../../../src/dev/minimap_unit_marker_visual_spec_v1.md) — **DESIGN-M3-UNITS-001**

### M3-04 Replay scrub

| Token | Value |
|:---|:---|
| Tick | 1px vertical line, registration magenta @ 40% |
| When | replay timeline resource active |
| Toggle | `MinimapOverlayMask.replay_scrub` |

**Coder spec (SIGNED):** [`minimap_replay_scrub_visual_spec_v1.md`](../../../src/dev/minimap_replay_scrub_visual_spec_v1.md) — **DESIGN-M3-REPLAY-001**

---

## §11 Sign-off

| # | Item | Done |
|:---|:---|:---:|
| 1 | Channels M3-01…M3-04 defined | ☑ draft |
| 2 | Compositor order agreed | ☑ |
| 3 | Tokens aligned with palette v2 | ☑ draft |
| 4 | Target PNG or HUD capture | ☐ **optional** — [`assets/ui/minimap/m3_operational_target_v1.png`](../../../assets/ui/minimap/m3_operational_target_v1.png) |
| 5 | Designer **SIGNED** | ☑ |

**Done when (MINIMAP-DESIGN-M3-001):** this doc **SIGNED** + [`minimap_d_m3_signoff_v1.md`](../../../src/dev/minimap_d_m3_signoff_v1.md) design gate — **complete 2026-05-25**. Unblocks **UI-P3-M4-001**.

---

## @designer — MINIMAP-DESIGN-M3-001 (complete)

**Status:** **SIGNED** — no further gate work unless adding optional mock PNG.

| Step | Action |
|:---:|:---|
| 1 | Confirm M3-01…M3-04 tokens in § Per-channel spec (FoW / EW / units / replay) |
| 2 | Confirm compositor stack order in § Layer stack |
| 3 | *(Optional)* Capture sim minimap with FoW+EW mocked → `assets/ui/minimap/m3_operational_target_v1.png` |
| 4 | Acknowledge [`minimap_d_m3_signoff_v1.md`](../../../src/dev/minimap_d_m3_signoff_v1.md) §11 |

**Unblocks:** `@coder` **UI-P3-M4-001** (FoW + EW in `minimap_composite.wgsl`).

```text
@designer MINIMAP-DESIGN-M3-001 — DONE (SIGNED 2026-05-25)
Read: prompts/guides/ui/minimap_m3_operational_overlay_spec_v1.md
      src/dev/minimap_d_m3_signoff_v1.md
Optional: assets/ui/minimap/m3_operational_target_v1.png
Coder: UI-P3-M4-001 unblocked — fog + EW compositor passes
```

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v0.1.0 | 2026-05-24 | DRAFT for D-MINIMAP-M3 coder unblocks |
| v0.1.1 | 2026-05-25 | **SIGNED** — MINIMAP-DESIGN-M3-001 done; PNG deferred |
