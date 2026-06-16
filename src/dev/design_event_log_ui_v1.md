# Player event log UI `v1` (DESIGN-EVENT-LOG-001 / EVENT-LOG-UI-001)

| Field | Value |
|:---|:---|
| **Program** | **DESIGN-EVENT-LOG-001** (designer) · **EVENT-LOG-UI-001** (coder) |
| **Parent** | **PLAN-SIM-EFFECT-SPINE-001** · P3 |
| **Date** | 2026-06-12 |
| **Owner** | `@designer` (charter) · `@coder` (wire) |
| **Verdict** | **PASS** |
| **Guide** | [`guide_sim_effect_spine_v1.md`](guide_sim_effect_spine_v1.md) §Observe plane |
| **Exec** | [`plan_sim_effect_spine_exec_001_v1.md`](plan_sim_effect_spine_exec_001_v1.md) §P3 |
| **Witness** | [`debug_runs/design_event_log_ui_live.json`](../debug_runs/design_event_log_ui_live.json) |
| **Prereq** | **SIM-EFFECT-TEL-001** (`SimEffectTelemetryLedger` on disk) |
| **Unblocks** | **EVENT-LOG-UI-001** · **FACTION-REACT-001** (read model) |

**No Rust in this doc.** Observation-plane UX + adapter contract only.

---

## Mission

Players and operators need an **RTS-style structured event log** — tick, category, severity, target — derived from sim-effect telemetry. **Not** prose-first narrative, **not** F3 diagnostics, **not** designer pressure tooling.

**Acceptance test:** *Lightning or grid overload fires in normal play → within one tick the Events tray shows a row with tick + category + target chunk — without opening F3.*

---

## Authority model (observe plane)

```text
SimEffectQueue (drain)     → SimEffectTelemetryLedger (P1 telemetry — engineer/analytics)
                                    │
                                    ▼  projection adapter (PostUpdate, after drain)
                             PlayerEventLog (capped RAM ring — player read)
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
            ContextTray Events   OpsStrip ALERTS   AI snapshot (P3.2 stub)
```

| Surface | Reads | Must NOT |
|:---|:---|:---|
| `SimEffectTelemetryLedger` | adapter only | UI panels query ledger directly |
| `PlayerEventLog` | HUD systems | write sim / enqueue effects |
| `NarrativeObservationBus` | separate P6 | replace structured log |
| `StrategicEmergenceLog` | pressure composer only | player HUD |

**Rule:** Sim systems **never** subscribe to UI. UI **never** mutates `SimEffectQueue`. Projection runs **once per frame** on drain delta only.

---

## 1. Row schema — `PlayerEventRow`

Mapped from [`SimEffectTelemetryRecord`](../src/sim/effects/telemetry.rs) + kind decode:

| Field | Type | Player read | Example |
|:---|:---|:---|:---|
| `tick` | `u64` | When | `1240` |
| `category` | enum | Filter chip | `FIRE` · `GRID` · `WEATHER` · `BUILD` · `SCRIPT` |
| `severity` | enum | Color + ops priority | `INFO` · `WARN` · `CRIT` |
| `target_ref` | string | Pan/ping anchor | `ch(12,34)` · `cell(12,34,#2)` |
| `label` | string | One line, **structured** | `Lightning strike · 3 cells` |
| `effect_id` | `u64` | Causal drill-down (P3.1) | `#22` |
| `parent_id` | `Option<u64>` | Chain link | `#21` |
| `dispatch_ok` | `bool` | Strikethrough if false | failed adapter |

### 1a. Category map (`SimEffectSource` → player)

| `SimEffectSource` | Player category | Tray filter id |
|:---|:---|:---|
| `Lightning` | **WEATHER** | `weather` |
| `GridOverload` | **GRID** | `grid` |
| `Ecology` | **FIRE** | `fire` |
| `Construction` | **BUILD** | `build` |
| `ScenarioScript` | **SCRIPT** | `script` |
| `SimEffectTest` | *(hidden in sim)* | — |

### 1b. Severity map (`SimEffectKind` + dispatch)

| Kind | Default severity | Upgrade to CRIT when |
|:---|:---|:---|
| `LightningStrike` | **WARN** | `spark >= 0.8` or cells ≥ 8 |
| `IgniteCells` | **WARN** | heat max ≥ 0.7 |
| `StructureHeat` | **CRIT** | always (transformer / overload) |
| `HydroDirty` | **INFO** | — |
| `dispatch_ok == false` | **INFO** | show as failed · muted |

### 1c. Label templates (locked — no NL generator)

| `kind_tag` | Template |
|:---:|:---|
| 1 (IgniteCells) | `Fire ignition · {n} cells` |
| 2 (Lightning) | `Lightning strike · {n} cells` |
| 3 (Hydro) | `Hydrology update · {cause_id}` |
| 4 (StructureHeat) | `Structure overload · {cause_id}` |

**Forbidden:** full-sentence story text on hot path — defer to **NARRATIVE-GEN-001** (P6).

---

## 2. Storage — RAM ring first

| Parameter | Value | Rationale |
|:---|:---:|:---|
| `PLAYER_EVENT_LOG_CAP` | **512** | Within guide 256–1k band |
| Dedupe window | **30 ticks** | same `cause_id` + `kind_tag` → suppress repeat |
| Export | none in P3 | JSONL stays on telemetry ledger (dev) |
| Embedded DB | **🧊 defer** | GAME-STORE-GATE◈ — not P3 |

