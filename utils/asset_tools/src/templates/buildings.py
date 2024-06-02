
# Buildings
BUILDING_TYPES_MAP_UI = {"Residencey": lambda Parent_param: ResidenceyUI.create_residency_ui(Parent_param),
                  "Depanneur": lambda Parent_param: DepanneurUI.create_depanneur_ui(Parent_param), 
                  "Burocracy": lambda Parent_param: BurocracyUI.create_burocracy_ui(Parent_param), 
                  "Feild Depo": lambda Parent_param: FeildDepoUI.create_feild_depo_ui(Parent_param), 
                  "Warehouse": lambda Parent_param: WarehouseUI.create_warehouse_ui(Parent_param), 
                  "Factory": lambda Parent_param: FactoryUI.create_factory_ui(Parent_param),
                  "Mine": lambda Parent_param: MineUI.create_mine_ui(Parent_param),
                  "Fuel": lambda Parent_param: FuelUI.create_fuel_ui(Parent_param),
                  "Power": lambda Parent_param: PowerUI.create_power_ui(Parent_param),
                  "Reserch": lambda Parent_param: ReserchUI.create_resech_ui(Parent_param),
                  "Farm": lambda Parent_param: FarmUI.create_farm_ui(Parent_param),
                  "Rail": lambda Parent_param: RailUI.create_rail_ui(Parent_param)
                }
BUILDING_TYPES= [ "Residencey","Depanneur", "Burocracy", "Feild Depo","Warehouse","Factory","Mine","Fuel","Power","Reserch","Farm","Rail"]

RESEDENCY_TYPES = ["Small House", "Medium House", "Large House", "Estate", "Apartments"]

APARTMENT_TYPES = ["HighRise","Duplex", "Quadplex","Three Story Block", "Five Story Block"]
APARTMENT_UNIT_TYPES = [ "Studio", "Single", "Double", "ThreeBedrooms", "Family", "Luxury"]

MINE_TYPE = [ "Metal", "RareEarth", "Gravel", "Oil"]
FACTORY_TYPES = ["Ammunition","Electronics", "processed Metals","Chemicals","Woood","Fertilizer","Refinery","Paper Mill","Concreate"]