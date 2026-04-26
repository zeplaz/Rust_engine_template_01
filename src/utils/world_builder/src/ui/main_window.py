"""
Main window for the World Builder tool
"""

import os
import sys
import json
import numpy as np
from enum import Enum, auto
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, 
                           QHBoxLayout, QLabel, QPushButton, QTabWidget, 
                           QStackedWidget, QSplitter, QScrollArea, QFileDialog,
                           QMessageBox, QGroupBox, QComboBox, QSpinBox,
                           QDoubleSpinBox, QSlider, QCheckBox)
from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtGui import QPixmap, QImage

# Try to import PyQt-Fluent-Widgets if available
try:
    from qfluentwidgets import (NavigationInterface, NavigationItemPosition,
                              FluentIcon, SplitFluentWindow, Theme, setTheme, 
                              FluentTranslator, MessageBox, PushButton)
    HAS_FLUENT = True
except ImportError:
    HAS_FLUENT = False
    print("PyQt-Fluent-Widgets not found. Falling back to standard PyQt widgets.")

# Import UI components
from .world_preview import WorldPreviewWidget

# Define terrain and biome enums
class BiomeType(Enum):
    DEEP_WATER = auto()
    SHALLOW_WATER = auto()
    BEACH = auto()
    DESERT = auto()
    GRASSLAND = auto()
    FOREST = auto()
    DENSE_FOREST = auto()
    MOUNTAIN = auto()
    SNOW_CAPPED_MOUNTAIN = auto()
    TUNDRA = auto()
    SWAMP = auto()

class RegionMethod(Enum):
    REGULAR = auto()
    MANHATTAN = auto()
    WEIGHTED = auto()
    CENTROIDAL = auto()
    CIRCULAR = auto()
    POWER = auto()

# Import generators if available
try:
    from src.generators.terrain_generator import TerrainGenerator
    from src.generators.region_generator import RegionGenerator
    HAS_GENERATORS = True
except ImportError:
    HAS_GENERATORS = False
    print("Generator modules not found. Some functionality will be disabled.")

