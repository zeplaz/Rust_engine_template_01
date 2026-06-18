# Power repair panel UX `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-REPAIR-PANEL-001** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track C |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Authority** | [`power_damage_ui_persistence_v1.md`](../../docs/reference/designer_questions/production_economy/power_damage_ui_persistence_v1.md) |
| **HUD tier** | **P2 Docked** or **P1 Tray** Logistics tab — not floating corner |
| **Handoff** | COD-POWER-REPAIR-QUEUE-001 |
| **Verdict** | **PASS** |

```text
DES-POWER-REPAIR-PANEL-001 Q✓
Arrangeable repair queue — priority 1–100 — parts from supply chain
```

---

## 0. Purpose

After cuts and knockouts, player **queues repairs** for lines and transformers. UX: visible queue, reorder, priority, parts read — matches uber damage model (one track).

---

## 1. Placement

| Option | Verdict |
|:---|:---|
| **Context tray → Logistics tab** | **P0** — section **Power repairs** |
| Product shell dock **Construction queue** | P1 merge — filter chip `Power` |
| Floating window | **Ban** |

```text
┌ [Alerts] [Logistics] [Build] ─────────────────────┐
│ Power repairs (3)                    [+ Queue all] │
│ ┌──────────────────────────────────────────────┐ │
│ │ ≡ MV segment · yard feed · parts 2/2  P [72] │ │
│ │ ≡ Transformer T-12 · parts 0/4  P [90] blocked│ │
│ │ ≡ HV span north · parts 1/3  P [45]         │ │
│ └──────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

---

## 2. Row fields

| Column | Content |
|:---|:---|
| Drag handle | ≡ reorder |
| Label | Human target — `MV segment · yard feed` / `Transformer {id}` |
| Parts | `{have}/{need}` — `blocked` if short |
| Priority | `P [{1-100}]` spinner |
| ETA | `~{ticks}` when parts satisfied |
| Cancel | × remove job |

---

## 3. Interactions

| Action | Behaviour |
|:---|:---|
| Click row | Pan map to target + highlight segment/node |
| Drag reorder | Stable sort within same priority band |
| Priority change | Integer 1–100 · tie-break: severity then queue order |
| Queue all damaged | Adds all `Damaged` segments in view — confirm toast |
| Cancel job | `○ Repair cancelled — {label}` |

---

## 4. Parts & blocked-why

| State | Copy |
|:---|:---|
| Parts ready | `parts {n}/{n}` mono green |
| Short | `parts {have}/{need} · blocked: need {resource}` |
| No spares warehouse | `blocked: no spares in range` |
| Specialist boost | caption `repair crew +{pct}%` when active |

**Authority:** supply chain + spare parts sim — panel displays only.

---

## 5. Completion feedback

| Event | UX |
|:---|:---|
| Job complete | Row flash green · toast `✓ Repaired — {label}` |
| Line restored | Overlay **live** stroke + brief pulse |
| Transformer restored | Consumer badges clear `offline` |

---

## 6. Copy registry

| Key | String |
|:---|:---|
| `power.repair.title` | `Power repairs` |
| `power.repair.empty` | `○ No power repairs queued` |
| `power.repair.parts` | `parts {have}/{need}` |
| `power.repair.blocked` | `blocked: need {resource}` |
| `power.repair.complete` | `✓ Repaired — {label}` |
| `power.repair.queue_all` | `Queue all damaged` |

---

## 7. Witness

```json
{
  "power_repair_panel_wired": true,
  "panel_tier": "tray_logistics",
  "floating_repair_window": false,
  "priority_range": [1, 100]
}
```

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
