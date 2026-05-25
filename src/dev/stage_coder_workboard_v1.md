# Coder workboard `v1` (active)

| Field | Value |
|:---|:---|
| **Version** | `1.0.0` |
| **Date** | 2026-05-24 |
| **Sign-off ledger** | [`stage_tracks_signoff_ledger_v1.md`](stage_tracks_signoff_ledger_v1.md) |
| **Copy-paste detail** | Per-track plans under [`stages/`](stages/) |

**Rule:** One primary slice per session (≤3 files). Check **DONE** — do not re-implement.

---

## Done — do not redo

| ID | Track |
|:---|:---|
| S7P-IND-001 | S7-PLAY |
| FX-WATER-SHADER-001/002, PARTICLE-001/002 | FX-WATER |
| FX-FIRE-SPARK-001…006 | FX-FIRE |
| P2-VFX-VISUAL-001 | VFX-P2 |
| P2-FIRE-SPARK-010 | VFX-P2 |
| UI-WP-LAYOUT-001 | UI-P4 |
| UI-P3-M1, M2, UI-P3-001 | UI-P3 |

---

## Primary queue (pick one)

### Lane A — render / WGSL

| Priority | ID | Copy-paste starter |
|:---:|:---|:---|
| 1 | **WATER-W1-OCEAN-001** | See [`stages/water_vfx_closure_plan_v1.md`](stages/water_vfx_closure_plan_v1.md) § Coder A ocean |
| 2 | **WATER-W1-RIVER-001** | Strategic river ribbon — `water_overlay.wgsl` |
| 3 | **P2-FIRE-SPARK-011** | `fire_spark_compute.wgsl` tuning — F-T01/F-T03 |

### Lane B — policy / witness / emission

| Priority | ID | Copy-paste starter |
|:---:|:---|:---|
| 1 | **WATER-W2-FOAM-001** | `gpu_water_particles.rs` — coast foam + bend foam (river_foam=1 partial) |
| 2 | **WATER-STRATEGIC-001** | D-W09: particles 0 @ strategic, shaders on |
| 3 | **WATER-WITNESS-001** | Harness gates for foam/ocean/strategic |
| 4 | **P2-VFX-WITNESS-001** | Close partial — tactical lib tests (already green) |

### Lane C — UI / shell

| Priority | ID | Notes |
|:---:|:---|:---|
| 1 | **UI-SHELL-REFRESH-001** | Replay sim interactions; fix witness only if code broken |
| 2 | **UI-P2A-F03** | Ops zone hover → `witness.ops_zone_hover_token: true` |
| 3 | **UI-P2A-P4-AUTH** | Already true in stale JSON for rail — verify on refresh |
| 4 | **UI-WP-LAYOUT-002** | **Blocked** until UI4-DESIGN-001 |

---

## UI-SHELL-REFRESH-001 (new)

**Problem:** `ui_shell_migration_live.json` shows `phase2b_closed: false` but `egui_pass_count_in_sim: 0` and code was green historically — likely **stale proof frame** (no interaction replay).

```
Lane: UI-SHELL-REFRESH-001
Read: src/dev/stage_tracks_signoff_ledger_v1.md § UI-P2 STALE
First: cargo run --test visual OR manual sim — expand tray, hover ops, click build rail, ESC
Do NOT: re-open Phase 2B architecture unless code regressed
Verify: ui_shell_migration_live.json → phase2b_closed, witness.* gated true
```

---

## Global regression

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Coder queue from sign-off ledger |
