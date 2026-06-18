# APS Design System v1.1 Delta — status_atom migration `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-DS-V11-001** |
| **Program** | APS UI/UX phase 2 professional polish |
| **Date** | 2026-06-17 |
| **Owner** | `@designer` |
| **Authority** | [`aps_design_system_v1.md`](aps_design_system_v1.md) §3.4 · sign-off **N1** |
| **Implements** | `OVR-P5-TAIL-001` |
| **Verdict** | **PASS** |

```text
DES-APS-DS-V11-001 Q✓
Retire ● dialect · route every status surface through status_atom / apply_status_atom
```

---

## 0. Goal

Close sign-off note **N1**: one status language everywhere. **Helper:** `aps_inline_feedback.status_atom()` · `apply_status_atom()` · `format_status_line()`.

**Retire:** material-card `●` · raw `PASS`/`FAIL` without glyph · `GEN`/`ERR` thumb codes · `register_green` in visible copy · PASS tinted `COLOR_ACCENT` (blue).

---

## 1. Canonical atom (unchanged from v1)

```text
{glyph} {word}[ — {detail}]
```

| State | Glyph | Default word |
|:---|:---:|:---|
| pass | `✓` | valid |
| fail | `✗` | blocked |
| warn | `◐` | partial |
| pending | `○` | pending |
| working | `⟳` | working |

---

## 2. Panel migration map

**Legend:** `✓` migrated · `△` partial · `✗` migrate in P5-TAIL

| Panel / file | Status surface | Today | Target | Priority |
|:---|:---|:---|:---|:---:|
| **pipeline_pills.py** | Pipeline pill text | `format_status_line` | ✓ keep | — |
| **landscape_extract_parity_panel.py** | Parity summary | `apply_status_atom` | ✓ keep | — |
| **atlas_panel.py** | Register row | `apply_status_atom` pass/fail | ✓ keep | — |
| **atlas_panel.py** | Inline QC / pack status | `set_inline_status` | `apply_status_atom` | P1 |
| **assembly_panel.py** | Ship check / validate line | `set_inline_status` | `apply_status_atom` + detail | **P0** |
| **assembly_preview_panel.py** | Preview status | mixed | all `apply_status_atom` | P1 |
| **catalog.py** | GLB validation | `set_inline_status` | `apply_status_atom` | P1 |
| **variants_panel.py** | File ops + bake line | `set_inline_status` | `apply_status_atom` | P1 |
| **grammar_iterate_panel.py** | Diff / iterate status | `set_inline_status` | `apply_status_atom` | P2 |
| **landscape_presets_panel.py** | Validate preset | `set_inline_status` | `apply_status_atom` | P1 |
| **landscape_states_panel.py** | Catalog validate + tree Status col | `set_inline_status` + `status_display` | glyph+word in tree; atom on banner | P1 |
| **material_library_widget.py** | Card status + list row | `format_material_texture_status` | ✓ atom path · fix `GEN`/`ERR` placeholders | **P0** |
| **material_library_widget.py** | `_status_var` footer | raw `ready` string | `✓ ready — {id}` | P1 |
| **slot_preview_panel.py** | Preview cells | `aps_preview_state` | align with preview spec §1 | P2 |
| **pipeline_status_bar.py** | Advance blocked hint | raw text + `COLOR_FAIL` | `✗ blocked — {reason}` prefix | P1 |
| **job_strip.py** | Running step | plain text | `⟳ {step}…` when active | P2 |
| **footprint_canvas.py** | Selection hint | subtitle only | no status atom | — |
| **grammar_inspector.py** | Rule chain | labels only | no pass/fail row | — |
| **metadata_flow_panel.py** | Explainer | prose | no atom | — |
| **status_log_panel.py** | Log lines | monospace log | keep log format; spine uses atom | — |

---

## 3. Material card migration (retire ● / GEN / ERR)

| Old | New |
|:---|:---|
| Thumb placeholder `GEN` | `○` + caption `generating…` below thumb |
| Thumb placeholder `ERR` | `✗` + caption `error` |
| Card header without glyph | `format_material_texture_status(status)` → `✓ ready` / `◐ partial` / `○ missing` |
| `_status_foreground` only | use `material_texture_status` fg from atom |

---

## 4. Remaining literal debt (from G0 audit + N1–N3)

| Class | Examples | Action |
|:---|:---|:---|
| Font literals | `("Segoe UI", 9)` in panels | migrate to `FONT_*` tokens (P1 tail) |
| Hex outside theme | ad-hoc `#4a90d9` PASS-in-blue | route through `COLOR_PASS` / `COLOR_FAIL` |
| Engineer words in status | `register_green`, `validate_status` | artist copy only in labels |
| Preview fidelity | partial async | **N2** — separate preview v2 spec |

**Guard:** extend `test_aps_style_tokens.py` — fail new raw `PASS`/`FAIL` labels without `✓`/`✗` prefix in panel modules.

---

## 5. Implementation order (@coder-mcp · OVR-P5-TAIL-001)

```text
1. material_library_widget — GEN/ERR + card headers (highest visibility)
2. assembly_panel — validation line
3. atlas_panel — inline QC lines
4. catalog + variants + landscape_presets — validate rows
5. pipeline_status_bar — blocked hint
6. grammar_iterate + job_strip — P2
```

One commit per file group; `pytest -k aps` each step.

---

## 6. Acceptance

- [ ] No `●` in `tools/mcp/art_pipeline_suite/`
- [ ] No visible `GEN`/`ERR` on material thumbs
- [ ] Register / validate / bake lines use `✓`/`✗`/`◐`/`○`/`⟳` + word
- [ ] `test_aps_style_tokens.py` extended
- [ ] Sign-off N1 closed in witness

---

## 7. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-17 |
