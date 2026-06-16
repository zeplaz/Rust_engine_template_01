# APS-BEVY-QC-HUD-001-DESIGN — Designer sign-off `v1`

| Field | Value |
|:---|:---|
| **Program** | APS-BEVY-QC-HUD-001-DESIGN |
| **Parent** | APS-BEVY-QC-HUD-001 (Lane A′ egui QC) |
| **Owner** | `@designer` |
| **Verdict** | **PASS WITH NOTES** |
| **Date** | 2026-06-03 |
| **Brief** | [`aps_bevy_qc_hud_brief_v1.md`](../../prompts/designer_questions/aps_bevy_qc_hud_brief_v1.md) |
| **Shipped** | [`assembly_snapshot_qc_ui.rs`](../../src/gui/assembly_snapshot_qc_ui.rs) |
| **Witness** | [`debug_runs/aps_bevy_qc_hud_001_live.json`](../../debug_runs/aps_bevy_qc_hud_001_live.json) |

---

## Review vs brief

| Requirement | Shipped | Verdict |
|:---|:---|:---:|
| Snapshot path + Load | Text field, Load, example button | ✓ |
| Summary header | assembly_id · archetype · district · seed + grammar chain + atlas smoke | ✓ |
| Placement table | 6 columns (see below) | ✓ |
| Row status text | OK / WARN / FAIL — not color-only | ✓ |
| Plain errors | “Snapshot not found”, “0 placements”, missing material count | ✓ |
| Spawn preview | Queues `bevy_preview_worker` job + hint line | ✓ |
| Open in APS | Shell hint string (no Tk spawn from Bevy) | ✓ |
| egui dev gate | sim or editor; **not** player HUD | ✓ |
| Default collapsed | Hidden until Ctrl+Shift+Q or F3 entry | ✓ |
| Row → footprint highlight | Not wired | **V2** |
| P0 validate read-only strip | Not present | **V2** |

---

## Approved layout

**Toggle:** **Ctrl+Shift+Q** · **F3 → Open assembly snapshot QC panel**

**Window:** 720×480 default · title `Assembly snapshot QC (APS-BEVY-QC-HUD-001)`

```text
Read-only QC — Ctrl+Shift+Q toggle · loads assembly_snapshot JSON from disk
Path: [________________________] [Load]
[Use example warehouse snapshot]

Summary
  {assembly_id} · archetype {id} · district {style} · seed {n}
  {grammar chain one-liner}
  Atlas: {atlas_smoke}
  {N placements — all material_profile present | missing on N cells}

[Spawn preview]  [Open in APS (shell hint)]
{preview job hint line}

Placements (scroll max 280px)
| Cell      | module_id | material_profile | tags (≤48) | GLB (≤32) | status |
| (x,y,fz)  | …         | …                | …          | …         | OK     |
```

**Column / truncation rules (locked):**

| Column | Rule |
|:---|:---|
| Cell | `(grid_x,grid_y,f{floor})` |
| module_id | full |
| material_profile | `(missing)` when empty |
| tags | placement_tags + semantic_tags; **48 chars** + ellipsis |
| GLB | path **32 chars** + ellipsis |
| status | **OK** file exists · **WARN** path set, file missing · **FAIL** empty path |

**Selection:** row click sets `selected_row` — footprint highlight deferred to **APS-BEVY-QC-HUD-001-V2**.

---

## Copy approval

| String | Status |
|:---|:---:|
| Empty: “Load a snapshot to inspect placements.” | ✓ |
| Spawn hint: job path + PNG target | ✓ |
| APS hint: `python -m rust_engine_mcp.cli assembly open --snapshot <path>` | ✓ (keep) |
| Missing materials: “material_profile missing on N cell(s)” | ✓ |

---

## Accessibility

| Item | Status |
|:---|:---:|
| Status column text OK/WARN/FAIL | ✓ |
| Table readable at 720px width | ✓ |
| Scroll capped 280px — affordance OK | ✓ |
| No hover-only critical actions | ✓ |

---

## Notes → @coder (V2)

1. **Row select → footprint grid highlight** when preview entities exist (brief requirement)
2. Optional read-only **P0 gate** sentences (no MCP hot path in frame loop)
3. Consider **Browse** file picker beside path field (P2)

**Unblocks:** APS-BEVY-QC-HUD-001-V2

---

## Paste back

```text
APS-BEVY-QC-HUD-001 designer spec ready
Shortcut: Ctrl+Shift+Q (+ F3 entry)
Columns: Cell · module_id · material_profile · tags(48) · GLB(32) · status OK/WARN/FAIL
Spawn preview: yes
Sign-off: PASS WITH NOTES (V2: row highlight + P0 strip)
```
