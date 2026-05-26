# Wave S — active lane (post–Stage 6)

**Status:** **S6-S1 / S6-S3 code complete** (2026-05-23)  
**Runbook:** [`prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md`](../../prompts/guides/backlog_serialization_preview_streaming_runbook_v1.md) §4

---

## On-disk layout (beside `manifest.ron`)

| Artifact | Path | DTO | BQ |
|----------|------|-----|-----|
| Product shell | `product_shell.ron` | `ProductShellPersistenceBundleR8` | BQ-133 |
| Blueprint presets | `blueprints/presets.ron` | `BlueprintPresetCollectionR8` | BQ-128 — [`bq128_editor_path_plan_v1.md`](bq128_editor_path_plan_v1.md) · design [`bq128_editor_path_design_note_v1.md`](bq128_editor_path_design_note_v1.md) (**UX-E02 SIGNED**) · coder **BQ-128-APPLY-001** open |

IO: [`src/io/save/wave_s_artifacts.rs`](../io/save/wave_s_artifacts.rs)  
Loader docs: [`src/io/save/load.rs`](../io/save/load.rs)

---

## In-app capture

Command shell → **Capture layout to Wave S DTO** sets `WaveSShellCapturePending`; `apply_wave_s_shell_capture_requests` writes to `WorldSaveBundleSettings.bundle_dir` on the next frame.

If `PendingConstructionQueue` has entries, blueprints are written and `blueprint_preset_ref` points at `blueprints/presets.ron`.

---

## Fixtures

| File | Profile |
|------|---------|
| `debug_runs/wave_s_shell_roundtrip.json` | `WAVE_S_SHELL_ROUNDTRIP` |
| `debug_runs/wave_s_blueprint_roundtrip.json` | `WAVE_S_BLUEPRINT_ROUNDTRIP` |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib wave_s
```

---

## Follow-on (active board)

| ID | Task | Board |
|----|------|-------|
| WS-A01…A05 | Hydrate, restore UX, blueprint import, live witness | [`post_stage6_active_todos.md`](post_stage6_active_todos.md) Phase A |

## BQ-133 binary envelope (WS-A05)

**Decision:** **RON-only** for `ProductShellPersistenceBundleR8` in S6-S slice. A future binary envelope (`SAVE_BINARY_BULK_DEFERRED` pattern from chunk artifacts) requires a BQ row + schema bump — do not dual-write RON and binary in the same bundle version.

## Not in this slice

- Binary envelope implementation (see above)
- Autoload shell on world load — **WS-A01** (flag-gated per DQ-POST-01)
- Wave P preview authority — **Phase B** in design plan
