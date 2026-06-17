# DES-APS-PARITY-PANEL-UX-001 — Landscape ENGINE_READ_PATH parity panel `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-PARITY-PANEL-UX-001** |
| **Program** | APS-E5 |
| **Pattern** | Mirror [`aps_mat_auth_ui.py`](../../tools/mcp/python/rust_engine_mcp/aps_mat_auth_ui.py) `ENGINE_READ_PATH` |
| **Date** | 2026-06-16 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |

---

## Building reference (exists)

```text
ENGINE_READ_PATH (materials):
  placement.material_profile → registry → worker bake → Bevy preview → render extract
  Assembly snapshot is authority.
```

## Landscape parity string (NEW — `ENGINE_READ_PATH_LANDSCAPE`)

```text
Runtime: landscape_grammar preset (land_dna + topology_graph) → ecology program on chunk
  → succession/disturbance state → LG-5 tile atlas lookup → map stamp / minimap tint.
  Preset JSON on disk is authority — not world-gen preview alone or a single "green" witness.
```

## Panel placement

| Lane | Tab | Panel |
|:---|:---|:---|
| Landscape | **Presets** | Collapsible `Metadata → engine` variant (landscape context) |
| Landscape | **Grammar** | Pinned one-liner above graph + expandable detail |
| Landscape | **States** | One line: "States expand variant_set — preset remains authority" |
| Landscape | **Atlas** | G0–G5 scope line (see state labels doc) |

## Wireframe (Grammar tab header)

```text
┌─ ENGINE_READ_PATH (landscape) ─────────────────────────────────┐
│ Preset land_dna + topology_graph → runtime ecology / atlas      │
│ [Show full path ▼]                                              │
└─────────────────────────────────────────────────────────────────┘
```

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-16 |
