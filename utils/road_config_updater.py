import json
import numpy as np
from pathlib import Path
from PIL import Image, ImageOps



# Define the configuration file path
config_file_path = "assets/terrain/road_config.json"

# Open the tilemaps
road_tilemap = Image.open("%USERPROFILE%\\Downloads\\gen_art\\terrain\\road_out_scaleup.png")
vehicle_tilemap = Image.open("%USERPROFILE%\\Downloads\\gen_art\source\\truck_civ_01\\tilemaps\\tile_map_empty_miday.png")

# Set the size of the tiles
road_tile_width = 1920 // 6
road_tile_height = 1280 // 4
vehicle_tile_size = 512  # 1536 / 3

# Load the existing configuration data if the file exists
if Path(config_file_path).is_file():
    with open(config_file_path, "r") as config_file:
        config_data = json.load(config_file)
else:
    config_data = {"combined_tiles": []}






# Define the tile positions to combine
tile1_position = (1, 1)
tile2_position = (1, 1)

# Get the road tiles
tile1 = road_tilemap.crop(
    (
        road_tile_width * tile1_position[1],
        road_tile_height * tile1_position[0],
        road_tile_width * (tile1_position[1] + 1),
        road_tile_height * (tile1_position[0] + 1),
    )
)
tile2 = road_tilemap.crop(
    (
        road_tile_width * tile2_position[1],
        road_tile_height * tile2_position[0],
        road_tile_width * (tile2_position[1] + 1),
        road_tile_height * (tile2_position[0] + 1),
    )
)

# Create masks for the tiles
mask1 = ImageOps.invert(tile1.convert("L").point(lambda x: 255 if x < 128 else 0))
mask2 = ImageOps.invert(tile2.convert("L").point(lambda x: 255 if x < 128 else 0))

# Combine the tiles
combined_tile = Image.new("RGBA", (road_tile_width * 2, road_tile_height))
combined_tile.paste(tile1, (0, 0), mask1)
combined_tile.paste(tile2, (road_tile_width, 0), mask2)

# Save the combined tile as a new image
combined_tile.save("combined_road_tiles.png")


# Add the new combined tile information to the configuration data
config_data["combined_tiles"].append(
    {
        "tile1": {
            "position": [tile1_position[0], tile1_position[1]],
            "size": [road_tile_width, road_tile_height],
        },
        "tile2": {
            "position": [tile2_position[0], tile2_position[1]],
            "size": [road_tile_width, road_tile_height],
        },
        "combined_size": [road_tile_width * 2, road_tile_height],
    }
)

# Save the updated configuration data to the file
with open(config_file_path, "w") as config_file:
    json.dump(config_data, config_file, indent=4)