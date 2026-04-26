import os
import sys
from PyQt5.QtCore import Qt, QSize
from PyQt5.QtGui import QIcon, QPixmap
from PyQt5.QtWidgets import QApplication, QMainWindow, QWidget, QLabel, QVBoxLayout, QHBoxLayout

# Import PyQt-Fluent-Widgets
from qfluentwidgets import (NavigationInterface, NavigationItemPosition, MessageBox,
                          SplitFluentWindow, Theme, setTheme, isDarkTheme, PushButton)
from qfluentwidgets import FluentTranslator
from qfluentwidgets import FluentIcon

# Import common base utilities
from .pages.base_page import BasePage

# Import our pages
from .pages.vehicle_page import VehiclePage
from .pages.dynamic_building_page import DynamicBuildingPage
from .pages.texture_page import TexturePage
from .pages.power_page import PowerPage
from .config.asset_config import AssetConfig, ASSET_TYPES, RESOURCE_TYPES

class MainWindow(SplitFluentWindow):
    def __init__(self):
        super().__init__()
        self.asset_config = AssetConfig()
        self.initWindow()
        self.initNavigation()
        self.connectSignals()

    def initWindow(self):
        # Set window title and size
        self.setWindowTitle("Processor Alpha Dine - Asset Editor")
        self.resize(1200, 800)

    def initNavigation(self):
        # Set up the navigation interface - slim width for icon-only nav
        self.navigationInterface.setFixedWidth(48)

        # Create the pages
        self.vehicle_page = VehiclePage(self)
        self.building_page = DynamicBuildingPage(self)
        self.texture_page = TexturePage(self)
        self.power_page  = PowerPage(self)
        # Create a welcome/dashboard home page
        self.home_page = self.createHomePage()

        # Set object names and tooltips
        self.home_page.setObjectName("Home")
        self.vehicle_page.setObjectName("Vehicles")
        self.building_page.setObjectName("Buildings")
        self.texture_page.setObjectName("Textures")
        self.power_page.setObjectName("PowerPage")

        # Set tooltips directly on the pages
        self.home_page.setToolTip("Home")
        self.vehicle_page.setToolTip("Vehicles")
        self.building_page.setToolTip("Buildings")
        self.texture_page.setToolTip("Textures")
        self.power_page.setToolTip("Power")

        # Add pages to navigation with icons only, tooltip will show on hover
        self.addSubInterface(self.home_page, FluentIcon.HOME, "")
        self.addSubInterface(self.vehicle_page, FluentIcon.CAR, "")
        self.addSubInterface(self.building_page, FluentIcon.FOLDER, "")
        self.addSubInterface(self.texture_page, FluentIcon.PHOTO, "")
        self.addSubInterface(self.power_page, FluentIcon.PROJECTOR, "")

        # Set default page
        self.navigationInterface.setCurrentItem("Home")

    def createHomePage(self):
        """Create a welcome/dashboard page that shows a workflow overview"""
        home_page = QWidget()
        layout = QVBoxLayout(home_page)
        layout.setContentsMargins(20, 20, 20, 20)

        # Add welcome title
        title = QLabel("Welcome to Processor Alpha Dine Asset Editor")
        title.setStyleSheet("font-size: 24px; font-weight: bold; color: #5dca31;")
        layout.addWidget(title)

        # Add subtitle
        subtitle = QLabel("Create and modify game assets for Processor Alpha Dine")
        subtitle.setStyleSheet("font-size: 16px; color: #cccccc;")
        layout.addWidget(subtitle)

        # Add separator
        separator = QWidget()
        separator.setFixedHeight(2)
        separator.setStyleSheet("background-color: rgba(93, 202, 49, 0.3);")
        layout.addWidget(separator)
        layout.addSpacing(20)

        # Create workflow section
        workflow_title = QLabel("Asset Creation Workflow")
        workflow_title.setStyleSheet("font-size: 20px; font-weight: bold; color: #ffffff;")
        layout.addWidget(workflow_title)

        # Create workflow steps
        steps = [
            ("1. Choose Asset Type", "Select from the navigation menu: Vehicle, Building, or Texture"),
            ("2. Create or Edit", "Create a new asset or select an existing one to edit"),
            ("3. Configure Properties", "Set the asset's properties in the detail tabs"),
            ("4. Test Preview", "Check the preview tab to ensure the asset looks correct"),
            ("5. Save Changes", "Save your changes to update the asset configuration"),
            ("6. Export for Game", "Export the asset in FlatBuffer format for use in the game")
        ]

        for step, description in steps:
            step_widget = QWidget()
            step_layout = QHBoxLayout(step_widget)
            step_layout.setContentsMargins(0, 5, 0, 5)

            # Step number
            step_num = QLabel(step)
            step_num.setStyleSheet("font-size: 16px; font-weight: bold; color: #5dca31;")
            step_num.setFixedWidth(200)

            # Description
            desc = QLabel(description)
            desc.setStyleSheet("font-size: 14px; color: #cccccc;")
            desc.setWordWrap(True)

            step_layout.addWidget(step_num)
            step_layout.addWidget(desc, 1)  # 1 = stretch

            layout.addWidget(step_widget)

        # Add spacing and stretch to push content to top
        layout.addSpacing(20)
        layout.addStretch()

        # Add quick action buttons
        quick_actions = QWidget()
        actions_layout = QHBoxLayout(quick_actions)

        # Create buttons for quick actions
        new_vehicle_btn = PushButton("New Vehicle")
        new_vehicle_btn.setIcon(FluentIcon.CAR)

        new_building_btn = PushButton("New Building")
        new_building_btn.setIcon(FluentIcon.FOLDER)

        # Style buttons
        new_vehicle_btn.setProperty("class", "accentButton")
        new_building_btn.setProperty("class", "accentButton")

        # Connect buttons to navigation
        new_vehicle_btn.clicked.connect(lambda: self.switchToVehiclePage())
        new_building_btn.clicked.connect(lambda: self.switchToBuildingPage())

        # Add buttons to layout
        actions_layout.addStretch()
        actions_layout.addWidget(new_vehicle_btn)
        actions_layout.addSpacing(10)
        actions_layout.addWidget(new_building_btn)
        actions_layout.addStretch()

        layout.addWidget(quick_actions)

        return home_page

    def switchToVehiclePage(self):
        """Switch to the vehicle page and start creating a new vehicle"""
        self.navigationInterface.setCurrentItem("Vehicles")

        # Create a new vehicle
        self.vehicle_page.createNew()

    def switchToBuildingPage(self):
        """Switch to the building page and start creating a new building"""
        self.navigationInterface.setCurrentItem("Buildings")

        # Create a new building
        self.building_page.createNew()

    def connectSignals(self):
        # Connect signals between pages
        self.vehicle_page.assetConfigChanged.connect(self.updateAssetConfig)
        self.building_page.assetConfigChanged.connect(self.updateAssetConfig)
        self.texture_page.textureSelected.connect(self.updateTexture)

    def updateAssetConfig(self, config_data):
        # Check if this is a save action
        if config_data.get("_action") == "save":
            self.saveAsset()
            return

        # Update the asset configuration silently (without logging)
        for key, value in config_data.items():
            if key != "_action":  # Skip special action keys
                setattr(self.asset_config, key, value)

    def updateTexture(self, texture_path):
        # Update the texture in the asset configuration silently
        self.asset_config.texture_path = texture_path

    def saveAsset(self):
        # Save the asset configuration
        try:
            self.asset_config.save()
            w = MessageBox(
                "Save Successful",
                "Asset configuration was saved successfully.",
                self
            )
            w.exec_()
        except Exception as e:
            w = MessageBox(
                "Save Failed",
                f"Failed to save asset configuration: {str(e)}",
                self
            )
            w.exec_()

