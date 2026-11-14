# PLAYCHECK-1 — UAS + missile industrial / sensory loop

| Field | Value |
|:---|:---|
| **Id** | **PLAYCHECK-1** |
| **Parent** | [`plan_ref_books_upgrade_use_v1.md`](plan_ref_books_upgrade_use_v1.md) §5 Order 8 gate |
| **Status** | **SHIPPED** (2026-11-14) |
| **Unlocks** | **HAUL-AMPLIFY-1** — [`plan_haul_amplify_v1.md`](plan_haul_amplify_v1.md) |

## Goal

Prove one real `--test visual` run dumps countable evidence for the shipped UAS + missile path and FIDELITY sensory stack, then score it with `tools/sim_review` (no `*_green` in-engine).

## Exit (playcheck-green)

1. `python tools/sim_review/playcheck.py` completes without panic; writes `debug_runs/sim_run_dump.json`.
2. `debug_runs/sim_review/matrix.json` — **0 fail** on required rows (see below).
3. Dump `fidelity` block present with series for missile / blast / audio / trucks (counts may be 0 if scene never enqueues sorties — presence still required).
4. Existing logistics/economy/weather soft rows still pass.
5. Stamp this doc **SHIPPED** + plan Order 8 unblocked for haul-amplify charter.

**Witness:** longer playcheck (`--min-secs 360`) — panics 0; **60 pass / 3 fail**; all required fidelity + `weather.ao_count_tracked` pass; `weather.solved` pass. Residual soft fails (out of PLAYCHECK scope): `logistics.unserved_draw` (=2), `perf.debug.avg_ms` / `perf.debug.p95_ms` (debug sandbox). Dashboard: `debug_runs/sim_review/dashboard.html`.

## Required matrix ids (after fidelity dump lands)

| Id | Intent |
|:---|:---|
| `fidelity.samples` | Series collector ran |
| `fidelity.missile_dump_present` | `MissileSortieDump` hooked (max ≥ 1) |
| `fidelity.ballistic_flights_tracked` | `BallisticFlightBook` hooked |
| `fidelity.blast_craters_tracked` | `BlastCraterDump` hooked |
| `fidelity.audio_mix_requests_tracked` | `AudioMixFrame` hooked |
| `fidelity.truck_proxies_tracked` | `TruckPresentationIndex` hooked |
| existing `weather.ao_count_tracked` | Mission dump present |

## Out of scope

Haul-amplify impl, SCH Wave 2+, WeatherField rewrite, skeleton/MM, forcing non-zero sorties via cheat writers.

## How to run

```powershell
python tools/sim_review/playcheck.py
# or shorter smoke:
python tools/sim_review/playcheck.py --min-secs 120 --secs 240 --ticks 2000
```
