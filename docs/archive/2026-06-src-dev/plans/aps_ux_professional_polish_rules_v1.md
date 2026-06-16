# APS-UX-PROFESSIONAL-POLISH — interaction rules `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **APS-UX-PROFESSIONAL-POLISH** |
| **Status** | **ACTIVE** — rules signed; implementation Phases 1–6 |
| **Date** | 2026-06-03 |
| **Applies to** | `tools/mcp/art_pipeline_suite/*` (Tk APS shell) |
| **Prior audit** | [`design_aps_ux_audit_v1.md`](design_aps_ux_audit_v1.md) |
| **Exec index** | [`plan_aps_ux_polish_program_v1.md`](plan_aps_ux_polish_program_v1.md) |
| **Owner** | `@designer` (rules) · `@coder-mcp` (implementation) |

**Principle:** Token/theme polish alone does not fix frozen windows. **Responsiveness + non-blocking feedback** come first.

---

## 0. Viewport policy (1080p-class production)

**Design for daily use on a normal production monitor — not for minimum window size.**

| Tier | Size | Role |
|:---|:---|:---|
| **Design target** | **1280×800** window on **1920×1080** display class | Primary sign-off — must **look intentional** (split beside Blender/browser) |
| **Default launch** | **1280×800** (`aps_theme.DEFAULT_WINDOW_SIZE`) | Matches design target |
| **Comfortable max** | **1440×900** | Same display when more space — also supported, not required |
| **Comfortable range** | **1280×720 – 1440×900** | Panes grow; dynamic `wraplength`; collapsibles stay collapsed until opened |
| **Minimum floor** | **960×600** (`minsize`) | **Regression only** — must not crash or trap scroll; not required to look “designed” |

**Not in scope yet:** ultrawide / 4K layout modes (future **APS-UX-LAYOUT-002** — use horizontal space, remembered pane weights).

**Rule:** New APS UX work is reviewed at **1280×800** first. Min-size checks run in CI/witness — they do not define product aesthetics.

---

## 1. Responsiveness contract

| Tier | Latency | UI behavior | Examples |
|:---|:---|:---|:---|
| **Instant ack** | **< 100 ms** | Button press state, status log line, disable triggering control | Save path chosen, tab switch, filter change |
| **Inline work** | **100 ms – 2 s** | **Inline spinner** on the control or panel header; UI stays interactive elsewhere | `validate-report`, P0 gate, atlas meta validate, catalog GLB validate |
| **Job work** | **≥ 2 s** | **Job strip** (not modal): label + step `2/3` + **Cancel**; tab switch OK | `tile-batch-run`, `tile-atlas-pack`, variant bake, material generate batch |
| **Background** | **minutes** | Job strip + optional job drawer; append to status log; cancel where subprocess supports it | `assembly-build-run`, lod0 batch, keyframe pack |

**Hard rules:**

- Never block `mainloop` on MCP/CLI/subprocess for tier ≥ inline work.
- **Cancel** must be visible when `cancellable=True` (subprocess or cooperative thread event).
- Tab switch and scroll **always** work during jobs unless the job owns a modal confirm (data loss only).

---

## 2. Non-blocking feedback stack

Use the **lowest** layer that communicates without stealing focus:

```text
(1) Status log     — persistent, full text, timestamp optional
(2) Inline panel   — validation line, QC strip, panel header message
(3) Toast          — transient 4s banner (success/info); no action required
(4) Job drawer     — multi-line job log + history (Phase 2+)
(5) Modal          — ONLY data loss, irreversible delete, or unsaved navigate away
```

| Current anti-pattern | Replacement |
|:---|:---|
| `messagebox.showinfo` after every save/pack/validate | Status log + inline PASS/FAIL line |
| `messagebox.showerror` with 2000-char CLI log | Status log full text + inline FAIL + optional “Copy log” |
| `messagebox.askyesno` for bake confirm | Inline confirm row or job strip preflight |
| `status_var` truncated to **240 chars** | Full line in scrollable status log; status_var = last line summary ≤80 chars |

