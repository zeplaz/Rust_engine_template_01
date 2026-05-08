import os
import json
from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                            QListWidget, QListWidgetItem, QSplitter, QTabWidget,
                            QFormLayout, QGroupBox, QFileDialog, QTextEdit,
                            QTabBar)
from PyQt5.QtGui import QFont, QColor, QPixmap

# Import base page
from .base_page import BasePage, EntityBasePage

# Import all widgets from base_page - we now always use Fluent widgets
from .base_page import (LineEdit, SpinBox, ComboBox, PushButton,
                       DoubleSpinBox, CheckBox, RadioButton, ScrollArea,
                       FluentIcon, ImageLabel)

from ..config.asset_config import (
    VEHICLE_TYPES,
    ROAD_VEHICLE_TYPES,
    SEGMENT_MEMBERSHIP,
    FUEL_SOURCE_CATEGORY,
    SOUND_CLASSES,
    fuel_types_for_category,
    normalize_fuel_type,
    normalize_fuel_source_category,
)

class VehiclePage(EntityBasePage):
    """Vehicle configuration page"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.entity_type = "Vehicle"  # Set entity type for button labels
        self.current_entity = None    # Will store the current vehicle
        self.is_new_entity = False    # For creating new vehicles
        self.is_editing = False       # For editing state
        self.initUI()
        
    def onTabChanged(self, index):
        """Handle tab changes
        
        Args:
            index: New tab index
        """
        # Force the tab to change - this is a critical fix for tab navigation
        self.tabs.setCurrentIndex(index)
        
        # Update the content for the selected tab
        if index == 0:  # Basic tab
            # Update basic tab if needed
            pass
        elif index == 1:  # Advanced tab
            # Update advanced tab if needed
            pass
        elif index == 2:  # Preview tab
            # Update preview with latest data
            self.updatePreview()
        
    def initUI(self):
        """Initialize the UI"""
        # === LEFT PANEL: Vehicle List ===
        self.vehicle_list_panel = self.createVehicleListPanel()
        
        # === RIGHT PANEL: Vehicle Details ===
        details_panel = QWidget()
        details_layout = QVBoxLayout(details_panel)
        
        # Dictionary of tabs for details panel
        tabs_dict = {
            "Basic": self.createBasicTab,
            "Advanced": self.createAdvancedTab,
            "Preview": self.createPreviewTab
        }
        
        # Create tabs widget for details panel
        self.tabs = QTabWidget()
        self.tabs.setTabsClosable(False)
        self.tabs.setMovable(False)
        self.tabs.setDocumentMode(True)  # Make tabs look cleaner
        
        # Subclass the tab bar to capture clicks directly
        tabBar = self.tabs.tabBar()
        
        # Override the tab click event to force tab switching
        def mouseReleaseEvent(self, event):
            index = self.tabAt(event.pos())
            if index >= 0:
                self.tabs.setCurrentIndex(index)
            QTabBar.mouseReleaseEvent(self, event)
            
        # Apply the override to the tab bar
        tabBar.__class__.mouseReleaseEvent = mouseReleaseEvent
        
        # Create and add tabs
        for tab_name, create_method in tabs_dict.items():
            tab = create_method()
            self.tabs.addTab(tab, tab_name)
        
        # Add tabs to details layout
        details_layout.addWidget(self.tabs)
        
        # Connect tab change signal - use direct connection for reliability
        self.tabs.currentChanged.connect(self.onTabChanged, Qt.DirectConnection)
        
        # Use the base class method to initialize the UI with our panels
        self.initEntityUI(
            tabs_dict=tabs_dict,
            entity_list_panel=self.vehicle_list_panel,
            details_panel=details_panel
        )
        
    def createVehicleListPanel(self):
        """Create panel for vehicle list"""
        panel = QWidget()
        layout = QVBoxLayout(panel)
        
        # Create search box
        search_box = LineEdit()
        search_box.setPlaceholderText("Search vehicles...")
        search_box.setClearButtonEnabled(True)
        
        layout.addWidget(search_box)
        
        # Create vehicle list
        self.list_widget = QListWidget()
        self.list_widget.setSelectionMode(QListWidget.SingleSelection)
        
        # Style the list
        self.list_widget.setStyleSheet("""
            QListWidget {
                background-color: rgba(93, 202, 49, 0.05);
                border: 1px solid #5dca31;
                border-radius: 5px;
            }
            QListWidget::item {
                padding: 8px;
                border-bottom: 1px solid rgba(93, 202, 49, 0.2);
            }
            QListWidget::item:selected {
                background-color: rgba(93, 202, 49, 0.2);
            }
            QListWidget::item:hover {
                background-color: rgba(93, 202, 49, 0.1);
            }
        """)
        
        # Sample vehicles - in a real app, these would come from a database or config
        sample_vehicles = [
            {"name": "Ural Truck", "type": "Road", "subtype": "Heavy Truck"},
            {"name": "City Bus", "type": "Road", "subtype": "Bus"},
            {"name": "Cargo Train", "type": "Rail", "subtype": "Locomotive"},
            {"name": "Semi Trailer", "type": "Road", "subtype": "Semi"},
            {"name": "Delivery Van", "type": "Road", "subtype": "Light Truck"}
        ]
        
        # Add sample vehicles to list
        for vehicle in sample_vehicles:
            item = QListWidgetItem(f"{vehicle['name']} ({vehicle['subtype']})")
            item.setData(Qt.UserRole, vehicle)
            self.list_widget.addItem(item)
        
        # Add list to scrollable area
        scroll_area = ScrollArea()
        scroll_area.setWidget(self.list_widget)
        scroll_area.setWidgetResizable(True)
        
        layout.addWidget(scroll_area)
        
        # Add type filter
        filter_layout = QHBoxLayout()
        filter_layout.addWidget(QLabel("Filter by type:"))
        
        type_filter = ComboBox()
        type_filter.addItems(["All"] + VEHICLE_TYPES)
        
        filter_layout.addWidget(type_filter)
        layout.addLayout(filter_layout)
        
        # Connect signals
        search_box.textChanged.connect(self.filterVehicles)
        type_filter.currentTextChanged.connect(self.filterByType)
        self.list_widget.itemSelectionChanged.connect(self.onVehicleSelected)
        
        return panel
        
    def createPreviewTab(self):
        """Create tab for preview"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create preview label
        self.preview_label = ImageLabel("No texture selected")
        self.preview_label.setFixedSize(400, 300)
        
        # Create info text box
        self.info_preview = QTextEdit()
        self.info_preview.setReadOnly(True)
        self.info_preview.setPlaceholderText("Vehicle information will appear here")
        
        # Add widgets to layout
        preview_layout = QVBoxLayout()
        preview_layout.addWidget(QLabel("Vehicle Preview:"))
        preview_layout.addWidget(self.preview_label)
        
        info_layout = QVBoxLayout()
        info_layout.addWidget(QLabel("Vehicle Information:"))
        info_layout.addWidget(self.info_preview)
        
        layout.addLayout(preview_layout)
        layout.addLayout(info_layout)
        
        return tab
        
    def filterVehicles(self, text):
        """Filter vehicles by search text"""
        # In a real app, this would search through the vehicle database
        for i in range(self.list_widget.count()):
            item = self.list_widget.item(i)
            if text.lower() in item.text().lower():
                item.setHidden(False)
            else:
                item.setHidden(True)
                
    def filterByType(self, vehicle_type):
        """Filter vehicles by type"""
        # In a real app, this would filter based on actual vehicle data
        if vehicle_type == "All":
            # Show all vehicles
            for i in range(self.list_widget.count()):
                self.list_widget.item(i).setHidden(False)
        else:
            # Filter by the selected type
            for i in range(self.list_widget.count()):
                item = self.list_widget.item(i)
                vehicle_data = item.data(Qt.UserRole)
                if vehicle_data["type"] == vehicle_type:
                    item.setHidden(False)
                else:
                    item.setHidden(True)
                    
    def onVehicleSelected(self):
        """Handle vehicle selection from list"""
        selected_items = self.list_widget.selectedItems()
        if selected_items:
            # Get the selected vehicle data
            vehicle_data = selected_items[0].data(Qt.UserRole)
            
            # Set current entity for EntityBasePage
            self.current_entity = vehicle_data
            
            # In a real app, this would load the vehicle's data into the form
            self.name_input.setText(vehicle_data["name"])
            self.vehicle_type_combo.setCurrentText(vehicle_data["type"])
            
            if vehicle_data["type"] == "Road":
                self.road_vehicle_type_combo.setCurrentText(vehicle_data["subtype"])
                self.road_vehicle_type_combo.setVisible(True)
            else:
                self.road_vehicle_type_combo.setVisible(False)

            if hasattr(self, "fuel_class_combo"):
                raw_class = vehicle_data.get("fuel_class", FUEL_SOURCE_CATEGORY[0])
                category = normalize_fuel_source_category(raw_class) or FUEL_SOURCE_CATEGORY[0]
                self.fuel_class_combo.blockSignals(True)
                ci = self.fuel_class_combo.findText(category)
                self.fuel_class_combo.setCurrentIndex(ci if ci >= 0 else 0)
                self.fuel_class_combo.blockSignals(False)
                self.limitFuelTypesByCategory(category)
                raw_fuel = vehicle_data.get("fuel_type", "")
                fuel = normalize_fuel_type(raw_fuel) if raw_fuel else self.fuel_type_combo.currentText()
                fi = self.fuel_type_combo.findText(fuel)
                if fi >= 0:
                    self.fuel_type_combo.setCurrentIndex(fi)
                elif self.fuel_type_combo.count() > 0:
                    self.fuel_type_combo.setCurrentIndex(0)
            
            # Update the preview
            self.updatePreview()
            
            # Update button states using base class method
            self.updateButtonStates()
        else:
            # No selection, update button states
            self.current_entity = None
            self.updateButtonStates()
            
    def updatePreview(self):
        """Update the preview with current vehicle information"""
        # Skip if no current entity and no form
        if not self.current_entity and not hasattr(self, 'name_input'):
            if hasattr(self, 'preview_data'):
                self.preview_data.setText("No vehicle data available.")
            return
            
        # Get data from widgets if available, otherwise use current_entity
        if hasattr(self, 'name_input') and self.name_input.text():
            try:
                vehicle_data = {
                    "name": self.name_input.text(),
                    "type": self.vehicle_type_combo.currentText(),
                    "subtype": self.road_vehicle_type_combo.currentText() if self.vehicle_type_combo.currentText() == "Road" else "N/A",
                    "capacity": self.capacity_spin.value(),
                    "mass": self.mass_spin.value(),
                    "max_speed": self.max_speed_spin.value(),
                    "fuel_type": self.fuel_type_combo.currentText(),
                    "fuel_class": self.fuel_class_combo.currentText(),
                    "fuel_efficiency": self.fuel_efficiency_spin.value(),
                    "sound_class": self.sound_class_combo.currentText(),
                    "sound_emission": self.sound_emission_spin.value(),
                    "detection_multiplier": self.detection_multiplier_spin.value()
                }
            except (AttributeError, RuntimeError):
                # If any widgets are not initialized yet
                if hasattr(self, 'preview_data'):
                    self.preview_data.setText("Loading vehicle data...")
                return
        else:
            # Use current entity data if widgets not available
            vehicle_data = self.current_entity
            
        # Safety check - if vehicle_data is None, create an empty dict
        if vehicle_data is None:
            vehicle_data = {}
            
        # Format for display in the preview data text area
        if hasattr(self, 'preview_data'):
            try:
                formatted_data = json.dumps(vehicle_data, indent=2)
                self.preview_data.setText(formatted_data)
            except (TypeError, AttributeError):
                self.preview_data.setText(str(vehicle_data))
                
        # Show formatted data in info preview if exists
        if hasattr(self, 'info_preview') and vehicle_data:
            try:
                vehicle_info = {
                    "Name": vehicle_data.get('name', 'Unknown'),
                    "Type": vehicle_data.get('type', 'Unknown'),
                    "Subtype": vehicle_data.get('subtype', 'N/A'),
                    "Capacity": f"{vehicle_data.get('capacity', 0)} units",
                    "Mass": f"{vehicle_data.get('mass', 0)} tons",
                    "Max Speed": f"{vehicle_data.get('max_speed', 0)} km/h",
                    "Fuel Type": vehicle_data.get('fuel_type', 'Unknown'),
                    "Fuel Efficiency": f"{vehicle_data.get('fuel_efficiency', 0)} km/L",
                    "Sound Level": f"{vehicle_data.get('sound_emission', 0)} dB"
                }
                
                # Format the info text
                info_text = "\n".join([f"{key}: {value}" for key, value in vehicle_info.items()])
                self.info_preview.setText(info_text)
            except Exception as e:
                print(f"Error formatting info preview: {e}")
                if hasattr(self, 'info_preview'):
                    self.info_preview.setText("Error formatting vehicle information")
            
        # Update image preview if available
        if hasattr(self, 'preview_image'):
            # In a real app, we would load the vehicle image
            # For now, just show a placeholder
            if hasattr(self.preview_image, 'setText'):
                self.preview_image.setText(f"Preview for: {vehicle_data.get('name', 'Unknown')}")
                
    def exportEntityData(self):
        """Export vehicle data to a file"""
        # Create safe filename from vehicle name
        safe_name = self.name_input.text().lower().replace(" ", "_")
        default_filename = f"{safe_name}_vehicle.fb"
        
        # Get export directory
        export_dir = os.path.expanduser("~")
        if hasattr(self, 'last_export_dir') and self.last_export_dir:
            export_dir = self.last_export_dir
            
        # Show file dialog
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Export Vehicle FlatBuffer",
            os.path.join(export_dir, default_filename),
            "FlatBuffer Files (*.fb);;JSON Files (*.json);;All Files (*.*)"
        )
        
        if not file_path:
            return  # User cancelled
            
        try:
            # Extract current vehicle data
            vehicle_data = {
                "name": self.name_input.text(),
                "type": self.vehicle_type_combo.currentText(),
                "subtype": self.road_vehicle_type_combo.currentText() if self.vehicle_type_combo.currentText() == "Road" else "",
                "capacity": self.capacity_spin.value(),
                "mass": self.mass_spin.value(),
                "max_speed": self.max_speed_spin.value(),
                "fuel_type": self.fuel_type_combo.currentText(),
                "fuel_class": self.fuel_class_combo.currentText(),
                "fuel_efficiency": self.fuel_efficiency_spin.value(),
                "sound_class": self.sound_class_combo.currentText(),
                "sound_emission": self.sound_emission_spin.value(),
                "detection_multiplier": self.detection_multiplier_spin.value()
            }
            
            # Save as JSON 
            with open(file_path, 'w') as f:
                json.dump(vehicle_data, f, indent=2)
                
            self.showMessage("Export Successful", f"Vehicle exported to {file_path}", is_error=False)
                
        except Exception as e:
            self.showMessage("Export Error", f"Failed to export vehicle: {str(e)}", is_error=True)
        
    def createNew(self):
        """Create a new vehicle (overrides base class method)"""
        # Create new empty vehicle data
        vehicle_data = {
            "name": "New Vehicle",
            "type": VEHICLE_TYPES[0],
            "subtype": ROAD_VEHICLE_TYPES[0] if VEHICLE_TYPES[0] == "Road" else "",
            "capacity": 0,
            "mass": 1.0,
            "max_speed": 60,
            "fuel_type": fuel_types_for_category(FUEL_SOURCE_CATEGORY[0])[0],
            "fuel_class": FUEL_SOURCE_CATEGORY[0],
            "fuel_efficiency": 10.0,
            "sound_class": SOUND_CLASSES[0],
            "sound_emission": 60.0,
            "detection_multiplier": 1.0
        }
        
        # Set as current entity
        self.current_entity = vehicle_data
        
        # Clear form fields
        self.name_input.setText("New Vehicle")
        self.vehicle_type_combo.setCurrentIndex(0)
        self.road_vehicle_type_combo.setCurrentIndex(0)
        self.segment_combo.setCurrentIndex(0)
        self.capacity_spin.setValue(0)
        self.mass_spin.setValue(1.0)
        self.max_speed_spin.setValue(60)
        
        # Clear advanced settings (category first so fuel type list matches)
        self.fuel_class_combo.setCurrentIndex(0)
        self.limitFuelTypesByCategory(self.fuel_class_combo.currentText())
        self.fuel_type_combo.setCurrentIndex(0)
        self.fuel_efficiency_spin.setValue(10.0)
        self.sound_class_combo.setCurrentIndex(0)
        self.sound_emission_spin.setValue(60.0)
        self.detection_multiplier_spin.setValue(1.0)
        
        # Update button states using the base class method
        super().updateButtonStates(selected=False, editing=True)
    
    # Keep old method name for compatibility
    def createNewVehicle(self):
        """Alias for createNew for backward compatibility"""
        self.createNew()
    
    def editEntity(self):
        """Edit the selected vehicle (overrides base class method)"""
        if not self.current_entity:
            return
        
        # Enter editing mode
        self.is_editing = True
        
        # Update button states using base class method
        self.updateButtonStates()
    
    # Keep old method name for compatibility
    def editVehicle(self):
        """Alias for editEntity for backward compatibility"""
        self.editEntity()
    
    def copyEntity(self):
        """Copy the selected vehicle (overrides base class method)"""
        if not self.current_entity:
            return
        
        # Make a copy of the current vehicle data
        self.is_new_entity = True
        
        # Update name to indicate it's a copy
        self.name_input.setText(f"{self.name_input.text()} (Copy)")
        
        # Update button states using base class method
        self.updateButtonStates()
    
    # Keep old method name for compatibility
    def copyVehicle(self):
        """Alias for copyEntity for backward compatibility"""
        self.copyEntity()
    
    def cancelEdit(self):
        """Cancel editing and reset UI (overrides base class method)"""
        # Reset editing state
        self.is_editing = False
        self.is_new_entity = False
        
        # Reselect the current item to restore values
        selected_items = self.list_widget.selectedItems()
        if selected_items:
            self.onVehicleSelected()
        else:
            # Clear form if no item is selected
            self.name_input.clear()
            self.current_entity = None
        
        # Update button states
        self.updateButtonStates()
    
    def createBasicTab(self):
        """Create tab for basic properties"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create form layout for vehicle properties
        form_layout = QFormLayout()
        
        # Basic properties widgets
        self.name_input = LineEdit()
        self.vehicle_type_combo = ComboBox()
        self.road_vehicle_type_combo = ComboBox()
        self.segment_combo = ComboBox()
        self.capacity_spin = SpinBox()
        self.mass_spin = DoubleSpinBox()
        self.max_speed_spin = SpinBox()
        
        # Set up controls
        self.name_input.setPlaceholderText("Vehicle name")
        
        # Vehicle type combo
        self.vehicle_type_combo.addItems(VEHICLE_TYPES)
        self.vehicle_type_combo.currentTextChanged.connect(self.onVehicleTypeChanged)
        
        # Road vehicle type combo (initially hidden)
        self.road_vehicle_type_combo.addItems(ROAD_VEHICLE_TYPES)
        self.road_vehicle_type_combo.setVisible(False)
        
        # Segment combo
        self.segment_combo.addItems(SEGMENT_MEMBERSHIP)
        
        # Capacity spin
        self.capacity_spin.setRange(0, 10000)
        self.capacity_spin.setValue(0)
        
        # Mass spin
        self.mass_spin.setRange(0.1, 10000.0)
        self.mass_spin.setValue(1.0)
        self.mass_spin.setSingleStep(0.1)
        
        # Max speed spin
        self.max_speed_spin.setRange(0, 500)
        self.max_speed_spin.setValue(60)
        
        # Add widgets to form layout
        form_layout.addRow("Name:", self.name_input)
        form_layout.addRow("Vehicle Type:", self.vehicle_type_combo)
        form_layout.addRow("Road Vehicle Type:", self.road_vehicle_type_combo)
        form_layout.addRow("Segment:", self.segment_combo)
        form_layout.addRow("Capacity:", self.capacity_spin)
        form_layout.addRow("Mass (tons):", self.mass_spin)
        form_layout.addRow("Max Speed (km/h):", self.max_speed_spin)
        
        # Create a group box for the form
        group_box = QGroupBox("Vehicle Properties")
        group_box.setLayout(form_layout)
        
        # Add group box to tab layout
        layout.addWidget(group_box)
        layout.addStretch()
        
        # Connect signals
        self.name_input.textChanged.connect(self.updateConfig)
        self.vehicle_type_combo.currentTextChanged.connect(self.updateConfig)
        self.vehicle_type_combo.currentTextChanged.connect(self.onVehicleTypeChanged)
        self.road_vehicle_type_combo.currentTextChanged.connect(self.updateConfig)
        self.segment_combo.currentTextChanged.connect(self.updateConfig)
        self.capacity_spin.valueChanged.connect(self.updateConfig)
        self.mass_spin.valueChanged.connect(self.updateConfig)
        self.max_speed_spin.valueChanged.connect(self.updateConfig)
        
        return tab
    
    def createAdvancedTab(self):
        """Create tab for advanced properties"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create advanced widgets
        # Fuel type options are filled from ``fuel_types_for_category`` (see ``limitFuelTypesByCategory``).
        self.fuel_type_combo = ComboBox()
        self.fuel_class_combo = ComboBox()
        self.fuel_efficiency_spin = DoubleSpinBox()
        self.sound_class_combo = ComboBox()
        self.sound_emission_spin = DoubleSpinBox()
        self.detection_multiplier_spin = DoubleSpinBox()
        
        # Fuel type and source category (serialized field remains fuel_class)
        self.fuel_class_combo.addItems(FUEL_SOURCE_CATEGORY)
        
        # Fuel efficiency spin
        self.fuel_efficiency_spin.setRange(0.1, 1000.0)
        self.fuel_efficiency_spin.setValue(10.0)
        self.fuel_efficiency_spin.setSingleStep(0.1)
        
        # Sound class combo
        self.sound_class_combo.addItems(SOUND_CLASSES)
        
        # Sound emission spin
        self.sound_emission_spin.setRange(0.0, 150.0)
        self.sound_emission_spin.setValue(60.0)  # Default 60 dB
        self.sound_emission_spin.setSingleStep(1.0)
        
        # Detection multiplier
        self.detection_multiplier_spin.setRange(0.1, 10.0)
        self.detection_multiplier_spin.setValue(1.0)
        self.detection_multiplier_spin.setSingleStep(0.1)
        
        # Fuel properties section
        fuel_group = QGroupBox("Fuel Properties")
        fuel_layout = QFormLayout()
        fuel_layout.addRow("Fuel Type:", self.fuel_type_combo)
        fuel_layout.addRow("Fuel source category:", self.fuel_class_combo)
        fuel_layout.addRow("Fuel Efficiency (km/L):", self.fuel_efficiency_spin)
        fuel_group.setLayout(fuel_layout)
        
        # Sound properties section
        sound_group = QGroupBox("Sound Properties")
        sound_layout = QFormLayout()
        sound_layout.addRow("Sound Class:", self.sound_class_combo)
        sound_layout.addRow("Sound Emission (dB):", self.sound_emission_spin)
        sound_layout.addRow("Detection Multiplier:", self.detection_multiplier_spin)
        sound_group.setLayout(sound_layout)
        
        # Add groups to tab layout
        layout.addWidget(fuel_group)
        layout.addWidget(sound_group)
        layout.addStretch()
        
        # Connect signals
        self.fuel_type_combo.currentTextChanged.connect(self.updateConfig)
        self.fuel_class_combo.currentTextChanged.connect(self.updateConfig)
        self.fuel_efficiency_spin.valueChanged.connect(self.updateConfig)
        self.sound_class_combo.currentTextChanged.connect(self.updateConfig)
        self.sound_emission_spin.valueChanged.connect(self.updateConfig)
        self.detection_multiplier_spin.valueChanged.connect(self.updateConfig)
        
        # Connect fuel type changes to sound emission adjustment
        self.fuel_type_combo.currentTextChanged.connect(self.adjustSoundBasedOnFuel)
        
        # Connect fuel source category changes to limit fuel types
        self.fuel_class_combo.currentTextChanged.connect(self.limitFuelTypesByCategory)

        self.limitFuelTypesByCategory(self.fuel_class_combo.currentText())

        return tab

    def limitFuelTypesByCategory(self, fuel_source_category):
        """Limit available fuel types based on selected fuel source category.

        Serialized JSON field is still ``fuel_class``; values are ``FUEL_SOURCE_CATEGORY``.
        """
        current_selection = normalize_fuel_type(self.fuel_type_combo.currentText())

        self.fuel_type_combo.blockSignals(True)
        self.fuel_type_combo.clear()
        allowed = fuel_types_for_category(fuel_source_category)
        self.fuel_type_combo.addItems(allowed)

        index = self.fuel_type_combo.findText(current_selection)
        if index >= 0:
            self.fuel_type_combo.setCurrentIndex(index)
        elif self.fuel_type_combo.count() > 0:
            self.fuel_type_combo.setCurrentIndex(0)

        self.fuel_type_combo.blockSignals(False)

        self.adjustSoundBasedOnFuel(self.fuel_type_combo.currentText())
        
    def onVehicleTypeChanged(self, vehicle_type):
        """Handle vehicle type changes"""
        # Show/hide road vehicle type combo based on selected vehicle type
        if vehicle_type == "Road":
            self.road_vehicle_type_combo.setVisible(True)
        else:
            self.road_vehicle_type_combo.setVisible(False)
    
    def adjustSoundBasedOnFuel(self, fuel_type):
        """Automatically adjust sound emission based on fuel type
        
        Args:
            fuel_type: Selected fuel type
        """
        base_level = 70.0
        ft = normalize_fuel_type(fuel_type)

        if ft in ("Battery_Electric", "Grid_Electric_Trolley"):
            self.sound_emission_spin.setValue(base_level - 30.0)
            self.detection_multiplier_spin.setValue(0.5)
            self.sound_class_combo.setCurrentText("Electrical")
        elif ft == "Hydrogen_FuelCell":
            self.sound_emission_spin.setValue(base_level - 20.0)
            self.detection_multiplier_spin.setValue(0.7)
            self.sound_class_combo.setCurrentText("Electrical")
        elif ft in ("Diesel", "Gasoline", "Biodiesel", "Bio_Ethanol", "LPG", "Crude_Transport_Only"):
            self.sound_emission_spin.setValue(base_level + 10.0)
            self.detection_multiplier_spin.setValue(1.3)
            self.sound_class_combo.setCurrentText("Engine")
        elif ft in ("Jet_Kerosene", "Avgas"):
            self.sound_emission_spin.setValue(base_level + 30.0)
            self.detection_multiplier_spin.setValue(2.0)
            self.sound_class_combo.setCurrentText("Engine")
        elif ft == "Marine_Bunker":
            self.sound_emission_spin.setValue(base_level + 20.0)
            self.detection_multiplier_spin.setValue(1.6)
            self.sound_class_combo.setCurrentText("Engine")
        elif ft in ("Coal", "Steam_External"):
            self.sound_emission_spin.setValue(base_level + 15.0)
            self.detection_multiplier_spin.setValue(1.4)
            self.sound_class_combo.setCurrentText("Machinery")
        elif ft == "Nuclear_Heat":
            self.sound_emission_spin.setValue(base_level - 5.0)
            self.detection_multiplier_spin.setValue(1.0)
            self.sound_class_combo.setCurrentText("Machinery")
        elif ft in ("Hybrid_Battery_Diesel", "Hybrid_Battery_Gasoline"):
            self.sound_emission_spin.setValue(base_level + 5.0)
            self.detection_multiplier_spin.setValue(1.1)
            self.sound_class_combo.setCurrentText("Engine")
        elif ft in ("Solid_Rocket", "Liquid_Rocket"):
            self.sound_emission_spin.setValue(base_level + 40.0)
            self.detection_multiplier_spin.setValue(2.5)
            self.sound_class_combo.setCurrentText("Explosive")
        else:
            self.sound_emission_spin.setValue(base_level)
            self.detection_multiplier_spin.setValue(1.0)
            self.sound_class_combo.setCurrentText("Engine")
    
    def updateConfig(self):
        """Update the asset configuration and emit signal"""
        config = {
            "is_vehicle": True,
            "is_building": False,
            "is_power": False,
            "asset_name": self.name_input.text(),
            "asset_type": "Vehicle",
            "vehicle_type": self.vehicle_type_combo.currentText(),
            "road_vehicle_type": self.road_vehicle_type_combo.currentText(),
            "segment": self.segment_combo.currentText(),
            "fuel_type": self.fuel_type_combo.currentText(),
            "fuel_class": self.fuel_class_combo.currentText(),
            "fuel_efficiency": self.fuel_efficiency_spin.value(),
            "capacity": self.capacity_spin.value(),
            "mass": self.mass_spin.value(),
            "max_speed": self.max_speed_spin.value(),
            "sound_class": self.sound_class_combo.currentText(),
            "sound_emission": self.sound_emission_spin.value(),
            "detection_multiplier": self.detection_multiplier_spin.value()
        }
        
        self.assetConfigChanged.emit(config)
    
    def validateEntity(self):
        """Validate that the vehicle has required fields before saving"""
        if not self.name_input.text():
            # Show error message if name is empty
            self.showMessage("Error", "Vehicle name is required", is_error=True)
            return False
            
        return True
        
    def performSave(self):
        """Perform the actual save operation"""
        # Update config one last time before saving
        self.updateConfig()
        
        # Create vehicle type ID (for real world, this would be a unique ID)
        vehicle_type_id = self.name_input.text().lower().replace(" ", "_")
        
        # Add save action flags
        config = {
            "_action": "save",
            "type_id": vehicle_type_id
        }
        
        if self.is_new_entity:
            config["_is_new"] = True
        
        # Emit special signal to trigger save
        self.assetConfigChanged.emit(config)
        
        # Extract current vehicle data
        vehicle_data = {
            "name": self.name_input.text(),
            "type": self.vehicle_type_combo.currentText(),
            "subtype": self.road_vehicle_type_combo.currentText() if self.vehicle_type_combo.currentText() == "Road" else "",
            "capacity": self.capacity_spin.value(),
            "mass": self.mass_spin.value(),
            "max_speed": self.max_speed_spin.value(),
            "fuel_type": self.fuel_type_combo.currentText(),
            "fuel_class": self.fuel_class_combo.currentText(),
            "fuel_efficiency": self.fuel_efficiency_spin.value(),
            "sound_class": self.sound_class_combo.currentText(),
            "sound_emission": self.sound_emission_spin.value(),
            "detection_multiplier": self.detection_multiplier_spin.value()
        }
        
        # Set as current entity
        self.current_entity = vehicle_data
        
        # Add or update the vehicle in the list
        selected_items = self.list_widget.selectedItems()
        
        if selected_items and not self.is_new_entity:
            # Update existing item
            item = selected_items[0]
            item.setText(f"{vehicle_data['name']} ({vehicle_data['subtype'] if vehicle_data['subtype'] else vehicle_data['type']})")
            item.setData(Qt.UserRole, vehicle_data)
        else:
            # Add new item
            item = QListWidgetItem(f"{vehicle_data['name']} ({vehicle_data['subtype'] if vehicle_data['subtype'] else vehicle_data['type']})")
            item.setData(Qt.UserRole, vehicle_data)
            self.list_widget.addItem(item)
            self.list_widget.setCurrentItem(item)
            
        # Update the preview
        self.updatePreview()
    
    # Keep old method name for compatibility
    def saveVehicle(self):
        """Alias for saveEntity for backward compatibility"""
        self.saveEntity()
        
    def exportFlatBuffer(self):
        """Export vehicle configuration as a FlatBuffer file (overrides base class method)"""
        if not self.current_entity:
            self.showMessage("Export Error", "No vehicle selected to export", is_error=True)
            return
            
        # Create safe filename from vehicle name
        safe_name = self.name_input.text().lower().replace(" ", "_")
        default_filename = f"{safe_name}_vehicle.fb"
        
        # Get export directory
        export_dir = os.path.expanduser("~")
        if hasattr(self, 'last_export_dir') and self.last_export_dir:
            export_dir = self.last_export_dir
            
        # Show file dialog
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Export Vehicle FlatBuffer",
            os.path.join(export_dir, default_filename),
            "FlatBuffer Files (*.fb);;All Files (*.*)"
        )
        
        if not file_path:
            return  # User cancelled
            
        try:
            # In a real app, this would use actual FlatBuffer serialization
            # For demo purposes, we'll just save the JSON representation
            vehicle_data = {
                "name": self.name_input.text(),
                "type": self.vehicle_type_combo.currentText(),
                "subtype": self.road_vehicle_type_combo.currentText() if self.vehicle_type_combo.currentText() == "Road" else "",
                "capacity": self.capacity_spin.value(),
                "mass": self.mass_spin.value(),
                "max_speed": self.max_speed_spin.value(),
                "fuel_type": self.fuel_type_combo.currentText(),
                "fuel_class": self.fuel_class_combo.currentText(),
                "fuel_efficiency": self.fuel_efficiency_spin.value(),
                "sound_class": self.sound_class_combo.currentText(),
                "sound_emission": self.sound_emission_spin.value(),
                "detection_multiplier": self.detection_multiplier_spin.value()
            }
            
            # Save as JSON for demonstration purposes
            with open(file_path, 'w') as f:
                json.dump(vehicle_data, f, indent=2)
                
            self.showMessage("Export Successful", f"Vehicle exported to {file_path}", is_error=False)
                
        except Exception as e:
            self.showMessage("Export Error", f"Failed to export vehicle: {str(e)}", is_error=True)