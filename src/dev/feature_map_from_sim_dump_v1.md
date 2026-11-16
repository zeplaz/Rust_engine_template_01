# Feature map from live `sim_run_dump` — why finish the spine

| Field | Value |
|:---|:---|
| **Source dump** | `debug_runs/sim_run_dump.json` (`sim_run_dump_v2`) |
| **Matrix** | `debug_runs/sim_review/matrix.json` — **60 pass / 10 fail / 4 skip** |
| **Dashboard** | `debug_runs/sim_review/dashboard.html` |
| **Dump scene** | `launch.test_scene: Visual` (not `BuildReadability` — refresh with `--test buildread` + `RUST_ENGINE_SIM_DUMP=1` for a buildread-scoped stamp) |
| **Date read** | 2026-11-15 |

**Compile warnings ≠ dump truth.** `cargo run` can be **stable** (0 errors) while printing a Bevy-0.20 `Interaction` deprecation wall + unused imports. The dump/matrix is how subsystems report whether they are **alive, starved, or silent**.

---

## 1. What the dump is saying (feature map)

```text
World generated + sim entered
  ├─ combat     fronts / formations / territory change  → PASS (engaged stack alive)
  ├─ weather    wind/rain/haul/sensor multipliers       → PASS (atmosphere coupled)
  ├─ economy    plants producing, storage filling       → PASS
  ├─ logistics  deliveries OK, but unserved_draw=2      → FAIL (honest starvation signal)
  ├─ construction sites UnderConstruction / Operational → COUNTS present (phase spine)
  ├─ fire chain ember/threat/collapse counters          → FAIL/0 (calm Visual — OK-if-zero notes)
  ├─ fidelity   audio mix, truck proxies sampled        → PASS (presentation listens)
  └─ perf.debug avg/p95                                 → FAIL (debug budget; not product release)
```

| Domain | Dump signal | Why it matters |
|:---|:---|:---|
| **Logistics** | `unserved_draw_demands: 2` (personnel draws unserved) | Malfunction **reports**: demand exists, no serving line — systems stay responsive instead of silent green |
| **Construction** | 4 sites, 3 under construction, 1 operational | Build funnel is on the clock; massing/ghost is **readability**, not this counter |
| **Combat / AI** | formations engaged, decisions issued | Strategic loop still ticks while we polish presentation |
| **Weather ↔ haul / air / radio / EO** | multipliers tracked | Cross-domain synergy: weather shifts should move logistics/combat feel — dump proves coupling |
| **Fire entity chain** | tracked counters 0 on calm Visual | Fail rows are **instrumentation honesty**, not “fire broken” — finish means counters move when heat is on |
| **Perf debug** | avg ~42 ms / p95 ~62 ms | Debug build opt-level 1; finish bar for product is **release** matrix (`when_build`) |

---

## 2. How the studio / massing / buildread work helps

| Finished piece | Dump / operator role |
|:---|:---|
| Grammar → `massing_preview` → `StudioMassingOverride` | Operator can **see** footprint height intent without lying to `FootprintMatrix` |
| `--test buildread` | Deterministic entry to non-rect pilots + PixelWorld chrome — same class of harness as Visual for **readability**, not VFX |
| Republic QC / `massing_export` | When presentation malfunctions (empty columns, bad seed), HTML reports it before play |
| Reject ledger (no voxel physics / AI-on-columns) | Keeps dump domains authoritative — we do not invent parallel occupation that would desync construction/logistics counts |

Presentation finished well → operators trust ghosts; sim finished well → matrix rows move when the world shifts. Both are required for **integrated, responsive** play.

---

## 3. Why “finish” — not leave half-wired

1. **Malfunction should shout.** `unserved_draw=2` is useful. A missing matrix row or a silent counter is not. Studio/massing without a demo path is the same class of silence.
2. **Synergy needs closed loops.** Weather multipliers only help if logistics/combat **read** them; grammar only helps if export → override → paint is one path operators can run.
3. **Warning walls hide real fails.** 100+ `Interaction` deprecations drown the 10 matrix fails. Hygiene (unused imports) + scheduled UI migration keep the signal channel clear.
4. **Scoped green.** Schema-green ≠ bake-green ≠ dump-green ≠ art-ship-green. Say which scope you closed.

---

## 4. Finish bar (next stamps)

| Action | Scope |
|:---|:---|
| Re-run dump under buildread | `$env:RUST_ENGINE_SIM_DUMP=1; cargo run -p proc_A_dine01 -- --test buildread` then `score_dump.py` |
| Logistics unserved draws | Product: serve or explicitly allowlist personnel front draws |
| Fire counters | Only prove under fire-seeded scene (`--test fire` / Visual with heat) — do not chase zeros on calm Visual |
| Perf | Judge on release + `sim_dump` feature, not debug avg |
| Warning hygiene | Unused imports now; Bevy `Interaction` → `Hovered`/`Pressed` as a **dedicated** slice (do not mix into massing) |

---

## 5. Compile warning triage (from operator paste)

| Bucket | Count (approx) | Finish? |
|:---|:---|:---|
| Deprecated `bevy_ui::Interaction` | Vast majority | Separate Bevy-0.20 UI migration program |
| Unused imports / unused_parens | ~15 listed | Cheap hygiene — clear signal |
| Deprecated `Mat4::orthographic_rh` | 1 | Render camera follow-on |

**Verdict:** build is **stable** (errors=0). Warnings are mostly **migration debt**, not evidence that buildread or the massing spine is broken. Use **dump + matrix** as the feature map; use warnings as a backlog, not a panic.
