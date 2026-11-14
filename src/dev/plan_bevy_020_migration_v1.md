# PLAN-BEVY-020-MIG-v1 — Bevy 0.19 → 0.20 (TOP PRIORITY)

| Field | Value |
|:---|:---|
| **Id** | **PLAN-BEVY-020-MIG-v1** |
| **Status** | **PROGRAM CLOSE 2026-11-13** — Bevy **0.20** on master; gate `debug_runs/mig_bevy_020/mig_v2_gate.json` |
| **Date** | 2026-11-13 |
| **From → To** | Bevy **0.19** → **0.20** |
| **Guide** | https://bevy.org/learn/migration-guides/0-19-to-0-20/ |
| **Companion** | [`plan_ref_books_upgrade_use_v1.md`](plan_ref_books_upgrade_use_v1.md) — book/ECS hygiene **after M0 SIGN-OFF** |
| **Gate JSON** | `debug_runs/mig_bevy_020/mig_v2_gate.json` (written at PROGRAM CLOSE) |
| **Authority** | [bevy-simulation-grade](../../.cursor/skills/bevy-simulation-grade/SKILL.md) · [`07-repo-authority-map.md`](../../.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md) |
| **Prior** | [`plan_bevy_019_migration_v1.md`](plan_bevy_019_migration_v1.md) — **CLOSED**; do not reopen MIG-0.19 rows |

## Sign-off stamps

| Phase | Stamp (fill on exit) |
|:---|:---|
| **M0** | **M0 SIGN-OFF 2026-11-13** — cargo 0 errors; lib 1558 pass / 25 defer ([`mig_bevy_020_m0_test_defer.md`](mig_bevy_020_m0_test_defer.md)) |
| **M1** | **M1 SIGN-OFF 2026-11-13** — ECS extract/observers/pointer/`If`/`or_else`; [`mig_bevy_020_m1_weak_order_note.md`](mig_bevy_020_m1_weak_order_note.md); authority map **0.20** |
| **M2** | **M2 SIGN-OFF 2026-11-13** — `#[extract_app(RenderApp)]`, `ViewDepthStencilTexture`, Vertex/Fragment `constants`; cargo 0 errors (playcheck deferred to operator; no extract panic at compile) |
| **M3** | **M3 SIGN-OFF 2026-11-13** — `PointerClick` flatten; `ui_widgets::Button`; `or_else` run_if |
| **M4** | **M4 SIGN-OFF 2026-11-13** — **N/A** (no `bsn!` product call sites in `src/`) |
| **M5 / PROGRAM CLOSE** | **PROGRAM CLOSE 2026-11-13** — ecosystem bumped; `mig_v2_gate.json` written; AGENTS.md on 0.20 |

## Rule

1. **Engine bump first (M0)** — master compiles and lib-tests on Bevy 0.20; **M0 SIGN-OFF** before book Slice A code.
2. **Linear gates** — do not start Mn+1 until Mn SIGN-OFF is stamped below.
3. **Every later stage** does a **0.20 feature sweep** in that domain while touching it.
4. Preserve **Sim → View → Render → UI**, one writer per resource, named **SystemSets** (note 0.20 **weak set ordering**).
5. Ecosystem crates gate on crates.io **0.20** — **DR-MIG-020-ECO**; core still bumps.
6. **Precedence: 0.20 always wins.** Legacy = opportunistic only (§ Legacy).

## Legacy from older migrations (keep in mind — **not** a priority)

While editing a file for **M0–M5** or a book slice, if you notice leftover **0.17 / 0.18 / 0.19-era** patterns that 0.20 replaces, clean them **opportunistically in that same PR** — only when cheap and on the touched path. Never delay M0 for a purge lane. Do not reopen closed **DR-MIG-*** 0.19 / `MIG-*` rows.

## Hot migration themes (from official guide)

| Theme | Why it hits *us* | Sweep stage |
|:---|:---|:---|
| **Weak schedule set ordering** | SystemSet chains / SCH / Slice A | **M0 + M1** |
| **Sprite → Mesh2d + SpriteMaterial** | Map/air sprites | **M2** |
| **Observers / exclusive / WorldQuery / FilteredResources** | ECS APIs | **M1** |
| **WESL shaders** | Owned fire/atmos shaders | **M2** (or DR-MIG-020-WESL) |
| **Pointer / UI text / Val** | Play UI + pick | **M3** |
| **BSN syntax** | City grammar if used | **M4** |
| **bevy_shape / bevy_curve splits** | Math imports | **M0** |

