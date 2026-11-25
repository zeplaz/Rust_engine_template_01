import sys
import numpy as np
from PIL import Image

# Open the tilemaps
road_tilemap = Image.open(sys.argv[1])
vehicle_tilemap = Image.open(sys.argv[2])

# Set the size of the tiles
road_tile_width = 1920 // 6
road_tile_height = 1280 // 4
vehicle_tile_size = 512  # 1536 / 3

# Define the location where the vehicle should be placed on the road
vehicle_position = (1, 1)  # (row, column) of the road tile
vehicle_index = 3  # Index of the vehicle in the vehicle_tilemap (0 to 7)

# Get the road and vehicle tiles
road_tile = road_tilemap.crop(
    (
        road_tile_width * vehicle_position[1],
        road_tile_height * vehicle_position[0],
        road_tile_width * (vehicle_position[1] + 1),
        road_tile_height * (vehicle_position[0] + 1),
    )
)

vehicle_row = vehicle_index // 3
vehicle_col = vehicle_index % 3
vehicle_tile = vehicle_tilemap.crop(
    (
        vehicle_tile_size * vehicle_col,
        vehicle_tile_size * vehicle_row,
        vehicle_tile_size * (vehicle_col + 1),
        vehicle_tile_size * (vehicle_row + 1),
    )
)

# Combine the road and vehicle tiles
combined_tile = Image.alpha_composite(
    road_tile.convert("RGBA"),
    vehicle_tile.resize((road_tile_width, road_tile_height), Image.ANTIALIAS).convert("RGBA"),
)

# Paste the combined tile back into the road tilemap
road_tilemap.paste(
    combined_tile, (road_tile_width * vehicle_position[1], road_tile_height * vehicle_position[0])
)


# Save the result as a new image
road_tilemap.save(sys.argv[3])