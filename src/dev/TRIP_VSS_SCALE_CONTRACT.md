# TRIP-VSS-SCALE — World scale contract trip stub

```text
@orchestrator-status: ACTIVE
@orchestrator-owner: @planner
Program: VSS-001 · $ref:src/dev/plan_vfx_spectator_scale_program_v1.md
```

**You touched world scale / chunk / camera limits.** Stop and answer:

1. What is **one tile** in world units? **`TILE_SIM_UNIT` = 1.0** (`world_scale_contract.rs`)
2. What is **one chunk slab**? **`CHUNK_TILES_SIM` = 32** (display split 8 deferred T1-004)
3. How many **building footprints** should fit in one operational viewport frame? **Measured 10.15** (linear proxy) @ 3×3 — **T1-003 signed off** with bpf dissent
4. Does your change preserve **strategic sim chunk** vs **tactical display tile** separation? **Yes** — sim 32 unchanged; display 8 not split yet

**Before merge:**

- [x] Read `plan_world_scale_contract_v1.md`
- [x] Refresh `debug_runs/world_scale_contract_live.json` (T1-002 + T1-003 designer sign-off)
- [ ] File tribunal packet if changing constants without designer sign-off
- [ ] Run `symbolic_build_digest_tool` — do not paste raw cargo output

**Dissent:** Minority agents may file `dissent[]` in tribunal YAML with alternate scale proposal.
