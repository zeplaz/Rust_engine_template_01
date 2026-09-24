# VSS-T4-005 — EffectBrowser implement packet (`@coder-mcp`)

| Field | Value |
|:---|:---|
| **Slice** | **VSS-T4-005** |
| **Charter** | [`src/dev/design_aps_effect_browser_v1.md`](../../../src/dev/design_aps_effect_browser_v1.md) **DES-APS-EFFECT-BROWSER-001 PASS** |
| **Owner** | `@coder-mcp` |
| **Do not** | Fork `fire_vfx` · touch sim HUD chrome · invent live particle preview |
| **Pattern** | Clone `material_browser.py` + `materials_panel.py` structure |

---

## Deliverables (files to add/edit)

| File | Action |
|:---|:---|
| `tools/mcp/art_pipeline_suite/effect_browser.py` | **ADD** — `EffectBrowserPanel` + `mount_effect_library` (mirror `material_browser.py`) |
| `tools/mcp/art_pipeline_suite/effect_library_widget.py` | **ADD** — browse list + honesty badges + preview ladder pane |
| `tools/mcp/art_pipeline_suite/effects_panel.py` | **ADD** — Effects tab shell (mirror `materials_panel.py`) |
| `tools/mcp/art_pipeline_suite/app.py` | Wire tab after Materials on buildings notebook |
| `tools/mcp/art_pipeline_suite/domain_router.py` | Add `Effects` to `BUILDINGS_TAB_LABELS` + pipeline step `effects` |
| `tools/mcp/art_pipeline_suite/aps_tooltips.py` | Add `tab_effects`, `eff_*` keys from charter §7 |
| `tools/mcp/python/tests/test_effect_browser.py` | **ADD** — list ≥3 registry ids · honesty read · assign blocked when pending |
| `debug_runs/aps_effect_browser_live.json` | Write on panel smoke |
| `debug_runs/artist_vfx_pipeline_live.json` | Clear `pending.aps_panel`; set shipped path |

Optional helper (prefer under `rust_engine_mcp/` if non-UI):

| File | Action |
|:---|:---|
| `tools/mcp/python/rust_engine_mcp/effect_registry_browse.py` | List registry/staging specs · read `honest_gate` |
| `tools/mcp/python/rust_engine_mcp/scenario_trigger_write.py` | Write `assets/scenarios/triggers/{id}.trigger.ron` |

---

## Mount API (required)

```python
MOUNT_BROWSE = "browse"
MOUNT_ASSIGN = "assign"

class EffectBrowserPanel(EffectLibraryWidget):
    ...

def mount_effect_library(master, *, mount: str = MOUNT_BROWSE, **kwargs) -> EffectBrowserPanel:
    ...
```

`EffectsPanel` uses `mount="browse"`. Assign strip may remount `mount="assign"` or call the same widget's assign method.

---

## Data sources

| Need | Path |
|:---|:---|
| Registry specs | `assets/effects/registry/<effect_id>/effect_spec.json` |
| Staging | `assets/staging/<effect_id>/effect_spec.json` |
| Honesty | `preview_witness.honest_gate` on spec or rollup in `debug_runs/artist_vfx_pipeline_live.json` |
| Reference batch | `batch_id == vss_t4_reference_effects_v1` |

---

## Assign contract

When `honest_gate == "honest"` only:

1. Write `assets/scenarios/triggers/{marker_id}.trigger.ron` (schema match `demo_ignite.trigger.ron`).
2. Log success line with `effect_id` + `marker_id`.
3. Optional: append `TriggerEffect(effect_id: "…")` to a user-selected scenario — v1 may stub UI picker and only write trigger file.

When `pending` / `dishonest_gate` / missing: Assign button **disabled** + tooltip.

---

## Preview ladder (UI only)

| Level | Implementation |
|:---|:---|
| P0 | Spawn-hook badge + tile token (text/canvas) |
| P1 | Staging PNG thumb if present else lane placeholder |
| P2 | Read-only emit/lod labels from spec |
| P3 | Show capture frames iff `frames_captured >= 1` |
| P4 | Chip only if `honest_gate == honest` and `development_tier == production` |

**Forbidden:** importing Bevy, `fire_vfx.emit`, GPU particle frames for preview.

---

## Acceptance (pytest)

```text
test_effect_registry_lists_reference_batch
test_honesty_badge_reads_pending_for_reference
test_assign_blocked_when_not_honest
test_mount_effect_library_factory
```

Exit: pytest green + `aps_effect_browser_live.json` + `artist_vfx_pipeline_live.json` refresh.

---

## ΔWF joint

```yaml
joint:
  from: "@designer"
  to: "@coder-mcp"
  slice: VSS-T4-005
  charter: src/dev/design_aps_effect_browser_v1.md
  question: "Does Effects tab wiring stay Tk-only with no fire_vfx import?"
  expect: true
```

---

## Out of scope residuals (do not pick)

- In-game spark/smoke absence / HUD chrome jank (other P0 agents)
- G4 preview capture worker (`frames_captured` / `capture_hash`) — may remain `pending`; panel must surface that honesty
- VSS-T4-005b runtime emit (already done)
