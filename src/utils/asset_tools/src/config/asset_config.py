import os

import json

from datetime import datetime



# Entity types

ASSET_TYPES = ["Building", "Vehicle", "Power", "Transportable", "Productive", "Scenery"]

TEXTURE_MAP_STATES = ["Midday", "Night", "Full", "Empty", "Lights_On", "Destroyed", "On_Fire"]



# Membership/faction

SEGMENT_MEMBERSHIP = ["Civilian", "Military", "Police", "National_Guard", "Praetorian", "Intelligence"]



# Damage states

FIRE_STATES = ["None", "Burning", "Burned", "Smoldering", "Smoldered"]

DAMAGE_STATES = ["FullyOperational", "Damaged", "Disabled", "Wrecked"]



# Surface types

SURFACE_TYPES = ["Asphalt", "Cobblestone", "Gravel", "Dirt"]



# Concrete types (align with concrete_industry_sim_runbook_v1)

CONCRETE_TYPES = ["Limecrete", "Portland", "Geopolymer", "Gypsum", "RapidSet", "HighStrength", "Lightweight"]



# Power infrastructure (asset editor combos; pair with production_economy power specs over time)

POWER_PLANT_TYPES = [
    "Coal",
    "Nuclear",
    "Solar",
    "Wind",
    "Oil",
    "Gas",
    "Geothermal",
    "Hydro",
    "Biomass",
]

POWER_DISTRIBUTION_TYPES = [
    "ThreePhaseHeavyIndustrial",
    "ThreePhaseMediumIndustrial",
    "OnePhaseLightIndustrial",
    "ThreePhaseResidential",
    "OnePhaseResidential",
    "ThreePhaseLongDistance",
    "OnephaseLongDistance",
    "Mixed",
]

SWITCH_STATES = ["Open", "Closed"]

OPERATION_MECHANISMS = ["Manual", "Automatic"]

SWITCH_BEHAVIORS = ["NonReclosing", "AutoReclosing"]

OPERATIONAL_STATUSES = [
    "Standby",
    "Operational",
    "Maintenance",
    "OutOfFuel",
    "StartingUp",
    "ShuttingDown",
    "Decommissioned",
    "ExternalShutdown",
    "ReducedCapacity",
    "OverCapacity",
    "EnvironmentalShutdown",
]



# --- Petroleum chain (see prompts/guides/petroleum_industry_simulation_runbook_v1.md) ---

# Crude = feedstock at deposits / extraction; not normal road-vehicle tank fuel.

CRUDE_TYPES = [

    "LightSweet",

    "HeavySour",

    "OilSandsBitumen",

    "ShaleOil",

    "OffshoreDeepCrude",

    "SyntheticCrude",

    "Condensate",

]



# Refinery fractions (storage / economy / logistics; not a vehicle fuel_type by itself).

REFINERY_FRACTIONS = [

    "LPG",

    "Naphtha",

    "Gasoline_Fraction",

    "Kerosene_Jet_Fraction",

    "Diesel_Fraction",

    "HeavyFuelOil",

    "Lubricants",

    "Asphalt",

    "PetroleumCoke",

]



# Resource catalog: split vague "Oil" / "Fuel" into feedstock + refined products.

RESOURCE_TYPES = [

    "Labour",

    "Water",

    "Food",

    "Wood",

    "Steel",

    "Concrete",

    "Ammunition",

    "Fertilizer",

    "Chemicals",

    "RareEarth",

    "Electronics",

    "Metal",

    "Paper",

    "Electricity",

    "Energy",

    "Coal",

    "WarSupply",

    "Knowledge",

    # Petroleum (crude vs refined)

    "CrudeOil",

    "NaturalGas",

    "Refined_LPG",

    "Refined_Naphtha",

    "Refined_Gasoline",

    "Refined_JetKerosene",

    "Refined_Diesel",

    "Refined_HeavyFuelOil",

    "Refined_Lubricants",

    "Refined_Asphalt",

    "Refined_PetroleumCoke",

    "Petrochemical_Intermediates",

    # Legacy aggregate names (keep for old JSON; prefer CrudeOil + Refined_* in new assets)

    "Oil",

    "Fuel",

]



RESOURCE_CATEGORY = ["Raw Material", "Processed Materials", "Energy", "Military", "Human", "Essentials"]



# Cargo types

CARGO_TYPES = ["Fluid", "Gas", "People", "Dry Goods"]



# Vehicle types

VEHICLE_TYPES = ["Road", "Ship", "Train", "Military", "Construction"]

ROAD_VEHICLE_TYPES = ["Bus", "Truck", "Car", "Cargo"]

SHIP_TYPES = ["Passenger", "Freight", "Tanker"]



# Energy carrier the vehicle *consumes* (tank, battery, grid shoe, etc.).

# Distinct from refinery fractions and from crude types.

