# DESIGN-THEME-COLLAGE-001 — theme collage delta for Bevy HUD panels `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **DESIGN-THEME-COLLAGE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Owner** | `@designer` |
| **Verdict** | **PASS (qualified)** (token-only delta; Bevy HUD panel mapping, not egui-only) |
| **Unblocks** | `PLAN-UI-THEME-MERGE-001` coder slice |
| **Witness** | `debug_runs/ui_shell_migration_live.json` → `/ui_w3_theme_001/green` |
| **Do not break** | `/ui_w3_theme_001/green == true` |

---
## Scope
Document the delta of `design_theme.md` collage against the current Bevy HUD panel style mapping.

Rules:
- This doc defines tokens and “where they apply” to Bevy HUD chrome.
- Do NOT define egui theme application logic in this doc (that lives in `ui_theme_merge_impl_spec_v1.md`).
- No Rust changes claimed.

---
## Token-only delta (what coders should treat as canonical)
1. Panel background and elevated layers must use the Bevy HUD palette (not ad-hoc color literals).
2. Text primary/muted colors must keep contrast on existing minimap and overlay chrome.
3. Accent + border must preserve the same visual separators across:
   - dock shell
   - context rail
   - minimap chrome

---
## Acceptance checklist (designer)
1. Bevy HUD panels reflect the collage delta (tokens only).
2. No regression in the global theme gate: `ui_w3_theme_001.green`.

*** End Patch