class WorldBuilderWindow(SplitFluentWindow if HAS_FLUENT else QMainWindow):
    """Main window for the World Builder tool"""
    
    def __init__(self, settings):
        super().__init__()
        self.settings = settings
        self.initWindow()
        
        # Initialize generators if available
        self.terrain_generator = TerrainGenerator(settings) if HAS_GENERATORS else None
        self.region_generator = RegionGenerator(settings) if HAS_GENERATORS else None
        
        # Set up UI
        if HAS_FLUENT:
            self.initFluentUI()
        else:
            self.initStandardUI()
    
    def initWindow(self):
        """Initialize window properties"""
        self.setWindowTitle("Processor Alpha Dine - World Builder")
        self.resize(1280, 800)
    
    def initFluentUI(self):
        """Initialize UI with Fluent Widgets"""
        # Create navigation interface
        self.navigationInterface = NavigationInterface(self)
        self.navigationInterface.setFixedWidth(220)
        
        # Create pages
        self.homePage = QWidget()
        self.homePage.setObjectName("Home")
        
        # Create terrain page
        self.terrainPage = QWidget()
        self.terrainPage.setObjectName("Terrain")
        self.setupTerrainPage(self.terrainPage)
        
        # Create regions page
        self.regionsPage = QWidget()
        self.regionsPage.setObjectName("Regions")
        self.setupRegionsPage(self.regionsPage)
        
        # Create preview page
        self.previewPage = QWidget()
        self.previewPage.setObjectName("Preview")
        self.setupPreviewPage(self.previewPage)
        
        # Create export page
        self.exportPage = QWidget()
        self.exportPage.setObjectName("Export")
        self.setupExportPage(self.exportPage)
        
        # Add pages to navigation interface
        self.addSubInterface(self.homePage, FluentIcon.HOME, "Home")
        self.addSubInterface(self.terrainPage, FluentIcon.GLOBE, "Terrain")
        self.addSubInterface(self.regionsPage, FluentIcon.GLOBE, "Regions")  # Using GLOBE instead of MAP which doesn't exist
        self.addSubInterface(self.previewPage, FluentIcon.PHOTO, "Preview")  # Using PHOTO instead of VIEW which doesn't exist
        self.addSubInterface(self.exportPage, FluentIcon.SAVE, "Export")
        
        # Set default page
        self.navigationInterface.setCurrentItem("Home")
        
        # Set up home page
        self.setupHomePage(self.homePage)
    
    def initStandardUI(self):
        """Initialize UI with standard PyQt widgets"""
        # Create central widget
        self.centralWidget = QWidget()
        self.centralLayout = QHBoxLayout(self.centralWidget)
        self.setCentralWidget(self.centralWidget)
        
        # Create tab widget for navigation
        self.tabWidget = QTabWidget()
        
        # Create pages
        self.homePage = QWidget()
        self.setupHomePage(self.homePage)
        
        self.terrainPage = QWidget()
        self.setupTerrainPage(self.terrainPage)
        
        self.regionsPage = QWidget()
        self.setupRegionsPage(self.regionsPage)
        
        self.previewPage = QWidget()
        self.setupPreviewPage(self.previewPage)
        
        self.exportPage = QWidget()
        self.setupExportPage(self.exportPage)
        
        # Add pages to tab widget
        self.tabWidget.addTab(self.homePage, "Home")
        self.tabWidget.addTab(self.terrainPage, "Terrain")
        self.tabWidget.addTab(self.regionsPage, "Regions")
        self.tabWidget.addTab(self.previewPage, "Preview")
        self.tabWidget.addTab(self.exportPage, "Export")
        
        # Add tab widget to central layout
        self.centralLayout.addWidget(self.tabWidget)
    
    def setupHomePage(self, page):
        """Set up the home page"""
        layout = QVBoxLayout(page)
        
        # Add title
        title = QLabel("Processor Alpha Dine - World Builder")
        title.setAlignment(Qt.AlignCenter)
        title.setStyleSheet("font-size: 24px; font-weight: bold;")
        
        # Add description
        description = QLabel(
            "This tool allows you to create and customize worlds for Processor Alpha Dine. "
            "Use the navigation menu to access different aspects of world generation."
        )
        description.setWordWrap(True)
        description.setAlignment(Qt.AlignCenter)
        
        # Add quick start buttons
        quickStartLayout = QHBoxLayout()
        
        if HAS_FLUENT:
            newWorldButton = PushButton("New World")
            newWorldButton.setIcon(FluentIcon.ADD)
            
            loadWorldButton = PushButton("Load World")
            loadWorldButton.setIcon(FluentIcon.FOLDER)
        else:
            newWorldButton = QPushButton("New World")
            loadWorldButton = QPushButton("Load World")
        
        newWorldButton.clicked.connect(self.createNewWorld)
        loadWorldButton.clicked.connect(self.loadWorld)
        
        quickStartLayout.addWidget(newWorldButton)
        quickStartLayout.addWidget(loadWorldButton)
        
        # Add to layout
        layout.addStretch()
        layout.addWidget(title)
        layout.addWidget(description)
        layout.addSpacing(20)
        layout.addLayout(quickStartLayout)
        layout.addStretch()
    
    def setupTerrainPage(self, page):
        """Set up the terrain page"""
        layout = QVBoxLayout(page)
        layout.setContentsMargins(10, 10, 10, 10)
        
        # Add title
        title = QLabel("Terrain Generation")
        title.setStyleSheet("font-size: 18px; font-weight: bold; color: #f0f0f0;")
        
        # Add description
        description = QLabel(
            "Configure terrain generation parameters. This will determine the height map, "
            "moisture, temperature, and other basic features of your world."
        )
        description.setWordWrap(True)
        description.setStyleSheet("color: #f0f0f0;")
        
        # Create settings group box
        settingsGroup = QGroupBox("Terrain Settings")
        settingsGroup.setStyleSheet("""
            QGroupBox {
                background-color: #2e2e2e;
                border: 1px solid #555;
                border-radius: 5px;
                margin-top: 1ex;
                color: #f0f0f0;
            }
            QGroupBox::title {
                subcontrol-origin: margin;
                subcontrol-position: top center;
                padding: 0 5px;
            }
        """)
        settingsLayout = QVBoxLayout(settingsGroup)
        
        # World size settings
        sizeLayout = QHBoxLayout()
        widthLabel = QLabel("Width:")
        widthLabel.setStyleSheet("color: #f0f0f0;")
        self.worldWidthSpin = QSpinBox()
        self.worldWidthSpin.setRange(64, 2048)
        self.worldWidthSpin.setValue(256)
        self.worldWidthSpin.setSingleStep(64)
        self.worldWidthSpin.setStyleSheet("""
            QSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        heightLabel = QLabel("Height:")
        heightLabel.setStyleSheet("color: #f0f0f0;")
        self.worldHeightSpin = QSpinBox()
        self.worldHeightSpin.setRange(64, 2048)
        self.worldHeightSpin.setValue(256)
        self.worldHeightSpin.setSingleStep(64)
        self.worldHeightSpin.setStyleSheet("""
            QSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        sizeLayout.addWidget(widthLabel)
        sizeLayout.addWidget(self.worldWidthSpin)
        sizeLayout.addWidget(heightLabel)
        sizeLayout.addWidget(self.worldHeightSpin)
        settingsLayout.addLayout(sizeLayout)
        
        # Terrain parameters
        noiseLayout = QHBoxLayout()
        scaleLabel = QLabel("Noise Scale:")
        scaleLabel.setStyleSheet("color: #f0f0f0;")
        self.noiseScaleSpin = QDoubleSpinBox()
        self.noiseScaleSpin.setRange(0.01, 10.0)
        self.noiseScaleSpin.setValue(0.5)
        self.noiseScaleSpin.setSingleStep(0.05)
        self.noiseScaleSpin.setStyleSheet("""
            QDoubleSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        octavesLabel = QLabel("Octaves:")
        octavesLabel.setStyleSheet("color: #f0f0f0;")
        self.octavesSpin = QSpinBox()
        self.octavesSpin.setRange(1, 8)
        self.octavesSpin.setValue(4)
        self.octavesSpin.setStyleSheet("""
            QSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        noiseLayout.addWidget(scaleLabel)
        noiseLayout.addWidget(self.noiseScaleSpin)
        noiseLayout.addWidget(octavesLabel)
        noiseLayout.addWidget(self.octavesSpin)
        settingsLayout.addLayout(noiseLayout)
        
        # Climate parameters
        climateLayout = QHBoxLayout()
        moistureLabel = QLabel("Moisture Bias:")
        moistureLabel.setStyleSheet("color: #f0f0f0;")
        self.moistureBiasSpin = QDoubleSpinBox()
        self.moistureBiasSpin.setRange(-1.0, 1.0)
        self.moistureBiasSpin.setValue(0.0)
        self.moistureBiasSpin.setSingleStep(0.05)
        self.moistureBiasSpin.setStyleSheet("""
            QDoubleSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        tempLabel = QLabel("Temperature Bias:")
        tempLabel.setStyleSheet("color: #f0f0f0;")
        self.tempBiasSpin = QDoubleSpinBox()
        self.tempBiasSpin.setRange(-1.0, 1.0)
        self.tempBiasSpin.setValue(0.0)
        self.tempBiasSpin.setSingleStep(0.05)
        self.tempBiasSpin.setStyleSheet("""
            QDoubleSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        climateLayout.addWidget(moistureLabel)
        climateLayout.addWidget(self.moistureBiasSpin)
        climateLayout.addWidget(tempLabel)
        climateLayout.addWidget(self.tempBiasSpin)
        settingsLayout.addLayout(climateLayout)
        
        # Features
        featuresLayout = QHBoxLayout()
        islandLabel = QLabel("Island Mode:")
        islandLabel.setStyleSheet("color: #f0f0f0;")
        self.islandModeCheck = QCheckBox()
        self.islandModeCheck.setStyleSheet("""
            QCheckBox {
                color: #f0f0f0;
            }
            QCheckBox::indicator {
                width: 13px;
                height: 13px;
                background-color: #3e3e3e;
                border: 1px solid #555;
            }
            QCheckBox::indicator:checked {
                background-color: #8dc9c8;
            }
        """)
        
        mountainLabel = QLabel("Mountain Threshold:")
        mountainLabel.setStyleSheet("color: #f0f0f0;")
        self.mountainThresholdSpin = QDoubleSpinBox()
        self.mountainThresholdSpin.setRange(0.5, 0.95)
        self.mountainThresholdSpin.setValue(0.7)
        self.mountainThresholdSpin.setSingleStep(0.05)
        self.mountainThresholdSpin.setStyleSheet("""
            QDoubleSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        featuresLayout.addWidget(islandLabel)
        featuresLayout.addWidget(self.islandModeCheck)
        featuresLayout.addWidget(mountainLabel)
        featuresLayout.addWidget(self.mountainThresholdSpin)
        settingsLayout.addLayout(featuresLayout)
        
        # Add generate button
        if HAS_FLUENT:
            generateButton = PushButton("Generate Terrain")
            generateButton.setIcon(FluentIcon.GLOBE)
        else:
            generateButton = QPushButton("Generate Terrain")
            generateButton.setStyleSheet("""
                QPushButton {
                    background-color: #3e3e3e;
                    color: #f0f0f0;
                    border: 1px solid #555;
                    padding: 5px 15px;
                }
                QPushButton:hover {
                    background-color: #4e4e4e;
                }
                QPushButton:pressed {
                    background-color: #5e5e5e;
                }
            """)
        
        generateButton.clicked.connect(self.generateTerrain)
        
        # Add to layout
        layout.addWidget(title)
        layout.addWidget(description)
        layout.addWidget(settingsGroup)
        layout.addStretch()
        layout.addWidget(generateButton)
    
    def setupRegionsPage(self, page):
        """Set up the regions page"""
        layout = QVBoxLayout(page)
        layout.setContentsMargins(10, 10, 10, 10)
        
        # Add title
        title = QLabel("Region Generation")
        title.setStyleSheet("font-size: 18px; font-weight: bold; color: #f0f0f0;")
        
        # Add description
        description = QLabel(
            "Configure region generation parameters. Regions define political, economic, "
            "and resource distribution boundaries in your world."
        )
        description.setWordWrap(True)
        description.setStyleSheet("color: #f0f0f0;")
        
        # Create settings group box
        settingsGroup = QGroupBox("Region Settings")
        settingsGroup.setStyleSheet("""
            QGroupBox {
                background-color: #2e2e2e;
                border: 1px solid #555;
                border-radius: 5px;
                margin-top: 1ex;
                color: #f0f0f0;
            }
            QGroupBox::title {
                subcontrol-origin: margin;
                subcontrol-position: top center;
                padding: 0 5px;
            }
        """)
        settingsLayout = QVBoxLayout(settingsGroup)
        
        # Region count
        countLayout = QHBoxLayout()
        regionsLabel = QLabel("Number of Regions:")
        regionsLabel.setStyleSheet("color: #f0f0f0;")
        self.regionCountSpin = QSpinBox()
        self.regionCountSpin.setRange(4, 64)
        self.regionCountSpin.setValue(16)
        self.regionCountSpin.setSingleStep(4)
        self.regionCountSpin.setStyleSheet("""
            QSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        countLayout.addWidget(regionsLabel)
        countLayout.addWidget(self.regionCountSpin)
        countLayout.addStretch()
        settingsLayout.addLayout(countLayout)
        
        # Region method
        methodLayout = QHBoxLayout()
        methodLabel = QLabel("Generation Method:")
        methodLabel.setStyleSheet("color: #f0f0f0;")
        self.regionMethodCombo = QComboBox()
        self.regionMethodCombo.addItems([method.name.capitalize() for method in RegionMethod])
        self.regionMethodCombo.setCurrentIndex(0)
        self.regionMethodCombo.setStyleSheet("""
            QComboBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
                min-width: 120px;
            }
            QComboBox::drop-down {
                subcontrol-origin: padding;
                subcontrol-position: top right;
                width: 15px;
                border-left: 1px solid #555;
            }
            QComboBox QAbstractItemView {
                background-color: #3e3e3e;
                color: #f0f0f0;
                selection-background-color: #8dc9c8;
            }
        """)
        
        methodLayout.addWidget(methodLabel)
        methodLayout.addWidget(self.regionMethodCombo)
        methodLayout.addStretch()
        settingsLayout.addLayout(methodLayout)
        
        # Iteration count for voronoi-based methods
        iterationsLayout = QHBoxLayout()
        iterationsLabel = QLabel("Iterations:")
        iterationsLabel.setStyleSheet("color: #f0f0f0;")
        self.iterationsSpin = QSpinBox()
        self.iterationsSpin.setRange(1, 10)
        self.iterationsSpin.setValue(3)
        self.iterationsSpin.setStyleSheet("""
            QSpinBox {
                background-color: #3e3e3e;
                color: #f0f0f0;
                border: 1px solid #555;
                padding: 2px;
            }
        """)
        
        iterationsLayout.addWidget(iterationsLabel)
        iterationsLayout.addWidget(self.iterationsSpin)
        iterationsLayout.addStretch()
        settingsLayout.addLayout(iterationsLayout)
        
        # Region properties
        propertiesLayout = QVBoxLayout()
        propertiesLabel = QLabel("Region Properties:")
        propertiesLabel.setStyleSheet("color: #f0f0f0;")
        propertiesLayout.addWidget(propertiesLabel)
        
        # Resource generation checkbox
        resourcesCheck = QCheckBox("Generate Resources")
        resourcesCheck.setChecked(True)
        resourcesCheck.setStyleSheet("""
            QCheckBox {
                color: #f0f0f0;
            }
            QCheckBox::indicator {
                width: 13px;
                height: 13px;
                background-color: #3e3e3e;
                border: 1px solid #555;
            }
            QCheckBox::indicator:checked {
                background-color: #8dc9c8;
            }
        """)
        propertiesLayout.addWidget(resourcesCheck)
        
        # Political boundaries checkbox
        politicalCheck = QCheckBox("Political Boundaries")
        politicalCheck.setChecked(True)
        politicalCheck.setStyleSheet("""
            QCheckBox {
                color: #f0f0f0;
            }
            QCheckBox::indicator {
                width: 13px;
                height: 13px;
                background-color: #3e3e3e;
                border: 1px solid #555;
            }
            QCheckBox::indicator:checked {
                background-color: #8dc9c8;
            }
        """)
        propertiesLayout.addWidget(politicalCheck)
        
        # Economic zones checkbox
        economicCheck = QCheckBox("Economic Zones")
        economicCheck.setChecked(True)
        economicCheck.setStyleSheet("""
            QCheckBox {
                color: #f0f0f0;
            }
            QCheckBox::indicator {
                width: 13px;
                height: 13px;
                background-color: #3e3e3e;
                border: 1px solid #555;
            }
            QCheckBox::indicator:checked {
                background-color: #8dc9c8;
            }
        """)
        propertiesLayout.addWidget(economicCheck)
        
        settingsLayout.addLayout(propertiesLayout)
        
        # Add generate button
        if HAS_FLUENT:
            generateButton = PushButton("Generate Regions")
            generateButton.setIcon(FluentIcon.GLOBE)  # Using GLOBE instead of MAP which doesn't exist
        else:
            generateButton = QPushButton("Generate Regions")
            generateButton.setStyleSheet("""
                QPushButton {
                    background-color: #3e3e3e;
                    color: #f0f0f0;
                    border: 1px solid #555;
                    padding: 5px 15px;
                }
                QPushButton:hover {
                    background-color: #4e4e4e;
                }
                QPushButton:pressed {
                    background-color: #5e5e5e;
                }
            """)
        
        generateButton.clicked.connect(self.generateRegions)
        
        # Add to layout
        layout.addWidget(title)
        layout.addWidget(description)
        layout.addWidget(settingsGroup)
        layout.addStretch()
        layout.addWidget(generateButton)
    
    def setupPreviewPage(self, page):
        """Set up the preview page"""
        layout = QVBoxLayout(page)
        
        # Create preview widget
        self.worldPreview = WorldPreviewWidget(self.settings)
        
        # Add to layout
        layout.addWidget(self.worldPreview)
    
    def setupExportPage(self, page):
        """Set up the export page"""
        layout = QVBoxLayout(page)
        
        # Add title
        title = QLabel("Export World")
        title.setStyleSheet("font-size: 18px; font-weight: bold;")
        
        # Add description
        description = QLabel(
            "Export your world for use in Processor Alpha Dine. You can save the world "
            "as a file or directly import it into the game."
        )
        description.setWordWrap(True)
        
        # Add export options
        exportOptionsLayout = QVBoxLayout()
        
        if HAS_FLUENT:
            saveButton = PushButton("Save World")
            saveButton.setIcon(FluentIcon.SAVE)
            
            exportButton = PushButton("Export for Game")
            exportButton.setIcon(FluentIcon.SHARE)
        else:
            saveButton = QPushButton("Save World")
            exportButton = QPushButton("Export for Game")
        
        saveButton.clicked.connect(self.saveWorld)
        exportButton.clicked.connect(self.exportWorld)
        
        exportOptionsLayout.addWidget(saveButton)
        exportOptionsLayout.addWidget(exportButton)
        
        # Add to layout
        layout.addWidget(title)
        layout.addWidget(description)
        layout.addSpacing(20)
        layout.addLayout(exportOptionsLayout)
        layout.addStretch()
    
    # Event handlers
    def createNewWorld(self):
        """Create a new world"""
        # Reset generators
        if self.terrain_generator:
            self.terrain_generator.reset()
        
        if self.region_generator:
            self.region_generator.reset()
        
        # Reset preview
        self.worldPreview.resetPreview()
        
        # Show confirmation
        self.showMessage("New World", "Started a new world project.")
    
    def loadWorld(self):
        """Load a world from file"""
        # Show file dialog
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "Load World",
            "",
            "World Files (*.json);;All Files (*)"
        )
        
        if file_path and os.path.exists(file_path):
            try:
                with open(file_path, 'r') as f:
                    world_data = json.load(f)
                
                # TODO: Load world data into generators and update preview
                
                # Show confirmation
                self.showMessage("Load World", f"Loaded world from {file_path}")
            except Exception as e:
                self.showMessage("Error", f"Failed to load world: {str(e)}", error=True)
        else:
            # Show error
            self.showMessage("Error", "No file selected or file doesn't exist", error=True)
    
    def generateTerrain(self):
        """Generate terrain based on settings"""
        if self.terrain_generator:
            # Generate terrain
            try:
                # Create placeholder terrain data
                terrain_data = np.zeros((256, 256), dtype=np.int32)
                for y in range(256):
                    for x in range(256):
                        # Simple terrain with hills and valleys
                        nx = x / 256.0 * 5.0
                        ny = y / 256.0 * 5.0
                        value = np.sin(nx) * np.cos(ny) * 0.5 + 0.5
                        if value < 0.2:
                            terrain_data[y, x] = 0  # Deep water
                        elif value < 0.3:
                            terrain_data[y, x] = 1  # Shallow water
                        elif value < 0.35:
                            terrain_data[y, x] = 2  # Sand
                        elif value < 0.65:
                            terrain_data[y, x] = 3  # Grass
                        elif value < 0.8:
                            terrain_data[y, x] = 4  # Forest
                        elif value < 0.9:
                            terrain_data[y, x] = 5  # Mountain
                        else:
                            terrain_data[y, x] = 6  # Snow
                
                # Update preview
                self.worldPreview.terrain_map = terrain_data
                self.worldPreview.updateTerrainLayer()
                
                # Show confirmation
                self.showMessage("Terrain Generation", "Terrain generation successful!")
            except Exception as e:
                self.showMessage("Error", f"Failed to generate terrain: {str(e)}", error=True)
        else:
            self.showMessage("Error", "Terrain generator not available.", error=True)
    
    def generateRegions(self):
        """Generate regions based on settings"""
        if self.region_generator and self.worldPreview.terrain_map is not None:
            try:
                # Create placeholder region data
                width = self.worldPreview.terrain_map.shape[1]
                height = self.worldPreview.terrain_map.shape[0]
                region_data = np.zeros((height, width), dtype=np.int32)
                
                # Simple grid-based regions
                region_width = width // 4
                region_height = height // 4
                
                for y in range(height):
                    for x in range(width):
                        region_x = x // region_width
                        region_y = y // region_height
                        region_data[y, x] = region_y * 4 + region_x
                
                # Update preview
                self.worldPreview.region_map = region_data
                self.worldPreview.updateRegionLayer()
                
                # Show confirmation
                self.showMessage("Region Generation", "Region generation successful!")
            except Exception as e:
                self.showMessage("Error", f"Failed to generate regions: {str(e)}", error=True)
        else:
            self.showMessage("Error", "Region generator not available or terrain not generated yet.", error=True)
    
    def saveWorld(self):
        """Save the world to a file"""
        if not self.worldPreview.terrain_map is not None:
            self.showMessage("Error", "No world to save", error=True)
            return
        
        # Show file dialog
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Save World",
            "",
            "World Files (*.json);;All Files (*)"
        )
        
        if file_path:
            # Ensure file has .json extension
            if not file_path.lower().endswith('.json'):
                file_path += '.json'
            
            try:
                # Create world data to save
                world_data = {
                    "settings": {
                        "world_width": self.settings.world_width,
                        "world_height": self.settings.world_height,
                        "noise_scale": self.settings.noise_scale,
                        "noise_octaves": self.settings.noise_octaves,
                        "water_level": self.settings.water_level,
                        "mountain_level": self.settings.mountain_level,
                        "region_count": self.settings.region_count,
                        "region_generation_method": self.settings.region_generation_method
                    },
                    "terrain_data": self.worldPreview.terrain_map.tolist() if self.worldPreview.terrain_map is not None else None,
                    "region_data": self.worldPreview.region_map.tolist() if self.worldPreview.region_map is not None else None,
                    "resource_data": self.worldPreview.resource_map.tolist() if self.worldPreview.resource_map is not None else None
                }
                
                with open(file_path, 'w') as f:
                    json.dump(world_data, f, indent=4)
                
                # Show confirmation
                self.showMessage("Save World", f"World saved to {file_path}")
            except Exception as e:
                self.showMessage("Error", f"Failed to save world: {str(e)}", error=True)
    
    def exportWorld(self):
        """Export the world for game use"""
        if not self.worldPreview.terrain_map is not None:
            self.showMessage("Error", "No world to export", error=True)
            return
        
        # Show file dialog
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Export World for Game",
            "",
            "Game World Files (*.dat);;All Files (*)"
        )
        
        if file_path:
            # Ensure file has .dat extension
            if not file_path.lower().endswith('.dat'):
                file_path += '.dat'
            
            try:
                # Create a simple binary format for exporting
                with open(file_path, 'wb') as f:
                    # Write header with dimensions and types
                    header = f"WORLD_V1\n{self.worldPreview.terrain_map.shape[1]}\n{self.worldPreview.terrain_map.shape[0]}\n"
                    f.write(header.encode('utf-8'))
                    
                    # Write terrain data as binary
                    if self.worldPreview.terrain_map is not None:
                        terrain_bytes = self.worldPreview.terrain_map.astype(np.uint8).tobytes()
                        f.write(terrain_bytes)
                    
                    # Write region data as binary if available
                    if self.worldPreview.region_map is not None:
                        region_bytes = self.worldPreview.region_map.astype(np.uint8).tobytes()
                        f.write(region_bytes)
                
                # Show confirmation
                self.showMessage("Export World", f"World exported to {file_path}")
            except Exception as e:
                self.showMessage("Error", f"Failed to export world: {str(e)}", error=True)
    
    def showMessage(self, title, message, error=False):
        """Show a message dialog"""
        if HAS_FLUENT:
            if error:
                msg_box = MessageBox(title, message, self)
                msg_box.exec()
            else:
                msg_box = MessageBox(title, message, self)
                msg_box.exec()
        else:
            if error:
                QMessageBox.critical(self, title, message)
            else:
                QMessageBox.information(self, title, message)