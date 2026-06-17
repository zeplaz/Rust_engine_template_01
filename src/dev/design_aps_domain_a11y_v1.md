# DES-APS-DOMAIN-A11Y-001 — Lane switch keyboard + focus `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-DOMAIN-A11Y-001** |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Lane segmented control

| Key | Action |
|:---|:---|
| `Ctrl+1` | Buildings lane |
| `Ctrl+2` | Landscape lane |
| `Tab` / `Shift+Tab` | Focus lane control **before** Flow bar |
| `Left` / `Right` | Move between segments when focused |
| `Space` / `Enter` | Activate segment |

### Focus order (global)

```text
Lane switch → Flow bar (left→right) → Authority strip (read-only, skip) →
Pipeline bar (read-only) → Active tab content → Job strip → Status log
```

## Lane switch UX rules

1. Focus **returns** to first control in active tab after lane change (not lost).
2. Announce lane change in status line: `Lane: Landscape — Presets tab`.
3. Segment carries **visible label** `Buildings` / `Landscape` + underline tint — not color alone.
4. `takefocus=1` on both segments; `ttk` radiobutton style or custom `Frame`+`Button` pair.

## Guard expectation

New test: Tab from window entry reaches lane control within 3 Tab stops.

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |
