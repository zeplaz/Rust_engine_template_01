# VFX Phase 2 — @coder copy-paste starters

**Full queue:** [`src/dev/vfx_coder_phase2_queue_v1.md`](../../../src/dev/vfx_coder_phase2_queue_v1.md)

---

## Coder A — start here

```
Lane: P2-VFX-VISUAL-001 — tactical VFX visual proof
Read: src/dev/vfx_coder_phase2_queue_v1.md
Problem: stage5 JSON shows 0 particle rows at strategic zoom — need tactical proof
First: stage5_full_app_harness.rs — tactical zoom_alpha before witness stamp
Verify: fire_spark_rows > 0; water_particle_river_streaks > 0 in stage5_full_app_live.json
```

Then: **P2-FIRE-SPARK-010** (smoke under sparks) → **P2-WATER-POLISH-001** (river read).

---

## Coder B — start here

```
Lane: P2-VFX-WITNESS-001 — tactical witness unit tests
Read: src/dev/vfx_coder_phase2_queue_v1.md § P2-VFX-WITNESS-001
First: gpu_particles.rs + gpu_water_particles.rs tests at zoom_alpha = 0.8
Verify: cargo test -p proc_A_dine01 --lib gpu_particles gpu_water
```

Parallel: **UI-WP-LAYOUT-001** or **IND-E01** (disjoint from VFX shaders).

---

## Global regression

```powershell
cargo test -p proc_A_dine01 --lib gpu_particles gpu_water stage5
cargo run -p proc_A_dine01 --release -- --test visual
```
