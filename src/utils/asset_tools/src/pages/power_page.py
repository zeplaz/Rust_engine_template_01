from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                            QGroupBox, QFormLayout, QTabWidget, QTabBar)

# Import Fluent widgets consistently from base_page
from .base_page import (LineEdit, SpinBox, ComboBox, PushButton,
                       DoubleSpinBox, CheckBox, RadioButton, FluentIcon)
    
from ..config.asset_config import SEGMENT_MEMBERSHIP

# Power system enums
POWER_PLANT_TYPES = [
    "Coal", "Nuclear", "Solar", "Wind", "Oil", "Gas", "Geothermal", "Hydro", "Biomass"
]

POWER_DISTRIBUTION_TYPES = [
    "ThreePhaseHeavyIndustrial", "ThreePhaseMediumIndustrial", "OnePhaseLightIndustrial",
    "ThreePhaseResidential", "OnePhaseResidential", "ThreePhaseLongDistance",
    "OnephaseLongDistance", "Mixed"
]

SWITCH_STATES = ["Open", "Closed"]
OPERATION_MECHANISMS = ["Manual", "Automatic"]
SWITCH_BEHAVIORS = ["NonReclosing", "AutoReclosing"]
OPERATIONAL_STATUSES = [
    "Standby", "Operational", "Maintenance", "OutOfFuel", "StartingUp", 
    "ShuttingDown", "Decommissioned", "ExternalShutdown", "ReducedCapacity",
    "OverCapacity", "EnvironmentalShutdown"
]

