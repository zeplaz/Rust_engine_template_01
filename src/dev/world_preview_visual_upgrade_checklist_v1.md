# DESIGN-WP-QUALIFIED-UPGRADE-001 — qualified -> full PASS checklist `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-WP-QUALIFIED-UPGRADE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** (promotion checklist; runtime witness already shows qualified green) |
| **Unblocks** | `UI-WP-VISUAL-001` (after operator visual run) |
| **Witness** | `debug_runs/wave_p_live.json` → `/ui_wp_visual_001/green`, `/ui_wp_visual_001/lib_qualified`, `/ui_wp_visual_001/visual_signoff_pending` |
| **Do not break** | `/ui_wp_visual_001/green == true` and `/ui_wp_visual_001/lib_qualified == true` |

---
## Goal
After the coders land qualified library acceptance for world preview visual (UI-WP-VISUAL-001), promote to full PASS based on operator `--test visual` run.

This checklist is designer-side: it describes what must be visually re-verified after promotion (no Rust).

---
## Promotion trigger
Operator runs:
`cargo run -p proc_A_dine01 --release -- --test visual`

Promotion allowed only if witness indicates qualified green:
- `/ui_wp_visual_001/green == true`
- `/ui_wp_visual_001/lib_qualified == true`

---
## Checklist items (designer)
1. No duplicate/misaligned preview shell when generator sheet is open/closed.
2. Corner inset and unified workspace remain correct under default zoom.
3. Pixel-level sanity for: map dominance, slide sheet dim alpha, and composite layer bindings.
4. Visual scan: no stale preview binding; no missing panes.

---
## Evidence to record (what to copy into sign-off)
Record operator timestamp and whether any visual-run blockers were observed.

*** End Patch
