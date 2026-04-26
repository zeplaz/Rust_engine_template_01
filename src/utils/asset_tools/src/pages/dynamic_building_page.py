"""Dynamic building page that can adapt to different building types"""

import os
import json
import time
from PyQt5.QtCore import Qt, pyqtSignal, QDir
from PyQt5.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                            QLineEdit, QSpinBox, QDoubleSpinBox, QComboBox, 
                            QPushButton, QGroupBox, QFormLayout, QCheckBox,
                            QTabWidget, QScrollArea, QSplitter, QTextEdit, QMessageBox,
                            QFileDialog, QTabBar)
from PyQt5.QtGui import QFont, QPixmap

# Import base page and widgets
from .base_page import BasePage, EntityBasePage
from .base_page import (LineEdit, SpinBox, ComboBox, PushButton,
                      DoubleSpinBox, CheckBox, RadioButton, FluentIcon)

# Import additional widgets from base_page
from .base_page import (ScrollArea, ImageLabel, expandWidgetAnimation,
                      BodyLabel, StrongBodyLabel, CaptionLabel)
    
from ..config.asset_config import SEGMENT_MEMBERSHIP, RESOURCE_TYPES, SOUND_CLASSES
from ..config.building_templates import (BuildingTemplate, get_template, 
                                       BUILDING_TEMPLATES, BUILDING_CATEGORIES)
from ..integration.flatbuffers import FlatBuffersIntegration

