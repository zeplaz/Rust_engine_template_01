# Sim HUD ops strip `v2` — alert tiers, overflow, sort

| Field | Value |
|:---|:---|
| **ID** | **DES-SIM-HUD-OPS-002** |
| **Program** | PLAN-MULTI-PARALLEL-TRACKS-001 · T5 SIM-HUD |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Supersedes** | [`docs/archive/2026-06-src-dev/plans/design_sim_hud_ops_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_sim_hud_ops_v1.md) (v1 shipped — v2 adds tiers + overflow) |
| **Related** | [`design_power_grid_copy_v1.md`](design_power_grid_copy_v1.md) · IND-E03 overload |
| **Handoff** | COD-SIM-HUD-OPS-002 |
| **Verdict** | **PASS** |

```text
DES-SIM-HUD-OPS-002 Q✓
Ops strip v2 — alert sort · tier badges · PWR island · overflow ellipsis
```

---

## 0. Scope

Top **Operations strip** (Bevy native) — readability at 1280×720 and 2560×1440.

**Out of scope:** context tray body · floating egui panels · multiview parity.

---

## 1. Layout (unchanged height)

```text
[TIME  T+01234  RUN  ×1.0]  [◆2  ALERTS  2]  [INTEL  routes on]  [WX  r0.2 s0.1]  [PWR  78%]  [▼ TRAY]
```

**Font:** ≥11px mono on all zone bodies (v1 retained).

---

## 2. Alert tiers (v2)

| Tier | Glyph | Sort priority | Examples |
|:---|:---:|:---:|:---|
| **P0 critical** | `◆` | 0 | Grid overload · meltdown scram |
| **P1 warn** | `▲` | 1 | Island offline · damaged segment |
| **P2 info** | `●` | 2 | Repair queued · tool hint |

**ALERTS zone:** `{badge}  ALERTS  {n}` — count = P0+P1 only (P2 optional peek in tray).

**Sort:** highest tier left in tray list; strip shows aggregate count.

---

## 3. PWR zone extensions

| State | Copy |
|:---|:---|
| Idle | `PWR  {pct}%` |
| Overload | `PWR  ⚠ Grid overload — {n} segments` (IND-E03) |
| Island | `PWR  ○ Island — {n} offline` |
| Toast pairing | Toast body matches strip prefix for 8s then revert idle |

---

## 4. Overflow rules

| Width | Behavior |
|:---|:---|
| ≥1920 | All zones full copy |
| 1280–1919 | WX abbreviates `WX r{s} s{w}` · INTEL may drop `routes` word |
| &lt;1280 | Drop WX first · keep TIME + ALERTS + PWR + TRAY |

**Ellipsis:** only after 48 chars in toast — not in strip zones.

---

## 5. Flex priority (coder)

| Zone | Flex |
|:---|:---|
| TIME | fixed min |
| ALERTS | grow center |
| PWR | fixed min |
| INTEL / WX | shrink first |
| TRAY | fixed |

---

## 6. Witness (COD)

`debug_runs/sim_hud_ops_v2_live.json`:

```json
{
  "ops_strip_font_min_px": 11,
  "alert_tier_sort": true,
  "pwr_island_copy": true,
  "overflow_1280_green": true
}
```

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