```text
PlayerEventLog {
  rows: VecDeque<PlayerEventRow>   // newest at back
  unread_crit: u32                 // ops strip badge
  last_projected_effect_id: u64    // adapter cursor
}
```

---

## 3. HUD surfaces

### 3a. Context tray — new tab **Events** (primary)

Add `ContextTrayTab::Events` — fifth tab after Alerts.

| State | Behavior |
|:---|:---|
| Sim enter (PLAY-01) | Tray **collapsed**; Events tab **not** auto-open |
| Expanded + Events | Scroll list, **newest first**, mono 11px |
| Row height | 1 line default; 2 lines if `parent_id` shown |
| Empty | `No events yet · sim effects will appear here` |
| Filter chips (P3.0) | ALL · FIRE · GRID · WEATHER · BUILD |

**Row format (locked):**

```text
T{tick}  [{category}]  {label}  @ {target_ref}
```

Example:

```text
T1240  [WEATHER]  Lightning strike · 3 cells  @ ch(12,34)
T1241  [FIRE]     Fire ignition · 2 cells  @ ch(12,34)  ←#22
```

Parent link suffix `←#parent` when `parent_effect_id` present (causal chain teaser).

### 3b. Ops strip — ALERTS zone (secondary)

Keep mission count. **Append** latest **CRIT/WARN** when unread:

| Priority | Template |
|:---|:---|
| CRIT present | `ALERT · {category} · {short_label} · T{tick}` |
| WARN only | `WARN · {short_label}` |
| Idle | existing `ALERTS · msn {n} · …` only |

**Short label:** first 28 chars of `label` — no wrap.

Clear `unread_crit` when user opens Events tab or expands tray on Events.

### 3c. Explicit non-surfaces

| Surface | Status |
|:---|:---|
| F3 diagnostics | **No** duplicate log — link line only: `SimEffect rows: {n}` |
| Pressure composer log | **Unchanged** — emergence log stays dev-facing |
| Transmission / narrative | **P6** — may *quote* event rows later, not replace |
| Minimap ping on row click | **P3.1 defer** |

---

## 4. Interaction (P3.0)

| Action | Result |
|:---|:---|
| Expand tray → Events | Mark crit unread cleared |
| Click row (P3.1) | Pan main map to `target_ref` chunk center |
| Esc | Collapse tray (existing PLAY-01) |
| Filter chip | Client-side filter on `category` only |

**No** row click commit · **no** sim mutation from log.

---

## 5. Accessibility

| # | Requirement |
|:---:|:---|
| A1 | Category in **brackets + word** — `[FIRE]` not color-only |
| A2 | Severity uses **weight + prefix** — `CRIT` / `WARN` in ops strip |
| A3 | Failed dispatch rows show **failed** text, not hidden |
| A4 | Filter chips keyboard-focusable when tray expanded |
| A5 | Empty state is text — not blank panel |

---

## 6. Acceptance (operator + lib)

| Probe | Pass |
|:---|:---:|
| Drain produces ≥1 telemetry row → ≥1 player row same tick | ✓ |
| Ring cap enforced at 512 | ✓ |
| Dedupe suppresses spam within 30 ticks | ✓ |
| Events tab shows structured rows in sim | ✓ |
| Ops strip shows CRIT one-liner when unread | ✓ |
| `--test` harness rows hidden (`SimEffectTest`) | ✓ |
| No UI query of fire ECS / chunk weather | ✓ |

---

## 7. Coder handoff — EVENT-LOG-UI-001

```text
Read:  src/dev/design_event_log_ui_v1.md
       src/sim/effects/telemetry.rs
       src/gui/hud/simulation_shell_phase2.rs
Touch: src/sim/effects/ (PlayerEventLog + project adapter) — OR src/gui/hud/ if observation stays gui-owned
       simulation_shell_phase2.rs — ContextTrayTab::Events + body scroll
       in_game_hud.rs — spawn Events tab chrome
Do:    PostUpdate adapter after sim effect drain witness
Do NOT: UI → SimEffectQueue · ledger in egui every frame · embedded DB
Verify: cargo test -p proc_A_dine01 --lib sim::effects::
Witness: debug_runs/design_event_log_ui_live.json (flip impl_wired keys)
```

### Slice order

| Step | Deliverable |
|:---:|:---|
| 1 | `PlayerEventLog` + projection from drain delta |
| 2 | `ContextTrayTab::Events` + scroll body |
| 3 | Ops strip CRIT/WARN append |
| 4 | Lib tests: cap, dedupe, category map |
| 5 | Witness JSON green |

---

## 8. Non-goals (P3)

- Embedded DB / save-load log history
- Full causal graph panel (P3.1+)
- Minimap ping animation
- NARRATIVE-GEN prose
- FACTION-REACT behavior (P5 — reads same rows later)

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-12 |
| `@coder` | pending | — |
| `@planner` | SIGNED (parent spine) | 2026-06-11 |

```text
DESIGN-EVENT-LOG-001 complete
Verdict: PASS
Doc: src/dev/design_event_log_ui_v1.md
Unblocks: EVENT-LOG-UI-001
ΔWF→@coder EVENT-LOG-UI-001
```

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-12 | Initial PASS — Events tray + ops strip + RAM ring |
