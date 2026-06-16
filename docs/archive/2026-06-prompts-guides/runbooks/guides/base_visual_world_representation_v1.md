# World representation & GPU spine — strict execution order (v1)

**Status:** architecture lock-in for `base_visual_dev01` visual program.  
**Companion:** [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md) (cards + tables).  
**Governance:** [`stage5_convergence_directive_v1.md`](stage5_convergence_directive_v1.md) — convergent growth; Stage 5 primary lane, not feature freeze.

This document overrides **only** the sequencing among **theme-3**, **phase-e (E1)**, **VT-4**, **phase-d**, and **phase-f** when they conflict with older “suggestive” tracker ordering. Meta items (`meta-no-temp-side-paths`, `meta-schedule-contracts`, …) may still run in parallel where they do not violate the invariants below.

---

## 1. Dependency-locked order (non-negotiable for this spine)

| Step | Item | Why |
|------|------|-----|
| **1** | **`theme-3-lod-overlay-channel-matrix`** | Without LOD driving **real** GPU/extract cost, you over-extract, lock the wrong instance model, and bake inefficiency into buffers. |
| **2** | **`phase-e-cadence-scale` — E1 contract only first** | Without a committed snapshot cadence, GPU work debugs **phantom** bugs from sim/render timing. |
| **3** | **`visual-test-matrix-upgrade` — VT-4** | Triple agreement (minimap + preview + GPU fire field) is the **truth validator**; skipping it invites duplicate producers again. |
| **4** | **`phase-d-preview-render-target`** | **No further expansion as primary work** until 1–3 are stable. **Target DONE** = real Bevy camera → texture, `SwapImageBuffers` presentation, egui display-only — **not** CPU raster as the long-term owner. |
| **5** | **`phase-f-gpu-particles`** | Last: requires LOD + stamped snapshots + VT-4 discipline. |

**Repo reality (2026):** Phase **D** already contains partial/stub work (CPU swap, GPU stub camera, contract resources). Treat that as **exploratory** until step **4** is explicitly re-entered under this spine; **do not** grow new preview/GPU side paths until **1–3** meet their strict EXIT clauses.

**Repo reality (2026-05-13, stage5-08):** `AppStage5ReadinessReport` + `Stage5ReadinessProfile` (`HEADLESS` default; `FULL_APP` when `WorldPreviewPlugin` is present) enforce spine invariants at runtime; MAP REP + F3 diagnostics surface readiness; `phase_f_lod_proof.rs` records per-band GPU upload ordering; CI runs `cargo test --lib`. **Phase D (2026-05-13):** offscreen camera renders chunk quads into `SwapImageBuffers::back`, present swaps to **front**, egui samples `WorldPreviewTexture` only; CPU raster is gated when GPU is authoritative. These are **governance / proof hooks** plus preview architecture — not substitutes for strict Phase **F** instanced draw.

**Code anchor (theme-3 minimal slice):** `WorldLodBand` + `WorldRepresentationFrame` live in `src/gui/world_representation.rs` — one `Update` system runs after map camera smooth and `SimControlSystemSet::AdvanceSimTick`; no `FireVisualFrame` / GPU / minimap wiring yet.

---

## 2. Mental model rename

- **Stop:** “LOD = level of detail (render hack).”
- **Start:** **View domain resolution** — one policy layer decides **what exists at this scale**, how it is aggregated, culled, or symbolic, **before** `FireVisualFrame` and **before** GPU upload.

Target direction: a **`WorldRepresentationResolver`** (name TBD) consumes **ECS + camera intent + zoom + interest + zones + importance** and outputs a **`WorldLodMap`** / **`RepresentationResult`** that drives fire, units, terrain, overlay, compute dispatch, and GPU projection **once**. **Companion staging:** [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md) § *Representation domain resolution* and § *Stage staging*.

**Invariant:** **LOD decides representation** — not individual render systems, not GPU shaders, not minimap code, not preview or particle subsystems.

---

## 3. Single GPU fire path (hard rule)

There is **one** GPU fire spine:

`ECS (sim)` → **`FireVisualFrame`** (after LOD resolution) → **GPU buffer / instances** → render

**Not allowed:** “debug GPU buffer”, preview-only GPU lists, second emitter lists, duplicate extractors for the same semantic data.

**Overlay / UI:** `FireVisualFrame.chunk_heat` → `SharedOverlayFieldBuffers` → minimap + UI + debug — **no second ECS fire scan** for those consumers.

---

## 4. Strict “DONE” definitions (summary)

### Theme 3 — `theme-3-lod-overlay-channel-matrix`

**MUST:**

1. **LOD changes extraction / GPU payload**, not only an enum in memory:
   - **`FireVisualLod::Full`** — full instance path (subject to caps policy).
   - **`FireVisualLod::Clustered`** — **reduced** instance upload vs Full (measurable).
   - **`FireVisualLod::OverlayOnly`** — **`chunk_heat` only**; **no** fire instance GPU upload path for that frame/mode (or equivalent strict reduction — document the exact GPU path skipped).