class DynamicBuildingPage(EntityBasePage):
    """Dynamic building configuration page that adapts to the building type"""
    
    def updateButtons(self, selected=None, editing=None):
        """Alias for updateButtonStates for backward compatibility"""
        if selected is None:
            selected = self.current_template is not None
        if editing is None:
            editing = self.is_editing
        self.updateButtonStates()
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.flatbuffers = FlatBuffersIntegration()
        self.current_template = None
        self.current_entity = None  # Required by EntityBasePage
        self.entity_type = "Building"  # Set entity type for button labels
        self.produces_widgets = []
        self.consumes_widgets = []
        self.production_rate_widgets = {}
        self.consumption_rate_widgets = {}
        self.is_new_building = False
        self.is_editing = False
        self.initUI()
        
    def initUI(self):
        """Initialize the UI"""
        # Create template selection panel
        template_panel = self.createTemplatePanel()
        
        # Create building details panel with tabs
        details_panel = self.createDetailsPanel()
        
        # Dictionary of tabs for details panel
        tabs_dict = {
            "Basic": self.createBasicPropertiesTab,
            "Resources": self.createResourcesTab,
            "Advanced": self.createAdvancedTab,
            "Preview": self.createPreviewTab
        }
        
        # Use the base class method to initialize the UI
        self.initEntityUI(
            tabs_dict=tabs_dict, 
            entity_list_panel=template_panel, 
            details_panel=details_panel
        )
    
    def createTemplatePanel(self):
        """Create panel for template selection"""
        group_box = QGroupBox("Building Templates")
        layout = QVBoxLayout()
        
        # Add search box
        search_box = LineEdit()
        search_box.setPlaceholderText("Search templates...")
        search_box.setClearButtonEnabled(True)
        
        layout.addWidget(search_box)
        
        # Create tabs for categories
        # Always use QTabWidget directly for compatibility
        category_tabs = QTabWidget()
        
        # Store reference to category tabs
        self.category_tabs = category_tabs
        
        # Add "All" category
        all_templates_widget = self.createTemplateList(BUILDING_TEMPLATES.keys())
        category_tabs.addTab(all_templates_widget, "All")
        
        # Add other categories
        for category, templates in BUILDING_CATEGORIES.items():
            category_widget = self.createTemplateList(templates)
            category_tabs.addTab(category_widget, category)
        
        layout.addWidget(category_tabs)
        
        # Connect search box
        search_box.textChanged.connect(self.filterTemplates)
        
        group_box.setLayout(layout)
        return group_box
    
    def createTemplateList(self, template_ids):
        """Create a list widget for templates
        
        Args:
            template_ids: List of template IDs to include
        """
        widget = QWidget()
        layout = QVBoxLayout(widget)
        layout.setSpacing(5)
        layout.setContentsMargins(2, 2, 2, 2)
        
        # Add sort controls
        sort_widget = QWidget()
        sort_layout = QHBoxLayout(sort_widget)
        sort_layout.setContentsMargins(0, 0, 0, 0)
        sort_layout.setSpacing(2)
        
        # Add sort label
        sort_label = QLabel("Sort by:")
        sort_label.setStyleSheet("color: #b0b0b0; font-size: 11px;")
        
        # Add sort dropdown
        sort_combo = QComboBox()
        sort_combo.addItems(["Name", "Category", "Power", "Last Modified"])
        sort_combo.setStyleSheet("""
            QComboBox {
                background-color: #333333;
                color: #f0f0f0;
                border: 1px solid #555555;
                padding: 2px;
                font-size: 11px;
                max-height: 20px;
            }
        """)
        
        # Add sort buttons
        up_button = QPushButton("▲")
        up_button.setFixedSize(20, 20)
        up_button.setStyleSheet("""
            QPushButton {
                background-color: #333333;
                color: #f0f0f0;
                border: 1px solid #555555;
                font-size: 10px;
            }
        """)
        
        down_button = QPushButton("▼")
        down_button.setFixedSize(20, 20)
        down_button.setStyleSheet("""
            QPushButton {
                background-color: #333333;
                color: #f0f0f0;
                border: 1px solid #555555;
                font-size: 10px;
            }
        """)
        
        # Add to sort layout
        sort_layout.addWidget(sort_label)
        sort_layout.addWidget(sort_combo)
        sort_layout.addWidget(up_button)
        sort_layout.addWidget(down_button)
        
        # Add sort widget to main layout
        layout.addWidget(sort_widget)
        
        # Create a scroll area for templates
        scroll_area = ScrollArea()
        scroll_content = QWidget()
        scroll_layout = QVBoxLayout(scroll_content)
        scroll_layout.setContentsMargins(0, 0, 0, 0)
        scroll_layout.setSpacing(4)
        
        # Add templates
        for template_id in template_ids:
            template = get_template(template_id)
            if template:
                template_widget = self.createTemplateItem(template)
                scroll_layout.addWidget(template_widget)
        
        # Add stretch to push templates to top
        scroll_layout.addStretch()
        
        # Set up scroll area
        scroll_content.setLayout(scroll_layout)
        scroll_area.setWidget(scroll_content)
        scroll_area.setWidgetResizable(True)
        
        layout.addWidget(scroll_area)
        return widget
    
    def createTemplateItem(self, template):
        """Create a clickable template item with collapsible details
        
        Args:
            template: BuildingTemplate object
        """
        # Create widget
        widget = QWidget()
        widget.setCursor(Qt.PointingHandCursor)
        widget.setProperty("template_id", template.type_id)
        widget.setMinimumHeight(30)  # Initially collapsed height
        
        # Create layout
        layout = QVBoxLayout(widget)
        layout.setContentsMargins(5, 3, 5, 3)
        layout.setSpacing(2)
        
        # Create header layout with icon, title and expand button
        header_layout = QHBoxLayout()
        header_layout.setContentsMargins(2, 2, 2, 2)
        header_layout.setSpacing(5)
        
        # Create icon based on category
        icon_label = QLabel()
        icon_label.setFixedSize(16, 16)
        
        # Set icon based on category
        if hasattr(template, 'category'):
            if "power" in template.category.lower():
                icon_label.setStyleSheet("background-color: #88cc88; border-radius: 8px;")
            elif "production" in template.category.lower():
                icon_label.setStyleSheet("background-color: #cc8888; border-radius: 8px;")
            elif "transport" in template.category.lower():
                icon_label.setStyleSheet("background-color: #8888cc; border-radius: 8px;")
            else:
                icon_label.setStyleSheet("background-color: #cccc88; border-radius: 8px;")
        
        # Add icon to header
        header_layout.addWidget(icon_label)
        
        # Add title
        title = StrongBodyLabel(template.name)
        
        header_layout.addWidget(title, 1)  # 1 = stretch
        
        # Add expand/collapse button
        expand_button = PushButton("▼")
        expand_button.setFixedSize(16, 16)
        expand_button.setObjectName("expandButton")
        expand_button.setStyleSheet("""
            QPushButton {
                background-color: transparent;
                border: none;
                color: #5dca31;
                font-weight: bold;
            }
            QPushButton:hover {
                color: #7dea51;
            }
        """)
        
        header_layout.addWidget(expand_button)
        layout.addLayout(header_layout)
        
        # Create container for expandable content
        content_widget = QWidget()
        content_layout = QVBoxLayout(content_widget)
        content_layout.setContentsMargins(4, 0, 4, 4)
        content_layout.setSpacing(4)
        
        # Add description
        desc = BodyLabel(template.description)
        desc.setWordWrap(True)
        
        content_layout.addWidget(desc)
        
        # Add resources info
        resource_text = ""
        if template.produces:
            resource_text += f"Produces: {', '.join(template.produces)}\n"
        if template.consumes:
            resource_text += f"Consumes: {', '.join(template.consumes)}"
        
        resources = CaptionLabel(resource_text)
        resources.setStyleSheet("color: #cccccc;")
        
        content_layout.addWidget(resources)
        
        # Add version and export status
        status_widget = QWidget()
        status_layout = QHBoxLayout(status_widget)
        status_layout.setContentsMargins(0, 2, 0, 0)
        status_layout.setSpacing(10)
        
        # Version info (fixed for now, should come from template metadata)
        version_label = QLabel("Version: 1.0")
        version_label.setStyleSheet("font-size: 10px; color: #8dc9c8;")
        
        # Export status (fixed for now, should come from template metadata)
        export_label = QLabel("Not Exported")
        export_label.setStyleSheet("font-size: 10px; color: #ff9966;")
        
        status_layout.addWidget(version_label)
        status_layout.addStretch(1)
        status_layout.addWidget(export_label)
        
        content_layout.addWidget(status_widget)
        
        # Add container to main layout
        layout.addWidget(content_widget)
        
        # Initially hide the content
        content_widget.setVisible(False)
        
        # Store expanded state as a property
        widget.setProperty("expanded", False)
        
        # Define toggle function
        def toggle_expand():
            expanded = widget.property("expanded")
            if expanded:
                # Collapse
                widget.setFixedHeight(50)
                content_widget.setVisible(False)
                expand_button.setText("▼")
            else:
                # Expand
                widget.setFixedHeight(140)
                content_widget.setVisible(True)
                expand_button.setText("▲")
            widget.setProperty("expanded", not expanded)
        
        # Connect expand button
        expand_button.clicked.connect(toggle_expand)
        
        # Make widget selectable with new green styling
        widget.setStyleSheet("""
            QWidget {
                background-color: rgba(93, 202, 49, 0.05);
                border: 1px solid #5dca31;
                border-radius: 5px;
            }
            QWidget:hover {
                background-color: rgba(93, 202, 49, 0.1);
                border: 1px solid #7dea51;
            }
            QLabel {
                background-color: transparent;
                border: none;
                color: #CCCCCC;
            }
        """)
        
        # Connect click event (but exclude the expand button from triggering selection)
        def on_click(event):
            # Check if the click was on the expand button
            child = widget.childAt(event.pos())
            if child and child.objectName() == "expandButton":
                # Let the button handle its own click
                return
            # Otherwise select the template
            self.selectTemplate(template.type_id)
            
        widget.mousePressEvent = on_click
        
        # Store template info for version tracking and export status
        widget.version = "1.0"
        widget.export_status = "Not Exported"
        widget.last_modified = time.time()
        
        # Add double-click to edit
        widget.mouseDoubleClickEvent = lambda event: self.editEntity()
        
        return widget
    
    def createDetailsPanel(self):
        """Create panel for building details"""
        # Create tab widget for details
        # Always use QTabWidget directly for compatibility
        tabs = QTabWidget()
        
        # Configure tab behavior for better usability
        tabs.setTabsClosable(False)
        tabs.setMovable(False)
        tabs.setDocumentMode(True)  # Make tabs look cleaner
        
        # Store tabs reference for later use
        self.detail_tabs = tabs
        
        # Subclass the tab bar to capture clicks directly
        tabBar = tabs.tabBar()
        
        # Override the tab click event to force tab switching
        def mouseReleaseEvent(self, event):
            index = self.tabAt(event.pos())
            if index >= 0:
                tabs.setCurrentIndex(index)
            QTabBar.mouseReleaseEvent(self, event)
            
        # Apply the override to the tab bar
        tabBar.__class__.mouseReleaseEvent = mouseReleaseEvent
        
        # Create basic properties tab
        basic_tab = self.createBasicPropertiesTab()
        tabs.addTab(basic_tab, "Basic")
        
        # Create resources tab
        resources_tab = self.createResourcesTab()
        tabs.addTab(resources_tab, "Resources")
        
        # Create advanced tab
        advanced_tab = self.createAdvancedTab()
        tabs.addTab(advanced_tab, "Advanced")
        
        # Create preview tab
        preview_tab = self.createPreviewTab()
        tabs.addTab(preview_tab, "Preview")
        
        # Connect currentChanged signal to handle tab changes - use direct connection for reliability
        tabs.currentChanged.connect(self.onTabChanged, Qt.DirectConnection)
        
        return tabs
    
    def createBasicPropertiesTab(self):
        """Create tab for basic properties"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create form for basic properties
        form_layout = QFormLayout()
        
        # Name input
        self.name_input = LineEdit()
        self.description_input = QTextEdit()
        self.segment_combo = ComboBox()
        self.size_x_spin = SpinBox()
        self.size_y_spin = SpinBox()
        self.height_spin = SpinBox()
        self.construction_cost_spin = SpinBox()
        self.construction_time_spin = SpinBox()
        
        # Set up controls
        self.name_input.setPlaceholderText("Building name")
        self.description_input.setPlaceholderText("Building description")
        
        # Segment combo
        self.segment_combo.addItems(SEGMENT_MEMBERSHIP)
        
        # Size spins
        self.size_x_spin.setRange(1, 100)
        self.size_x_spin.setValue(1)
        
        self.size_y_spin.setRange(1, 100)
        self.size_y_spin.setValue(1)
        
        self.height_spin.setRange(1, 50)
        self.height_spin.setValue(1)
        
        # Construction properties
        self.construction_cost_spin.setRange(10, 100000)
        self.construction_cost_spin.setValue(100)
        self.construction_cost_spin.setSingleStep(10)
        
        self.construction_time_spin.setRange(1, 500)
        self.construction_time_spin.setValue(10)
        
        # Add widgets to form layout
        form_layout.addRow("Name:", self.name_input)
        form_layout.addRow("Description:", self.description_input)
        form_layout.addRow("Segment:", self.segment_combo)
        form_layout.addRow("Size X:", self.size_x_spin)
        form_layout.addRow("Size Y:", self.size_y_spin)
        form_layout.addRow("Height:", self.height_spin)
        form_layout.addRow("Construction Cost:", self.construction_cost_spin)
        form_layout.addRow("Construction Time:", self.construction_time_spin)
        
        # Create a group box for the form
        group_box = QGroupBox("Building Properties")
        group_box.setLayout(form_layout)
        
        # Add group box to tab layout
        layout.addWidget(group_box)
        layout.addStretch()
        
        # Connect signals
        self.name_input.textChanged.connect(self.updateConfig)
        self.description_input.textChanged.connect(self.updateConfig)
        self.segment_combo.currentTextChanged.connect(self.updateConfig)
        self.size_x_spin.valueChanged.connect(self.updateConfig)
        self.size_y_spin.valueChanged.connect(self.updateConfig)
        self.height_spin.valueChanged.connect(self.updateConfig)
        self.construction_cost_spin.valueChanged.connect(self.updateConfig)
        self.construction_time_spin.valueChanged.connect(self.updateConfig)
        
        return tab
    
    def createResourcesTab(self):
        """Create tab for resource production and consumption"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create a splitter for production/consumption
        resources_splitter = QSplitter(Qt.Vertical)
        
        # ===== PRODUCTION SECTION =====
        production_widget = QWidget()
        production_layout = QVBoxLayout(production_widget)
        production_layout.setContentsMargins(5, 5, 5, 5)
        
        production_group = QGroupBox("Production Resources")
        production_inner_layout = QVBoxLayout()
        
        # Production controls section
        production_controls = QWidget()
        production_controls_layout = QHBoxLayout(production_controls)
        production_controls_layout.setContentsMargins(0, 0, 0, 0)
        
        # Resource selection
        self.produces_combo = ComboBox()
        add_production_button = PushButton("Add")
        add_production_button.setIcon(FluentIcon.ADD)
        
        # Setup resource combo and button
        self.produces_combo.addItems(RESOURCE_TYPES)
        self.produces_combo.setMinimumWidth(150)
        
        # Info label
        production_info = QLabel("Select resources produced by this building")
        production_info.setWordWrap(True)
        
        # Add controls
        production_controls_layout.addWidget(QLabel("Resource:"))
        production_controls_layout.addWidget(self.produces_combo, 1)
        production_controls_layout.addWidget(add_production_button)
        
        # Resource list header
        headers_widget = QWidget()
        headers_layout = QHBoxLayout(headers_widget)
        headers_layout.setContentsMargins(0, 0, 0, 0)
        
        resource_header = QLabel("Resource")
        resource_header.setStyleSheet("font-weight: bold;")
        rate_header = QLabel("Production Rate")
        rate_header.setStyleSheet("font-weight: bold;")
        
        headers_layout.addWidget(resource_header, 2)
        headers_layout.addWidget(rate_header, 2)
        headers_layout.addWidget(QLabel(""), 1)  # Spacer for remove button column
        
        # Container for production items
        production_items_container = QWidget()
        self.production_list = QVBoxLayout(production_items_container)
        self.production_list.setContentsMargins(0, 0, 0, 0)
        self.production_list.setSpacing(5)
        
        # Add everything to production group
        production_inner_layout.addWidget(production_info)
        production_inner_layout.addWidget(production_controls)
        production_inner_layout.addWidget(headers_widget)
        production_inner_layout.addWidget(production_items_container)
        
        # Set group layout and add to parent
        production_group.setLayout(production_inner_layout)
        production_layout.addWidget(production_group)
        
        # ===== CONSUMPTION SECTION =====
        consumption_widget = QWidget()
        consumption_layout = QVBoxLayout(consumption_widget)
        consumption_layout.setContentsMargins(5, 5, 5, 5)
        
        consumption_group = QGroupBox("Consumption Resources")
        consumption_inner_layout = QVBoxLayout()
        
        # Consumption controls section
        consumption_controls = QWidget()
        consumption_controls_layout = QHBoxLayout(consumption_controls)
        consumption_controls_layout.setContentsMargins(0, 0, 0, 0)
        
        # Resource selection
        self.consumes_combo = ComboBox()
        add_consumption_button = PushButton("Add")
        add_consumption_button.setIcon(FluentIcon.ADD)
        
        # Setup resource combo and button
        self.consumes_combo.addItems(RESOURCE_TYPES)
        self.consumes_combo.setMinimumWidth(150)
        
        # Info label
        consumption_info = QLabel("Select resources consumed by this building")
        consumption_info.setWordWrap(True)
        
        # Add controls
        consumption_controls_layout.addWidget(QLabel("Resource:"))
        consumption_controls_layout.addWidget(self.consumes_combo, 1)
        consumption_controls_layout.addWidget(add_consumption_button)
        
        # Resource list header
        cons_headers_widget = QWidget()
        cons_headers_layout = QHBoxLayout(cons_headers_widget)
        cons_headers_layout.setContentsMargins(0, 0, 0, 0)
        
        cons_resource_header = QLabel("Resource")
        cons_resource_header.setStyleSheet("font-weight: bold;")
        cons_rate_header = QLabel("Consumption Rate")
        cons_rate_header.setStyleSheet("font-weight: bold;")
        
        cons_headers_layout.addWidget(cons_resource_header, 2)
        cons_headers_layout.addWidget(cons_rate_header, 2)
        cons_headers_layout.addWidget(QLabel(""), 1)  # Spacer for remove button column
        
        # Container for consumption items
        consumption_items_container = QWidget()
        self.consumption_list = QVBoxLayout(consumption_items_container)
        self.consumption_list.setContentsMargins(0, 0, 0, 0)
        self.consumption_list.setSpacing(5)
        
        # Add everything to consumption group
        consumption_inner_layout.addWidget(consumption_info)
        consumption_inner_layout.addWidget(consumption_controls)
        consumption_inner_layout.addWidget(cons_headers_widget)
        consumption_inner_layout.addWidget(consumption_items_container)
        
        # Set group layout and add to parent
        consumption_group.setLayout(consumption_inner_layout)
        consumption_layout.addWidget(consumption_group)
        
        # Add widgets to splitter
        resources_splitter.addWidget(production_widget)
        resources_splitter.addWidget(consumption_widget)
        
        # ===== POWER SECTION =====
        power_group = QGroupBox("Power Requirements")
        power_layout = QFormLayout()
        
        # Power description label
        power_info = QLabel("Specify the building's power consumption (positive) or generation (negative)")
        power_info.setWordWrap(True)
        power_layout.addRow(power_info)
        
        # Power widgets with HorizontalLayout for better organization
        power_input_layout = QHBoxLayout()
        
        self.power_consumption_spin = SpinBox()
        
        # Configure power spinner
        self.power_consumption_spin.setRange(-10000, 10000)
        self.power_consumption_spin.setValue(0)
        self.power_consumption_spin.setSingleStep(10)
        self.power_consumption_spin.setMinimumWidth(120)
        
        # Label for power consumption
        power_unit_label = QLabel("kW")
        
        # Add widgets to horizontal layout
        power_input_layout.addWidget(QLabel("Power:"))
        power_input_layout.addWidget(self.power_consumption_spin)
        power_input_layout.addWidget(power_unit_label)
        power_input_layout.addStretch()
        
        power_layout.addRow(power_input_layout)
        
        # Add note about negative values for generators
        power_note = QLabel("Note: Use negative values for power generation buildings")
        power_note.setStyleSheet("color: #5dca31; font-style: italic;")
        power_layout.addRow(power_note)
        
        power_group.setLayout(power_layout)
        
        # Add all sections to main layout
        layout.addWidget(resources_splitter, 3)  # Higher stretch for resource sections
        layout.addWidget(power_group, 1)  # Lower stretch for power section
        
        # Connect signals
        add_production_button.clicked.connect(self.addProduction)
        add_consumption_button.clicked.connect(self.addConsumption)
        self.power_consumption_spin.valueChanged.connect(self.updateConfig)
        
        return tab
    
    def createAdvancedTab(self):
        """Create tab for advanced properties"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create form for advanced properties
        form_layout = QFormLayout()
        
        # Advanced widgets
        self.maintenance_cost_spin = DoubleSpinBox()
        self.workers_required_spin = SpinBox()
        self.tech_level_spin = SpinBox()
        self.sound_class_combo = ComboBox()
        self.sound_emission_spin = DoubleSpinBox()
        self.detection_multiplier_spin = DoubleSpinBox()
        
        # Set up controls
        self.maintenance_cost_spin.setRange(0.0, 1000.0)
        self.maintenance_cost_spin.setValue(1.0)
        self.maintenance_cost_spin.setSingleStep(0.1)
        
        self.workers_required_spin.setRange(0, 1000)
        self.workers_required_spin.setValue(0)
        
        self.tech_level_spin.setRange(1, 10)
        self.tech_level_spin.setValue(1)
        
        # Sound controls
        self.sound_class_combo.addItems(SOUND_CLASSES)
        self.sound_class_combo.setCurrentText("Machinery")
        
        self.sound_emission_spin.setRange(0.0, 150.0)
        self.sound_emission_spin.setValue(50.0)  # Default 50 dB for buildings
        self.sound_emission_spin.setSingleStep(1.0)
        
        self.detection_multiplier_spin.setRange(0.1, 10.0)
        self.detection_multiplier_spin.setValue(1.0)
        self.detection_multiplier_spin.setSingleStep(0.1)
        
        # Add widgets to form layout
        form_layout.addRow("Maintenance Cost:", self.maintenance_cost_spin)
        form_layout.addRow("Workers Required:", self.workers_required_spin)
        form_layout.addRow("Tech Level:", self.tech_level_spin)
        
        # Sound properties section
        sound_group = QGroupBox("Sound Properties")
        sound_layout = QFormLayout()
        sound_layout.addRow("Sound Class:", self.sound_class_combo)
        sound_layout.addRow("Sound Emission (dB):", self.sound_emission_spin)
        sound_layout.addRow("Detection Multiplier:", self.detection_multiplier_spin)
        sound_group.setLayout(sound_layout)
        form_layout.addRow(sound_group)
        
        # Create a group box for the form
        group_box = QGroupBox("Advanced Properties")
        group_box.setLayout(form_layout)
        
        # Add group box to tab layout
        layout.addWidget(group_box)
        
        # Special features section
        features_group = QGroupBox("Special Features")
        features_layout = QVBoxLayout()
        
        self.features_list = QVBoxLayout()
        features_layout.addLayout(self.features_list)
        
        # Define available features
        AVAILABLE_FEATURES = [
            "Power Generation", "Power Storage", "Power Consumption",
            "Noise Reduction", "Environmental Impact", "Worker Housing",
            "Research Capability", "Production Bonus", "Maintenance Reduction",
            "Security Enhancement", "Transport Hub", "Resource Storage"
        ]
        
        # Add feature dropdown
        feature_selection_layout = QHBoxLayout()
        feature_selection_label = QLabel("Select feature:")
        self.feature_combo = ComboBox()
        self.feature_combo.addItems(AVAILABLE_FEATURES)
        
        add_feature_button = PushButton("Add Feature")
        add_feature_button.setIcon(FluentIcon.ADD)
        
        feature_selection_layout.addWidget(feature_selection_label)
        feature_selection_layout.addWidget(self.feature_combo, 1)
        feature_selection_layout.addWidget(add_feature_button)
        
        features_layout.addLayout(feature_selection_layout)
        
        # Add Power Features section
        power_features_group = QGroupBox("Power Integration")
        power_layout = QFormLayout()
        
        self.has_power_checkbox = CheckBox("Enable Power Integration")
        self.power_consumption_spin = SpinBox()
        self.power_generation_spin = SpinBox()
        self.power_storage_spin = SpinBox()
        
        # Configure power widgets
        self.power_consumption_spin.setRange(-10000, 10000)
        self.power_consumption_spin.setValue(0)
        self.power_consumption_spin.setSingleStep(10)
        self.power_consumption_spin.setEnabled(False)
        
        self.power_generation_spin.setRange(0, 10000)
        self.power_generation_spin.setValue(0)
        self.power_generation_spin.setSingleStep(10)
        self.power_generation_spin.setEnabled(False)
        
        self.power_storage_spin.setRange(0, 100000)
        self.power_storage_spin.setValue(0)
        self.power_storage_spin.setSingleStep(100)
        self.power_storage_spin.setEnabled(False)
        
        # Add widgets to power layout
        power_layout.addRow("", self.has_power_checkbox)
        power_layout.addRow("Power Consumption (kW):", self.power_consumption_spin)
        power_layout.addRow("Power Generation (kW):", self.power_generation_spin)
        power_layout.addRow("Power Storage (kWh):", self.power_storage_spin)
        
        # Connect power checkbox
        self.has_power_checkbox.stateChanged.connect(self.togglePowerControls)
        
        power_features_group.setLayout(power_layout)
        features_layout.addWidget(power_features_group)
        
        features_group.setLayout(features_layout)
        
        # Add features group to tab layout
        layout.addWidget(features_group)
        layout.addStretch()
        
        # Connect signals
        self.maintenance_cost_spin.valueChanged.connect(self.updateConfig)
        self.workers_required_spin.valueChanged.connect(self.updateConfig)
        self.tech_level_spin.valueChanged.connect(self.updateConfig)
        self.sound_class_combo.currentTextChanged.connect(self.updateConfig)
        self.sound_emission_spin.valueChanged.connect(self.updateConfig)
        self.detection_multiplier_spin.valueChanged.connect(self.updateConfig)
        self.power_consumption_spin.valueChanged.connect(self.updateConfig)
        self.power_generation_spin.valueChanged.connect(self.updateConfig)
        self.power_storage_spin.valueChanged.connect(self.updateConfig)
        add_feature_button.clicked.connect(self.addFeatureFromCombo)
        
        # Connect power consumption to sound adjustment
        self.power_consumption_spin.valueChanged.connect(self.adjustSoundBasedOnPower)
        
        return tab
    
    def createPreviewTab(self):
        """Create tab for preview"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create preview label
        self.preview_label = ImageLabel("No texture selected")
        self.preview_label.setFixedSize(400, 300)
        
        # Create FlatBuffers preview
        self.fb_preview = QTextEdit()
        self.fb_preview.setReadOnly(True)
        self.fb_preview.setPlaceholderText("FlatBuffers export preview will appear here")
        
        # Add widgets to layout
        preview_layout = QVBoxLayout()
        preview_layout.addWidget(QLabel("Texture Preview:"))
        preview_layout.addWidget(self.preview_label)
        
        fb_layout = QVBoxLayout()
        fb_layout.addWidget(QLabel("FlatBuffers Export Preview:"))
        fb_layout.addWidget(self.fb_preview)
        
        layout.addLayout(preview_layout)
        layout.addLayout(fb_layout)
        
        return tab
    
    def selectTemplate(self, template_id):
        """Handle template selection
        
        Args:
            template_id: ID of selected template
        """
        template = get_template(template_id)
        if not template:
            return
        
        # Update both current_template and current_entity for consistency
        self.current_template = template
        self.current_entity = template
        self.is_new_building = False
        self.is_editing = False
        
        # Update UI with template values
        self.name_input.setText(template.name)
        self.description_input.setText(template.description)
        self.size_x_spin.setValue(template.default_size[0])
        self.size_y_spin.setValue(template.default_size[1])
        self.construction_cost_spin.setValue(template.construction_cost)
        self.construction_time_spin.setValue(template.construction_time)
        self.maintenance_cost_spin.setValue(template.maintenance_cost)
        self.workers_required_spin.setValue(template.workers_required)
        self.tech_level_spin.setValue(template.tech_level)
        # Convert float to int for power consumption
        self.power_consumption_spin.setValue(int(template.power_consumption))
        
        # Clear existing production/consumption widgets
        self.clearResourceWidgets()
        
        # Add production resources
        for resource in template.produces:
            rate = template.production_rate.get(resource, 1.0)
            self.addProduction(resource, rate)
        
        # Add consumption resources
        for resource in template.consumes:
            rate = template.consumption_rate.get(resource, 1.0)
            self.addConsumption(resource, rate)
        
        # Add special features
        self.clearFeatures()
        for feature in template.special_features:
            self.addFeature(feature)
        
        # Update button states using base class method
        self.updateButtonStates()
        
        # Update preview
        self.updatePreview()
        
        # Switch to the details tab
        self.detail_tabs.setCurrentIndex(0)
    
    def clearResourceWidgets(self):
        """Clear all resource widgets"""
        # Clear production widgets
        for widget, _ in self.produces_widgets:
            widget.setParent(None)
            widget.deleteLater()
        self.produces_widgets = []
        self.production_rate_widgets = {}
        
        # Clear consumption widgets
        for widget, _ in self.consumes_widgets:
            widget.setParent(None)
            widget.deleteLater()
        self.consumes_widgets = []
        self.consumption_rate_widgets = {}
    
    def clearFeatures(self):
        """Clear all special feature widgets"""
        # Check if features_list exists and is initialized
        if not hasattr(self, 'features_list') or self.features_list is None:
            return
            
        # Find and clear feature widgets
        for i in reversed(range(self.features_list.count())):
            item = self.features_list.itemAt(i)
            if item and item.widget():
                widget = item.widget()
                self.features_list.removeWidget(widget)
                widget.setParent(None)
                widget.deleteLater()
    
    def addProduction(self, resource=None, rate=1.0):
        """Add a production resource
        
        Args:
            resource: Resource name, or None to use selected resource
            rate: Production rate
        """
        # Handle the case when this method is called directly from a button click
        if isinstance(resource, bool) or resource is None:
            resource = self.produces_combo.currentText()
        
        # Ensure resource is a string
        if not isinstance(resource, str):
            print(f"Error: resource must be a string, got {type(resource)}")
            return
            
        # Check if resource already exists
        for _, res in self.produces_widgets:
            if res == resource:
                return
        
        # Create a widget for this resource
        widget = QWidget()
        widget_layout = QHBoxLayout(widget)
        widget_layout.setContentsMargins(0, 0, 0, 0)
        
        # Add resource label
        resource_label = QLabel(resource)
        widget_layout.addWidget(resource_label)
        
        # Add rate spin box
        rate_spin = DoubleSpinBox()
        
        rate_spin.setRange(0.1, 1000.0)
        rate_spin.setValue(rate)
        rate_spin.setSingleStep(0.1)
        widget_layout.addWidget(rate_spin)
        
        # Add remove button
        remove_button = PushButton("Remove")
        remove_button.setIcon(FluentIcon.DELETE)
        
        widget_layout.addWidget(remove_button)
        
        # Connect signals
        remove_button.clicked.connect(lambda: self.removeProduction(widget, resource))
        rate_spin.valueChanged.connect(self.updateConfig)
        
        # Add to list
        self.production_list.addWidget(widget)
        self.produces_widgets.append((widget, resource))
        self.production_rate_widgets[resource] = rate_spin
        
        # Update configuration
        self.updateConfig()
    
    def removeProduction(self, widget, resource):
        """Remove a production resource
        
        Args:
            widget: Widget to remove
            resource: Resource name
        """
        # Remove from UI
        self.production_list.removeWidget(widget)
        widget.setParent(None)
        widget.deleteLater()
        
        # Remove from list
        self.produces_widgets = [(w, r) for w, r in self.produces_widgets if r != resource]
        if resource in self.production_rate_widgets:
            del self.production_rate_widgets[resource]
        
        # Update configuration
        self.updateConfig()
    
    def addConsumption(self, resource=None, rate=1.0):
        """Add a consumption resource
        
        Args:
            resource: Resource name, or None to use selected resource
            rate: Consumption rate
        """
        # Handle the case when this method is called directly from a button click
        if isinstance(resource, bool) or resource is None:
            resource = self.consumes_combo.currentText()
            
        # Ensure resource is a string
        if not isinstance(resource, str):
            print(f"Error: resource must be a string, got {type(resource)}")
            return
        
        # Check if resource already exists
        for _, res in self.consumes_widgets:
            if res == resource:
                return
        
        # Create a widget for this resource
        widget = QWidget()
        widget_layout = QHBoxLayout(widget)
        widget_layout.setContentsMargins(0, 0, 0, 0)
        
        # Add resource label
        resource_label = QLabel(resource)
        widget_layout.addWidget(resource_label)
        
        # Add rate spin box
        rate_spin = DoubleSpinBox()
        
        rate_spin.setRange(0.1, 1000.0)
        rate_spin.setValue(rate)
        rate_spin.setSingleStep(0.1)
        widget_layout.addWidget(rate_spin)
        
        # Add remove button
        remove_button = PushButton("Remove")
        remove_button.setIcon(FluentIcon.DELETE)
        
        widget_layout.addWidget(remove_button)
        
        # Connect signals
        remove_button.clicked.connect(lambda: self.removeConsumption(widget, resource))
        rate_spin.valueChanged.connect(self.updateConfig)
        
        # Add to list
        self.consumption_list.addWidget(widget)
        self.consumes_widgets.append((widget, resource))
        self.consumption_rate_widgets[resource] = rate_spin
        
        # Update configuration
        self.updateConfig()
    
    def removeConsumption(self, widget, resource):
        """Remove a consumption resource
        
        Args:
            widget: Widget to remove
            resource: Resource name
        """
        # Remove from UI
        self.consumption_list.removeWidget(widget)
        widget.setParent(None)
        widget.deleteLater()
        
        # Remove from list
        self.consumes_widgets = [(w, r) for w, r in self.consumes_widgets if r != resource]
        if resource in self.consumption_rate_widgets:
            del self.consumption_rate_widgets[resource]
        
        # Update configuration
        self.updateConfig()
    
    def togglePowerControls(self, state):
        """Enable or disable power controls based on checkbox state
        
        Args:
            state: Checkbox state
        """
        enabled = state == Qt.Checked
        self.power_consumption_spin.setEnabled(enabled)
        self.power_generation_spin.setEnabled(enabled)
        self.power_storage_spin.setEnabled(enabled)
        
        # Update config
        self.updateConfig()
    
    def addFeatureFromCombo(self):
        """Add a feature from the dropdown combo box"""
        feature = self.feature_combo.currentText()
        if feature:
            self.addFeature(feature)
    
    def addFeature(self, feature=None):
        """Add a special feature
        
        Args:
            feature: Feature name, or None to use input text
        """
        # Check if we're being called from a clicked signal (in which case feature will be a bool)
        if isinstance(feature, bool):
            feature = None
            
        # Get feature from combo if not specified
        if feature is None:
            feature = self.feature_combo.currentText()
            if not feature:
                return
        
        # Make sure feature is a string
        if not isinstance(feature, str):
            self.showMessage("Error", "Invalid feature type", is_error=True)
            return
            
        # Check if feature already exists
        for i in range(self.features_list.count()):
            item = self.features_list.itemAt(i)
            if item and item.widget():
                widget_layout = item.widget().layout()
                if widget_layout and widget_layout.count() > 0:
                    label_widget = widget_layout.itemAt(0).widget()
                    if isinstance(label_widget, QLabel) and label_widget.text() == feature:
                        # Feature already exists
                        self.showMessage("Warning", f"Feature '{feature}' already added", is_error=True)
                        return
            
        # Create a widget for this feature
        widget = QWidget()
        widget.setStyleSheet("background-color: rgba(93, 202, 49, 0.1); border-radius: 4px; padding: 2px;")
        widget_layout = QHBoxLayout(widget)
        widget_layout.setContentsMargins(8, 4, 8, 4)
        
        # Add feature label
        feature_label = QLabel(feature)
        feature_label.setStyleSheet("background-color: transparent; font-weight: bold;")
        widget_layout.addWidget(feature_label)
        
        # Add remove button
        remove_button = PushButton("Remove")
        remove_button.setIcon(FluentIcon.DELETE)
        remove_button.setFixedWidth(100)
        
        widget_layout.addStretch(1)
        widget_layout.addWidget(remove_button)
        
        # Connect signals
        remove_button.clicked.connect(lambda: self.removeFeature(widget))
        
        # Add to list
        self.features_list.addWidget(widget)
        
        # Update configuration
        self.updateConfig()
    
    def removeFeature(self, widget):
        """Remove a special feature
        
        Args:
            widget: Widget to remove
        """
        # Remove from UI
        self.features_list.removeWidget(widget)
        widget.setParent(None)
        widget.deleteLater()
        
        # Update configuration
        self.updateConfig()
    
    def filterTemplates(self, text):
        """Filter templates by search text
        
        Args:
            text: Search text
        """
        # Get current tab index
        current_tab = self.category_tabs.currentIndex()
        
        # Get the tab's widget
        tab_widget = self.category_tabs.widget(current_tab)
        
        # Find the scroll area in the tab widget
        scroll_area = None
        for i in range(tab_widget.layout().count()):
            widget = tab_widget.layout().itemAt(i).widget()
            if isinstance(widget, ScrollArea) or isinstance(widget, QScrollArea):
                scroll_area = widget
                break
        
        if not scroll_area:
            return
            
        # Get the content widget from scroll area
        content_widget = scroll_area.widget()
        if not content_widget:
            return
            
        # Check each template widget for a match
        for i in range(content_widget.layout().count()):
            item = content_widget.layout().itemAt(i)
            if not item or not item.widget():
                continue
                
            template_widget = item.widget()
            
            # Skip if it's not a template widget
            if not template_widget.property("template_id"):
                continue
                
            # Find title label (first label in the header layout)
            title_text = ""
            header_layout = template_widget.layout().itemAt(0)
            if header_layout and hasattr(header_layout, "itemAt"):
                for j in range(header_layout.count()):
                    widget = header_layout.itemAt(j).widget()
                    if isinstance(widget, QLabel) or isinstance(widget, StrongBodyLabel):
                        title_text = widget.text()
                        break
            
            # Show/hide based on if text is in title
            if text.lower() in title_text.lower():
                template_widget.setVisible(True)
            else:
                template_widget.setVisible(False)
    
    def updateConfig(self):
        """Update the asset configuration and emit signal"""
        if not self.current_template:
            return
        
        # Collect special features
        special_features = []
        for i in range(self.features_list.count()):
            item = self.features_list.itemAt(i)
            if item and item.widget():
                layout = item.widget().layout()
                if layout and layout.count() > 0:
                    label_widget = layout.itemAt(0).widget()
                    if isinstance(label_widget, QLabel):
                        special_features.append(label_widget.text())
        
        # Build configuration
        # Get power integration settings
        has_power_integration = hasattr(self, 'has_power_checkbox') and self.has_power_checkbox.isChecked()
        
        config = {
            "is_building": True,
            "is_vehicle": False,
            "is_power": has_power_integration,
            "is_productive": len(self.produces_widgets) > 0,
            "asset_name": self.name_input.text(),
            "description": self.description_input.toPlainText(),
            "asset_type": "Building",
            "type_id": self.current_template.type_id,
            "segment": self.segment_combo.currentText(),
            "building_size_x": self.size_x_spin.value(),
            "building_size_y": self.size_y_spin.value(),
            "building_height": self.height_spin.value(),
            "construction_cost": self.construction_cost_spin.value(),
            "construction_time": self.construction_time_spin.value(),
            "maintenance_cost": self.maintenance_cost_spin.value(),
            "workers_required": self.workers_required_spin.value(),
            "tech_level": self.tech_level_spin.value(),
            "produces_resources": [r for _, r in self.produces_widgets],
            "consumes_resources": [r for _, r in self.consumes_widgets],
            "production_rate": {
                r: w.value() for r, w in self.production_rate_widgets.items()
            },
            "consumption_rate": {
                r: w.value() for r, w in self.consumption_rate_widgets.items()
            },
            "special_features": special_features,
            # Sound properties
            "sound_class": self.sound_class_combo.currentText() if hasattr(self, 'sound_class_combo') else "Machinery",
            "sound_emission": self.sound_emission_spin.value() if hasattr(self, 'sound_emission_spin') else 50.0,
            "detection_multiplier": self.detection_multiplier_spin.value() if hasattr(self, 'detection_multiplier_spin') else 1.0
        }
        
        # Add power integration settings if enabled
        if has_power_integration:
            config.update({
                "power_consumption": self.power_consumption_spin.value() if hasattr(self, 'power_consumption_spin') else 0,
                "power_generation": self.power_generation_spin.value() if hasattr(self, 'power_generation_spin') else 0,
                "power_storage": self.power_storage_spin.value() if hasattr(self, 'power_storage_spin') else 0,
            })
        else:
            # Default power values
            config.update({
                "power_consumption": 0,
                "power_generation": 0,
                "power_storage": 0,
            })
            
        # Add special flags based on features
        if "Power Generation" in special_features:
            config["is_power_generator"] = True
            
        if "Power Storage" in special_features:
            config["has_power_storage"] = True
        
        # Update FlatBuffers preview
        self.updatePreview(config)
        
        # Emit signal
        self.assetConfigChanged.emit(config)
    
    def onTabChanged(self, index):
        """Handle tab changes
        
        Args:
            index: New tab index
        """
        # Force the tab to change - this is a critical fix for tab navigation
        self.detail_tabs.setCurrentIndex(index)
        
        # Update the content for the selected tab
        if index == 0:  # Basic tab
            # Refresh basic settings if needed
            pass
        elif index == 1:  # Resources tab
            # Refresh resource widgets if needed
            self.updateResourceTab()
        elif index == 2:  # Advanced tab
            # Update advanced settings if needed
            self.updateAdvancedTab()
        elif index == 3:  # Preview tab
            # Update preview if needed
            self.updatePreview()
            
    def refreshTemplatesList(self):
        """Refresh the templates list after saving"""
        current_tab = self.category_tabs.currentIndex()
        current_template_id = self.current_template.type_id if self.current_template and hasattr(self.current_template, 'type_id') else None
        
        # Clear existing tabs
        while self.category_tabs.count() > 0:
            self.category_tabs.removeTab(0)
        
        # Force reload of templates to ensure we get the latest data
        from ..config.building_templates import reload_templates, BUILDING_TEMPLATES, BUILDING_CATEGORIES
        reload_templates()
        
        # DEBUG - print a message showing how many templates are loaded
        print(f"Refreshing templates list. Found {len(BUILDING_TEMPLATES)} templates.")
        
        # Rebuilt "All" category tab
        all_templates_widget = self.createTemplateList(BUILDING_TEMPLATES.keys())
        self.category_tabs.addTab(all_templates_widget, "All")
        
        # Rebuild other category tabs
        for category, templates in BUILDING_CATEGORIES.items():
            category_widget = self.createTemplateList(templates)
            self.category_tabs.addTab(category_widget, category)
        
        # Restore the previously selected tab
        if current_tab < self.category_tabs.count():
            self.category_tabs.setCurrentIndex(current_tab)
        
        # If the current template exists, reselect it
        if current_template_id:
            try:
                self.selectTemplate(current_template_id)
                print(f"Reselected template: {current_template_id}")
            except Exception as e:
                print(f"Error reselecting template: {e}")
        
        # Show a confirming message about the refresh
        self.showMessage("Templates Updated", "Template list has been refreshed", is_error=False)
            
    def updatePreview(self, config=None):
        """Update the preview with current data (overrides base class method)"""
    
    def updateResourceTab(self):
        """Update the resources tab when it's selected"""
        # This ensures resource widgets are properly displayed
        # Re-layout the containers
        for i in range(self.production_list.count()):
            widget = self.production_list.itemAt(i).widget()
            if widget:
                widget.setVisible(True)
                
        for i in range(self.consumption_list.count()):
            widget = self.consumption_list.itemAt(i).widget()
            if widget:
                widget.setVisible(True)
    
    def adjustSoundBasedOnPower(self, power_value):
        """Automatically adjust sound emission based on power consumption/generation
        
        Args:
            power_value: Power consumption value (negative for generation)
        """
        # Skip during initialization
        if not hasattr(self, 'sound_emission_spin'):
            return
            
        # Base value
        base_level = 50.0
        
        # If it's a power generator (negative value)
        if power_value < 0:
            # Power plants are louder
            abs_power = abs(power_value)
            
            # Scale based on power generation amount
            if abs_power > 1000:
                self.sound_emission_spin.setValue(base_level + 30.0)
                self.detection_multiplier_spin.setValue(1.8)
                self.sound_class_combo.setCurrentText("Machinery")
            elif abs_power > 500:
                self.sound_emission_spin.setValue(base_level + 20.0)
                self.detection_multiplier_spin.setValue(1.5)
                self.sound_class_combo.setCurrentText("Machinery")
            elif abs_power > 100:
                self.sound_emission_spin.setValue(base_level + 10.0)
                self.detection_multiplier_spin.setValue(1.2)
            else:
                self.sound_emission_spin.setValue(base_level + 5.0)
                self.detection_multiplier_spin.setValue(1.0)
        else:
            # Regular buildings with power consumption
            if power_value > 500:
                self.sound_emission_spin.setValue(base_level + 15.0)
                self.detection_multiplier_spin.setValue(1.3)
            elif power_value > 100:
                self.sound_emission_spin.setValue(base_level + 5.0)
                self.detection_multiplier_spin.setValue(1.1)
            else:
                self.sound_emission_spin.setValue(base_level)
                self.detection_multiplier_spin.setValue(1.0)
                
    def updateAdvancedTab(self):
        """Update the advanced tab when it's selected"""
        # Refresh feature widgets
        for i in range(self.features_list.count()):
            widget = self.features_list.itemAt(i).widget()
            if widget:
                widget.setVisible(True)
            
    def updatePreview(self, config=None):
        """Update preview with current config
        
        Args:
            config: Configuration to preview, or None to use current config
        """
        if config is None:
            # Call updateConfig to get current config
            self.updateConfig()
            return
        
        # Update FlatBuffers preview
        if self.flatbuffers:
            try:
                fb_data = self.flatbuffers.create_building_data(config)
                self.fb_preview.setText(json.dumps(fb_data, indent=2))
            except Exception as e:
                self.fb_preview.setText(f"Error creating FlatBuffers preview: {str(e)}")
                
    def exportFlatBuffer(self):
        """Export building configuration as a FlatBuffer file"""
        if not self.current_template:
            self.showMessage("Export Error", "No building selected to export", is_error=True)
            return
            
        # Switch to preview tab to ensure config is up to date
        self.detail_tabs.setCurrentIndex(3)
        
        # Update the configuration to get the latest values
        self.updateConfig()
        
        # Create safe filename from building name
        safe_name = self.name_input.text().lower().replace(" ", "_")
        default_filename = f"{safe_name}_building.fb"
        
        # Get export directory
        export_dir = os.path.expanduser("~")
        if hasattr(self, 'last_export_dir') and self.last_export_dir:
            export_dir = self.last_export_dir
            
        # Show file dialog
        file_path, _ = QFileDialog.getSaveFileName(
            self,
            "Export FlatBuffer",
            os.path.join(export_dir, default_filename),
            "FlatBuffer Files (*.fb);;All Files (*.*)"
        )
        
        if not file_path:
            return  # User cancelled
            
        # Remember the directory for next time
        self.last_export_dir = os.path.dirname(file_path)
        
        try:
            # Get config data
            config = {
                "is_building": True,
                "is_vehicle": False,
                "is_power": self.power_consumption_spin.value() < 0,
                "is_productive": len(self.produces_widgets) > 0,
                "asset_name": self.name_input.text(),
                "description": self.description_input.toPlainText(),
                "asset_type": "Building",
                "type_id": self.current_template.type_id,
                "segment": self.segment_combo.currentText(),
                "building_size_x": self.size_x_spin.value(),
                "building_size_y": self.size_y_spin.value(),
                "building_height": self.height_spin.value(),
                "construction_cost": self.construction_cost_spin.value(),
                "construction_time": self.construction_time_spin.value(),
                "maintenance_cost": self.maintenance_cost_spin.value(),
                "workers_required": self.workers_required_spin.value(),
                "tech_level": self.tech_level_spin.value(),
                "power_consumption": self.power_consumption_spin.value(),
                "produces_resources": [r for _, r in self.produces_widgets],
                "consumes_resources": [r for _, r in self.consumes_widgets],
                "production_rate": {
                    r: w.value() for r, w in self.production_rate_widgets.items()
                },
                "consumption_rate": {
                    r: w.value() for r, w in self.consumption_rate_widgets.items()
                }
            }
            
            # Export config
            if self.flatbuffers:
                fb_data = self.flatbuffers.create_building_data(config)
                
                # For demonstration - in a real app, this would write binary FlatBuffer data
                # Here we'll just write the JSON for demonstration purposes
                with open(file_path, 'w') as f:
                    json.dump(fb_data, f, indent=2)
                    
                self.showMessage("Export Successful", f"Building exported to {file_path}", is_error=False)
            else:
                self.showMessage("Export Error", "FlatBuffers integration not available", is_error=True)
                
        except Exception as e:
            self.showMessage("Export Error", f"Failed to export building: {str(e)}", is_error=True)
    
    def createNew(self):
        """Create a new building from scratch (overrides base class method)"""
        # Create an empty template as a starting point
        empty_template = BuildingTemplate(
            type_id="new_building",
            name="New Building",
            description="A new custom building",
            default_size=(1, 1),
            construction_cost=100,
            construction_time=10,
            produces=[],
            consumes=[],
            special_features=[],
            maintenance_cost=1.0,
            workers_required=0,
            tech_level=1,
            power_consumption=0,
            production_rate={},
            consumption_rate={}
        )
        
        # Update both current_template and current_entity for consistency
        self.current_template = empty_template
        self.current_entity = empty_template
        self.is_new_building = True
        self.is_editing = True
        
        # Update UI with template values
        self.name_input.setText("New Building")
        self.description_input.setText("A new custom building")
        self.size_x_spin.setValue(1)
        self.size_y_spin.setValue(1)
        self.construction_cost_spin.setValue(100)
        self.construction_time_spin.setValue(10)
        self.maintenance_cost_spin.setValue(1.0)
        self.workers_required_spin.setValue(0)
        self.tech_level_spin.setValue(1)
        self.power_consumption_spin.setValue(0)
        
        # Clear existing production/consumption widgets
        self.clearResourceWidgets()
        self.clearFeatures()
        
        # Update button states
        self.updateButtonStates()
        
        # Switch to details panel
        self.detail_tabs.setCurrentIndex(0)
    
    # Keep old method name for compatibility with any existing connections
    def createNewBuilding(self):
        """Alias for createNew for backward compatibility"""
        self.createNew()
        
    def editEntity(self):
        """Edit the selected building (overrides base class method)"""
        if not self.current_template:
            return
            
        self.is_editing = True
        
        # Update button states
        self.updateButtonStates()
    
    # Keep old method name for compatibility with any existing connections
    def editBuilding(self):
        """Alias for editEntity for backward compatibility"""
        self.editEntity()
        
    def copyEntity(self):
        """Copy the current building as a template (overrides base class method)"""
        if not self.current_template:
            return
            
        # Make a copy of the current template
        self.is_new_building = True
        self.is_editing = True
        
        # Update name to indicate it's a copy
        self.name_input.setText(f"{self.name_input.text()} (Copy)")
        
        # Update button states
        self.updateButtonStates()
    
    # Keep old method name for compatibility with any existing connections
    def copyAsTemplate(self):
        """Alias for copyEntity for backward compatibility"""
        self.copyEntity()
        
    def cancelEdit(self):
        """Cancel editing and reset UI (overrides base class method)"""
        self.is_editing = False
        self.is_new_building = False
        
        # Reset UI if a template was selected
        if self.current_template and self.current_template.type_id in BUILDING_TEMPLATES:
            self.selectTemplate(self.current_template.type_id)
        else:
            # Clear UI if no template
            self.name_input.clear()
            self.description_input.clear()
            self.clearResourceWidgets()
            self.clearFeatures()
            self.current_template = None
            self.current_entity = None
        
        # Update button states
        self.updateButtonStates()
    
    def saveEntity(self):
        """Save the building configuration (overrides base class method)"""
        if not self.name_input.text():
            # Show error message if name is empty
            self.showMessage("Error", "Building name is required", is_error=True)
            return
        
        # Update config one last time before saving
        self.updateConfig()
        
        # Add version tracking
        import time
        current_time = time.time()
        
        # If the template has a version, increment it, otherwise start at 1.0
        current_version = getattr(self.current_template, "version", "1.0") if self.current_template else "1.0"
        try:
            major, minor = current_version.split(".")
            new_version = f"{major}.{int(minor) + 1}"
        except (ValueError, AttributeError):
            new_version = "1.0"
            
        # Add save action flag
        config = {"_action": "save"}
        if self.is_new_building:
            config["_is_new"] = True
            
        # Create/update type_id for new buildings
        if self.is_new_building:
            # Create safe type_id from name
            type_id = self.name_input.text().lower().replace(" ", "_")
            config["type_id"] = type_id
            
            # Update current template with new type_id
            if self.current_template:
                self.current_template.type_id = type_id
                self.current_entity = self.current_template  # Keep current_entity in sync
        
        # Add version information to config
        config["version"] = new_version
        config["last_modified"] = current_time
        
        # Update the template with the new version info
        if self.current_template:
            self.current_template.version = new_version
            self.current_template.last_modified = current_time
            self.current_template.is_exported = False
            
        # Update status displays with version information
        if hasattr(self, 'version_label'):
            self.version_label.setText(f"Version: {new_version}")
            
        if hasattr(self, 'export_status'):
            self.export_status.setText("Not exported")
            self.export_status.setStyleSheet("color: #ff9966; font-size: 10px;")
        
        # Emit special signal to trigger save
        self.assetConfigChanged.emit(config)
        
        # Reset editing state
        self.is_editing = False
        self.is_new_building = False
        
        # Refresh the templates list to show the updated version
        self.refreshTemplatesList()
        
        # Update button states using base class method
        super().updateButtonStates(selected=True, editing=False)
        
        # Switch to preview tab
        self.detail_tabs.setCurrentIndex(3)
        
        # Show success message using the BasePage method
        self.showMessage("Success", f"Building configuration saved (version {new_version})", is_error=False)
    
    # Keep old method name for compatibility with any existing connections
    def saveBuilding(self):
        """Alias for saveEntity for backward compatibility"""
        self.saveEntity()