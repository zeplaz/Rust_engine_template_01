# Minimap replay scrub tick — visual spec `v1` (DESIGN-M3-REPLAY-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-M3-REPLAY-001** |
| **Coder queue** | **UI-P3-M3-REPLAY-001** (Coder B wave 3 **#6**) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Parent** | [`minimap_m3_operational_overlay_spec_v1.md`](../docs/archive/2026-06-prompts-guides/ui_phases/guides/ui/minimap_m3_operational_overlay_spec_v1.md) § M3-04 |
| **Replay data** | [`CommittedSimReplayRing`](../systems/sim_frame_delta.rs) |
| **Compositor** | [`composite.rs`](../render/minimap_compositor/composite.rs) `paint_replay_scrub` |
| **Witness** | `replay_scrub_enabled`, `ui_p3_m3_replay_001_green` |
| **Parity** | [`replay_editor_parity_live.json`](../../debug_runs/replay_editor_parity_live.json) — orthogonal gate |

---

## Purpose

When **sim replay / timeline** has depth, show a **single vertical scrub index** on the minimap margin so operators see “where we are in history” without opening the editor replay panel.

**Not:** full timeline UI, tick labels, or scrub interaction on minimap (v1).

---

## Visual intent

| Principle | Spec |
|:---|:---|
| **Read** | Thin **time needle** on minimap — margin column, not map center |
| **When** | `CommittedSimReplayRing::stamps.len() >= 2` |
| **Toggle** | `MinimapOverlayMask.replay_scrub` — default **on** in sim |
| **Color** | Registration **magenta** `#c040c0` @ **40%** — distinct from EW amber and unit gray |

---

## Glyph spec

| Token | Value | Notes |
|:---|:---|:---|
| **Geometry** | **1 px** vertical line, full minimap height | |
| **Horizontal position** | **Right third** — `x = (w * 2 / 3)` clamped to valid texel | Avoids left chrome / dock |
| **Channel** | FoW buffer luminance bump `+102` (v1 impl) | Future: dedicated replay pass or magenta RGBA |
| **Interaction** | None in v1 | Scrub control stays in replay/editor systems |

### States

| State | Minimap |
|:---|:---|
| **Inactive** | No ring or `< 2` stamps — no line |
| **Active** | Vertical tick visible |
| **Replay disabled** | Toggle off — no line regardless of ring |

---

## Layer stack

```text
… → unit markers (M3-03) → replay scrub tick (M3-04)  [top of M3 stack]
```

| Rule | Spec |
|:---|:---|
| **Z-order** | Scrub draws **after** units — always visible |
| **FoW** | Line visible through light veil (needle is UI chrome, not terrain truth) |
| **Multiview** | Minimap-only — no scrub on World Preview / editor map |

---

## Data contract (coder)

| Input | Condition | Witness |
|:---|:---|:---|
| `CommittedSimReplayRing` | `stamps.len() >= 2` | `replay_scrub_enabled: true` |
| `MinimapOverlayMask.replay_scrub` | `true` | Required for paint |
| Rollup | — | `ui_p3_m3_replay_001_green` |

**Seed (lib):** ring capacity ≥8, ≥2 stamps committed in test harness — acceptable for witness.

**Product:** runtime ring from sim commits; must not set green without real stamp growth in sim session.

---

## Relationship to replay parity

| Lane | Scope |
|:---|:---|
| **REPLAY-PARITY-001** | Deterministic replay + editor parity |
| **UI-P3-M3-REPLAY-001** | **Presentation only** — minimap tick when ring has depth |

Designer accepts **witness green** before full **REPLAY-PARITY-001** close; scrub must **hide** when ring empty.

---

## Acceptance (playtest)

| # | Pass | Fail |
|:---:|:---|:---|
| 1 | Vertical tick at ~⅔ width when replay active | No indicator with 10+ stamps |
| 2 | Tick absent when replay off or `<2` stamps | Permanent line in fresh sim |
| 3 | Toggle **Replay scrub** off → line gone | Line remains |
| 4 | `ui_p3_m3_replay_001_green: true` when active | Hand-edited JSON only |

```powershell
cargo test -p proc_A_dine01 --lib ui_p3_m3_replay
cargo test -p proc_A_dine01 --lib minimap_compositor
```

---

## Coder handoff — UI-P3-M3-REPLAY-001

```
Lane: UI-P3-M3-REPLAY-001
Read: docs/archive/2026-06-src-dev/plans/minimap_replay_scrub_visual_spec_v1.md
Touch: composite.rs, live_proof.rs, sim_frame_delta hook (≤3 files)
Do: paint_replay_scrub per spec; wire ring in sim OnEnter if needed
Do NOT: minimap click-to-scrub; second replay UI
Exit: replay_scrub_enabled · ui_p3_m3_replay_001_green
```

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-26 | **SIGNED** |
| Coder | — | Witness green on disk — interaction polish deferred |

**On-disk (2026-05-26):** `replay_scrub_enabled: true`, `ui_p3_m3_replay_001_green: true` in [`debug_runs/minimap_compositor_live.json`](../../debug_runs/minimap_compositor_live.json).

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | **DESIGN-M3-REPLAY-001** SIGNED |
