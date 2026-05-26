# Wave 3 icon atlas sheet — `DESIGN-W3-P4-ATLAS-001` `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-W3-P4-ATLAS-001** |
| **Track** | Wave 3 / UI Phase 4 |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED — PASS** |
| **Primary asset** | `assets/textures/ui/icon_atlas_phase4_v1.png` |
| **Layout mock** | `assets/ui/phase4/icon_atlas_phase4_layout_mock.png` |
| **Manifest** | `assets/configs/ui/icon_atlas_phase4.icon_atlas.ron` |
| **Bake script** | `tools/orchestrator/scripts/bake_icon_atlas_phase4.py` |
| **Witness JSON** | `debug_runs/ui_shell_migration_live.json` (`phase4.icon_atlas_loaded`) |

---

## Deliverable intent

Wave 3 Phase 4 atlas sheet is the designer handoff that locks icon language for build rail cells and preserves the existing atlas manifest contract.

Verdict: `SIGNED — PASS`. The atlas and layout mock are present on disk and already integrated with shell witness fields.

---

## Acceptance checklist

| # | Item | Result |
|:---:|:---|:---:|
| 1 | Atlas PNG exists at canonical texture path | PASS |
| 2 | Layout mock exists at canonical Phase 4 mock path | PASS |
| 3 | RON atlas manifest path unchanged | PASS |
| 4 | Bake script emits 256x128 atlas sheet | PASS |
| 5 | Shell witness reports `phase4.icon_atlas_loaded: true` | PASS |

---

## Verification commands

```powershell
python tools/orchestrator/scripts/bake_icon_atlas_phase4.py
cargo test -p proc_A_dine01 --lib icon_atlas
```

---

## Notes for coder handoff

- This deliverable unblocks Phase 4 coder consumption paths; it does not reopen atlas manifest indexing.
- Any future icon art iteration must preserve atlas cell contract unless a separate manifest migration is approved.

---

## History

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | Initial Wave 3 atlas sheet deliverable record |
