# APS-BEVY-QC-HUD-001-V2 — Designer sign-off `v2` (EGUI-DEV-UX-001)

| Field | Value |
|:---|:---|
| **Program** | **EGUI-DEV-UX-001** · **APS-BEVY-QC-HUD-001-V2** |
| **Parent** | APS-BEVY-QC-HUD-001-DESIGN (`design_aps_bevy_qc_hud_v1.md`) |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Date** | 2026-06-03 |
| **Brief** | [`aps_bevy_qc_hud_brief_v1.md`](../../prompts/designer_questions/aps_bevy_qc_hud_brief_v1.md) |
| **Shipped** | [`assembly_snapshot_qc_ui.rs`](../../src/gui/assembly_snapshot_qc_ui.rs) |
| **Witness** | [`debug_runs/aps_bevy_qc_hud_001_v2_live.json`](../../debug_runs/aps_bevy_qc_hud_001_v2_live.json) |

---

## Mission

Close the **v1 PASS WITH NOTES** tail: row-select footprint highlight and read-only P0 gate strip — without turning the panel into an editor or MCP hot path.

---

## Review vs v1 notes

| Requirement | Shipped | Verdict |
|:---|:---|:---:|
| Row click → footprint cell highlight | Plan grid under **Footprint highlight**; selection uses `palette.selection_bg`; gated on **Spawn preview** | ✓ |
| Plain-language P0 strip | **P0 gate (read-only)** section; sentences + `→ fix_hint` on Load only | ✓ |
| No per-frame MCP / validate hot path | `evaluate_p0_readonly` runs on Load / snapshot change only | ✓ |
| v1 table + copy preserved | Six columns, OK/WARN/FAIL, Ctrl+Shift+Q + F3 entry | ✓ |
| Empty / guidance copy | “Spawn preview to enable footprint grid highlight.” · “Click a placement row…” | ✓ |
| Green witness | `footprint_highlight_on_preview: true` · `p0_readonly_strip: true` | ✓ |

---

## Approved V2 layout (delta)

```text
… Summary + atlas smoke (unchanged) …

P0 gate (read-only)
  {sentence}
  → {fix_hint}
  — or —
  P0 gate: no blocking issues detected (read-only).

[Spawn preview]  [Open in APS (shell hint)]

Footprint highlight          ← visible after Spawn preview
  Selected row highlighted on plan grid (preview active).
  [W×D monospace token grid — selected cell filled]

Placements (table unchanged)
```

**Interaction contract (locked):**

| Step | Behavior |
|:---|:---|
| Load snapshot | P0 issues evaluated once; strip populated or “no blocking issues” |
| Click placement row | `selected_row` set; grid highlights matching `(grid_x, grid_y, floor)` |
| Before Spawn preview | Muted hint only — no grid (avoids false “highlight” without preview context) |
| Spawn preview | Enables grid section; worker job unchanged from v1 |

**P0 copy style:** Plain English sentences (not error codes). Fix hints use `→` prefix in muted label — matches APS Assembly tab tone.

---

## Accessibility

| Item | Status |
|:---|:---:|
| Selection uses fill + token text — not color-only | ✓ |
| P0 issues are full sentences | ✓ |
| Grid cells 22px — readable at 720×480 panel | ✓ |

---

## Deferred (P2 — not blocking)

1. **Browse** file picker beside path field (v1 note #3)
2. Highlight footprint in **3D viewport** when preview entities exist (brief “if preview entities spawned” — plan grid satisfies minimum; 3D hook optional)

**Closes:** EGUI-DEV-UX-001 · APS-BEVY-QC-HUD-001-V2 designer review

---

## Paste back

```text
EGUI-DEV-UX-001 designer sign-off ready
V2: footprint grid + P0 read-only strip — PASS
Witness: debug_runs/aps_bevy_qc_hud_001_v2_live.json
Coder: no further QC HUD slices unless Browse (P2) requested
```
