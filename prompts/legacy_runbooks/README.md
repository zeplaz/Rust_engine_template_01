# Legacy runbooks — closed / maintenance packs

> **Purpose:** Single index for **Applied** or **maintenance-only** step history so active runbook READMEs stay short. **Canonical execution** still lives under [`../matrix/`](../matrix/) and [`../guides/`](../guides/); this tree is **archive + capsule pointers** (2026-05-01 cleanup wave).

**Do not** treat files here as the first stop for new work — use the **Active** links below.

---

## Active entrypoints (by domain tag)

| Tag | Meaning | Go here first |
|:---|:---|:---|
| **gap_active** | G2–G5, G3 GUI | [`../matrix/gap_remediation/runbook/README.md`](../matrix/gap_remediation/runbook/README.md) · [`../guides/gap_remediation_runbook_v1.md`](../guides/gap_remediation_runbook_v1.md) |
| **terrain_active** | U3–U7 maintenance | [`../matrix/terrain_biome/runbook/README.md`](../matrix/terrain_biome/runbook/README.md) · [`terrain/terrain_u_applied_maintenance_v1.md`](terrain/terrain_u_applied_maintenance_v1.md) |
| **gui_tail** | G3B–G3D follow-ups | [`../guides/gui_runbook_v1.md`](../guides/gui_runbook_v1.md) · G3 sub-packs under gap runbook |
| **assets** | Bevy assets / tooling | [`../guides/world_assets_tools_rulebook_v1.md`](../guides/world_assets_tools_rulebook_v1.md) · [`../matrix/assets/runbook/`](../matrix/assets/runbook/) |
| **production** | Power, serialization, manufacturing | [`../guides/rulebook_backlog_designer_brief_v1.md`](../guides/rulebook_backlog_designer_brief_v1.md) §4 (**BQ-###**) |

---

## Archived full copies (zero-loss history)

| Domain | File | Notes |
|:---|:---|:---|
| **gap_closed** | [`gap_remediation/g1_hydrology_steps_v1_FULL.md`](gap_remediation/g1_hydrology_steps_v1_FULL.md) | Pre-capsule **G1-S01–S06** atomic steps; routing status **Applied** |
| **terrain** | [`terrain/terrain_u_applied_maintenance_v1.md`](terrain/terrain_u_applied_maintenance_v1.md) | U3–U7 **Applied** summary; step packs remain at `matrix/terrain_biome/runbook/u*_steps_v1.md` |

---

## Layout

```
legacy_runbooks/
├── README.md                 (this file)
├── gap_remediation/
│   └── g1_hydrology_steps_v1_FULL.md
└── terrain/
    └── terrain_u_applied_maintenance_v1.md
```
