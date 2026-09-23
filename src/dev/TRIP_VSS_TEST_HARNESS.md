# TRIP-VSS-HARNESS — Test harness ≠ game trip stub

```text
@orchestrator-status: ACTIVE
@orchestrator-owner: @coder
Program: VSS-001 · $ref:src/dev/plan_vfx_spectator_scale_program_v1.md
```

**You touched test harness, launch args, or ActiveTestScene.**

The harness is a **scaffold**, not product truth. Operators must see fire from:

- Save load
- Campaign state
- Scenario script (`play_scenario.rs`)
- SimEffect / ECS ignition — **not** `apply_test_scene_fire_seeds` alone

**Before merge:**

- [x] Entry path documented in witness (`entry_path`: `harness` | `scenario` | `save` | `interactive`) — VSS-T2-001
- [x] `spectator_parity_live.json` updated — scenario product path green (T2-002); save/interactive pending
- [ ] No new dual-write to `SharedOverlayFieldBuffers` from harness
- [ ] `symbolic_build_digest_tool` after build — agents reason on digest only

**Forbidden as sole proof:** `--test vfx` green without scenario parity.

**Dissent allowed:** If harness shortcut is necessary, file tribunal with `verdict: dissent` and time-boxed DR row.
