//! Parse legacy text tables of per-building resource rates (`input:` / `output:` lines).
//! Boundary: `serialization_hybrid_migration_matrix_v1.md` — this path is transitional until RON/config assets are canonical.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use crate::entities::types::p_enumz::ResourceType;
use crate::entities::types::s_flagz::{
    BuildingType, ConcreteType, FactoryType, MineType,
};

#[derive(Debug, Clone, Default)]
pub struct ResourceRates {
    pub input: HashMap<ResourceType, f32>,
    pub output: HashMap<ResourceType, f32>,
}

impl ResourceRates {
    pub fn from_file(path: &str) -> io::Result<HashMap<BuildingType, Self>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut rates: HashMap<BuildingType, Self> = HashMap::new();
        let mut current_building: Option<BuildingType> = None;

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("FactoryType::")
                || trimmed.starts_with("MineType::")
                || trimmed.starts_with("BuildingType::")
            {
                current_building = Some(parse_building_type(trimmed));
            } else if trimmed.starts_with("input:") {
                let input_resources = parse_resources(trimmed);
                if let Some(building_type) = current_building {
                    rates
                        .entry(building_type)
                        .or_default()
                        .input = input_resources;
                }
            } else if trimmed.starts_with("output:") {
                let output_resources = parse_resources(trimmed);
                if let Some(building_type) = current_building {
                    rates
                        .entry(building_type)
                        .or_default()
                        .output = output_resources;
                }
            }
        }

        Ok(rates)
    }
}

fn parse_resources(line: &str) -> HashMap<ResourceType, f32> {
    let mut m = HashMap::new();
    let rest = line
        .split_once(':')
        .map(|(_, r)| r)
        .unwrap_or(line)
        .trim();

    for part in rest.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut kv = part.split('=');
        let name = kv.next().unwrap_or("").trim();
        let val: f32 = kv
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0.0);
        if let Some(rt) = parse_resource_type(name) {
            m.insert(rt, val);
        }
    }
    m
}

fn parse_resource_type(s: &str) -> Option<ResourceType> {
    match s {
        "Wood" => Some(ResourceType::Wood),
        "Coal" => Some(ResourceType::Coal),
        "Oil" => Some(ResourceType::Oil),
        "RareEarth" | "Rare Earth" => Some(ResourceType::RareEarth),
        "Metal" => Some(ResourceType::Metal),
        "Steel" => Some(ResourceType::Steel),
        "Concrete" => Some(ResourceType::Concrete),
        "Fertilizer" => Some(ResourceType::Fertilizer),
        "Chemicals" | "Chemical" => Some(ResourceType::Chemicals),
        "Electronics" => Some(ResourceType::Electronics),
        "Energy" => Some(ResourceType::Energy),
        "Fuel" => Some(ResourceType::Fuel),
        "Ammunition" => Some(ResourceType::Ammunition),
        "WarSupply" | "War Supply" => Some(ResourceType::WarSupply),
        "Knowledge" => Some(ResourceType::Knowledge),
        "Labour" | "Labor" => Some(ResourceType::Labour),
        "Food" => Some(ResourceType::Food),
        "Water" => Some(ResourceType::Water),
        "Paper" => Some(ResourceType::Paper),
        "Electricity" => Some(ResourceType::Electricity),
        _ => None,
    }
}

fn parse_building_type(line: &str) -> BuildingType {
    let parts: Vec<&str> = line.split("::").map(str::trim).collect();
    match parts.first().copied() {
        Some("FactoryType") => BuildingType::FactoryType(parse_factory_type(&parts[1..])),
        Some("MineType") => {
            let sub = parts.get(1).copied().unwrap_or("");
            let mine = match sub {
                "Gravel" => MineType::Gravel,
                "Metal" => MineType::Metal,
                "RareEarth" => MineType::RareEarth,
                "Oil" => MineType::Oil,
                _ => panic!("Invalid MineType: {line}"),
            };
            BuildingType::MineType(mine)
        }
        Some("BuildingType") => {
            let sub = parts.get(1).copied().unwrap_or("");
            match sub {
                "Farm" => BuildingType::Farm,
                "House" => BuildingType::House,
                "RaiLDepot" => BuildingType::RaiLDepot,
                "Burocracy" => BuildingType::Burocracy,
                "WareHouse" => BuildingType::WareHouse,
                "Depanneur" => BuildingType::Depanneur,
                "FeildDepot" => BuildingType::FeildDepot,
                _ => panic!("Invalid BuildingType variant: {line}"),
            }
        }
        _ => panic!("Invalid building line: {line}"),
    }
}

fn parse_factory_type(parts: &[&str]) -> FactoryType {
    match parts.first().copied() {
        Some("ConcreteType") | Some("ConcreateType") => {
            let sub = parts.get(1).copied().unwrap_or("");
            let ct = match sub {
                "Limecrete" => ConcreteType::Limecrete,
                "Portland" => ConcreteType::Portland,
                "Geopolymer" => ConcreteType::Geopolymer,
                "Gypsum" => ConcreteType::Gypsum,
                _ => panic!("Invalid ConcreteType: {:?}", parts),
            };
            FactoryType::ConcreteType(ct)
        }
        Some("Ammunition") => FactoryType::Ammunition,
        Some("Electronics") => FactoryType::Electronics,
        Some("WarSupply") => FactoryType::WarSupply,
        Some("Chemical") => FactoryType::Chemical,
        Some("Wood") => FactoryType::Wood,
        Some("Fertilizer") => FactoryType::Fertilizer,
        Some("Refinery") => FactoryType::Refinery,
        Some("MetalProcessing") => FactoryType::MetalProcessing,
        _ => panic!("Invalid FactoryType: {:?}", parts),
    }
}
