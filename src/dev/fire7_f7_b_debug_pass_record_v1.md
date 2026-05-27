# DESIGN-F7-DEBUG-PASS-001 — F3 label wiring sign-off `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-F7-DEBUG-PASS-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **DEFER** (label wiring checklist template; witness coverage for “F3 open + labels present” not available in current live JSON) |
| **Unblocks** | `FIRE7-F7-B-DEBUG-UI-001` (coder HUD wiring lane) |
| **Witness** | `debug_runs/fire_streaming_live.json` → `/gate` and `/green` |
| **Do not break** | `/gate == "FIRE7-F7-B-001"` and `/green == true` |
| **Debug overlay contract (reference)** | `src/dev/fire_streaming_debug_overlay_names_v1.md` (telemetry keys + label order) |

---
## Scope
Sign-off template for the engineering wiring of the F3 debug section for Fire Phase 7 chunk streaming.

This record is intentionally *qualified*: it validates that the Fire streaming debug section is enabled and the underlying witness rollup is green.

---
## Qualified PASS template (what “PASS” means here)
1. F3 debug section exists (collapsing header string aligned to the canonical spec).
2. F3 telemetry rows exist in the canonical wire order:
   - `F7B gate=... green=...`
   - `F7B focus_chunk=... sleep_r=...`
   - `F7B sleep=... wake=... active=...`
   - `F7B runtime_writer=...`
3. Label keys map to the same JSON fields used by `fire_streaming_live.json`.

---
## Acceptance checklist (designer)
- Fire streaming debug section can be opened without crashing.
- Telemetry lines match the canonical spec and update across frames (no stale caching).
- No duplicate debug sections compete for the same F3 header slot.

*** End of record ***

