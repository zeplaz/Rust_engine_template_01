# Experience layer — UX / HUD designer brief `v1`

> **STATUS:** Locked product input for **GPU minimap**, **transmission shell**, **construction blueprint UX**, and **command/overlay shell** — **no Rust**. Rendering and interaction **contracts** land after Stage-5 EXIT; full strategic AI is **not** a prerequisite.

Version: `v1.0.0`  
**Parent:** [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md) · [`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md)  
**Companions:** [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md), [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md), [`stage7_behavioral_world_designer_brief_v1.md`](stage7_behavioral_world_designer_brief_v1.md) §8 (**UX-D**), [`scenario_campaign_scripted_tools_runbook_v1.md`](scenario_campaign_scripted_tools_runbook_v1.md)  
**Backlog:** [`rulebook_backlog_designer_brief_v1.md`](rulebook_backlog_designer_brief_v1.md) §4 (**BQ-119+**)

---

## 1. Gates and parallel UX tracks

**After Stage-5 EXIT:**

- unified representation + VT gates stable  
- preview parity stable  
- GPU draw authoritative  

**Then parallel UX tracks (do not wait for full strategic AI):**

| Track | Scope |
|:---|:---|
| **UX-A** | Minimap / overlay shell |
| **UX-B** | Media / transmission widget |
| **UX-C** | Construction & blueprint UX |
| **UX-D** | Command shell / overlays / intel timeline |

**Later:** campaign scripting + mission-driven transmissions.

**Architectural rule:** minimap, construction preview, transmission widget, overlays, and strategic shell **must not** invent separate world extraction paths. All consume **`WorldRepresentationResolver` → representation frames → overlay buffers → GPU/UI presentation** — same rule as the visual spine ([`base_visual_world_representation_v1.md`](base_visual_world_representation_v1.md) §2–3).

---

## 2. Minimap overhaul (UX-A)

**Problem:** egui texture presentation is acceptable for **tooling**; it is not the long-term tactical minimap shell. Overlay interaction, zoom, drag, and GPU compositing become expensive and awkward as a paint-heavy widget.

**Direction:** split into **world overlay producers → `SharedOverlayFieldBuffers` → GPU minimap compositor → `RenderTarget::Image`**.

**Presentation modes:**

- **A)** embedded HUD minimap  
- **B)** fullscreen strategic map  
- **C)** detached floating window  

**egui role only:** host controls, toggle overlays, dock/fold windows, **display the texture** — not per-frame world raster ownership.

### 2.1 Phases

| Phase | Features |
|:---|:---|
| **M1 — foundation** | GPU minimap render target; pan/zoom; overlay toggles; chunk streaming visualization; click-to-focus camera; detachable window |
| **M2 — strategic shell** | fog-of-war overlays; logistics heat; EW coverage; rail/pipeline overlays; construction overlays; unit aggregation markers; replay scrub markers |
| **M3 — operational shell** | theater command overlays; intel confidence; command latency visualization; corridor/path plans; mission zones |

### 2.2 Compositor inputs (no duplicate paths)

Terrain field + fire field + logistics + recon + construction + units + markers → minimap composite texture.

Consumers: **`FireVisualFrame`**, **`SharedOverlayFieldBuffers`**, domain projection outputs — not parallel minimap-only extracts.

### 2.3 Core resource (contract sketch)

```rust
#[derive(Resource)]
pub struct MinimapState {
    pub visible: bool,
    pub detached: bool,
    pub zoom: f32,
    pub world_center: Vec2,
    pub overlays: OverlayMask,
    pub mode: MinimapMode,
}
```

---

## 3. Transmission / media widget (UX-B)

**Role:** campaign/intel layer — not “play a video” only. Shell for briefings, propaganda/news feed, intercepted comms, emergency alerts, mission updates. Fits strategic atmosphere and comms planes ([`stage7_behavioral_world_designer_brief_v1.md`](stage7_behavioral_world_designer_brief_v1.md)).

**Separate concerns:** transmission **logic** · video/audio **playback** · HUD **shell** · **campaign scripting**. Do **not** hardwire playback into egui repaint loops.

### 3.1 Widget states

`OFF` · `MINIMIZED` · `SMALL LIVE WINDOW` · `EXPANDED` · `FULLSCREEN`

### 3.2 v1 features

Movable widget; hide/show; persistent corner icon; animated “signal active”; queued transmissions; subtitles; image/video/audio playback; pause/replay recent transmission.

### 3.3 Later

Channel knob; faction channels; interrupted/degraded signals; static/noise; picture corruption from EW; emergency override broadcasts.

**Thematic channels (examples):** civilian news, command net, field reports, logistics, emergency alerts, enemy intercepts — aligned with comms planes and belief/intel separation.

### 3.4 Render path

Video frame decode → GPU texture upload → HUD material quad (`RenderTarget::Image`, custom material, draggable HUD anchor; egui for **controls only**).

### 3.5 Core resource (contract sketch)

```rust
#[derive(Resource)]
pub struct TransmissionState {
    pub active: bool,
    pub minimized: bool,
    pub current_channel: ChannelId,
    pub queue: VecDeque<TransmissionEvent>,
    pub current: Option<ActiveTransmission>,
}
```

---

## 4. Construction + blueprint UX (UX-C)

First-class **construction domain** — placement clarity, feedback, reversibility, planning flow. Pairs with [`infrastructure_construction_runbook_v1.md`](infrastructure_construction_runbook_v1.md) sim phases; this brief owns **player-facing** flow.

**Model:**

```text
Player intent
    → Construction planner
    → Blueprint entity
    → Validation pipeline
    → Approval/commit
    → Construction simulation
```

Not: click instantly spawns building.

### 4.1 Phases

| Phase | Scope |
|:---|:---|
| **CSTR-1 — placement foundation** | Ghost placement; rotate; valid/invalid feedback; terrain conformity; collision checks; cost preview; cancel/revert |
| **CSTR-2 — approval workflow** | Place blueprint → pending queue → approve → resources assigned → construction starts |
| **CSTR-3 — strategic construction** | Delayed approval; workforce; supply dependency; corridor upgrades; phased construction; sabotage; blueprint templates |

### 4.2 Placement UX

Select tool → ghost follows cursor → overlays show terrain, logistics access, power/water, threat, cost, footprint, adjacency.

| Input | Action |
|:---|:---|
| Left click | place ghost |
| Shift+click | queue multiple |
| Right click | cancel |
| Enter | approve all pending |
| Checkbox panel | approve/reject individually |

### 4.3 ECS separation

**`BlueprintGhost`** · **`PendingConstruction`** · **`ConstructionProgress`** / **`ConstructionSite`** · **`CompletedStructure`** — do not mutate one entity through all states.

Construction overlays integrate **`WorldRepresentationResolver`**: FULL → scaffold meshes + workers; MID → construction zones; FAR → strategic build markers.

---

## 5. Command shell (UX-D)

Build order (product): overlay toggles → command tray → intel timeline → command table — see [`stage7_behavioral_world_designer_brief_v1.md`](stage7_behavioral_world_designer_brief_v1.md) §8. Map-primary posture: [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md).

---

## 6. Engineering roadmap (UX-1–UX-6)

| ID | Deliverable |
|:---|:---|
| **UX-1** | GPU minimap compositor |
| **UX-2** | Detached HUD window framework |
| **UX-3** | Transmission / media widget |
| **UX-4** | Construction blueprint system |
| **UX-5** | Overlay interaction shell |
| **UX-6** | Strategic command shell |

**Recommended order:**

1. Stage-5 exit stabilization  
2. GPU minimap compositor (**UX-1**)  
3. Construction blueprint UX (**UX-4**)  
4. Detached HUD framework (**UX-2**)  
5. Transmission widget (**UX-3**)  
6. Strategic overlay shell (**UX-5** / **UX-6**)  
7. Campaign transmission scripting  

---

## 7. Explicit non-goals (this wave)

- egui-owned long-term minimap raster or video frames  
- Instant-build without blueprint/approval path for strategic construction  
- Campaign transmission scripting before transmission shell + decode/upload path exist  
- Per-widget duplicate fire/overlay ECS scans  

---

**Document history:** `v1.0.0` (2026-05-14) — designer lock for UX-A–D and UX-1–6 roadmap.
