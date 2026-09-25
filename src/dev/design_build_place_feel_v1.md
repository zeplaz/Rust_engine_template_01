# Build place feel `v1` (DES-BUILD-PLACE-FEEL)

| Field | Value |
|:---|:---|
| **ID** | **DES-BUILD-PLACE-FEEL** |
| **Owner** | `@designer` |
| **Date** | 2026-09-25 |
| **Parent** | [`design_build_ux_redesign_v1.md`](design_build_ux_redesign_v1.md) · [`design_place_feedback_anim_v1.md`](design_place_feedback_anim_v1.md) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) — preview ≠ commit; two-click FSM unchanged |
| **Feel target** | Stronghold / Crusader locked footprint + SimCity tray discipline |

The icon, the tray line, and the ghost must name the thing being placed. Clicks stay on the shipped two-click path.

---

## Mission

Placement chrome is dense and readable. A catalog row is the thing you will stamp. The ghost is that thing’s footprint, held still when locked. Success does not flash white or swap hue in the first half of the settle.

**Acceptance:** *Pick “Victorian row house” (or a defense row) → tray and context say that name → first click locks the same green footprint → second click settles without a bright punch → crosshair stays one cursor.*

---

## 1. Tray and copy

| Rule | Spec |
|:---|:---|
| Subject | When `building_intent.label` or `DefenseKind::player_label` is set, lead with that string. Category words (`Industry`, `building`) are not a substitute. |
| Lock | Adjust + origin: `locked footprint`. Preview: `follows cursor`. |
| Modifiers | Keep shipped `Ctrl rotate · Shift scale` (tests and prior charters lock this phrase). |
| Idle | No armed subject → existing `BUILD · queue N (M ready) · …` line. |
| Snap | If `ConstructionPathFeedback.snap_hint` is set, append it. Prefix `Snap ·` only when the hint does not already start with “snap”. |
| Grammar v2 strip | Still contains `Valid ✓` / `Blocked`. Insert the subject name before validity when a subject exists. Demolish and defense strips stay on their locked copy helpers. |

---

## 2. Picker hit targets (Bevy sheet is the sim path)

| Token | Value | Was |
|:---|---:|---:|
| Catalog row min height | **40px** | 32 |
| Close control | **40×40px** | 36 |
| Row label | **13px** | 12 |

Rail width stays **52px**. Do not add a second picker window.

---

## 3. Unified crosshair

OS-cursor hide policy is unchanged. When the game crosshair draws:

| Token | Value |
|:---|:---|
| Ring radius | **5px** |
| Ring | 1.25px white, with a 2px dark halo one pixel outside |
| Cross arms | gold, length `1.6 × radius`, 1px |

One cursor. No second sprite.

---

## 4. Ghost and settle (presentation only)

| Rule | Spec |
|:---|:---|
| Hues | Do not change valid / risky / invalid / Planned settle tokens. |
| Lock ring token | `footprint_lock_ring_color` gold `#E8C460` @ 230α, stroke **2px**. Painter hook for a later slice — this slice does not add a second ghost entity. |
| Pulse | Duration stays **280ms**. `t=0` valid fill. `t=1` Planned settle fill. |
| Hold | Until raw `t=0.62`, fill stays the valid token (locked footprint). Ease-out cubic only on the tail crossfade. |
| Flash | Alpha multiplier **1.0**. Never punch above the source alpha. No white, no camera nudge. |
| FSM | `Adjust → Preview` on commit stays as shipped. Pulse remains disposable. |

---

## 5. Out of scope

Two-click input FSM, pixel pipeline, scan debug, orchestrator dashboard, grammar-label tests, fire shader tests.
