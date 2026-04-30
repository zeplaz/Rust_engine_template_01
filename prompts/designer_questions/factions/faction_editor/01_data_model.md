# Faction editor — data model `01`

**Layers:** Serializable DTOs first; ECS mirrors only where sim needs queries (see `prompts/matrix/repo/repo_boundary_matrix_v1.md`).

## Core: `FactionBlueprint` (FE-01)

- **Stable id:** `EntityId` or dedicated `FactionId` 📎 — must round-trip in saves; document chosen type in schema.
- **Display:** `name`, optional `short_code`, `emblem_asset_path` 📎.
- **Color:** store **HSL** (or HSV) in file; convert to `LinearRgba` at load (see `implementation_questions_v1.md`).
- **Tags / traits:** `Vec<String>` or interned ids (FE-02); reserve extension for **research/tech overlays** (runtime revision vs separate `FactionRuntimeModifiers` resource 📎).

## AI prebuilts (matrix § v1)

- **Archetype:** optional `archetype_id: Option<String>` pointing at shared defaults; full row still valid for humans and mods.

## Diplomacy v1 (FE-05)

- **Serializable graph:** e.g. directed or symmetric edges with `stance` enum + numeric modifiers + treaty keys; **interlocking rules** (trade ↔ military ↔ intel) described as data-driven rows or code tables 📎.
- **Events:** stance change emits events consumed by trade/combat/intel systems (shape TBD).
- **Permissions bridge:** `PermissionDomain::DiplomaticRelations` grants *who may edit* stance; automated sim can still apply rules — conflict policy: `implementation_questions_v1.md` §11.

## Versioning

- `schema_version: u32` on container that holds blueprints + graph snapshot 📎.