2. **CPU → GPU remains one producer:** `ECS → FireVisualFrame → (LOD-shaped) → GPU buffer`.
3. **Overlays** keep consuming the **same** frame-derived heat view (`SharedOverlayFieldBuffers` from frame only).

**DONE when:** GPU upload size/resolution **measurably** reflects LOD; `chunk_heat` stays consistent with minimap intent; grep/architecture shows **no** second fire extract for visuals.

### Phase E — E1 only (first slice)

**MUST:**

1. **`SimStepStamp` as a real struct** (not only a type alias), e.g. `{ tick: u64, sim_time: f64 }`, carried on the snapshot.
2. **`FireVisualFrame` includes `stamp: SimStepStamp`** (or an agreed wrapper) so render can assert “complete frame for tick N”.
3. **`FixedUpdate` runs only the agreed minimal sim slice** (e.g. fire + LOD-critical sim) — **not** full atmosphere/UI in the first E1 slice.
4. **Render reads only the latest committed frame** — no mid-step partial reads.

**Repo note:** Today `SimStepStamp` may still alias `SimTick`; **E1 DONE** replaces that with the struct + wiring above.

**DONE when:** No silent tick mismatch; render never consumes a half-updated `FireVisualFrame`.

### VT-4 — visual agreement

**MUST:**

1. **Three surfaces in one test harness or deterministic scenario:** sim minimap path, world preview (CPU fallback or render target — whichever the test pins), GPU fire visualization path.
2. **Same** fire positions / chunk heat / intensity zones within defined tolerance.
3. **Mismatch detection:** if divergence > threshold → log + include **frame stamp** (once E1 exists).

**DONE when:** disagreement is **detectable**, not silent.

### Phase D — preview render target

**MUST (target state, not stub):**

1. Real Bevy **`RenderTarget::Image`** camera path produces **world** pixels (not only clear color).
2. **`SwapImageBuffers`** is the presentation layer (front/back), used to avoid tearing.
3. **egui** registers/displays the texture only — no egui-owned raster loop as the primary path.

**DONE when:** stable camera → texture → swap → egui; CPU full-map raster is not the idle default where GPU path is active.

### Phase F — GPU particles

**MUST:**

1. Real GPU buffer + instanced draw fed from **`FireVisualFrame`** (post-LOD), not toy arrays.
2. At least one **`WorldFireFx`**-class effect (e.g. smoke **or** embers) is GPU-driven.
3. **LOD must reduce GPU particle cost** when bands tighten.

**Not allowed:** “GPU-ready” labels on CPU-only sim; debug-only particle paths.

---

## 5. What **not** to do next (anti-patterns)

- Do **not** add LOD branches **inside** GPU prepare code as the primary policy.
- Do **not** duplicate LOD decisions per subsystem (fire vs minimap vs particles).
- Do **not** open a second fire visual extract because one path is “convenient” for preview.

---

## 6. Suggested implementation sequence (PR-sized)

1. **Contracts:** `WorldLodBand` (or equivalent) + resolver output type (`WorldLodMap` / per-domain selections) — even if the first PR only wires **fire**.
2. **Schedule:** resolver **before** `FireVisualFrameSet::BuildProfiles` (policy before snapshot).
3. **Extract:** make `FireVisualExtractPlan` / GPU upload honor resolver output measurably.
4. **Tests:** VT-4 harness + mismatch log; then tighten phase-d; then phase-f.

---

## 7. Canonical render pipeline (target)

```text
ECS SIM
  → WORLD EVENTS
  → VIEW DOMAIN RESOLUTION (LOD / representation policy)
  → FireVisualFrame (STAMPED, SCALED)
  → OverlayFieldFrame / SharedOverlayFieldBuffers (from frame)
  → GPU instances & fields (SCALED)
  → Render / UI / Minimap
```

---

## 8. Cross-links

| Doc | Role |
|:---|:---|
| [`backlog_serialization_preview_streaming_runbook_v1.md`](backlog_serialization_preview_streaming_runbook_v1.md) | Wave **S/P/C** before scaling streamable overlay/belief consumers |
| [`legacy_cpp_repos_agent_communication_maps_v1.md`](legacy_cpp_repos_agent_communication_maps_v1.md) | Stage-7 behavior/comms contracts (after spine EXIT) |
| [`strategic_overlay_runbook_v1.md`](strategic_overlay_runbook_v1.md) | Overlay field owners feeding `FireVisualFrame` / shared buffers |
| [`experience_layer_ux_hud_designer_brief_v1.md`](experience_layer_ux_hud_designer_brief_v1.md) | GPU minimap, transmission, construction blueprint, command shell — consume spine only (**BQ-119+**) |

---

*Last edited: 2026-05-14 — cross-link UX/HUD designer brief.*
