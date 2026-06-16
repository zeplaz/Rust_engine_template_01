# Stage 5.5 active todos

**Opened:** 2026-05-23  
**Status:** **DONE** (2026-05-23) — implementation + lib tests green.

**Rule:** One primary track per cycle; attach to spine contracts.

**Next:** [`stage6_plan_open.md`](stage6_plan_open.md) when opening virtualization lane.

---

## 5.5-A — View runtime (VM-A → VM-C slices)

| ID | Task | Status | Proof |
|----|------|--------|-------|
| S55-A01 | VM-A1: `view_runtime` module registered (`ViewRuntimePlugin`) | [x] | `engine_with_worldgen.rs` |
| S55-A02 | VM-A2: Writer tags on all `commit_*` paths | [x] | `ViewAuthorityWriter` + `last_pose_writer` |
| S55-A03 | VM-A3: Minimap focus → authority only (no `MapCameraDesired`) | [x] | `live_proof.rs` witness |
| S55-A04 | VM-A4: `VIEW_RUNTIME_AUDIT=1` trace + isolation tests green | [x] | `plugin.rs`, `isolation_tests.rs` |
| S55-A05 | VM-A5: Single-writer map in `recovery_viewport.md` | [x] | § single-writer table |
| S55-A06 | VM-B1: `viewport_pipeline` resolve → authority only | [x] | `commit_resolved_viewports_to_authority` |
| S55-A07 | VM-B4: `PerViewRepresentationPolicy` fire extract caps | [x] | `per_view_policy.rs` + `fire_view_extract.rs` |
| S55-A08 | VM-C witness: `infrastructure_view_isolation_green` | [x] | `infrastructure_view_isolation_live.json` (sim writer) |

---

## 5.5-B — GPU tile authoritative

| ID | Task | Status | Proof |
|----|------|--------|-------|
| S55-B01 | Construction phase tiles on instanced path | [x] | `site_phase_tile_instances.rs` |
| S55-B02 | Document CPU gizmo fallback retirement criteria | [x] | `base_finsh_5.md` § retirement |
| S55-B03 | Gate CPU/egui fallback when instanced count > 0 | [x] | `construction_phase_on_instanced_path` |

---

## 5.5-C — Perf shell

| ID | Task | Status | Proof |
|----|------|--------|-------|
| S55-C01 | `perf_attribution_60s.md` capture template + one sample | [x] | `debug_runs/perf_attribution_60s.md` |
| S55-C02 | Frame budget diagnostics wired to HUD consumer stub | [x] | `frame_budget_diagnostics.rs` + `dock_shell` |
| S55-C03 | World-gen chrome logging gated | [x] | `worldgen_chrome_debug_enabled()` |

---

## 5.5-D — Wave S (one slice)

| ID | Task | Status | Proof |
|----|------|--------|-------|
| S55-D01 | `ProductShellPersistenceBundleR8` round-trip test | [x] | `shell_persistence.rs` |
| S55-D02 | Wave S manifest path documented in save loader | [x] | `io/save/load.rs` header |
| S55-D03 | Fixture JSON under `debug_runs/wave_s_shell_roundtrip.json` | [x] | test writes fixture |

---

## 5.5-E — Fire sim depth

| ID | Task | Status | Proof |
|----|------|--------|-------|
| S55-E01 | Standing water gate (lake ≠ dry) | [x] | `surface_water.rs` tests |
| S55-E02 | Fire ecology F1 witness refresh | [x] | `fire_ecology_live.json` writer in sim |
| S55-E03 | Per-view fire extract policy stub (beyond WorldMain) | [x] | `PerViewRepresentationPolicy` |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib
```
