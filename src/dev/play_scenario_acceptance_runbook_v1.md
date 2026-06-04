# G-PLAY-01 — Play scenario acceptance runbook `v1`

| Field | Value |
|:---|:---|
| **Gate** | **G-PLAY-01** |
| **Designer** | **DESIGN-G-PLAY-001** |
| **Planner** | **PLAN-G-PLAY-001** (shared deliverable) |
| **Scenario** | `PlayScenarioId::DefaultIndustrial` ([`play_scenario.rs`](../engine/play_scenario.rs)) |
| **Version** | `1.1.0` |
| **Date** | 2026-06-02 |
| **Audit** | [`planner_status_audit_v18.md`](planner_status_audit_v18.md) |
| **Owner** | `@designer` + `@planner` |
| **Verdict** | **PASS (qualified)** — operator must execute once for gate close |
| **Unblocks** | **G-PLAY-001-BLOCKERS** · **OPS-PLAY-001** tail · **PLAY-TRUTH-001-TAIL** |
| **No Rust** | Operator + designer acceptance script |

---

## Purpose

**G-PLAY-01** is the ship bar for **10 minutes of unaided default industrial play** in **release Simulation** without harness bootstrap (`--test visual`, `test_harness`, or `RUST_ENGINE_STAGE7_PLAY_SEED`).

Lib fixture green ≠ this gate. Witness JSON is evidence, not a substitute for the manual run.

---

## Preconditions

| # | Requirement |
|:---:|:---|
| P1 | `cargo build -p proc_A_dine01 --release` succeeds |
| P2 | Launch **without** `--test visual` and **without** `RUST_ENGINE_STAGE7_PLAY_SEED` |
| P3 | Reach `BaseState::Simulation` with generated world ≥ **32×32** tiles |
| P4 | `ActivePlayScenario` = **DefaultIndustrial** (default resource) |

**Recommended launch:**

```powershell
.\tools\orchestrator\scripts\run_visual_test_clean.ps1 -Release -NoStayOpen
# Or product entry that enters WorldGen → Full world → Simulation
cargo run -p proc_A_dine01 --release
```

**Forbidden for G-PLAY-01 sign-off:** `--test visual`, `ActiveTestScene` harness paths, env seeds that auto-complete construction.

---

## Session clock

| Phase | Duration | Cumulative |
|:---|:---|:---|
| Setup (world gen → sim enter) | ≤ 5 min | 5 min |
| Play (build + observe logistics) | ≥ 10 min | 15 min |
| Pause / resume smoke | ≤ 1 min | — |

**Gate:** uninterrupted Simulation play **≥ 10:00** after enter, no panic, no soft-lock.

---

## Acceptance checklist (operator)

| ☐ | # | Action | Pass criteria | Fail |
|:---:|:---:|:---|:---|:---|
| ☐ | 1 | Enter **Simulation** | Tactical map visible; PLAY-01 chrome ([`simulation_session.rs`](../gui/hud/simulation_session.rs)) — collapsed tray, no WorldGen panel | Editor shells overlay sim |
| ☐ | 2 | Confirm **no egui Construction window** | Only Bevy **build rail** (Rd/Rl/Ut/…) — see [`ui_construction_playtest_v1.md`](ui_construction_playtest_v1.md) | Full Construction floating window open |
| ☐ | 3 | Place **concrete_aggregate_mine** near Portland origin | Ghost + commit via construction funnel; no harness auto-build | Instant buildings without tool |
| ☐ | 4 | Place **concrete_cement_kiln** + **concrete_mixer_plant** | Three sites on map; corridor/tool per playbook | Missing chain |
| ☐ | 5 | Advance sites toward **Operational** | At least one site operational OR clear progress UI | Stuck with no feedback |
| ☐ | 6 | Observe **logistics on minimap** | Heat/rows visible (`logistics_rows > 0` when witness refreshed) | Blank logistics layer |
| ☐ | 7 | **Pause** → menu → **Resume** | Sim resumes; camera stable | Panic / black screen |
| ☐ | 8 | Pan/zoom 10 min | No sustained viewport blink (VR-04); playable frame rate | Hard lock / repeated ortho flip |
| ☐ | 9 | Optional: place **one road segment** | Corridor phase overlay readable (R4 amber/blue) if R4 product active | N/A if tools gated |

---

## Designer acceptance (PLAY-01 alignment)

| Rule | Check |
|:---|:---|
| WorldGen preview dismissed in sim | No preview raster stealing focus |
| Scenario script panel closed | `map_editor` chrome off |
| Minimap movable | Floating window draggable ([`ui_construction_playtest_v1.md`](ui_construction_playtest_v1.md) §1) |
| Diagnostics collapsed | Verbose sections not default-open in sim |

---

## Witness evidence (optional refresh)

After session, operator may refresh proofs (not required to **start** G-PLAY-01):

| File | Fields | Expected |
|:---|:---|:---|
| `debug_runs/play_scenario_live.json` | `green`, `default_industrial` | `true` when writer wired |
| `debug_runs/construction_stage_live.json` | `operational_green` | `true` when chain complete |
| `debug_runs/minimap_compositor_live.json` | `logistics_rows` | `> 0` |
| `debug_runs/industrial_activation_live.json` | `activation_green` | `true` when production runs |

**G-PLAY-01 closes** when checklist §1–8 pass on disk log (operator sign-off row below) even if JSON not refreshed same day.

---

## Fail / stop rules

Stop clock and file blocker under **G-PLAY-001-BLOCKERS** if:

- Panic or `unwrap` dialog
- Cannot enter Simulation after 5 min setup
- Construction commit does not appear on map
- Pause menu broken
- Requires `--test visual` or env seed to progress

---

## Sign-off

| Role | Verdict | Date | Notes |
|:---|:---|:---|:---|
| `@designer` | **PASS (qualified)** — script ready | 2026-05-28 | Awaiting operator execution |
| `@planner` | **PASS** | 2026-06-02 | PLAN-G-PLAY-001 — audit v18 per-witness grades |
| Operator | ☐ **EXECUTED** | — | Fill date when checklist §1–8 complete |

**Qualified:** Gate **G-PLAY-01** remains **OPEN** until operator row signed. Lib `play_scenario_live.json` green is **PARTIAL** playability per audit v18 — not a substitute for this runbook.
