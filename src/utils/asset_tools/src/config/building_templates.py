"""Building templates configuration for dynamic building type creation"""

from typing import Dict, List, Any, Optional

class BuildingMatrixGrid:
    """Interactive grid for defining building tile occupancy"""

    def __init__(self, width=1, height=1):
        self.width = width
        self.height = height
        self.matrix = [[0 for _ in range(width)] for _ in range(height)]

    def resize_grid(self, new_width, new_height):
        """Dynamically resize the grid"""
        self.width = new_width
        self.height = new_height
        self.matrix = [[0 for _ in range(new_width)] for _ in range(new_height)]

    def set_tile(self, x, y, value=1):
        """Set a specific tile's value"""
        if 0 <= x < self.width and 0 <= y < self.height:
            self.matrix[y][x] = value

    def get_matrix(self):
        """Return the current matrix"""
        return self.matrix

    def to_dict(self):
        """Convert matrix to dictionary representation"""
        return {
            "width": self.width,
            "height": self.height,
            "matrix": self.matrix
        }

    @classmethod
    def from_dict(cls, data):
        """Create matrix from dictionary"""
        grid = cls(data.get("width", 1), data.get("height", 1))
        grid.matrix = data.get("matrix", [[0]])
        return grid

class BuildingMatrixGridWidget(QWidget):
    """Interactive grid for defining building tile occupancy"""

    def __init__(self, width=10, height=10):
        super().__init__()
        self.width = width
        self.height = height
        self.matrix = [[0 for _ in range(width)] for _ in range(height)]
        self.initUI()

    def initUI(self):
        layout = QVBoxLayout(self)
        self.grid_widget = QWidget()
        self.grid_layout = QGridLayout(self.grid_widget)
        self.grid_layout.setSpacing(2)

        self.tile_buttons = []
        for y in range(self.height):
            row = []
            for x in range(self.width):
                button = QPushButton()
                button.setCheckable(True)
                button.setFixedSize(30, 30)
                button.clicked.connect(lambda checked, x=x, y=y: self.toggle_tile(x, y))
                self.grid_layout.addWidget(button, y, x)
                row.append(button)
            self.tile_buttons.append(row)

        layout.addWidget(self.grid_widget)

    def toggle_tile(self, x, y):
        """Toggle tile occupancy and update matrix"""
        button = self.tile_buttons[y][x]
        self.matrix[y][x] = 1 if button.isChecked() else 0
        button.setStyleSheet(
            "background-color: green;" if button.isChecked()
            else "background-color: lightgray;"
        )

    def resize_grid(self, new_width, new_height):
        """Resize the grid dynamically"""
        # Clear existing layout
        for i in reversed(range(self.grid_layout.count())):
            self.grid_layout.itemAt(i).widget().setParent(None)

        self.width = new_width
        self.height = new_height
        self.matrix = [[0 for _ in range(new_width)] for _ in range(new_height)]
        self.tile_buttons = []

        # Recreate grid
        for y in range(self.height):
            row = []
            for x in range(self.width):
                button = QPushButton()
                button.setCheckable(True)
                button.setFixedSize(30, 30)
                button.clicked.connect(lambda checked, x=x, y=y: self.toggle_tile(x, y))
                self.grid_layout.addWidget(button, y, x)
                row.append(button)
            self.tile_buttons.append(row)

    def get_matrix(self):
        """Return the current matrix representing tile occupancy"""
        return self.matrix

EMPTY_Matrix = BuildingMatrixGrid(3, 2)

