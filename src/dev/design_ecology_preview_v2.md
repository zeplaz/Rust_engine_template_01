# Ecology preview panel `v2` — fuel + old-growth read

| Field | Value |
|:---|:---|
| **ID** | **DES-ECOLOGY-PREVIEW-V2-001** |
| **Program** | PLAN-DESIGNER-WORK-202606-001 · Track D |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md) · F1 sim witness |
| **Verdict** | **PASS** |

```text
DES-ECOLOGY-PREVIEW-V2-001 Q✓
Fuel band · old-growth gate · ignition hint — sim diagnostics only
```

---

## 1. Panel placement

| Context | Surface |
|:---|:---|
| WorldGen preview | collapsed strip under fire ecology section |
| Simulation | diagnostics only — not PLAY chrome |

---

## 2. Rows

| Row | Copy pattern |
|:---|:---|
| Fuel load | `FUEL  ·  {band}` — Low / Med / High |
| Old growth | `OLD-GROWTH  ·  {pct}%` |
| Ignition gate | `IGNITION  ·  {open|closed}` + reason word |
| Moisture | `MOISTURE  ·  {dry|normal|wet}` |

---

## 3. Color tokens (glyph + word)

| Band | Glyph | Never color-alone |
|:---|:---:|:---|
| Low fuel | `○` | word always |
| High fuel | `●` | amber word |
| Gate closed | `✗` | `closed` |
| Gate open | `✓` | `open` |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
