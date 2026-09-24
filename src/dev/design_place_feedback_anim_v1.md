# Place feedback animation `v1` (DES-P3-PLACE-ANIM)

| Field | Value |
|:---|:---|
| **ID** | **DES-P3-PLACE-ANIM** |
| **Queue** | **P3-PLACE-ANIM** |
| **Priority** | **P3** |
| **Owner** | `@designer` (charter) · `@coder` **COD-P3-PLACE-ANIM-001** |
| **Verdict** | **PASS** |
| **Date** | 2026-09-24 |
| **Prereq** | **TRIAGE-BUILD-CLICK-PLACE-001** · **COD-MIL-DEFENSE-PLACE-001** |
| **Parent UX** | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · [`construction_parametric_ghost_visual_v1.md`](construction_parametric_ghost_visual_v1.md) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |
| **Witness** | [`debug_runs/des_place_feedback_anim_live.json`](../debug_runs/des_place_feedback_anim_live.json) |
| **Unblocks** | **COD-P3-PLACE-ANIM-001** |

**No Rust in this doc.** Tokens + timing + interaction rules only. Preview ≠ commit preserved.

---

## Mission

After a successful **Building** or **Defense** place (second LMB / Enter), the locked ghost must **settle into the map** with an intentional short motion — not vanish in one frame (flash / glitch).

**Acceptance test:** *Two-click place a building or defense site → footprint holds and eases for ~280ms into Planned-site read → Preview ghost resumes under cursor — no white flash, no second commit writer, FSM unchanged.*

---

## Problem (current)

| Surface | Today | Defect |
|:---|:---|:---|
| `try_commit_active_building_ghost` | queues `CommitConstructionSiteEvent` then `ghost.unlock_to_preview()` | `origin = None` same frame → footprint overlay gone |
| Site paint | Planned phase appears 0–N frames later | Gap reads as flash / teleport |
| Defense path | Same two-click funnel as Building | Same flash for Wall / Trench / Bunker |

---

## 1. Interaction model (presentation only)

```text
Adjust ─LMB/Enter(valid)▶ CommitConstructionSiteEvent
                       ─parallel▶ PlaceFeedbackPulse (disposable visual)
                       ─FSM▶ Preview (unlock_to_preview — unchanged)
PlaceFeedbackPulse ─ease 280ms▶ fade/settle ─done▶ clear pulse
```

| Rule | Spec |
|:---|:---|
| **Trigger** | Only after a **successful** commit message is queued (`allows_commit` already true) |
| **FSM** | Still `Adjust → Preview` immediately — pulse is **not** a placement mode |
| **Paths** | All `uses_two_click_place()` tools (Building **and** Defense) — one pulse path |
| **Cancel / invalid** | No pulse on blocked second click, RMB/Esc cancel, or staging-only path |
| **Overlap** | New commit while pulse active → **replace** pulse (latest footprint wins) |
| **Tool change** | Leaving build strip / clearing tool → clear pulse |

**Authority (hard):**

```text
CommitConstructionSiteEvent     → sole execute funnel (invariant 2)
BuildGhostState.unlock_to_preview → FSM only (unchanged)
PlaceFeedbackPulse              → disposable egui / ConstructionVisualRequests (invariant 9)
ConstructionVisualRequests      → single visual buffer — no parallel extract (invariant 16/18)
```

---

## 2. Motion (locked)

| Param | Value | Notes |
|:---|:---|:---|
| **Duration** | **280ms** (±40ms ok) | Short RTS settle — not cinematic |
| **Curve** | Ease-out cubic | Fast start, soft land |
| **t=0** | Full footprint at **valid** fill (`footprint_valid_color`) + lock-ring weight | Same tiles as Adjust ghost |
| **t=0→0.45** | Alpha punch **×1.15** then ease toward 1.0 | Intention, not flash |
| **t=0.45→1.0** | Crossfade fill toward **Planned** phase hue (`#6490DC` family — matches `phase_visual` Planned) while outline stays 1–2px | Ghost → site continuity |
| **t=1.0** | Pulse cleared; site phase paint owns read | No leftover overlay |

**Forbidden:**

- White / full-screen flash
- Hue jump to invalid red on success
- Scale bounce > **4%** of tile extent (zoom trust)
- Sprite detach / second ghost entity
- Camera nudge / focus steal

### Reduced motion

If a reduced-motion preference resource exists (or is added later): **skip punch + crossfade** — show **1 frame** of settle outline then clear. Until that resource exists, keep 280ms ease (no new OS hook required for this slice).

---

## 3. Tokens (reuse `ghost_visual` / phase hues)

| Token | Source | Role |
|:---|:---|:---|
| `footprint_valid_color()` | existing | Pulse start fill |
| Planned phase `#6490DC` @ ~200α | `phase_visual::phase_color(Planned)` family | Settle target fill |
| Outline | 1–2px same-hue @ 90% | Footprint bound during pulse |
| Lock ring | existing Adjust accent (if drawn) | Fade out by t=0.35 |

**Coder may add** thin helpers in `ghost_visual.rs` only if needed:

| Suggested name | Spec |
|:---|:---|
| `footprint_place_settle_color()` | Planned-family RGBA (align with phase Planned — do not invent a third green) |
| `place_feedback_duration_secs()` | `0.28` |

Do **not** fork road/corridor tokens. Do **not** change valid/risky/invalid hues.

---

## 4. Visual hierarchy / layering

Overlay priority (unchanged global order; pulse sits with construction preview):

```text
Critical alerts ═▶ Selection ═▶ PlaceFeedbackPulse ═▶ Active ghost (Preview) ═▶ Staged ghosts ═▶ Site phase tiles ═▶ Terrain
```

Pulse draws **above** Planned site tiles for the settle window so the handoff is readable, then clears so phase labels win.

---

## 5. Accessibility

| Check | Spec |
|:---|:---|
| Colorblind | Shape = same footprint tiles; settle uses Planned **hue family already used for sites** — not a success-only green→red cue |
| Contrast | Outline remains ≥1px through fade |
| Motion | No camera motion; optional reduced-motion = 1-frame settle |
| Operable without color | Footprint silhouette + fade is enough; no toast required for success pulse |

---

## 6. Viewport / multiview

| View | Pulse |
|:---|:---|
| Simulation map (primary) | **Yes** |
| Minimap | **No** |
| World preview | **No** |
| Editor-only chrome | **No** |

Uses `ConstructionMapProjection` / existing footprint request path only — no camera ownership.

---

## 7. Coder handoff — COD-P3-PLACE-ANIM-001

```text
Read:  src/dev/design_place_feedback_anim_v1.md
       src/dev/construction_invariants.md
       src/dev/design_build_ux_redesign_v1.md
       src/construction/ghost_visual.rs
       src/construction/visual_authority.rs
       src/construction/build_interaction.rs (try_commit_active_building_ghost)
Touch: ghost_visual.rs (optional settle token)
       visual_authority.rs OR small place_feedback.rs (Resource + tick + push FootprintTileRequest)
       build_interaction.rs (arm pulse AFTER successful queue_commit — do not fork commit)
       construction witness / AtomicU32 counters for place_feedback_starts / completes
Do:    Building + Defense two-click place → 280ms settle via ConstructionVisualRequests
       Keep unlock_to_preview + CommitConstructionSiteEvent unchanged
Do NOT: second commit writer · bypass allows_commit · change two-click FSM
        WALL-ARCHETYPE · Phase 9 · logistics · building look · G-PLAY
        parallel extract / new GPU path · camera motion
Verify: validate-report cargo --cached --compress 4
        cargo test -p proc_A_dine01 --lib construction::
Witness: debug_runs/des_place_feedback_anim_live.json (set impl_wired + counters after code)
```

### joint: @coder

Does a `PlaceFeedbackPulse` resource that only writes into `ConstructionVisualRequests` violate single visual authority? Prefer one sync system appending footprint requests after `sync_footprint_visual_requests` (or extend that sync) — never a second painter with its own camera math fork.

---

## 8. Residuals (explicit — do not expand)

| ID | Scope | Owner |
|:---|:---|:---|
| **WALL-ARCHETYPE** | Replace `MilitaryBase` stub for Defensive wall | `@planner` (packet only) |
| Phase 9 / logistics / G-PLAY / APS / Building Look | Out of slice | Skip |

---

## 9. Review gate (designer)

| Check | Pass |
|:---|:---:|
| Intentional settle ≠ flash | ✓ |
| Reuses ghost / phase tokens — no parallel extract | ✓ |
| Two-click FSM + pointer gate + commit funnel untouched | ✓ |
| Building + Defense share one pulse path | ✓ |
| Invalid / cancel never pulses | ✓ |
| construction_invariants 1–2, 7, 9, 16/18 respected | ✓ |
| WALL-ARCHETYPE left for @planner | ✓ |
| Code-heavy impl → @coder | ✓ |