# Define building template structure
class BuildingTemplate:
    """Template for creating specific building types"""

    def __init__(self,
                 type_id: str,
                 name: str,
                 description: str,
                 produces: List[str] = None,
                 consumes: List[str] = None,
                 production_rate: Dict[str, float] = None,
                 consumption_rate: Dict[str, float] = None,
                 power_consumption: float = 0.0,
                 default_size: tuple = (1, 1),
                 tile_matrix: BuildingMatrixGrid = None,
                 workers_required: int = 0,
                 construction_cost: int = 100,
                 construction_time: int = 10,
                 maintenance_cost: float = 1.0,
                 tech_level: int = 1,
                 special_features: List[str] = None,
                 icon_name: str = None):
        """Initialize building template

        Args:
            type_id: Unique identifier for this building type
            name: Display name
            description: Building description
            produces: Resources this building produces
            consumes: Resources this building consumes
            production_rate: Rate at which resources are produced
            consumption_rate: Rate at which resources are consumed
            power_consumption: Power required to operate
            default_size: Default building size (width, height)
            workers_required: Number of workers required to operate
            construction_cost: Cost to build
            construction_time: Time to build in game units
            maintenance_cost: Ongoing maintenance cost
            tech_level: Technology level required
            special_features: Special features or capabilities
            icon_name: Icon for UI representation
        """
        self.type_id = type_id
        self.name = name
        self.description = description
        self.produces = produces or []
        self.consumes = consumes or []
        self.production_rate = production_rate or {}
        self.consumption_rate = consumption_rate or {}
        self.power_consumption = power_consumption
        self.default_size = default_size
        self.tile_matrix = EMPTY_Matrix
        self.workers_required = workers_required
        self.construction_cost = construction_cost
        self.construction_time = construction_time
        self.maintenance_cost = maintenance_cost
        self.tech_level = tech_level
        self.special_features = special_features or []
        self.icon_name = icon_name or type_id.lower()

              # Use provided tile matrix or create a default 1x1 matrix
        self.tile_matrix = tile_matrix or BuildingMatrixGrid(1, 1)

        # Update default size based on tile matrix
        self.default_size = (self.tile_matrix.width, self.tile_matrix.height)


    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        return {
            "type_id": self.type_id,
            "name": self.name,
            "description": self.description,
            "produces": self.produces,
            "consumes": self.consumes,
            "production_rate": self.production_rate,
            "consumption_rate": self.consumption_rate,
            "power_consumption": self.power_consumption,
            "default_size": self.default_size,
            "tile_matrix": self.tile_matrix,
            "workers_required": self.workers_required,
            "construction_cost": self.construction_cost,
            "construction_time": self.construction_time,
            "maintenance_cost": self.maintenance_cost,
            "tech_level": self.tech_level,
            "special_features": self.special_features,
            "icon_name": self.icon_name
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'BuildingTemplate':
        """Create from dictionary"""
        return cls(
            type_id=data["type_id"],
            name=data["name"],
            description=data["description"],
            produces=data.get("produces", []),
            consumes=data.get("consumes", []),
            production_rate=data.get("production_rate", {}),
            consumption_rate=data.get("consumption_rate", {}),
            power_consumption=data.get("power_consumption", 0.0),
            default_size=tuple(data.get("default_size", (1, 1))),
            tile_matrix=data.get("tile_matrix", EMPTY_Matrix),
            workers_required=data.get("workers_required", 0),
            construction_cost=data.get("construction_cost", 100),
            construction_time=data.get("construction_time", 10),
            maintenance_cost=data.get("maintenance_cost", 1.0),
            tech_level=data.get("tech_level", 1),
            special_features=data.get("special_features", []),
            icon_name=data.get("icon_name", data["type_id"].lower())
        )





# Define concrete templates
CONCRETE_PLANT = BuildingTemplate(
    type_id="concrete_plant",
    name="Concrete Production Plant",
    description="Produces concrete from raw materials",
    produces=["Concrete"],
    consumes=["Water", "Chemicals", "Labour"],
    production_rate={"Concrete": 10.0},
    consumption_rate={"Water": 5.0, "Chemicals": 2.0, "Labour": 1.0},
    power_consumption=50.0,
    default_size=(3, 2),
    tile_matrix =EMPTY_Matrix,
    workers_required=5,
    construction_cost=1000,
    construction_time=20,
    tech_level=1,
    special_features=["Dust_Emission"]
)

# Define aluminum templates
ALUMINUM_SMELTER = BuildingTemplate(
    type_id="aluminum_smelter",
    name="Aluminum Smelter",
    description="Smelts aluminum from bauxite",
    produces=["Metal", "Aluminum"],
    consumes=["RareEarth", "Electricity", "Labour"],
    production_rate={"Metal": 5.0, "Aluminum": 10.0},
    consumption_rate={"RareEarth": 15.0, "Electricity": 100.0, "Labour": 3.0},
    power_consumption=200.0,
    default_size=(4, 3),
    tile_matrix =EMPTY_Matrix,
    workers_required=8,
    construction_cost=2500,
    construction_time=30,
    tech_level=2,
    special_features=["High_Temperature", "Pollution"]
)

# Define power generation templates
COAL_POWER_PLANT = BuildingTemplate(
    type_id="coal_power_plant",
    name="Coal Power Plant",
    description="Generates electricity using coal",
    produces=["Electricity"],
    consumes=["Coal", "Water", "Labour"],
    production_rate={"Electricity": 500.0},
    consumption_rate={"Coal": 50.0, "Water": 80.0, "Labour": 5.0},
    power_consumption=-500.0,  # Negative because it produces power
    default_size=(5, 4),
    tile_matrix =EMPTY_Matrix,
    workers_required=10,
    construction_cost=5000,
    construction_time=50,
    tech_level=1,
    special_features=["Air_Pollution", "Cooling_Tower"]
)

SOLAR_POWER_PLANT = BuildingTemplate(
    type_id="solar_power_plant",
    name="Solar Power Plant",
    description="Generates electricity using solar panels",
    produces=["Electricity"],
    consumes=["Labour"],
    production_rate={"Electricity": 200.0},
    consumption_rate={"Labour": 1.0},
    power_consumption=-200.0,  # Negative because it produces power
    default_size=(6, 6),
    tile_matrix =EMPTY_Matrix,
    workers_required=2,
    construction_cost=3000,
    construction_time=30,
    tech_level=2,
    special_features=["Clean_Energy", "Weather_Dependent"]
)

# Define manufacturing templates
ELECTRONICS_FACTORY = BuildingTemplate(
    type_id="electronics_factory",
    name="Electronics Factory",
    description="Manufactures electronic components",
    produces=["Electronics"],
    consumes=["Metal", "Chemicals", "Electricity", "Labour"],
    production_rate={"Electronics": 20.0},
    consumption_rate={"Metal": 10.0, "Chemicals": 5.0, "Electricity": 30.0, "Labour": 10.0},
    power_consumption=50.0,
    default_size=(4, 4),
    tile_matrix =EMPTY_Matrix,
    workers_required=15,
    construction_cost=4000,
    construction_time=40,
    tech_level=3,
    special_features=["Precision_Manufacturing"]
)

# Dictionary of all templates by ID
BUILDING_TEMPLATES = {
    "concrete_plant": CONCRETE_PLANT,
    "aluminum_smelter": ALUMINUM_SMELTER,
    "coal_power_plant": COAL_POWER_PLANT,
    "solar_power_plant": SOLAR_POWER_PLANT,
    "electronics_factory": ELECTRONICS_FACTORY
}

# Building categories for organization in UI
BUILDING_CATEGORIES = {
    "Production": ["concrete_plant", "aluminum_smelter"],
    "Power": ["coal_power_plant", "solar_power_plant"],
    "Manufacturing": ["electronics_factory"]
}

def get_template(type_id: str) -> Optional[BuildingTemplate]:
    """Get a building template by its type ID"""
    return BUILDING_TEMPLATES.get(type_id)