FUEL_TYPES = [

    "Diesel",

    "Gasoline",

    "Jet_Kerosene",

    "Marine_Bunker",

    "LPG",

    "Avgas",

    "Biodiesel",

    "Bio_Ethanol",

    "Battery_Electric",

    "Grid_Electric_Trolley",

    "Hydrogen_FuelCell",

    "Coal",

    "Steam_External",

    "Nuclear_Heat",

    "Hybrid_Battery_Diesel",

    "Hybrid_Battery_Gasoline",

    "Crude_Transport_Only",

    "Solid_Rocket",

    "Liquid_Rocket",

]



# What supply chain / physics bucket this vehicle is grouped under (replaces vague FUEL_CLASS "Fossil").

# Serialized JSON field remains "fuel_class" for compatibility; UI may label this "Fuel source category".

FUEL_SOURCE_CATEGORY = [

    "Refined_Petroleum",

    "Crude_Feedstock",

    "Biofuel",

    "Electric_Battery",

    "Electric_Grid",

    "Hydrogen",

    "Nuclear",

    "Coal_Or_Solid",

    "Hybrid",

    "Rocket_Propellant",

]



# Backward compatibility: old code imports FUEL_CLASS.

FUEL_CLASS = FUEL_SOURCE_CATEGORY



# Allowed energy carriers per source category (asset editor constraint).

FUEL_TYPES_BY_CATEGORY = {

    "Refined_Petroleum": [

        "Diesel",

        "Gasoline",

        "Jet_Kerosene",

        "Marine_Bunker",

        "LPG",

        "Avgas",

    ],

    "Crude_Feedstock": ["Crude_Transport_Only"],

    "Biofuel": ["Biodiesel", "Bio_Ethanol"],

    "Electric_Battery": ["Battery_Electric"],

    "Electric_Grid": ["Grid_Electric_Trolley"],

    "Hydrogen": ["Hydrogen_FuelCell"],

    "Nuclear": ["Nuclear_Heat"],

    "Coal_Or_Solid": ["Coal", "Steam_External"],

    "Hybrid": ["Hybrid_Battery_Diesel", "Hybrid_Battery_Gasoline"],

    "Rocket_Propellant": ["Solid_Rocket", "Liquid_Rocket"],

}



# Old vehicle JSON / UI strings → new canonical fuel_type.

LEGACY_FUEL_TYPE_ALIASES = {

    "Jet Fuel": "Jet_Kerosene",

    "Prop Fuel": "Avgas",

    "Electric": "Battery_Electric",

    "Steam": "Steam_External",

    "Hydrogen": "Hydrogen_FuelCell",

    "Nuclear": "Nuclear_Heat",

    "Solar": "Battery_Electric",

    "Hybrid": "Hybrid_Battery_Diesel",

}



# Old fuel_class → FUEL_SOURCE_CATEGORY.

LEGACY_FUEL_CLASS_ALIASES = {

    "Fossil": "Refined_Petroleum",

    "Electric": "Electric_Battery",

    "Renewable": "Biofuel",

    "Nuclear": "Nuclear",

    "Hybrid": "Hybrid",

}





def normalize_fuel_type(value: str) -> str:

    if not value:

        return ""

    return LEGACY_FUEL_TYPE_ALIASES.get(value, value)





def normalize_fuel_source_category(value: str) -> str:

    if not value:

        return ""

    return LEGACY_FUEL_CLASS_ALIASES.get(value, value)





def fuel_types_for_category(category: str):

    """Energy carriers valid for a fuel source category (empty → full list)."""

    c = normalize_fuel_source_category(category)

    return FUEL_TYPES_BY_CATEGORY.get(c, list(FUEL_TYPES))





# Sound and detection system

SOUND_CLASSES = ["Engine", "Machinery", "Human", "Electrical", "Mechanical", "Explosive", "None"]

SOUND_DAMPENING_FACTORS = ["Insulation", "Distance", "Material", "Water", "Vegetation"]





