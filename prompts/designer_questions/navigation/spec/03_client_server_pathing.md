# Navigation — client / server pathing `03`

**Checklist:** `implementation_questions_v1.md` §6–8.

## Proposal payload (draft)

- `request_id`, polyline waypoints (quantized), **cost estimate**, **LOD span**, optional **rail reservation** ids 📎.

## Server

- Validate against authoritative graph + **faction permissions** (e.g. closed border) — overlaps `factions/` diplomacy when that graph exists.
- **Reject** reasons: enum for UI/telemetry (optional).

## Anti-cheat posture

- Competitive modes: trust model per milestone (`implementation_questions_v1.md` §8).
