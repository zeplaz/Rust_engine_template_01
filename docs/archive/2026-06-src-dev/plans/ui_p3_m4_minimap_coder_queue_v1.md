# UI-P3-M4-001 — minimap design M3 (FoW + EW) `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **UI-P3-M4-001** |
| **Not** | **UI-P3-M3-001** (M2 construction + ecology — `ui_p3_m3_green`) |
| **Design gate** | **MINIMAP-DESIGN-M3-001** / **D-MINIMAP-M3** — **SIGNED** |
| **Owner** | `@coder` |
| **Witness** | [`debug_runs/minimap_compositor_live.json`](../debug_runs/minimap_compositor_live.json) |

---

## Naming (read first)

[`ui_phase3_minimap_track_naming_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/ui_phase3_minimap_track_naming_v1.md)

| ID | Design phase | Witness |
|:---|:---|:---|
| **UI-P3-M3-001** | M2 construction + ecology | `ui_p3_m3_green` |
| **UI-P3-M4-001** | M3 fog + EW (+ units/replay in follow-on slices) | `ui_p3_m4_green` |

---

## Scope (this slice)

| Channel | Spec | Code |
|:---|:---|:---|
| **M3-01** FoW veil | [`minimap_m3_operational_overlay_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md) | `MinimapOverlayMask.fow`, `fow_tex`, WGSL mix |
| **M3-02** EW stress | same | `MinimapOverlayMask.ew`, `ew_tex`, WGSL tint |

**Out of scope here:** **UI-P3-M3-UNITS-001**, **UI-P3-M3-REPLAY-001** (separate slices).

---

## Copy-paste (@coder)

```
Lane: UI-P3-M4-001 — design M3 fog/EW (NOT UI-P3-M3-001)
Read: ui_p3_m4_minimap_coder_queue_v1.md + minimap_m3_operational_overlay_spec_v1.md
First: MinimapOverlayMask.fow/ew → composite.rs upload → minimap_composite.wgsl
Data: MinimapOperationalSnapshot + seed_minimap_m3_fow_ew_witness (sim + --test visual)
Exit: minimap_compositor_live.json → fow_enabled, ew_overlay_enabled, ui_p3_m4_green: true
Test: cargo test -p proc_A_dine01 --lib minimap_compositor stage5
```

---

## Acceptance

| Field | Pass |
|:---|:---:|
| `fow_enabled` | `true` |
| `ew_overlay_enabled` | `true` |
| `fow_rows` | `> 0` |
| `ew_rows` | `> 0` |
| `ui_p3_m4_green` | `true` |
| `ui_p3_m3_green` | still `true` (M2 regression) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Initial queue — FoW + EW after design sign-off |
