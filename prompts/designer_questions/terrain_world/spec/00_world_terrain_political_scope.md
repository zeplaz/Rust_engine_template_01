# World scope — terrain, generation, politics `00`

## What “terrain_world” covers here

1. **World:** scenario container, seed, time, global rules — not just heightmap.
2. **Generation:** offline/headless + in-editor passes that produce **authoritative** height/biome/hydro (determinism TBD in §01).
3. **Terrain runtime:** chunk streaming, LOD sim, hydrology dirty regions — see sibling `*_v1.md` topic files.
4. **Political layer:** **territory** / **claims** / **borders** on tiles or macro regions; drives nav permissions and production ownership; full **stance graph** in `factions/spec/` + diplomacy runtime.

## Non-goals (defer)

- Pure cosmetics without sim hooks (decals only) — `tile_sprites_v1.md`.

## Read order

`01` → `02` (gen↔terrain) → `03` (politics) → `04` (cities) → `05` (units & logistics).
