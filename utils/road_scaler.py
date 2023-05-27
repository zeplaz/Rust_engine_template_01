from PIL import Image
import os

def resize_tile_map(input_path, output_path, scale_factor):
    # Open the tile map image
    img = Image.open(input_path)

    # Calculate the new dimensions
    width, height = img.size
    new_width = int(width * scale_factor)
    new_height = int(height * scale_factor)

    # Resize the image
    resized_img = img.resize((new_width, new_height), Image.ANTIALIAS)

    # Save the resized image
    resized_img.save(output_path)


# Set your input and output paths, and the desired scaling factor
input_path = "%USERPROFILE%\\Downloads\\gen_art\\terrain\\hjm-cityroads-64.png"
output_path = "%USERPROFILE%\\Downloads\\gen_art\\terrain\\road_out_scaleup.png"
scale_factor = 6.375  # Adjust this value as needed

resize_tile_map(input_path, output_path, scale_factor)