## Phase SIGN-OFF exits (linear)

### M0 — Engine bump

**Owner:** `@coder`

**Do:** `bevy = "0.20"` (+ matching deps); fix compile; gate lagging optional crates.

**SIGN-OFF M0 (all required):**

1. `validate_cargo_report` / `validate-report cargo` → **0 errors**
2. `cargo test -p proc_A_dine01 --lib` green **or** failures listed in [`mig_bevy_020_m0_test_defer.md`](mig_bevy_020_m0_test_defer.md)
3. Lockfile / Cargo.toml show Bevy **0.20**
4. Stamp **M0 SIGN-OFF** in table above; close **DR-MIG-020-M0**

**Unblocks:** M1 · book Slice A code

### M1 — ECS / schedule (+ book Slice A)

**Owner:** `@coder` + `@sim-steward`

**Do:** Observers, exclusive/`SystemAccess`, WorldQuery, FilteredResources→Query; weak-order audit; authority map **0.20** note; Slice A graph on 0.20 APIs.

**SIGN-OFF M1:** cargo 0 errors · steward weak-order note (paths/sets) · authority map says 0.20 · stamp **M1 SIGN-OFF**

**Unblocks:** M2

### M2 — Render / extract

**Owner:** `@coder`

**Do:** Sprite/Mesh2d where touched; render imports; transmission if needed; FireVisualFrame after view snaps; WESL only if blocking.

**SIGN-OFF M2:** cargo 0 errors · scoped render tests · playcheck or short run + sim_review smoke · stamp **M2 SIGN-OFF**

**Unblocks:** M3

### M3 — UI / input

**Owner:** `@coder` (+ `@designer` if pick/HUD)

**Do:** Flat pointer events; Text/EditableText; Val units; play UI only.

**SIGN-OFF M3:** cargo 0 errors · gui/input lib tests · pan/zoom + one panel or playcheck · stamp **M3 SIGN-OFF**

**Unblocks:** M4

### M4 — BSN / scenes

**Owner:** `@coder`

**Do:** BSN `@` / `--` migrate **or** document N/A if no product BSN call sites.

**SIGN-OFF M4:** cargo green + migrated **or** N/A noted · stamp **M4 SIGN-OFF**

**Unblocks:** M5

### M5 — Ecosystem + PROGRAM CLOSE

**Owner:** `@coder` / steward

**Do:** Bump or DEFER hanabi/tilemap/vector_shapes; write gate JSON; update AGENTS.md.

**SIGN-OFF M5 / PROGRAM CLOSE:**

1. **DR-MIG-020-ECO** closed or DEFER with reason
2. Final validate_cargo 0 errors (default features)
3. Lib tests green or remaining defer list
4. Write `debug_runs/mig_bevy_020/mig_v2_gate.json` (`gate_pass`, `bevy`, `phases`)
5. AGENTS.md: Bevy **0.20** on master; 0.20 mig closed
6. Stamp **PROGRAM CLOSE**

## Per-stage checklist (each PR)

```text
[ ] Compiles on bevy 0.20 (M0 SIGN-OFF done for post-M0 work)
[ ] 0.20 takes precedence over any legacy cleanup
[ ] Touched SystemSets: strict edges where interior mutability / channels (weak-order)
[ ] No new dual writers; Sim↛UI inverted
[ ] Domain-relevant 0.20 APIs adopted if this PR edits that code
[ ] Opportunistic legacy only on touched paths
[ ] validate_cargo_report + scoped lib tests
[ ] Migration guide section cited in note
```

## Deferral IDs

| Id | Item | Unblock |
|:---|:---|:---|
| **DR-MIG-020-M0** | Engine on 0.20 compile-green | M0 SIGN-OFF |
| **DR-MIG-020-ECO** | Optional crates 0.20 | crates.io + steward |
| **DR-MIG-020-WESL** | Full shader port | After M0; may lag M2 |
| **DR-MIG-020-LEGACY** | Opportunistic pre-0.20 leftovers | Never blocks |

## Agent routing

- **P0:** `@coder` M0 → stamp SIGN-OFF before Slice A code.
- `@sim-steward`: M1 weak-order / dual-writer.
- Do **not** pick closed 0.19 `MIG-*`.
- Legacy: same-PR only; never outrank 0.20.
