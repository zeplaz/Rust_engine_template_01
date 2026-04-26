

struct ResourceRates{
    input: HashMap<ResourceType, f32>,
    output: HashMap<ResourceType, f32>,
}

impl ResourceRates {
    fn from_file(path: &str) -> io::Result<HashMap<BuildingType, Self>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut rates: HashMap<BuildingType, Self> = HashMap::new();
        let mut current_building: Option<BuildingType> = None;

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("FactoryType::") || line.starts_with("MineType::") || line.starts_with("BuildingType::") {
                let building_type = parse_building_type(&line);  // You will need to implement this
                current_building = Some(building_type);
            } else if line.starts_with("input:") {
                let input_resources = parse_resources(&line);  // You will need to implement this
                if let Some(building_type) = current_building.clone() {
                    rates.entry(building_type).or_insert_with(|| Self { input: HashMap::new(), output: HashMap::new() }).input = input_resources;
                }
            } else if line.starts_with("output:") {
                let output_resources = parse_resources(&line);  // You will need to implement this
                if let Some(building_type) = current_building.clone() {
                    rates.entry(building_type).or_insert_with(|| Self { input: HashMap::new(), output: HashMap::new() }).output = output_resources;
                }
            }
        }

        Ok(rates)
    }
}


fn parse_building_type(line: &str) -> BuildingType {    
    let parts: Vec<&str> = line.split("::").collect();
    match parts[0] {
        "FactoryType" => {
            match parts[1] {
                "ConcreteType" => {
                    match parts[2] {
                        "Limecrete" => BuildingType::FactoryType(FactoryType::ConcreateType(ConcreateType::Limecrete)),
                        "Portland" => BuildingType::FactoryType(FactoryType::ConcreateType(ConcreateType::Portland)),
                        "Geopolymer" => BuildingType::FactoryType(FactoryType::ConcreateType(ConcreateType::Geopolymer)),
                        "Gypsum" => BuildingType::FactoryType(FactoryType::ConcreateType(ConcreateType::Gypsum)),
                        _ => panic!("Invalid ConcreteType"),
                    }
                },
                "Ammunition" => BuildingType::FactoryType(FactoryType::Ammunition),
                "Electronics" => BuildingType::FactoryType(FactoryType::Electronics),
                "WarSupply" => BuildingType::FactoryType(FactoryType::WarSupply),
                "Chemical" => BuildingType::FactoryType(FactoryType::Chemical),
                "Wood" => BuildingType::FactoryType(FactoryType::Wood),
                "Fertilizer" => BuildingType::FactoryType(FactoryType::Fertilizer),
                "Refinery" => BuildingType::FactoryType(FactoryType::Refinery),
                "MetalProcessing"=> BuildingType::FactoryType(FactoryType::MetalProcessing),
                _ => panic!("Invalid FactoryType"),
            }
        },

        "MineType" => {
            match parts[1] {
            "Gravel" => BuildingType::MineType(MineType::Gravel),
            "Metal" => BuildingType::MineType(MineType::Metal),
            "RareEarth" => BuildingType::MineType(MineType::RareEarth),
            "Oil" => BuildingType::MineType(MineType::Oil),
            _ => panic!("Invalid MineType"),
            }
        },

        "BuildingType" => {
            match parts[1] {
            "Farm"=> BuildingType::BuildingType(BuildingType::Farm),
            "House"=> BuildingType::BuildingType(BuildingType::House),
            "RaiLDepot"=> BuildingType::BuildingType(BuildingType::RaiLDepot),
            "Burocracy" => BuildingType::BuildingType(BuildingType::Burocracy),
            "WareHouse" => BuildingType::BuildingType(BuildingType::WareHouse),
            "Depanneur"=> BuildingType::BuildingType(BuildingType::Depanneur),
            "FeildDepot" => BuildingType::BuildingType(BuildingType::FeildDepot),
            
            _ => panic!("Invalid BuildingType"),
            }
        },
       
    }
        // handle other BuildingType variants here...
        
    }