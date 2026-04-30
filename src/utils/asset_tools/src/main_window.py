import os
import sys
from PyQt5.QtCore import Qt, QSize
from PyQt5.QtGui import QIcon, QPixmap
from PyQt5.QtWidgets import (
    QApplication,
    QMainWindow,
    QWidget,
    QLabel,
    QVBoxLayout,
    QHBoxLayout,
    QFileDialog,
    QListWidget,
)

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
from .pages.transport_page import TransportPage
from .pages.worldgen_page import WorldGenPage
from .pages.terrain_registry_pages import (
    MaterialRegistryPage,
    MaterialRulesPage,
    TagRegistryPage,
)
from .config.asset_config import AssetConfig, ASSET_TYPES, RESOURCE_TYPES
from .config.content_constants import MASTER_ENTITY_FILTER_ROLES
from .integration.const_game_entities import load_power_voltage_levels_raw
from . import repo_paths

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
        self.power_page = PowerPage(self)
        self.transport_page = TransportPage(self)
        self.worldgen_page = WorldGenPage(self)
        self.material_registry_page = MaterialRegistryPage(self)
        self.tag_registry_page = TagRegistryPage(self)
        self.material_rules_page = MaterialRulesPage(self)
        # Create a welcome/dashboard home page
        self.home_page = self.createHomePage()

        # Set object names and tooltips
        self.home_page.setObjectName("Home")
        self.vehicle_page.setObjectName("Vehicles")
        self.building_page.setObjectName("Buildings")
        self.texture_page.setObjectName("Textures")
        self.worldgen_page.setObjectName("WorldGen")
        self.material_registry_page.setObjectName("Materials")
        self.tag_registry_page.setObjectName("Tags")
        self.material_rules_page.setObjectName("MatRules")
        self.transport_page.setObjectName("Transport")
        self.power_page.setObjectName("PowerPage")

        # Set tooltips directly on the pages
        self.home_page.setToolTip("Home")
        self.vehicle_page.setToolTip("Vehicles")
        self.building_page.setToolTip("Buildings")
        self.texture_page.setToolTip("Textures")
        self.worldgen_page.setToolTip("World generation")
        self.material_registry_page.setToolTip("Terrain material registry (JSON)")
        self.tag_registry_page.setToolTip("Terrain tag registry (JSON)")
        self.material_rules_page.setToolTip("Material rules (RON)")
        self.transport_page.setToolTip("Roads & Rails")
        self.power_page.setToolTip("Power")

        # Add pages to navigation with icons only, tooltip will show on hover
        self.addSubInterface(self.home_page, FluentIcon.HOME, "")
        self.addSubInterface(self.vehicle_page, FluentIcon.CAR, "")
        self.addSubInterface(self.building_page, FluentIcon.FOLDER, "")
        self.addSubInterface(self.texture_page, FluentIcon.PHOTO, "")
        _map = getattr(FluentIcon, "MAP", getattr(FluentIcon, "GLOBE", FluentIcon.FOLDER))
        self.addSubInterface(self.worldgen_page, _map, "")
        _doc = getattr(FluentIcon, "DOCUMENT", FluentIcon.FOLDER)
        self.addSubInterface(self.material_registry_page, _doc, "")
        _tag = getattr(FluentIcon, "TAG", FluentIcon.FOLDER)
        self.addSubInterface(self.tag_registry_page, _tag, "")
        _code = getattr(FluentIcon, "CODE", FluentIcon.FOLDER)
        self.addSubInterface(self.material_rules_page, _code, "")
        _bus = getattr(FluentIcon, "BUS", FluentIcon.CAR)
        self.addSubInterface(self.transport_page, _bus, "")
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

        # Content studio: canonical repo paths + legacy parity (folder browse, consts preview)
        studio_title = QLabel("Content studio")
        studio_title.setStyleSheet("font-size: 20px; font-weight: bold; color: #ffffff;")
        layout.addWidget(studio_title)

        roles_lbl = QLabel("Asset roles (tags): " + ", ".join(MASTER_ENTITY_FILTER_ROLES))
        roles_lbl.setStyleSheet("font-size: 13px; color: #aaaaaa;")
        roles_lbl.setWordWrap(True)
        layout.addWidget(roles_lbl)

        paths_lbl = QLabel(
            "<b>Repository root:</b> {}<br/>"
            "<b>Power definitions:</b> {}<br/>"
            "<b>World gen tuning (active):</b> {}<br/>"
            "<b>World gen tuning (example):</b> {}<br/>"
            "<b>Terrain materials (active):</b> {}<br/>"
            "<b>Terrain materials (example):</b> {}<br/>"
            "<b>Terrain tags (active):</b> {}<br/>"
            "<b>Terrain tags (example):</b> {}<br/>"
            "<b>Material rules (active):</b> {}<br/>"
            "<b>Material rules (example):</b> {}<br/>"
            "<b>Building types index:</b> {}<br/>"
            "<b>Buildings configs:</b> {}<br/>"
            "<b>Roads configs:</b> {}<br/>"
            "<b>Rails configs:</b> {}<br/>"
            "<b>Vehicles configs:</b> {}<br/>"
            "<b>Tiled assets:</b> {}".format(
                repo_paths.REPO_ROOT,
                repo_paths.plant_definitions_json,
                repo_paths.world_gen_tuning_json,
                repo_paths.world_gen_tuning_example_json,
                repo_paths.material_registry_json,
                repo_paths.material_registry_example_json,
                repo_paths.tag_registry_json,
                repo_paths.tag_registry_example_json,
                repo_paths.material_rules_ron,
                repo_paths.material_rules_example_ron,
                repo_paths.building_types_index_json,
                repo_paths.buildings_configs_dir,
                repo_paths.roads_configs_dir,
                repo_paths.rails_configs_dir,
                repo_paths.vehicles_configs_dir,
                repo_paths.tiled_assets_dir,
            )
        )
        paths_lbl.setTextFormat(Qt.RichText)
        paths_lbl.setWordWrap(True)
        paths_lbl.setStyleSheet("font-size: 13px; color: #cccccc;")
        layout.addWidget(paths_lbl)

        volts = load_power_voltage_levels_raw()
        if volts:
            preview = ", ".join(volts[:24])
            if len(volts) > 24:
                preview += ", …"
            vol_lbl = QLabel("<b>Power voltages (MV, from consts):</b> " + preview)
            vol_lbl.setTextFormat(Qt.RichText)
            vol_lbl.setWordWrap(True)
            vol_lbl.setStyleSheet("font-size: 12px; color: #88cc88;")
            layout.addWidget(vol_lbl)

        self._package_list = QListWidget()
        self._package_list.setMaximumHeight(140)
        browse_btn = PushButton("Browse package folder…")
        browse_btn.clicked.connect(self._on_browse_package_folder)
        layout.addWidget(browse_btn)
        layout.addWidget(self._package_list)

        dep_note = QLabel(
            "Legacy PySide editor under <code>utils/asset_tools/</code> is deprecated — see "
            "<code>prompts/designer_questions/tools_ui/spec/07_asset_editor_dual_chain_audit_v1.md</code>."
        )
        dep_note.setTextFormat(Qt.RichText)
        dep_note.setWordWrap(True)
        dep_note.setStyleSheet("font-size: 11px; color: #888888;")
        layout.addWidget(dep_note)

        layout.addSpacing(16)

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
            ("1. Choose Asset Type", "Select: Vehicle, Building, Texture, World generation, Materials, Tags, Rules, Transport, or Power"),
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

    def _on_browse_package_folder(self):
        """List JSON / image / Tiled files under a chosen folder (shallow legacy parity)."""
        start = str(repo_paths.assets_dir)
        folder = QFileDialog.getExistingDirectory(self, "Content package folder", start)
        if not folder:
            return
        self._package_list.clear()
        exts = {".json", ".png", ".tsx", ".tmx", ".const", ".dat"}
        count = 0
        try:
            for dirpath, _dirnames, filenames in os.walk(folder):
                for fn in sorted(filenames):
                    if count >= 500:
                        break
                    low = os.path.splitext(fn)[1].lower()
                    if low in exts:
                        self._package_list.addItem(os.path.join(dirpath, fn))
                        count += 1
                if count >= 500:
                    break
        except OSError:
            pass

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
