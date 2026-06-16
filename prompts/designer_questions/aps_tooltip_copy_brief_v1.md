# APS-UX-TOOLTIPS-002 — Tooltip copy brief

**Program:** [`plan_aps_artist_tool_exec_v1.md`](../../src/dev/plan_aps_artist_tool_exec_v1.md) Phase 4  
**Audit:** [`design_aps_ux_audit_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_aps_ux_audit_v1.md)  
**Owner:** `@designer`  
**Consumer:** `@coder-mcp` → [`aps_tooltips.py`](../../tools/mcp/art_pipeline_suite/aps_tooltips.py)

---

## Deliverable

Create **`aps_tooltip_copy_v1.md`** in this folder with approved strings.

## Format

```markdown
### Catalog
| Control | Tooltip | On-screen hint (optional) |
|:---|:---|:---|
| Validate GLB | … | … |
```

## Must cover

- Flow bar: Send to Assembly, Bake variants, Pack atlas
- Catalog: validate, batch filter, metadata save
- Assembly: Generate, P0 gate, Save, Preview, Iterate grammar controls
- Materials: generate textures, Use in Assembly
- Variants: New from assembly, bake mismatch warning
- Atlas: pack, register hint, cell strip

## Rules

- ≤120 chars per tooltip where possible
- No hover-only critical paths — pair with visible hint where audit flagged accessibility
- Plain language — not validator codes (`StylePackDrift` → “Module from wrong style pack”)
- ARCH-MAT-001: materials assigned on snapshot, not Blender

## Paste back

```text
APS-UX-TOOLTIPS-002 complete
File: prompts/designer_questions/aps_tooltip_copy_v1.md
Row count: _
Sign-off: PASS | PASS WITH NOTES
```
