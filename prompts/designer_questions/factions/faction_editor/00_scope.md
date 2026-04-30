# Faction editor — scope `00`

**Parent matrix:** [`../faction_editor_tooling_matrix_v1.md`](../faction_editor_tooling_matrix_v1.md)

## In scope (v1 tool)

- Create/edit **faction blueprints**: name, **HSL-derived** colors, **tag bag** (traits placeholder).
- Embed in **scenario** or **save**; **rules-based** spawn when preset rows are missing.
- **Export/import** for modding (format: RON per `prompts/matrix/assets/bevy_asset_config_migration_matrix_v1.md` or project convention).
- **Diplomacy v1:** relationship matrix / stances / treaties with **interlocking** effects (see `01_data_model.md`, parent matrix § v1 product decisions).

## Out of scope (defer)

- VoIP, storefront, or any live service outside the game and editor tools.

## MP note

Faction **definitions** are data; **runtime faction state** and live edits are **server-authoritative** (pair with `_legacy/_legacy_designer_questions_v1.md` §A3 if still referenced, and `implementation_questions_v1.md` §8–9).

**Spec index:** [`README.md`](README.md).