class PowerPage(QWidget):
    """Power infrastructure configuration page"""
    
    # Signal to notify when configuration changes
    assetConfigChanged = pyqtSignal(dict)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.initUI()
        
    def initUI(self):
        """Initialize the UI"""
        # Main layout
        main_layout = QVBoxLayout(self)
        
        # Create tab widget for different power entity types
        self.tab_widget = QTabWidget()
        
        # Configure tab behavior for better usability
        self.tab_widget.setTabsClosable(False)
        self.tab_widget.setMovable(False)
        self.tab_widget.setDocumentMode(True)  # Make tabs look cleaner
        
        # Subclass the tab bar to capture clicks directly
        tabBar = self.tab_widget.tabBar()
        
        # Override the tab click event to force tab switching
        def mouseReleaseEvent(self, event):
            index = self.tabAt(event.pos())
            if index >= 0:
                self.tab_widget.setCurrentIndex(index)
            QTabBar.mouseReleaseEvent(self, event)
            
        # Apply the override to the tab bar
        tabBar.__class__.mouseReleaseEvent = mouseReleaseEvent
        
        # Create tabs for different power entity types
        power_plant_tab = self.createPowerPlantTab()
        substation_tab = self.createSubstationTab()
        transformer_tab = self.createTransformerTab()
        switch_tab = self.createSwitchTab()
        
        # Add tabs to tab widget
        self.tab_widget.addTab(power_plant_tab, "Power Plant")
        self.tab_widget.addTab(substation_tab, "Substation")
        self.tab_widget.addTab(transformer_tab, "Transformer")
        self.tab_widget.addTab(switch_tab, "Switch")
        
        # Add tab widget to main layout
        main_layout.addWidget(self.tab_widget)
        
        # Add save button
        self.save_button = PushButton("Save Power Entity")
        self.save_button.setIcon(FluentIcon.SAVE)
        main_layout.addWidget(self.save_button)
        main_layout.addStretch()
        
        # Connect signals
        self.save_button.clicked.connect(self.savePower)
        # Connect with direct connection for better reliability
        self.tab_widget.currentChanged.connect(self.onTabChanged, Qt.DirectConnection)
        
    def onTabChanged(self, index):
        """Handle tab changes
        
        Args:
            index: New tab index
        """
        # Force the tab to change - this is a critical fix for tab navigation
        self.tab_widget.setCurrentIndex(index)
        
        # Update config after tab change
        self.updateConfig()
        
    def createPowerPlantTab(self):
        """Create power plant configuration tab"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create form layouts
        building_form = QFormLayout()
        plant_form = QFormLayout()
        electrical_form = QFormLayout()
        
        # Building properties
        self.pp_name_input = LineEdit()
        self.pp_segment_combo = ComboBox()
        self.pp_size_x_spin = SpinBox()
        self.pp_size_y_spin = SpinBox()
        self.pp_height_spin = SpinBox()
        self.pp_construction_cost_spin = SpinBox()
        
        # Power plant specific properties
        self.pp_type_combo = ComboBox()
        self.pp_max_output_spin = DoubleSpinBox()
        self.pp_efficiency_spin = DoubleSpinBox()
        self.pp_status_combo = ComboBox()
        
        # Electrical properties
        self.pp_base_load_spin = DoubleSpinBox()
        self.pp_max_transfer_spin = DoubleSpinBox()
        self.pp_capacity_spin = DoubleSpinBox()
        
        # Set up controls
        self.pp_name_input.setPlaceholderText("Power plant name")
        self.pp_segment_combo.addItems(SEGMENT_MEMBERSHIP)
        
        self.pp_size_x_spin.setRange(1, 100)
        self.pp_size_x_spin.setValue(2)
        
        self.pp_size_y_spin.setRange(1, 100)
        self.pp_size_y_spin.setValue(2)
        
        self.pp_height_spin.setRange(1, 50)
        self.pp_height_spin.setValue(2)
        
        self.pp_construction_cost_spin.setRange(1000, 10000000)
        self.pp_construction_cost_spin.setValue(50000)
        self.pp_construction_cost_spin.setSingleStep(1000)
        
        self.pp_type_combo.addItems(POWER_PLANT_TYPES)
        
        self.pp_max_output_spin.setRange(0, 2000000)
        self.pp_max_output_spin.setValue(10000)
        self.pp_max_output_spin.setSingleStep(1000)
        self.pp_max_output_spin.setSuffix(" kW")
        
        self.pp_efficiency_spin.setRange(0, 1.0)
        self.pp_efficiency_spin.setValue(0.75)
        self.pp_efficiency_spin.setSingleStep(0.05)
        
        self.pp_status_combo.addItems(OPERATIONAL_STATUSES)
        self.pp_status_combo.setCurrentText("Standby")
        
        self.pp_base_load_spin.setRange(0, 10000)
        self.pp_base_load_spin.setValue(500)
        self.pp_base_load_spin.setSingleStep(100)
        self.pp_base_load_spin.setSuffix(" kW")
        
        self.pp_max_transfer_spin.setRange(0, 2000000)
        self.pp_max_transfer_spin.setValue(15000)
        self.pp_max_transfer_spin.setSingleStep(1000)
        self.pp_max_transfer_spin.setSuffix(" kW")
        
        self.pp_capacity_spin.setRange(0, 2000000)
        self.pp_capacity_spin.setValue(20000)
        self.pp_capacity_spin.setSingleStep(1000)
        self.pp_capacity_spin.setSuffix(" kW")
        
        # Add widgets to form layouts
        building_form.addRow("Name:", self.pp_name_input)
        building_form.addRow("Segment:", self.pp_segment_combo)
        building_form.addRow("Size X:", self.pp_size_x_spin)
        building_form.addRow("Size Y:", self.pp_size_y_spin)
        building_form.addRow("Height:", self.pp_height_spin)
        building_form.addRow("Construction Cost:", self.pp_construction_cost_spin)
        
        plant_form.addRow("Plant Type:", self.pp_type_combo)
        plant_form.addRow("Max Output:", self.pp_max_output_spin)
        plant_form.addRow("Efficiency:", self.pp_efficiency_spin)
        plant_form.addRow("Status:", self.pp_status_combo)
        
        electrical_form.addRow("Base Load:", self.pp_base_load_spin)
        electrical_form.addRow("Max Transfer:", self.pp_max_transfer_spin)
        electrical_form.addRow("Capacity:", self.pp_capacity_spin)
        
        # Create group boxes
        building_group = QGroupBox("Building Properties")
        building_group.setLayout(building_form)
        
        plant_group = QGroupBox("Power Plant Properties")
        plant_group.setLayout(plant_form)
        
        electrical_group = QGroupBox("Electrical Properties")
        electrical_group.setLayout(electrical_form)
        
        # Add groups to layout
        layout.addWidget(building_group)
        layout.addWidget(plant_group)
        layout.addWidget(electrical_group)
        layout.addStretch()
        
        # Connect signals for live updates
        self.pp_name_input.textChanged.connect(self.updateConfig)
        self.pp_segment_combo.currentTextChanged.connect(self.updateConfig)
        self.pp_size_x_spin.valueChanged.connect(self.updateConfig)
        self.pp_size_y_spin.valueChanged.connect(self.updateConfig)
        self.pp_height_spin.valueChanged.connect(self.updateConfig)
        self.pp_construction_cost_spin.valueChanged.connect(self.updateConfig)
        self.pp_type_combo.currentTextChanged.connect(self.updateConfig)
        self.pp_max_output_spin.valueChanged.connect(self.updateConfig)
        self.pp_efficiency_spin.valueChanged.connect(self.updateConfig)
        self.pp_status_combo.currentTextChanged.connect(self.updateConfig)
        self.pp_base_load_spin.valueChanged.connect(self.updateConfig)
        self.pp_max_transfer_spin.valueChanged.connect(self.updateConfig)
        self.pp_capacity_spin.valueChanged.connect(self.updateConfig)
        
        return tab
    
    def createSubstationTab(self):
        """Create substation configuration tab"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create form layouts
        building_form = QFormLayout()
        substation_form = QFormLayout()
        electrical_form = QFormLayout()
        
        # Building properties
        self.sub_name_input = LineEdit()
        self.sub_segment_combo = ComboBox()
        self.sub_size_x_spin = SpinBox()
        self.sub_size_y_spin = SpinBox()
        self.sub_height_spin = SpinBox()
        self.sub_construction_cost_spin = SpinBox()
        
        # Substation specific properties
        self.sub_input_voltage_type = ComboBox()
        self.sub_input_voltage_value = DoubleSpinBox()
        self.sub_output_voltage_type = ComboBox()
        self.sub_output_voltage_value = DoubleSpinBox()
        self.sub_radius_spin = DoubleSpinBox()
        
        # Electrical properties
        self.sub_base_load_spin = DoubleSpinBox()
        self.sub_max_transfer_spin = DoubleSpinBox()
        self.sub_capacity_spin = DoubleSpinBox()
        
        # Set up controls
        self.sub_name_input.setPlaceholderText("Substation name")
        self.sub_segment_combo.addItems(SEGMENT_MEMBERSHIP)
        
        self.sub_size_x_spin.setRange(1, 50)
        self.sub_size_x_spin.setValue(1)
        
        self.sub_size_y_spin.setRange(1, 50)
        self.sub_size_y_spin.setValue(1)
        
        self.sub_height_spin.setRange(1, 20)
        self.sub_height_spin.setValue(1)
        
        self.sub_construction_cost_spin.setRange(500, 1000000)
        self.sub_construction_cost_spin.setValue(10000)
        self.sub_construction_cost_spin.setSingleStep(500)
        
        self.sub_input_voltage_type.addItems(POWER_DISTRIBUTION_TYPES)
        
        self.sub_input_voltage_value.setRange(0, 1000)
        self.sub_input_voltage_value.setValue(132)
        self.sub_input_voltage_value.setSingleStep(1)
        self.sub_input_voltage_value.setSuffix(" kV")
        
        self.sub_output_voltage_type.addItems(POWER_DISTRIBUTION_TYPES)
        
        self.sub_output_voltage_value.setRange(0, 1000)
        self.sub_output_voltage_value.setValue(33)
        self.sub_output_voltage_value.setSingleStep(1)
        self.sub_output_voltage_value.setSuffix(" kV")
        
        self.sub_radius_spin.setRange(10, 1000)
        self.sub_radius_spin.setValue(100)
        self.sub_radius_spin.setSingleStep(10)
        self.sub_radius_spin.setSuffix(" m")
        
        self.sub_base_load_spin.setRange(0, 5000)
        self.sub_base_load_spin.setValue(50)
        self.sub_base_load_spin.setSingleStep(10)
        self.sub_base_load_spin.setSuffix(" kW")
        
        self.sub_max_transfer_spin.setRange(0, 1000000)
        self.sub_max_transfer_spin.setValue(50000)
        self.sub_max_transfer_spin.setSingleStep(1000)
        self.sub_max_transfer_spin.setSuffix(" kW")
        
        self.sub_capacity_spin.setRange(0, 1000000)
        self.sub_capacity_spin.setValue(100000)
        self.sub_capacity_spin.setSingleStep(1000)
        self.sub_capacity_spin.setSuffix(" kW")
        
        # Add widgets to form layouts
        building_form.addRow("Name:", self.sub_name_input)
        building_form.addRow("Segment:", self.sub_segment_combo)
        building_form.addRow("Size X:", self.sub_size_x_spin)
        building_form.addRow("Size Y:", self.sub_size_y_spin)
        building_form.addRow("Height:", self.sub_height_spin)
        building_form.addRow("Construction Cost:", self.sub_construction_cost_spin)
        
        substation_form.addRow("Input Voltage Type:", self.sub_input_voltage_type)
        substation_form.addRow("Input Voltage:", self.sub_input_voltage_value)
        substation_form.addRow("Output Voltage Type:", self.sub_output_voltage_type)
        substation_form.addRow("Output Voltage:", self.sub_output_voltage_value)
        substation_form.addRow("Service Radius:", self.sub_radius_spin)
        
        electrical_form.addRow("Base Load:", self.sub_base_load_spin)
        electrical_form.addRow("Max Transfer:", self.sub_max_transfer_spin)
        electrical_form.addRow("Capacity:", self.sub_capacity_spin)
        
        # Create group boxes
        building_group = QGroupBox("Building Properties")
        building_group.setLayout(building_form)
        
        substation_group = QGroupBox("Substation Properties")
        substation_group.setLayout(substation_form)
        
        electrical_group = QGroupBox("Electrical Properties")
        electrical_group.setLayout(electrical_form)
        
        # Add groups to layout
        layout.addWidget(building_group)
        layout.addWidget(substation_group)
        layout.addWidget(electrical_group)
        layout.addStretch()
        
        # Connect signals for live updates
        self.sub_name_input.textChanged.connect(self.updateConfig)
        self.sub_segment_combo.currentTextChanged.connect(self.updateConfig)
        self.sub_size_x_spin.valueChanged.connect(self.updateConfig)
        self.sub_size_y_spin.valueChanged.connect(self.updateConfig)
        self.sub_height_spin.valueChanged.connect(self.updateConfig)
        self.sub_construction_cost_spin.valueChanged.connect(self.updateConfig)
        self.sub_input_voltage_type.currentTextChanged.connect(self.updateConfig)
        self.sub_input_voltage_value.valueChanged.connect(self.updateConfig)
        self.sub_output_voltage_type.currentTextChanged.connect(self.updateConfig)
        self.sub_output_voltage_value.valueChanged.connect(self.updateConfig)
        self.sub_radius_spin.valueChanged.connect(self.updateConfig)
        self.sub_base_load_spin.valueChanged.connect(self.updateConfig)
        self.sub_max_transfer_spin.valueChanged.connect(self.updateConfig)
        self.sub_capacity_spin.valueChanged.connect(self.updateConfig)
        
        return tab
    
    def createTransformerTab(self):
        """Create transformer configuration tab"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create form layouts
        building_form = QFormLayout()
        transformer_form = QFormLayout()
        thermal_form = QFormLayout()
        
        # Building properties
        self.tf_name_input = LineEdit()
        self.tf_segment_combo = ComboBox()
        self.tf_size_x_spin = SpinBox()
        self.tf_size_y_spin = SpinBox()
        self.tf_height_spin = SpinBox()
        self.tf_construction_cost_spin = SpinBox()
        
        # Transformer specific properties
        self.tf_input_voltage_spin = DoubleSpinBox()
        self.tf_output_voltage_spin = DoubleSpinBox()
        self.tf_max_transfer_spin = DoubleSpinBox()
        self.tf_capacity_spin = DoubleSpinBox()
        
        # Thermal properties
        self.tf_current_temp_spin = DoubleSpinBox()
        self.tf_max_temp_spin = DoubleSpinBox()
        
        # Set up controls
        self.tf_name_input.setPlaceholderText("Transformer name")
        self.tf_segment_combo.addItems(SEGMENT_MEMBERSHIP)
        
        self.tf_size_x_spin.setRange(1, 10)
        self.tf_size_x_spin.setValue(1)
        
        self.tf_size_y_spin.setRange(1, 10)
        self.tf_size_y_spin.setValue(1)
        
        self.tf_height_spin.setRange(1, 5)
        self.tf_height_spin.setValue(1)
        
        self.tf_construction_cost_spin.setRange(100, 100000)
        self.tf_construction_cost_spin.setValue(5000)
        self.tf_construction_cost_spin.setSingleStep(100)
        
        self.tf_input_voltage_spin.setRange(1, 1000)
        self.tf_input_voltage_spin.setValue(33)
        self.tf_input_voltage_spin.setSingleStep(1)
        self.tf_input_voltage_spin.setSuffix(" kV")
        
        self.tf_output_voltage_spin.setRange(0.1, 500)
        self.tf_output_voltage_spin.setValue(11)
        self.tf_output_voltage_spin.setSingleStep(0.1)
        self.tf_output_voltage_spin.setSuffix(" kV")
        
        self.tf_max_transfer_spin.setRange(10, 100000)
        self.tf_max_transfer_spin.setValue(5000)
        self.tf_max_transfer_spin.setSingleStep(100)
        self.tf_max_transfer_spin.setSuffix(" kW")
        
        self.tf_capacity_spin.setRange(10, 100000)
        self.tf_capacity_spin.setValue(10000)
        self.tf_capacity_spin.setSingleStep(100)
        self.tf_capacity_spin.setSuffix(" kW")
        
        self.tf_current_temp_spin.setRange(0, 200)
        self.tf_current_temp_spin.setValue(25)
        self.tf_current_temp_spin.setSingleStep(1)
        self.tf_current_temp_spin.setSuffix(" °C")
        
        self.tf_max_temp_spin.setRange(50, 250)
        self.tf_max_temp_spin.setValue(120)
        self.tf_max_temp_spin.setSingleStep(5)
        self.tf_max_temp_spin.setSuffix(" °C")
        
        # Add widgets to form layouts
        building_form.addRow("Name:", self.tf_name_input)
        building_form.addRow("Segment:", self.tf_segment_combo)
        building_form.addRow("Size X:", self.tf_size_x_spin)
        building_form.addRow("Size Y:", self.tf_size_y_spin)
        building_form.addRow("Height:", self.tf_height_spin)
        building_form.addRow("Construction Cost:", self.tf_construction_cost_spin)
        
        transformer_form.addRow("Input Voltage:", self.tf_input_voltage_spin)
        transformer_form.addRow("Output Voltage:", self.tf_output_voltage_spin)
        transformer_form.addRow("Max Transfer:", self.tf_max_transfer_spin)
        transformer_form.addRow("Capacity:", self.tf_capacity_spin)
        
        thermal_form.addRow("Current Temperature:", self.tf_current_temp_spin)
        thermal_form.addRow("Max Temperature:", self.tf_max_temp_spin)
        
        # Create group boxes
        building_group = QGroupBox("Building Properties")
        building_group.setLayout(building_form)
        
        transformer_group = QGroupBox("Transformer Properties")
        transformer_group.setLayout(transformer_form)
        
        thermal_group = QGroupBox("Thermal Properties")
        thermal_group.setLayout(thermal_form)
        
        # Add groups to layout
        layout.addWidget(building_group)
        layout.addWidget(transformer_group)
        layout.addWidget(thermal_group)
        layout.addStretch()
        
        # Connect signals for live updates
        self.tf_name_input.textChanged.connect(self.updateConfig)
        self.tf_segment_combo.currentTextChanged.connect(self.updateConfig)
        self.tf_size_x_spin.valueChanged.connect(self.updateConfig)
        self.tf_size_y_spin.valueChanged.connect(self.updateConfig)
        self.tf_height_spin.valueChanged.connect(self.updateConfig)
        self.tf_construction_cost_spin.valueChanged.connect(self.updateConfig)
        self.tf_input_voltage_spin.valueChanged.connect(self.updateConfig)
        self.tf_output_voltage_spin.valueChanged.connect(self.updateConfig)
        self.tf_max_transfer_spin.valueChanged.connect(self.updateConfig)
        self.tf_capacity_spin.valueChanged.connect(self.updateConfig)
        self.tf_current_temp_spin.valueChanged.connect(self.updateConfig)
        self.tf_max_temp_spin.valueChanged.connect(self.updateConfig)
        
        return tab
    
    def createSwitchTab(self):
        """Create switch configuration tab"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create form layouts
        building_form = QFormLayout()
        switch_form = QFormLayout()
        
        # Building properties
        self.sw_name_input = LineEdit()
        self.sw_segment_combo = ComboBox()
        self.sw_size_x_spin = SpinBox()
        self.sw_size_y_spin = SpinBox()
        self.sw_height_spin = SpinBox()
        self.sw_construction_cost_spin = SpinBox()
        
        # Switch specific properties
        self.sw_state_combo = ComboBox()
        self.sw_max_current_spin = DoubleSpinBox()
        self.sw_operation_combo = ComboBox()
        self.sw_behavior_combo = ComboBox()
        self.sw_retry_duration_spin = DoubleSpinBox()
        self.sw_operation_time_spin = DoubleSpinBox()
        
        # Set up controls
        self.sw_name_input.setPlaceholderText("Switch name")
        self.sw_segment_combo.addItems(SEGMENT_MEMBERSHIP)
        
        self.sw_size_x_spin.setRange(1, 5)
        self.sw_size_x_spin.setValue(1)
        
        self.sw_size_y_spin.setRange(1, 5)
        self.sw_size_y_spin.setValue(1)
        
        self.sw_height_spin.setRange(1, 3)
        self.sw_height_spin.setValue(1)
        
        self.sw_construction_cost_spin.setRange(100, 10000)
        self.sw_construction_cost_spin.setValue(1000)
        self.sw_construction_cost_spin.setSingleStep(100)
        
        self.sw_state_combo.addItems(SWITCH_STATES)
        
        self.sw_max_current_spin.setRange(1, 10000)
        self.sw_max_current_spin.setValue(1000)
        self.sw_max_current_spin.setSingleStep(100)
        self.sw_max_current_spin.setSuffix(" A")
        
        self.sw_operation_combo.addItems(OPERATION_MECHANISMS)
        
        self.sw_behavior_combo.addItems(SWITCH_BEHAVIORS)
        
        self.sw_retry_duration_spin.setRange(0, 60)
        self.sw_retry_duration_spin.setValue(5)
        self.sw_retry_duration_spin.setSingleStep(1)
        self.sw_retry_duration_spin.setSuffix(" s")
        
        self.sw_operation_time_spin.setRange(0, 10)
        self.sw_operation_time_spin.setValue(0.2)
        self.sw_operation_time_spin.setSingleStep(0.1)
        self.sw_operation_time_spin.setSuffix(" s")
        
        # Add widgets to form layouts
        building_form.addRow("Name:", self.sw_name_input)
        building_form.addRow("Segment:", self.sw_segment_combo)
        building_form.addRow("Size X:", self.sw_size_x_spin)
        building_form.addRow("Size Y:", self.sw_size_y_spin)
        building_form.addRow("Height:", self.sw_height_spin)
        building_form.addRow("Construction Cost:", self.sw_construction_cost_spin)
        
        switch_form.addRow("Initial State:", self.sw_state_combo)
        switch_form.addRow("Max Current:", self.sw_max_current_spin)
        switch_form.addRow("Operation Type:", self.sw_operation_combo)
        switch_form.addRow("Behavior:", self.sw_behavior_combo)
        switch_form.addRow("Retry Duration:", self.sw_retry_duration_spin)
        switch_form.addRow("Operation Time:", self.sw_operation_time_spin)
        
        # Create group boxes
        building_group = QGroupBox("Building Properties")
        building_group.setLayout(building_form)
        
        switch_group = QGroupBox("Switch Properties")
        switch_group.setLayout(switch_form)
        
        # Add groups to layout
        layout.addWidget(building_group)
        layout.addWidget(switch_group)
        layout.addStretch()
        
        # Connect signals for live updates
        self.sw_name_input.textChanged.connect(self.updateConfig)
        self.sw_segment_combo.currentTextChanged.connect(self.updateConfig)
        self.sw_size_x_spin.valueChanged.connect(self.updateConfig)
        self.sw_size_y_spin.valueChanged.connect(self.updateConfig)
        self.sw_height_spin.valueChanged.connect(self.updateConfig)
        self.sw_construction_cost_spin.valueChanged.connect(self.updateConfig)
        self.sw_state_combo.currentTextChanged.connect(self.updateConfig)
        self.sw_max_current_spin.valueChanged.connect(self.updateConfig)
        self.sw_operation_combo.currentTextChanged.connect(self.updateConfig)
        self.sw_behavior_combo.currentTextChanged.connect(self.updateConfig)
        self.sw_retry_duration_spin.valueChanged.connect(self.updateConfig)
        self.sw_operation_time_spin.valueChanged.connect(self.updateConfig)
        
        return tab
    
    def updateConfig(self):
        """Update the asset configuration and emit signal"""
        current_tab_index = self.tab_widget.currentIndex()
        
        # Base config common to all types
        config = {
            "is_building": True,
            "is_vehicle": False,
            "is_power": True,
        }
        
        # Power Plant tab
        if current_tab_index == 0:
            config.update({
                "asset_name": self.pp_name_input.text(),
                "asset_type": "PowerPlant",
                "power_plant_type": self.pp_type_combo.currentText(),
                "segment": self.pp_segment_combo.currentText(),
                "building_size_x": self.pp_size_x_spin.value(),
                "building_size_y": self.pp_size_y_spin.value(),
                "building_height": self.pp_height_spin.value(),
                "construction_cost": self.pp_construction_cost_spin.value(),
                "power_generation": self.pp_max_output_spin.value(),
                "efficiency": self.pp_efficiency_spin.value(),
                "operational_status": self.pp_status_combo.currentText(),
                "base_load": self.pp_base_load_spin.value(),
                "max_transfer": self.pp_max_transfer_spin.value(),
                "capacity": self.pp_capacity_spin.value(),
            })
        
        # Substation tab
        elif current_tab_index == 1:
            config.update({
                "asset_name": self.sub_name_input.text(),
                "asset_type": "Substation",
                "segment": self.sub_segment_combo.currentText(),
                "building_size_x": self.sub_size_x_spin.value(),
                "building_size_y": self.sub_size_y_spin.value(),
                "building_height": self.sub_height_spin.value(),
                "construction_cost": self.sub_construction_cost_spin.value(),
                "input_voltage_type": self.sub_input_voltage_type.currentText(),
                "input_voltage": self.sub_input_voltage_value.value(),
                "output_voltage_type": self.sub_output_voltage_type.currentText(),
                "output_voltage": self.sub_output_voltage_value.value(),
                "service_radius": self.sub_radius_spin.value(),
                "base_load": self.sub_base_load_spin.value(),
                "max_transfer": self.sub_max_transfer_spin.value(),
                "capacity": self.sub_capacity_spin.value(),
            })
        
        # Transformer tab
        elif current_tab_index == 2:
            config.update({
                "asset_name": self.tf_name_input.text(),
                "asset_type": "Transformer",
                "segment": self.tf_segment_combo.currentText(),
                "building_size_x": self.tf_size_x_spin.value(),
                "building_size_y": self.tf_size_y_spin.value(),
                "building_height": self.tf_height_spin.value(),
                "construction_cost": self.tf_construction_cost_spin.value(),
                "input_voltage": self.tf_input_voltage_spin.value(),
                "output_voltage": self.tf_output_voltage_spin.value(),
                "max_transfer": self.tf_max_transfer_spin.value(),
                "capacity": self.tf_capacity_spin.value(),
                "current_temperature": self.tf_current_temp_spin.value(),
                "max_temperature": self.tf_max_temp_spin.value(),
            })
        
        # Switch tab
        elif current_tab_index == 3:
            config.update({
                "asset_name": self.sw_name_input.text(),
                "asset_type": "Switch",
                "segment": self.sw_segment_combo.currentText(),
                "building_size_x": self.sw_size_x_spin.value(),
                "building_size_y": self.sw_size_y_spin.value(),
                "building_height": self.sw_height_spin.value(),
                "construction_cost": self.sw_construction_cost_spin.value(),
                "switch_state": self.sw_state_combo.currentText(),
                "max_current": self.sw_max_current_spin.value(),
                "operation_mechanism": self.sw_operation_combo.currentText(),
                "switch_behavior": self.sw_behavior_combo.currentText(),
                "retry_duration": self.sw_retry_duration_spin.value(),
                "operation_time": self.sw_operation_time_spin.value(),
            })
        
        self.assetConfigChanged.emit(config)
    
    def savePower(self):
        """Signal to save the power configuration"""
        current_tab_index = self.tab_widget.currentIndex()
        
        # Check which tab is active and validate its fields
        if current_tab_index == 0:  # Power Plant
            if not self.pp_name_input.text():
                from PyQt5.QtWidgets import QMessageBox
                QMessageBox.critical(self, "Error", "Power plant name is required")
                return
        elif current_tab_index == 1:  # Substation
            if not self.sub_name_input.text():
                from PyQt5.QtWidgets import QMessageBox
                QMessageBox.critical(self, "Error", "Substation name is required")
                return
        elif current_tab_index == 2:  # Transformer
            if not self.tf_name_input.text():
                from PyQt5.QtWidgets import QMessageBox
                QMessageBox.critical(self, "Error", "Transformer name is required")
                return
        elif current_tab_index == 3:  # Switch
            if not self.sw_name_input.text():
                from PyQt5.QtWidgets import QMessageBox
                QMessageBox.critical(self, "Error", "Switch name is required")
                return
                
        # Update config one last time before saving
        self.updateConfig()
        
        # Emit special signal to trigger save
        self.assetConfigChanged.emit({"_action": "save"})