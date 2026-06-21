# Sim HUD pause menu `v2` — focus trap + keyboard nav

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-PAUSE-002** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 4 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Code** | `src/gui/pause_menu_bevy.rs` |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-PAUSE-002 Q✓
Pause overlay — focus trap · Esc cascade · keyboard order
```

---

## 1. Layout (unchanged size)

Center card · `bg_elevated` · `wire_magenta` border · max width 360px.

---

## 2. Focus trap

| Rule | Spec |
|:---|:---|
| Open pause | focus first actionable button |
| Tab | cycles within card only |
| Shift+Tab | reverse |
| Esc | close pause → return focus prior widget |
| Map clicks | blocked while open |

---

## 3. Button order (keyboard)

1. Resume  
2. Save  
3. Load  
4. Settings  
5. Quit to menu  

---

## 4. Esc cascade (signed)

```text
Tool sheet open → close sheet
Context tray expanded → peek
Pause open → close pause
Else → open pause
```

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