**Modal allowlist (Phase 2 migration):**

- Unsaved snapshot navigate away
- Delete profile / overwrite production path
- Cancel running job with partial writes (if any)

---

## 3. Progressive disclosure

| Surface | Default visible | Collapsed by default |
|:---|:---|:---|
| **Per tab** | Primary task + one inspector | Secondary/advanced |
| **Assembly** | Generate + footprint + placements + slot preview | Grammar inspector, variant tags accordion, agent strip |
| **Variants** | Layer comboboxes + bake selected | Agent patch strip |
| **Atlas** | Folder + pack + QC preview | Blender debug, lod0 advanced |
| **Catalog** | Filters + module list + detail | Metadata flow expanded on first run only |

**Rule:** At **1280×800** (design target), primary column shows footprint + slot preview without scrolling past advanced sections (tags/grammar collapsed). At **1440×900** (comfortable max) and **960×600** (floor), same path must remain **usable** — floor not required to look pretty.

---

## 4. One scroll owner per viewport

| Viewport | Scroll owner | Nested scroll |
|:---|:---|:---|
| **Notebook tab** | One `ScrollableFrame` per tab (`app._add_scrollable_tab`) | Accordions inside; no second full-height canvas scroll |
| **Catalog module list** | List canvas **or** tab scroll — **not both** fighting wheel | Phase 3: list captures wheel when pointer over list |
| **Materials tree + profile list** | Tree and list each own wheel when hovered | Tab scroll when pointer outside nested regions |
| **Assembly footprint grid** | Grid canvas owns wheel on hover | — |
| **Status log** | Own `Text` + scrollbar; wheel when hovered | — |

**Implementation rule:** On `<MouseWheel>`, scroll the **deepest** scrollable widget under cursor (hit-test chain). Parent `ScrollableFrame` scrolls only if child did not consume event.

---

## 5. Explicit interaction states

Every actionable control exposes **text + color** (not color/glyph alone):

| State | Visual | Text pattern |
|:---|:---|:---|
| **default** | Normal button/label | Verb label: `Pack atlas` |
| **hover** | Underline or relief | Tooltip reinforces |
| **disabled** | Muted + `state=disabled` | `Pack atlas (needs folder)` |
| **busy** | Spinner + disabled | `Packing atlas…` / `⟳ Packing atlas…` |
| **error** | `#a00000` foreground | `Pack failed — see log` |
| **success** | `#0a6b0a` | `Pack OK — tile_map_….png` |

Applies to: pipeline steps, validation lines, material status, bake status, atlas QC (per audit P0 fixes — enforce in tokens phase too).

---

## 6. APS shell layout (target)

```text
┌─ Flow bar ─────────────────────────────────────────────┐
│ Send to Assembly │ Bake variants │ Pack atlas            │
├─ Pipeline steps ───────────────────────────────────────┤
│ Catalog — done │ Assembly — done │ … │ Atlas — pending   │
├─ Job strip (only when busy) ───────────────────────────┤
│ ⟳ Packing atlas… 2/3                              [Cancel]│
├─ Notebook tabs ─ ONE ScrollableFrame each ─────────────┤
│  [ accordions inside tab ]                              │
├─ Status log (persistent, scrollable, not truncated) ───┤
│ 14:02:01 tile-batch-run started …                       │
│ 14:02:45 tile-batch-run OK                              │
└────────────────────────────────────────────────────────┘
```

---

## 7. Measurement (witness keys)

| Key | Target |
|:---|:---|
| `ui_ack_ms_p95` | < 100 ms for tab switch + log append |
| `mainloop_block_ms_max` | 0 during tier ≥ job work (threaded) |
| `messagebox_routine_count` | 0 after Phase 2 (modals = allowlist only) |
| `scroll_conflict_reports` | 0 — catalog list wheel fixed in Phase 3 |

Witness: `debug_runs/aps_ux_async_001_live.json` (Phase 1) · `aps_ux_nonblock_001_live.json` (Phase 2).

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Rules from post-audit responsiveness research |
