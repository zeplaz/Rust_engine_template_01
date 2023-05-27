use crate::entities::MilitaryCivilian;
use crate::entities::RoadVehicleConfig;
use serde::Deserialize;
use serde_json::Result;
use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

fn load_all_json_files_from_dir<P: AsRef<Path>>(path: P) -> io::Result<HashMap<String, String>> {
    let mut json_files = HashMap::new();

    for entry in fs::read_dir(path)? {
        let entry: DirEntry = entry?;
        let file_path = entry.path();
        if file_path.is_file() && file_path.extension().unwrap_or_default() == "json" {
            let file_stem = file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let contents = fs::read_to_string(&file_path)?;
            json_files.insert(file_stem, contents);
        }
    }

    Ok(json_files)
}

pub fn deserialize_road_vehicle_configs<P: AsRef<Path>>(path: P) -> Result<Vec<RoadVehicleConfig>> {
    let file_contents = fs::read_to_string(path)?;
    let road_vehicle_configs: Vec<RoadVehicleConfig> = serde_json::from_str(&file_contents)?;
    Ok(road_vehicle_configs)
}

fn deserialize_military_civilian<'de, D>(
    deserializer: D,
) -> Result<Option<MilitaryCivilian>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    let mc = s.and_then(|string| MilitaryCivilian::from_str(&string));
    Ok(mc)
}

#[derive(Debug, Deserialize)]
struct DrezTruckData {
    capacity: u32,
    mass: f32,
    max_speed: u32,
    military_civilian: MilitaryCivilian,
    texture_path: String,
    name: String,
}

fn load_truck_varyants() -> HashMap<String, DrezTruckData> {
    let bytes = std::fs::read("data/trucks.dat").expect("Failed to read trucks.dat");
    let file_content = String::from_utf8_lossy(&bytes);

    let mut trucks = HashMap::new();

    for line in file_content.lines() {
        let mut fields = line.split(",");
        let capacity = fields.next().unwrap().trim().parse().unwrap();
        let mass = fields.next().unwrap().trim().parse().unwrap();
        let max_speed = fields.next().unwrap().trim().parse().unwrap();
        let military_civilian = match fields.next().unwrap().trim() {
            "Civilian" => MilitaryCivilian::Civilian,
            "Military" => MilitaryCivilian::Military,
            _ => panic!("Invalid military/civilian value"),
        };
        let texture_path = fields.next().unwrap().trim().to_string();
        let name = fields.next().unwrap().trim().to_string();

        let truck = DrezTruckData {
            capacity,
            mass,
            max_speed,
            military_civilian,
            texture_path,
        };

        trucks.insert(name, truck);
    }

    trucks
}

#[derive(Debug, Deserialize)]
struct DrezTerrainData {
    roughness: f32,
    temperature: f32,
    base_height: f32,
    textures: Option<Vec<String>>,
}

fn deserialize_terrain_data() -> HashMap<TerrainType, DrezTerrainData> {
    let terrain_data_file = std::fs::read_to_string("data/base_terrains.dat")
        .expect("Unable to read terrain data file");
    let mut terrain_data: HashMap<TerrainType, DrezTerrainData> = HashMap::new();
    for line in terrain_data_file.lines() {
        let mut parts = line.trim().split(',');
        let terrain_type = parts.next().expect("Missing terrain type").trim();
        let terrain = DrezTerrainData {
            roughness: parts
                .next()
                .expect("Missing roughness")
                .trim()
                .parse()
                .unwrap(),
            temperature: parts
                .next()
                .expect("Missing temperature")
                .trim()
                .parse()
                .unwrap(),
            base_height: parts
                .next()
                .expect("Missing base height")
                .trim()
                .parse()
                .unwrap(),
            textures: parts
                .next()
                .map(|t| t.trim().split(' ').map(|s| s.to_string()).collect()),
        };
        let terrain_type = match terrain_type {
            "Grass" => TerrainType::Grass,
            "Forest" => TerrainType::Forest,
            "Swamp" => TerrainType::Swamp,
            "Water" => TerrainType::Water,
            "Cliff" => TerrainType::Cliff,
            "Concrete" => TerrainType::Concrete,
            "Sand" => TerrainType::Sand,
            "Dirt" => TerrainType::Dirt,
            "Snow" => TerrainType::Snow,
            "Stone" => TerrainType::Stone,
            _ => panic!("Unknown terrain type: {}", terrain_type),
        };
        terrain_data.insert(terrain_type, terrain);
    }
    terrain_data
}
