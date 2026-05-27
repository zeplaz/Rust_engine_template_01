# DESIGN-REPLAY-LIVE-001 — minimap replay live ring + scrub UX `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-REPLAY-LIVE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **DEFER** (live ring growth states require runtime evidence; base scrub affordance is already green) |
| **Unblocks** | `REPLAY-LIVE-RING-001` (coder B3/B4) |
| **Witness (base scrub)** | `debug_runs/minimap_compositor_live.json` → `/replay_scrub_enabled` and `/ui_p3_m3_replay_001_green` |
| **Do not break** | `/ui_p3_m3_replay_001_green` |

---
## Purpose
When the simulation replay ring is live (not only editor/lib seed), the minimap must show a scrub affordance as the ring grows.

This extends the existing replay scrub design (`DESIGN-M3-REPLAY-001`) with additional **runtime states**.

---
## Ring runtime states (minimap)
| State | Trigger | Visual requirement |
|:---|:---|:---|
| **Empty ring** | ring length too small | scrub indicator hidden or neutral |
| **Growing** | ring length increasing | scrub indicator present; indicates “current progress index” |
| **Scrubbing** | operator holds scrub intent | scrub indicator visible and stable; no jitter |
| **Paused** | growth not advancing | ring indicator remains; no alpha pulsing |

---
## Acceptance checklist (designer)
1. Scrub affordance appears when ring grows live in Simulation.
2. States (empty/growing/scrubbing/paused) are visually distinct but do not require editor UI.
3. Geometry/position remains consistent with the margin scrub column contract.

