# `debug_runs/` — sim review output

**Review path (only):** `sim_run_dump.json` (debug-only, `src/dev/sim_run_dump.rs`) → `python tools/sim_review/score_dump.py` → `sim_review/dashboard.html`. One-shot: `python tools/sim_review/playcheck.py` (build, run game, capture, score). DamFlood: `python tools/sim_review/playcheck.py --scene dam`. Full command list: [`docs/TESTING.md`](../docs/TESTING.md).

The witness / proof / readiness / gate JSON system (`*_live.json`, `_agent_meta` envelope, `unified_witness_index.json`) was deleted. Do not add new writers or read old `*_live.json` files — they are stale and untrusted.

Other files here: `orchestrator_thread_health.json` (`ORCHESTRATOR_EXPORT_HEALTH=1` only), `captures/` (`VFX_CAPTURE=1` real PNGs), `perf_frames/` (instrumentation), `mig_bevy_019/` (historical).
