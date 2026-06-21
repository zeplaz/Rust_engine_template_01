# Sim HUD accessibility charter `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-A11Y-001** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 3 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_sim_hud_cohesion_charter_v1.md`](design_sim_hud_cohesion_charter_v1.md) · [`design_aps_color_a11y_audit_v1.md`](design_aps_color_a11y_audit_v1.md) |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-A11Y-001 Q✓
WCAG AA tokens · invalid = word+glyph · focus visible on all sim chrome
```

---

## 1. Contrast (AA on vellum shell)

| Token pair | Min ratio |
|:---|:---:|
| `fg_primary` on `bg_vellum` | 4.5:1 |
| `fg_muted` on `bg_vellum` | 3:1 (large/caption only) |
| `accent_gold` border on dark map | 3:1 |
| Invalid `Blocked` on strip | 4.5:1 |

---

## 2. Non-color invalid states

| State | Required |
|:---|:---|
| Blocked place | word `Blocked` + `✗` |
| Valid ghost | word `Valid` + `✓` |
| Alert tier | `◆` + tier word (`INFO` / `WARN` / `CRIT`) |
| Power island | word `Island` + count |

Never red/green fill alone — ref OPS v2 + build read HUD v2.

---

## 3. Focus & keyboard

| Surface | Focus ring |
|:---|:---|
| Build rail slot | 2px gold outer |
| Picker card | 1px cyan outer + gold left bar |
| Tray tab | underline + ring |
| Pause menu item | gold ring 2px |

Focus must remain visible on keyboard Tab — no `outline: none` without replacement.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
