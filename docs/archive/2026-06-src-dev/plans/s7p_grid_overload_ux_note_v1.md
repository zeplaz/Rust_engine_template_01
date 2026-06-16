# S7P grid overload — player feedback routing `v1` (S7P-DESIGN-002)

| Field | Value |
|:---|:---|
| **Queue ID** | **S7P-DESIGN-002** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@designer` |
| **Status** | **SIGNED** |
| **Read** | [`power_damage_ui_persistence_v1.md`](../docs/reference/designer_questions/production_economy/power_damage_ui_persistence_v1.md) · [`industrial_grid_overload_impl_plan_v1.md`](industrial_grid_overload_impl_plan_v1.md) |
| **Unblocks** | **S7P-GRID-UX-UI-001** (Coder B **B3**) · optional **IND-E03-SIM-UX-001** |
| **Code anchor (v1)** | [`grid_overload_ux.rs`](../economy/activation/grid_overload_ux.rs) |

---

## Decision — one surface primary, two secondary

When a **smelter cluster** (or grid bus) trips **`GridOverloadEvent`** in **Simulation**, the player gets feedback in this order:

| Priority | Surface | Use for overload | Rationale |
|:---:|:---|:---|:---|
| **1 — Primary** | **Ops-strip PWR zone** (Bevy text, ~8s) | First-hit + sustained reminder while event window active | Always visible in sim PLAY-01; no egui product shell |
| **2 — Secondary** | **Alerts tray** (ops strip affordance) | Dismissible row: severity **Warning**, sortable list | Matches tiered alerts in power_damage brief; persists until dismissed |
| **3 — Tertiary** | **Diagnostics** (collapsed by default in sim) | `overload_events_total`, bus id, tick — engineer only | Never the only channel; avoids false “silent” overload |

**Rejected for v1**

| Option | Why not |
|:---|:---|
| Floating egui toast in sim | Violates PLAY-01 / Phase 2B egui gate |
| Modal blocking dialog | Breaks continuous sim; overload is recoverable |
| Minimap-only indicator | Too easy to miss during Portland chain play |

---

## Copy (canonical strings)

### Primary — ops strip PWR line

Shown while toast state active ([`GRID_OVERLOAD_TOAST_TICKS`](../economy/activation/grid_overload_ux.rs) = 240 ticks ≈ 8s @ 30 Hz):

```text
PWR  ⚠ Grid overload — reduce smelter load or add transformer capacity
```

**Shorter alt** (narrow layout / future localization key `grid.overload.toast.short`):

```text
PWR  ⚠ Grid overload — shed load or upgrade transformers
```

**Coder B3:** wire **`GRID_OVERLOAD_TOAST_MESSAGE`** to the **body** after the `PWR  ⚠` prefix (already close — align exact string above).

### Secondary — alerts tray row

| Field | Value |
|:---|:---|
| **Title** | Grid overload |
| **Body** | Smelter demand exceeded bus capacity. Reduce load or place a distribution transformer. |
| **Severity** | Warning (amber) — not Critical unless brownout cascade ships |
| **Dismiss** | User may hide; re-fire on next `GridOverloadEvent` |

### Tertiary — diagnostics (one line)

```text
Grid: overload_events={n}  last_bus={id}  tick={t}
```

---

## Behavior rules

| Rule | Spec |
|:---|:---|
| **First event** | Primary toast fires immediately; tray row enqueued same tick |
| **Repeat while overloaded** | Refresh primary window; do **not** stack modal spam — max **1** tray row per bus per 30s (coder throttle) |
| **Recovery** | Clear primary when load < threshold for 2s; tray row remains until dismissed |
| **Editor / WorldGen** | No toast — witness-only paths OK |
| **Simulation HUD** | Collapsed diagnostics; overload never requires opening F3 first |

---

## Witness / coder exit (B3)

| Field | Target |
|:---|:---|
| Lib / live | `s7p_grid_ux_toast_ui_wired: true` |
| Green | `s7p_grid_ux_001_green` when `overload_events_total > 0` or toast `show_count > 0` |
| Visual | PWR line visible in `--test visual` with `RUST_ENGINE_IND_E03_SEED=1` or `RUST_ENGINE_STAGE7_PLAY_SEED=1` |

```powershell
cargo test -p proc_A_dine01 --lib grid_overload
cargo test -p proc_A_dine01 --lib industrial_activation
```

---

## Sign-off

| Role | Date | Verdict |
|:---|:---|:---|
| Designer | 2026-05-25 | **SIGNED** — routing + copy for B3 |
| Coder B3 | — | Implement tray row + string alignment per **S7P-GRID-UX-UI-001** |

---

## Document history

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **S7P-DESIGN-002** — toast vs tray vs diagnostics |
