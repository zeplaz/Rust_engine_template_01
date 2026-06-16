# Operational feedback language v1

> **PURPOSE:** Vocabulary and patterns for **developmental UX** — teach causality in plain operational language. Aligns with `developmental_ux_runbook_v1.md` (L0–L1 first).

**Version:** v1.0.0

---

## Principles

1. **Answer why** — never “Cannot place”; always a reason or next step.
2. **Operational, not spreadsheet** — avoid internal field names; use mission meaninful nouns (supply, threat, corridor, grid).
3. **Actionable** — when blocked, hint what to change (toggle overlay, move site, fix dependency).
4. **Layer depth** — strip = one line; expand later to tray / egui for L2+ chains.

---

## BAD vs GOOD

| BAD | GOOD |
|-----|------|
| Cannot place structure | **Site placement blocked:** logistics reach too weak here — try closer to a corridor or wait for network solve. |
| Invalid | **Invalid:** excavation slope over tolerance for this archetype. |
| Error: network_access | **No logistics access** within effective range — connect or extend surface network. |
| utility_weight=0.32 | **Doctrine:** leaning toward consolidation — reducing offensive exposure until supply stabilizes. |

---

## Token glossary (machine → copy)

Map stable validation / diagnostic tokens to player strings in code (see `gui/hud/validation_feedback.rs`):

| Token / code | Player-facing line |
|--------------|---------------------|
| `terrain` | Terrain checks failed — slope, flood, or geology unsuitable. |
| `network_access` | No usable logistics access from this tile. |
| `sparse_logistics_reach` | Logistics field shows weak reach — site may be stranded. |
| `out_of_raster_bounds` (future) | Outside operational raster — cannot evaluate this map region. |

Extend as validators grow; keep tokens **stable** for saves / telemetry.

---

## L0 strip pattern (no window)

Single line, monospace-friendly:

`CONTEXT — Build: roads · tile @12,44 · Commit [Enter] when valid · cycle [;]`

When invalid:

`CONTEXT — **Blocked:** weak logistics reach — move toward a supply corridor or enable congestion overlay ([7]).`

---

## Severity wording

- **Info** —FYI, no blockage.
- **Warning** — allowed or risky; call out consequence.
- **Error** — commit blocked; must explain.

---

## Localization note

Keep sentences short; avoid nested subclauses. When i18n lands, keys should map **semantic id + parameters**, not English sentences only.
