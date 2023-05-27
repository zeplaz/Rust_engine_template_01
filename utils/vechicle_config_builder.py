import os
import re
import json
from pathlib import Path

def generate_vehicle_configs(texture_folder_path):
    vehicle_configs = []
    texture_folder = texture_folder_path / "vehicles"

    for vehicle_folder in texture_folder.iterdir():
        if vehicle_folder.is_dir():
            vehicle_config = {
                "name": vehicle_folder.name,
                "vtype": None,
                "capacity": 0,
                "mass": 0.0,
                "max_speed": 0,
                "military_civilian": None,
                "textures": {},
            }

            for texture_file in vehicle_folder.iterdir():
                if texture_file.name.startswith("tile_map_") and texture_file.name.endswith(".png"):
                    file_parts = re.split("_|\.", texture_file.name)
                    num_tiles = int(file_parts[1]) if file_parts[1].isdigit() else 8
                    
                    state = file_parts[3]
                    light_status = file_parts[4]

                    if file_parts[5].startswith("e"):
                        emission_level = float(file_parts[5][1:])
                    else:
                        emission_level = 0

                    if state not in vehicle_config["textures"]:
                        vehicle_config["textures"][state] = {}

                    if light_status not in vehicle_config["textures"][state]:
                        vehicle_config["textures"][state][light_status] = []
                    
                    texture_data = {
                        "path": f"vehicles/{vehicle_folder.name}/{texture_file.name}",
                        "tiles": num_tiles,
                    }

                    if emission_level:
                        texture_data["emission_level"] = emission_level
                    else: 
                        texture_data["emission_level"] = 0 

                    vehicle_config["textures"][state][light_status].append(texture_data)
                #print("file_parts:{},emission_level:{}, state:{}, light_status:{},texture_file.name:{}".format(file_parts,emission_level,state,light_status,texture_file.name))
            print("vc:{}".format(vehicle_config))
            vehicle_configs.append(vehicle_config)

    return vehicle_configs


def update_vehicle_configs(vehicle_configs, config_file_path):
    with open(config_file_path, "w") as output_file:
        json.dump(vehicle_configs, output_file, indent=2)


def main():
    script_dir = Path(os.path.dirname(os.path.abspath(__file__)))
    project_root = script_dir.parent
    texture_folder_path = project_root / "src" / "data" / "textures"
    vehicle_configs = generate_vehicle_configs(texture_folder_path)

    config_file_path = project_root / "src" / "data" / "vehicle_configs.json"
    update_vehicle_configs(vehicle_configs, config_file_path)

if __name__ == "__main__":
    main()



