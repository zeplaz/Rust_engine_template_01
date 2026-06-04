# DESIGN-M3-REPLAY-PASS-002 — Minimap live replay ring pass `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-M3-REPLAY-PASS-002** |
| **Baseline** | [`minimap_replay_live_ring_visual_v1.md`](minimap_replay_live_ring_visual_v1.md) (DESIGN-REPLAY-LIVE-001) |
| **Extends** | [`minimap_replay_scrub_visual_spec_v1.md`](minimap_replay_scrub_visual_spec_v1.md) (DESIGN-M3-REPLAY-001) |
| **Coder lane** | **REPLAY-RING-LIVE-001** · **M3-UNITS-DEPTH-001** (B-P2) |
| **Plan** | [`plan_replay_ring_exec_001_v1.md`](plan_replay_ring_exec_001_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | `REPLAY-RING-LIVE-001`, `plan_replay_ring_exec_001_v1.md` polish |
| **No Rust** | Live-ring state contract + witness alignment only |

---

## Witness alignment (current)

| Pointer | Value | Meaning |
|:---|:---|:---|
| `debug_runs/minimap_compositor_live.json` → `/replay_scrub_enabled` | `true` | Scrub affordance active |
| `debug_runs/minimap_compositor_live.json` → `/ui_p3_m3_replay_001_green` | `true` | M3 replay baseline green |
| `debug_runs/replay_editor_parity_live.json` → `/replay_ring_len` | `4` | Ring depth ≥ 2 (live/lib seed satisfied) |
| `debug_runs/replay_editor_parity_live.json` → `/replay_parity_001_green` | `true` | Parity spine green |

**Designer note:** `CommittedSimReplayRing` sim-time growth is a **coder** witness target per exec plan; this pass confirms **minimap presentation** is ready when ring depth exists on disk.

---

## Live ring states (minimap margin)

| State | Trigger | Visual |
|:---|:---|:---|
| **Hidden** | `stamps.len() < 2` OR `replay_scrub` mask off | No magenta needle |
| **Ready** | `stamps.len() >= 2` and scrub enabled | 1px magenta needle at `x = 2w/3` |
| **Growing** | ring length increased since last frame | needle position updates; no pulse |
| **Scrubbing** | operator scrubs in replay/editor UI | needle stable; no minimap interaction |
| **Paused** | sim tick paused / ring frozen | needle frozen at last index |

**Rejected:** alpha pulsing on needle; numeric tick labels on minimap; center-map timeline chrome.

---

## Acceptance checklist

1. Scrub column contract from DESIGN-M3-REPLAY-001 unchanged (geometry, color, position).
2. Live ring depth (`replay_ring_len >= 2`) maps to **Ready** state without editor-only UI.
3. Growing/scrubbing/paused states are distinct in spec for coder polish on `paint_replay_scrub`.
4. Does not break `ui_p3_m3_replay_001_green` or `replay_scrub_enabled`.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-05-27 |
