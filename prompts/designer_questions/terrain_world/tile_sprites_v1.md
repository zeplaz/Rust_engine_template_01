# Designer Q — tile sprites & art resolution `v1`

---

## Locked intent

- **Tolerant of mixed art quality** — multiple sprite resolutions / styles allowed in data.
- **Expansion later** — pipeline should not assume one global tile pixel size forever.

---

## Sub-questions (⏳) — superseded in spirit by **Answers** below (decouple + format abstraction)

See also: `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md`.

---

## Answers & implementation cues (append-only)

**Round:** 2026-04-27 · **For:** LLM + engineers

| Topic | Decision |
|:---|:---|
| Hard link **logical tile** ↔ **texture pixel size** | **Avoid** unless a subsystem *needs* it (e.g. one specific collision grid). Default: **decouple** — assets can have different texel densities; sim uses **logical** cells / meters. |
| Format lock-in | **Today:** PNG everywhere. **Future:** KTX2/Basis/BC/etc. — pipeline should hide format behind **loader + import settings** (Bevy `Image` / asset labels); **one abstraction** so switching is config, not code forks. |
| Brittle assumptions | No global “all tiles must be N×N px” rule; document per-atlas **texel→world** scale in asset metadata when needed. |

### 📎 Tradeoffs / questions for designer

1. Do we ever need **integer tile = integer texels** for modding contracts? (If yes, scope to **editor grid** only.)
2. Max **texture dimension** per target (mobile vs desktop) — caps as build profile, not game rule.

### Implementation hints

- Prefer **`Asset`/RON metadata** next to sprites: `pixels_per_world_unit`, optional `atlas_cell_size`.
- When adding a second format, add a row to `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md` (import path + fallback to PNG in dev).
