# Native game UI — production session

Playing session (`BaseState::Simulation`). The player looks at one world map. Chrome is Bevy UI. Egui stays on the editor and on world-gen.

Buildings and zone rectangles are in the world, locked to tiles. Zoom changes how large a tile looks. It does not slide those objects across the map. The map has no egui painter.

## Screen regions

1. **Map** — the window under the chrome. One terrain sprite (`TileWorldFallbackSprite`). Buildings, footprints, and committed zone rectangles sit on that tile grid and share the map camera. Default pixels are the current play raster (biome plus the relief shading already in that raster). A toggle on the same sprite switches those pixels to height and slope. The minimap stays the GPU compositor.

2. **Top status line** — the existing ops strip: full width, about 30px, 2px under the window top. Zones stay Time, Alerts, Intel, Weather, Power, tray affordance. The Power zone is one short line: power, then haul. The line does not grow down over the town.

3. **Left build rail** — existing dual column under the status line, 8px in from the left. The tool column is about 52px. Slots, top to bottom: Roads, Rail, Power, Defense, Industry, Ecology, Zones. A slot opens one Bevy sheet to its right (8px gap, about 320px wide, at most 480px tall), aligned to that slot. The sheet does the work of the build picker, the power tool sheet, and the road tool sheet.

4. **Town access** — a three-state Bevy control on the left stack, beside the rail, only while a town is selected. Words: Military closed, Restricted labor, Free labor. Each word has a shape (closed bar, dashed bar, open edge). It reads that town’s access policy. It does not draw the town.

5. **Context tray** — bottom edge. Tab row about 32px. Peek body about 48px; expanded body about 96px. The Build tab lives on this tray and peeks while a build tool is armed.

6. **Minimap** — bottom-right Bevy frame around the GPU compositor image. Drag and resize stay on that frame. It is the only extra view.

Chrome leaves clear the tile under the cursor, the rectangle being drawn, and the minimap frame. A sheet stops before the minimap’s left edge and before the tray’s top.

## Controls

- **Build.** Click a rail slot. The sheet opens on that category and stays open until the rail closes it, or Escape. Pick a row to arm the tool. Power rows arm the line tool. Road rows arm the road tool. Placement clicks stay as they are: roads use left-add, right-undo, Shift+left commit; defense uses two clicks, Ctrl rotate, Shift size.
- **Town access.** With a town selected, the three-state control sets that town’s policy. Escape returns focus to the map.
- **Zone rectangle.** Zone tool armed. Hold Control, press the left button on a tile, drag, release. The press starts the drag on the town. The drag moves the free corner in tile steps. Release commits the inclusive tile rectangle. One tile is a valid rectangle.
- **Zone ghost.** While the button is held, a rectangle lies on the tiles from the anchor corner to the cursor. It has a fill, an edge, and a hatch so a blocked drag is readable without color. Release clears the ghost; the committed zone remains the world rectangle. Zoom keeps the ghost on those tiles.

## Egui

Egui runs for the map editor, and for world-gen and world preview. Those two do not run in the same frame. The playing session has no egui: no build sheet, no power sheet, no road sheet, and no painter on the world. Remaining Simulation plugin passes (end pass, output, pointer capture) and the one-tile binding for fire and town marks are **PLN-1 → PLN-3** in [`plan_egui_game_retirement_v1.md`](plan_egui_game_retirement_v1.md).

## Forbidden

- A second world map, an egui minimap dock, or a screen copy of the terrain.
- Egui footprint quads and projected screen stamps of tiles or buildings.
- Screen-space building stamps. A building is a world mesh on its tiles.
- Chrome covering the placement tile, the zone ghost, or the minimap.
- The HUD committing a zone, writing power or haul, or moving the camera. Release commits the rectangle on the district book. The HUD shows state.

## Hooks

A world overlay reads the drag’s tile rectangle (and painted zone tiles) through the map camera. One writer each for the drag, the town policy, and the status line. Witness the sheet opening, the drag committing, and the playing session drawing the map with no egui painter.