def main():
    # Create application
    app = QApplication(sys.argv)

    # Set theme to dark
    setTheme(Theme.DARK)

    # Set translator
    fluentTranslator = FluentTranslator()
    app.installTranslator(fluentTranslator)

    # Apply custom stylesheet with green, blue and black colors
    custom_style = """
    QWidget {
        background-color: #111111;
        color: #CCCCCC;
    }
    QGroupBox {
        border: 1px solid #5dca31;
        border-radius: 4px;
        margin-top: 0.5em;
        background-color: rgba(93, 202, 49, 0.05);
        padding: 8px;
    }
    QGroupBox::title {
        subcontrol-origin: margin;
        left: 10px;
        padding: 0 5px;
        color: #5dca31;
        font-weight: bold;
    }

    /* Tab styling - IMPORTANT: Fixed tab hit area */
    QTabWidget {
        background-color: #111111;
    }
    QTabWidget::pane {
        border: 1px solid #5dca31;
        background-color: #111111;
        top: -1px; /* critical for tab appearance */
    }
    QTabBar {
        background-color: transparent;
    }
    QTabBar::tab {
        background-color: #222222;
        color: #CCCCCC;
        padding: 10px 20px;
        border: 1px solid #444444;
        min-width: 100px;
        margin-right: 4px;
        border-top-left-radius: 4px;
        border-top-right-radius: 4px;
    }
    QTabBar::tab:selected {
        background-color: #1a3a1a;
        color: #5dca31;
        border: 2px solid #5dca31;
        border-bottom: none;
    }
    QTabBar::tab:hover:!selected {
        background-color: #2a4a2a;
    }

    /* Form control styling */
    QLineEdit, QTextEdit, QSpinBox, QDoubleSpinBox, QComboBox {
        background-color: #222222;
        border: 1px solid #444444;
        border-radius: 3px;
        padding: 6px;
        color: #CCCCCC;
        selection-background-color: #3a5a3a;
    }
    QLineEdit:focus, QTextEdit:focus, QSpinBox:focus, QDoubleSpinBox:focus, QComboBox:focus {
        border: 1px solid #5dca31;
        background-color: #1a1a1a;
    }
    QComboBox::drop-down {
        border: 0px;
        background-color: #3a5a3a;
    }
    QComboBox QAbstractItemView {
        background-color: #222222;
        border: 1px solid #5dca31;
        selection-background-color: #3a5a3a;
    }

    /* Label styling */
    QLabel {
        color: #CCCCCC;
    }

    /* Accent button styling */
    .accentButton {
        background-color: #1a3a1a;
        border: 1px solid #5dca31;
        border-radius: 4px;
        padding: 8px 16px;
        color: #ffffff;
        font-weight: bold;
    }
    .accentButton:hover {
        background-color: #2a4a2a;
    }
    .accentButton:pressed {
        background-color: #3a5a3a;
    }
    """
    app.setStyleSheet(custom_style)

    # Create and show main window
    window = MainWindow()
    window.show()

    # Run application
    sys.exit(app.exec_())

if __name__ == "__main__":
    main()