class AssetConfig:

    """

    Class to manage asset configuration

    """



    def __init__(self):

        """Initialize with default values"""

        # Basic properties

        self.asset_name = ""

        self.asset_type = ""

        self.segment = "Civilian"

        self.description = ""



        # Asset type flags

        self.is_building = False

        self.is_vehicle = False

        self.is_power = False

        self.is_transportable = False

        self.is_productive = False

        self.is_scenery = False



        # Sound properties

        self.sound_emission = 0.0  # Base sound emission in decibels

        self.sound_class = "None"  # Type of sound emitted

        self.detection_multiplier = 1.0  # Affects how easily detected by agents



        # Texture properties

        self.texture_path = ""

        self.texture_states = {}



        # Vehicle properties

        self.vehicle_type = ""

        self.road_vehicle_type = ""

        self.fuel_type = ""

        self.fuel_class = ""  # stores FUEL_SOURCE_CATEGORY; JSON key unchanged

        self.fuel_efficiency = 0.0

        self.capacity = 0

        self.mass = 0.0

        self.max_speed = 0



        # Building properties

        self.building_size_x = 1

        self.building_size_y = 1

        self.tile_matrix_grid = []

        self.building_height = 1

        self.construction_cost = 100



        # Power properties

        self.power_generation = 0

        self.power_consumption = 0

        self.power_storage = 0



        # Resource properties

        self.produces_resources = []

        self.consumes_resources = []

        self.storage_capacity = {}



    def to_dict(self):

        """Convert to dictionary for serialization"""

        return {

            # Basic properties

            "asset_name": self.asset_name,

            "asset_type": self.asset_type,

            "segment": self.segment,

            "description": self.description,



            # Asset type flags

            "is_building": self.is_building,

            "is_vehicle": self.is_vehicle,

            "is_power": self.is_power,

            "is_transportable": self.is_transportable,

            "is_productive": self.is_productive,

            "is_scenery": self.is_scenery,



            # Sound properties

            "sound_emission": self.sound_emission,

            "sound_class": self.sound_class,

            "detection_multiplier": self.detection_multiplier,



            # Texture properties

            "texture_path": self.texture_path,

            "texture_states": self.texture_states,



            # Vehicle properties

            "vehicle_type": self.vehicle_type,

            "road_vehicle_type": self.road_vehicle_type,

            "fuel_type": self.fuel_type,

            "fuel_class": self.fuel_class,

            "fuel_efficiency": self.fuel_efficiency,

            "capacity": self.capacity,

            "mass": self.mass,

            "max_speed": self.max_speed,



            # Building properties

            "building_size_x": self.building_size_x,

            "building_size_y": self.building_size_y,

            "building_height": self.building_height,

            "construction_cost": self.construction_cost,



            # Power properties

            "power_generation": self.power_generation,

            "power_consumption": self.power_consumption,

            "power_storage": self.power_storage,



            # Resource properties

            "produces_resources": self.produces_resources,

            "consumes_resources": self.consumes_resources,

            "storage_capacity": self.storage_capacity,



            # Metadata

            "created": datetime.now().isoformat(),

            "version": "1.1.0",

        }



    def from_dict(self, data):

        """Load from dictionary"""

        # Basic properties

        self.asset_name = data.get("asset_name", "")

        self.asset_type = data.get("asset_type", "")

        self.segment = data.get("segment", "Civilian")

        self.description = data.get("description", "")



        # Asset type flags

        self.is_building = data.get("is_building", False)

        self.is_vehicle = data.get("is_vehicle", False)

        self.is_power = data.get("is_power", False)

        self.is_transportable = data.get("is_transportable", False)

        self.is_productive = data.get("is_productive", False)

        self.is_scenery = data.get("is_scenery", False)



        # Sound properties

        self.sound_emission = data.get("sound_emission", 0.0)

        self.sound_class = data.get("sound_class", "Machinery")

        self.detection_multiplier = data.get("detection_multiplier", 1.0)



        # Texture properties

        self.texture_path = data.get("texture_path", "")

        self.texture_states = data.get("texture_states", {})



        # Vehicle properties

        self.vehicle_type = data.get("vehicle_type", "")

        self.road_vehicle_type = data.get("road_vehicle_type", "")

        self.fuel_type = normalize_fuel_type(data.get("fuel_type", ""))

        self.fuel_class = normalize_fuel_source_category(data.get("fuel_class", ""))

        self.fuel_efficiency = data.get("fuel_efficiency", 0.0)

        self.capacity = data.get("capacity", 0)

        self.mass = data.get("mass", 0.0)

        self.max_speed = data.get("max_speed", 0)



        # Building properties

        self.building_size_x = data.get("building_size_x", 1)

        self.building_size_y = data.get("building_size_y", 1)

        self.building_height = data.get("building_height", 1)

        self.construction_cost = data.get("construction_cost", 100)



        # Power properties

        self.power_generation = data.get("power_generation", 0)

        self.power_consumption = data.get("power_consumption", 0)

        self.power_storage = data.get("power_storage", 0)



        # Resource properties

        self.produces_resources = data.get("produces_resources", [])

        self.consumes_resources = data.get("consumes_resources", [])

        self.storage_capacity = data.get("storage_capacity", {})



        return self



    def save(self, path=None):

        """Save configuration to JSON file"""

        if not self.asset_name:

            raise ValueError("Asset name is required")



        # Determine save path

        if path is None:

            # Default path is in the game data directory

            base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

            game_dir = os.path.dirname(os.path.dirname(base_dir))

            asset_dir = os.path.join(game_dir, "data", "game_entities")



            # Create subdirectory based on asset type

            if self.is_building:

                subdir = "buildings"

            elif self.is_vehicle:

                subdir = "vehicles"

            elif self.is_power:

                subdir = "power_stuff"

            else:

                subdir = "other"



            # Create directories if they don't exist

            save_dir = os.path.join(asset_dir, subdir)

            os.makedirs(save_dir, exist_ok=True)



            # Create filename

            filename = f"{self.asset_name.lower().replace(' ', '_')}.json"

            path = os.path.join(save_dir, filename)



        # Save to file

        with open(path, 'w') as f:

            json.dump(self.to_dict(), f, indent=4)



        return path



    def load(self, path):

        """Load configuration from JSON file"""

        with open(path, 'r') as f:

            data = json.load(f)



        return self.from_dict(data)

