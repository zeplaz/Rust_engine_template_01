# UX-E03 / S7B-DESIGN-003 — Transmission shell alignment (v1)

**Status:** SIGNED (design note) · **2026-05-25**  
**Board:** [`post_stage6_active_todos.md`](post_stage6_active_todos.md) UX-E03 · **S7B-DESIGN-003** in [`stages/stage7_behavioral_plan_v1.md`](stages/stage7_behavioral_plan_v1.md)  
**Code anchors:** [`src/gui/hud/transmission.rs`](../gui/hud/transmission.rs) · [`src/gui/hud/transmission_media.rs`](../gui/hud/transmission_media.rs) · [`src/strategic/comms_contract.rs`](../strategic/comms_contract.rs) · [`src/strategic/strategic_command_queue.rs`](../strategic/strategic_command_queue.rs)

---

## Purpose

One-page alignment so **behavioral comms (Stage 7B)** and the **transmission product shell** do not fork authority or duplicate chrome in `BaseState::Simulation`.

---

## Two comm lanes (do not merge writers)

| Lane | Authority | UI consumer today | Sim visibility |
|------|-----------|-------------------|----------------|
| **Narrative / field reports** | `NarrativeObservationBus` (sim-generated text) | `TransmissionShellState` ingest (`transmission_ingest_narrative_bus_system`) | Queue updates in sim; **no egui window** (PLAY-01) |
| **Strategic dispatch** | `StrategicCommandQueue` + `CommunicationPlane::StrategicCommand` | Stage 7 intel mocks (`stage7_ui_shell.rs`), map overlays (S7B-M3 recon/logistics) | Overlays on map; dispatch log is **editor egui mock only** |

**Rule:** `DispatchMessage` / queue ticks stay in `src/strategic/`; transmission shell **displays** envelopes — it does not enqueue gameplay orders.

---

## Module split (`transmission_media.rs` vs `transmission.rs`)

| Module | Role |
|--------|------|
| **`transmission_media.rs`** | Provider **kind** registry (`StaticText`, `StaticImage`, `FakeVideoFrames`, `TextTicker`) — no decode, no GPU upload |
| **`transmission.rs`** | Shell state: queue, severity, channel filter, `TransmissionMediaProvider` **frame** holder, egui drawer |

`TransmissionMediaProviderRegistry` (media kinds) is exported from `hud/mod.rs` for future BQ-124 wiring; the live shell uses `transmission::TransmissionMediaProvider` until BQ-126 binds kinds → frames.

---

## Editor vs simulation

| Session | Product egui shell | Transmission widget | Ingest / sim logic |
|---------|-------------------|---------------------|-------------------|
| **Editor** (`BaseState::Editor`, not WorldGen) | `product_egui_shell_active` → `draw_transmission_shell_egui` | Gated by `WidgetPresentationPolicy::transmission_enabled` (default **off**) | Narrative ingest runs (`in_simulation_or_editor`) |
| **Simulation** | **Off** — Bevy chrome only ([`ui_gates.rs`](../gui/ui_gates.rs)) | Policy forces widget off; layout collapsed on `OnEnter(Simulation)` | Queue may still fill; player sees comms via **map overlays / ops strip**, not floating egui |

**Do not** re-enable product transmission egui in sim without a **Bevy-native** surface (toast, ops strip slot, or docked Bevy panel) and an explicit `ScaffoldContract` if interim.

---

## Stage 7B behavioral relation

| Milestone | Relates to transmission how |
|-----------|----------------------------|
| **S7B-M2** (dispatch delay) | Proves `StrategicCommandQueue` timing — **not** wired into transmission queue yet |
| **S7B-M3** (recon/logistics overlays) | Map/readability — parallel to narrative transmission, not a replacement |
| **Future S7B-M4+** | Optional bridge: `DispatchEnvelope` → `TransmissionEvent` with `TransmissionChannelId::Command` when `transmission_enabled` or Bevy comms chrome exists |

Witness for behavioral spine: `debug_runs/stage7_behavioral_live.json` — independent of UX-E03.

---

## Recommended implementation order (coder, post-note)

1. **Policy / PLAY:** Keep `transmission_enabled: false` in sim; enable in editor only when playtesting comms chrome.
2. **BQ-126:** GPU texture upload path for `TransmissionMediaFrame` — still no decode in egui repaint loops.
3. **Bridge (optional):** Read-only adapter `StrategicCommandQueue` recent → `TransmissionShellState::enqueue` for editor briefing QA.
4. **Sim comms UX:** Bevy ops-strip or minimap-adjacent glyph (see S7B design worksheet: ghost contact / orders-pending) — **not** egui product shell.

---

## Out of scope (UX-E03)

- Campaign scripting, video decode authority, or duplicate mission writers  
- Closing S7B-M2/M3 gates (already on `stage7_behavioral_live.json`)  
- Enabling transmission egui in `BaseState::Simulation`

---

## Sign-off

| Role | Action |
|------|--------|
| **Designer** | This note satisfies **UX-E03** / **S7B-DESIGN-003** |
| **Coder** | No mandatory change; optional BQ-126 / bridge only when queued |
| **Steward** | No new witness field — proof is this doc + existing `transmission_media.rs` stub |

**Version:** v1.0.0 (2026-05-25)
