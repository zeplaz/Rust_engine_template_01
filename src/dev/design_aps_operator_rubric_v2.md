# APS operator pixel rubric `v2` — MIN window walk

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-OPERATOR-RUBRIC-002** |
| **Program** | PLAN-MULTI-PARALLEL-TRACKS-001 · T1 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_aps_interaction_v1.md`](design_aps_interaction_v1.md) · [`design_aps_onboard_spec_v2.md`](design_aps_onboard_spec_v2.md) · [`design_aps_preview_v2_spec_v1.md`](design_aps_preview_v2_spec_v1.md) |
| **Needs display** | true — operator session |
| **Verdict** | **PASS** |

```text
DES-APS-OPERATOR-RUBRIC-002 Q✓
Pixel walk — 1280×720 MIN · Buildings + Landscape · preview thumbs
```

---

## 0. Session setup

| Setting | Value |
|:---|:---|
| Window | **1280×720** (MIN) |
| Domain | Buildings first, then Landscape |
| Fresh prefs | Delete `onboarding_seen_v1` once for welcome pass |
| Capture | `assets/vfx/reference/aps_rubric_v2/` |

---

## 1. Buildings walk (≤15 min)

| # | Step | Pass if |
|:---:|:---|:---|
| B1 | Launch APS | Welcome card visible · metadata collapsed |
| B2 | `Start on Catalog` | Tab 0 · spine `▣ Catalog` |
| B3 | Select module | P1 thumb ≤300ms or `⟳ Rendering…` |
| B4 | State strip | Clean/Night/Damaged/Burning all labelled |
| B5 | Materials tab | Selection restored from B3 |
| B6 | Assembly generate | Inline `⟳` on button · no freeze >100ms silent |
| B7 | Disabled spine click | `✗` reason toast — no silent fail |
| B8 | Setup collapse | Manual fallback collapsed · no clip at MIN |
| B9 | Status log | Cap height — notebook tabs reachable |
| B10 | Help → getting started | Welcome returns non-modal |

---

## 2. Landscape walk (≤10 min)

| # | Step | Pass if |
|:---:|:---|:---|
| L1 | Switch domain | Landscape welcome once |
| L2 | Presets tab | Empty-state or list readable |
| L3 | Grammar tab | Tier chip shows G0 copy if applicable |
| L4 | Preview | At least P1 thumb or honest empty |
| L5 | Atlas spine | Step visible — bake may warn at G0 |

---

## 3. Anti-patterns (instant fail)

| # | Fail |
|:---:|:---|
| F1 | Engineer id in primary combo label |
| F2 | Two `▣` current indicators |
| F3 | Primary action ack log-only |
| F4 | Welcome modal blocks window resize |
| F5 | Preview thumb unlabelled state |

---

## 4. Scorecard

| Band | Criteria |
|:---|:---|
| **9/10** | All B1–B10 + L1–L5 green · no F* |
| **7/10** | ≤2 minor clip/readability issues documented |
| **≤6/10** | Any F* or >3 blockers — return to @coder-mcp |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** (rubric ready) | 2026-06-20 |
| Operator | pending session | — |
