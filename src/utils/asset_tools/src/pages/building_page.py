from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                            QLineEdit, QSpinBox, QDoubleSpinBox, QComboBox, 
                            QPushButton, QGroupBox, QFormLayout, QCheckBox)

# Import Fluent widgets from base_page
from .base_page import (LineEdit, SpinBox, ComboBox, PushButton,
                       DoubleSpinBox, CheckBox, RadioButton, FluentIcon,
                       InfoBar, InfoBarPosition)
    
from ..config.asset_config import SEGMENT_MEMBERSHIP, RESOURCE_TYPES

class BuildingPage(QWidget):
    """Building configuration page"""
    
    # Signal to notify when configuration changes
    assetConfigChanged = pyqtSignal(dict)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.initUI()
        
    def initUI(self):
        """Initialize the UI"""
        # Main layout
        main_layout = QVBoxLayout(self)
        
        # Create form layout for building properties
        form_layout = QFormLayout()
        
        # Name input
                    self.name_input = LineEdit()
            self.segment_combo = ComboBox()
            self.size_x_spin = SpinBox()
            self.size_y_spin = SpinBox()
            self.height_spin = SpinBox()
            self.construction_cost_spin = SpinBox()
            self.productive_check = CheckBox("Productive")
            self.save_button = PushButton("Save Building")
            self.save_button.setIcon(FluentIcon.SAVE)
            self.segment_combo = QComboBox()
            self.size_x_spin = QSpinBox()
            self.size_y_spin = QSpinBox()
            self.height_spin = QSpinBox()
            self.construction_cost_spin = QSpinBox()
            self.productive_check = QCheckBox("Productive")
            self.save_button = QPushButton("Save Building")
        
        # Set up controls
        self.name_input.setPlaceholderText("Building name")
        
        # Segment combo
        self.segment_combo.addItems(SEGMENT_MEMBERSHIP)
        
        # Size spins
        self.size_x_spin.setRange(1, 100)
        self.size_x_spin.setValue(1)
        
        self.size_y_spin.setRange(1, 100)
        self.size_y_spin.setValue(1)
        
        self.height_spin.setRange(1, 50)
        self.height_spin.setValue(1)
        
        # Construction cost
        self.construction_cost_spin.setRange(10, 100000)
        self.construction_cost_spin.setValue(100)
        self.construction_cost_spin.setSingleStep(10)
        
        # Add widgets to form layout
        form_layout.addRow("Name:", self.name_input)
        form_layout.addRow("Segment:", self.segment_combo)
        form_layout.addRow("Size X:", self.size_x_spin)
        form_layout.addRow("Size Y:", self.size_y_spin)
        form_layout.addRow("Height:", self.height_spin)
        form_layout.addRow("Construction Cost:", self.construction_cost_spin)
        form_layout.addRow("", self.productive_check)
        
        # Create a group box for the form
        group_box = QGroupBox("Building Properties")
        group_box.setLayout(form_layout)
        
        # Add resource production section
        resource_group = self.createResourceSection()
        
        # Add groups to main layout
        main_layout.addWidget(group_box)
        main_layout.addWidget(resource_group)
        
        # Add save button
        main_layout.addWidget(self.save_button)
        main_layout.addStretch()
        
        # Connect signals
        self.save_button.clicked.connect(self.saveBuilding)
        self.name_input.textChanged.connect(self.updateConfig)
        self.segment_combo.currentTextChanged.connect(self.updateConfig)
        self.size_x_spin.valueChanged.connect(self.updateConfig)
        self.size_y_spin.valueChanged.connect(self.updateConfig)
        self.height_spin.valueChanged.connect(self.updateConfig)
        self.construction_cost_spin.valueChanged.connect(self.updateConfig)
        self.productive_check.stateChanged.connect(self.updateConfig)
    
    def createResourceSection(self):
        """Create resource production/consumption section"""
        group_box = QGroupBox("Resource Production")
        layout = QVBoxLayout()
        
        # Production combobox
        combo_layout = QHBoxLayout()
        combo_layout.addWidget(QLabel("Produces:"))
        
                    self.produces_combo = ComboBox()
            add_button = PushButton("Add")
            add_button.setIcon(FluentIcon.ADD)
            add_button = QPushButton("Add")
        
        self.produces_combo.addItems(RESOURCE_TYPES)
        combo_layout.addWidget(self.produces_combo)
        combo_layout.addWidget(add_button)
        
        # Add selected resources list
        self.produces_list = QVBoxLayout()
        self.produces_widgets = []
        
        # Add to layouts
        layout.addLayout(combo_layout)
        layout.addLayout(self.produces_list)
        group_box.setLayout(layout)
        
        # Connect signals
        add_button.clicked.connect(self.addProducedResource)
        
        return group_box
    
    def addProducedResource(self):
        """Add a produced resource to the list"""
        resource = self.produces_combo.currentText()
        
        # Create a widget for this resource
        widget = QWidget()
        widget_layout = QHBoxLayout(widget)
        widget_layout.setContentsMargins(0, 0, 0, 0)
        
        # Add resource label
        widget_layout.addWidget(QLabel(resource))
        
        # Add remove button
                    remove_button = PushButton("Remove")
            remove_button.setIcon(FluentIcon.DELETE)
        
        widget_layout.addWidget(remove_button)
        widget_layout.addStretch()
        
        # Connect button to remove this resource
        remove_button.clicked.connect(lambda: self.removeProducedResource(widget, resource))
        
        # Add to list
        self.produces_list.addWidget(widget)
        self.produces_widgets.append((widget, resource))
        
        # Update configuration
        self.updateConfig()
    
    def removeProducedResource(self, widget, resource):
        """Remove a produced resource from the list"""
        # Remove from UI
        self.produces_list.removeWidget(widget)
        widget.setParent(None)
        widget.deleteLater()
        
        # Remove from list
        self.produces_widgets = [(w, r) for w, r in self.produces_widgets if r != resource]
        
        # Update configuration
        self.updateConfig()
    
    def updateConfig(self):
        """Update the asset configuration and emit signal"""
        config = {
            "is_building": True,
            "is_vehicle": False,
            "is_power": False,
            "is_productive": self.productive_check.isChecked(),
            "asset_name": self.name_input.text(),
            "asset_type": "Building",
            "segment": self.segment_combo.currentText(),
            "building_size_x": self.size_x_spin.value(),
            "building_size_y": self.size_y_spin.value(),
            "building_height": self.height_spin.value(),
            "construction_cost": self.construction_cost_spin.value(),
            "produces_resources": [r for _, r in self.produces_widgets]
        }
        
        self.assetConfigChanged.emit(config)
    
    def saveBuilding(self):
        """Signal to save the building configuration"""
        if not self.name_input.text():
            # Show error message if name is empty
                            InfoBar.error(
                    title="Error",
                    content="Building name is required",
                    orient=Qt.Horizontal,
                    isClosable=True,
                    position=InfoBarPosition.TOP,
                    duration=2000,
                    parent=self
                )
                QMessageBox.critical(self, "Error", "Building name is required")
            return
        
        # Update config one last time before saving
        self.updateConfig()
        
        # Emit special signal to trigger save
        self.assetConfigChanged.emit({"_action": "save"})