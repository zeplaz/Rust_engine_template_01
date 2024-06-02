from PySide6 import QtCore, QtGui, QtWidgets

#Entity roles
MASTER_ENTITY_FILTER_ROLES = ["Building", "Vehicle","Transportable","Power", "Productive", "Scenery"]

TEXTURE_MAP_STATES = ["Midday", "Night", "Full", "Empty", "Lights_On", "Destroyed", "On_Fire"]

# 
ASSET_TYPES = ["Building","Tree","Vehcle","Train","Rail","Road","Resource"]
# general
SEGMENT_MEMBERSHIP = ["Civilian", "Military", "Police", "National_Guard", "Praetorian","Intelligence"]



# damage
FIRE_STATES = [ "None", "Burning", "Burned", "Smoldering", "Smoldered"]
DAMAGE_STATES = [ "FullyOperationa","Damaged", "Disabled", "Wrecked"]

# Roads 
SURFACE_TYPES = ["Asphalt", "Cobblestone", "Gravel", "Dirt"]

# Resource 
CONCREATE_TYPES = ["Limecrete", "Portland", "Geopolymer", "Gypsum"]

RESOURCE_TYPES = [ "Labour", "Water", "Food", "Wood", "Steal", "Concreate", "Ammunition", "Fertilizer", "Chemicals", "RareEarth","Electronics",
                   "Metal","Oil","Paper","Electrcity","Coal","WarSupply","Knowledge","Fuel"]

RESOURCE_CATEGORY =[ "Raw Material", "Processed Materials","Energy","Military","Humnan", "Essentials"]
# metals  "Gold", "Iron", "Copper", "Silver", "Bronze", "Aluminum", "Titanium", "Lead", "Platinum", "Nickel", "Zinc"]

CARGO_TYPES = ["Fluid","Gas","People", "Dry Goods"]






# vehicles
VEHICAL_TYPES = [ "Road", "Ship", "Train", "Military", "Construction"]
ROAD_VEHICLE_TYPES = ["Bus", "Truck", "Car", "Cargo"]
SHIP_TYPES = ["PAsssenger", "Freight", "Tanker"]


#
Asset_defintions = {
    "Asset_Types":ASSET_TYPES,
    "Damage_States":DAMAGE_STATES,
    "Fire_States":FIRE_STATES,
    "Segment_Membership":SEGMENT_MEMBERSHIP,
    
    
}





class AssetConfig:
    def __init__(self):
        self.asset_name = None
        self.asset_type = None
        self.is_building = False
        self.is_destroyed = False
        self.is_vehicle = False
        self.is_transportable = False
        self.is_productive = False
        self.is_power = False
        self.texture_paths = []
        self.texture_maps = []


class SubstationAssetConfig(AssetConfig):
    def __init__(self):
        self.is_building = True
        self.is_power = True
        super().__init__()

class car(AssetConfig):
    def __init__(self):
        self.is_vehicle = True
        super().__init__()

class AssetConfigDialog(QtWidgets.QDialog):
    def on_asset_selected(self, asset_type):
        if asset_type == "substation":
            self.current_asset_config = SubstationAssetConfig()
            self.current_asset_config.setup_ui()