# Sim HUD Esc cascade `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-ESC-CASCADE-001** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 4 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_sim_hud_cohesion_charter_v1.md`](design_sim_hud_cohesion_charter_v1.md) · [`sim_hud_copy_registry_v1.md`](sim_hud_copy_registry_v1.md) |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-ESC-CASCADE-001 Q✓
Esc order: sheet → tray expand → pause — never skip a tier
```

---

## 1. Cascade order (Simulation session)

| Press # | If open… | Action |
|:---:|:---|:---|
| 1 | Build picker sheet | Close sheet only |
| 2 | Context tray expanded | Collapse tray to peek |
| 3 | Pause menu closed | Open pause menu |
| 4 | Pause menu open | Close pause (resume) |

**Never:** Esc closes sheet + tray + pause in one frame.

---

## 2. Exceptions

| Context | Esc behavior |
|:---|:---|
| WorldGen / Editor | Full editor cascade — not sim charter |
| Modal confirm (demolish) | Cancel confirm first |
| Text field focused | Clear field or blur — not pause |

---

## 3. Copy registry keys

| Key | Value |
|:---|:---|
| `esc.hint.sheet` | `Esc — close picker` |
| `esc.hint.tray` | `Esc — collapse tray` |
| `esc.hint.pause` | `Esc — pause menu` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
