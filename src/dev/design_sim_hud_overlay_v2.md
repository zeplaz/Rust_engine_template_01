# Sim HUD info overlay `v2` — panel IA + legend placement

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-OVERLAY-002** |
| **Program** | PLAN-SIM-HUD-PROFESSIONAL-POLISH-001 · Track 3 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Handoff** | COD-SIM-HUD-OVERLAY-002 (when queued) |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-OVERLAY-002 Q✓
Info panel tab merge · legend dock · sim-default collapsed
```

---

## 0. Scope

**Overlays / Info panel** (`HudWidgetId::Overlays`) — tab IA, legend placement, sim-session defaults.

---

## 1. Tab merge (v2)

| Tab | Contents | Sim default |
|:---|:---|:---:|
| **Map** | Layer toggles + minimap legend embed | collapsed |
| **Ecology** | Topology + burn frame read | collapsed |
| **Logistics** | Heat / routes summary | collapsed |
| **Debug** | Editor-only sections hidden in sim | hidden |

**Ban:** duplicate legend in Map + Minimap widget.

---

## 2. Legend placement

```text
┌ Info panel ─────────────────────┐
│ [Map] Ecology Logistics           │
│ ┌ Legend (scroll max 120px) ───┐  │
│ │ ● Network  ● Patch  ● Scar  │  │
│ └─────────────────────────────┘  │
│ Layer toggles…                   │
└──────────────────────────────────┘
```

---

## 3. Sim session rules

| Rule | Spec |
|:---|:---|
| Enter Simulation | Info panel **collapsed** |
| First open | Map tab · legend visible |
| Esc | panel stays state — no force close |
| Copy | locked registry §overlay |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
