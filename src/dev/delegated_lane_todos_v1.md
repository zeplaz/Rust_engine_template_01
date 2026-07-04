# Delegated lane todos `v1`

**Date:** 2026-07-03 · **Branch:** `master`

Items below are **owned by other sessions / agents** — not the migration or primary cross-front pick board in this chat. Track status here; do **not** duplicate picks from [`cross_front_pick_queue_v1.md`](cross_front_pick_queue_v1.md) unless reassigned.

**Hub:** [`coder_non_migration_todos_v1.md`](coder_non_migration_todos_v1.md) · [`coder_remaining_post_019_v1.md`](coder_remaining_post_019_v1.md)

---

## Active delegated rows

| ☐ | ID | Stream | Owner | Scope | Exit witness / command |
|:---:|:---|:---:|:---|:---|:---|
| ☑ | **BQ-C4-SCALE-001** | 2 | @coder | Scale-chain audit **bake → placement → iso_draw** | ✓ `bq_c4_scale_001_live.json` · `visual_only_iso_draw` |
| ☑ | **APSR-S1-001** | 2 | @coder-mcp | **EventBus** + **SuiteStateWriter** | ✓ `debug_runs/apsr_a1_s1_001_live.json` |
| ☑ | **APSR-S2-001** | 2 | @coder-mcp | **AssemblyService** stale-panel fix | ✓ `debug_runs/apsr_a1_s2_001_live.json` |
| ☐ | **PERF-INSTR-VFX-002** | 1 | @coder_b / operator | Lib witness + doc refresh; **display gate** for acceptance run | `cargo test -p proc_A_dine01 --lib triage_perf_vfx_002 -- --nocapture` · operator: `cargo run --release -- --test vfx` |
| ☑ | **BQ-H1-FACADE-001** | 4 | @coder | FacadeRule `by_massing` + door_rhythm propagation | ✓ `bq_h1_facade_001_live.json` |
| ☑ | **BQ-H3-V0-RETIRE-001** | 4 | @coder | v0 grammar freeze shim | ✓ `bq_h3_v0_retire_001_live.json` |

**All delegated rows closed except PERF display acceptance.**

---

## Stream map (historical)

```text
Stream 1  PERF-INSTR-VFX-002     @coder_b / operator   (lib ☑ — display pending)
Stream 2  BQ-C4 · APSR-S1/S2     ✓ closed
Stream 4  BQ-H1/H2/H3 · APSR-Q1  ✓ closed
Schedule  SCH-W1-E1              ✓ closed (see coder_todos_v1.md)
```

**File mutex (Stream 2 ∥ 4):** avoid same-session edits to `procedural_build_extract.rs` / `assembly_snapshot.rs` while APSR-S2 is in flight.

---

## Verify bundle (delegated lanes only)

```powershell
# BQ-C4 + procedural (coder)
cargo test -p proc_A_dine01 --lib procedural_build_extract assembly_snapshot building_quality -q

# APSR-S1/S2 (coder-mcp)
cd tools/mcp/python && python -m pytest tests/test_aps_mutation_inventory.py tests/test_aps_panel_sync_characterization.py -q

# PERF-INSTR-VFX-002 lib (no display)
cargo test -p proc_A_dine01 --lib triage_perf_vfx_002 -- --nocapture

# PERF-INSTR-VFX-002 acceptance (display required — operator)
cargo run -p proc_A_dine01 --release -- --test vfx
```

---

## Done when (session close checklist)

| ID | Green when |
|:---|:---|
| BQ-C4 | Scale chain documented · bake/placement/iso_draw authority decision recorded · cargo check green |
| APSR-S1 | EventBus + SuiteStateWriter shipped · mutation inventory pytest green |
| APSR-S2 | AssemblyService stale-panel characterization passes |
| PERF-INSTR-VFX-002 | Lib test writes witness · triage JSON `green: true` · doc notes display gate for operator acceptance |
| BQ-H1/H2 | H1 FacadeRule in grammar · H2 street-facing openings spec landed in procedural path |

---

## Do not pick here (other boards)

| Item | Board |
|:---|:---|
| Migration MIG-* | **CLOSED** — `mig_a_program_close.json` |
| DR-RTT-VR16 operator VFX | `cross_front_pick_queue_v1.md` P0 |
| CLN-P0 / SCH-W1 / GPU P0-C′ | `cross_front_pick_queue_v1.md` P1–P2 |
