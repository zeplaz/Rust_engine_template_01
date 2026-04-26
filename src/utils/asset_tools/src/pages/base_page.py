import os
import json
import time
from PyQt5.QtCore import Qt, pyqtSignal, QProcess
from PyQt5.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                          QLineEdit, QSpinBox, QDoubleSpinBox, QComboBox, 
                          QPushButton, QGroupBox, QFormLayout, QTabWidget, QCheckBox,
                          QScrollArea, QFileDialog, QMessageBox, QSplitter, QTreeView,
                          QProgressBar, QTextEdit)
from PyQt5.QtGui import QPixmap, QIcon, QFont, QColor

# Import Fluent widgets directly - assume they are always available
from qfluentwidgets import (LineEdit, SpinBox, ComboBox, PushButton,
                           DoubleSpinBox, CheckBox, RadioButton,
                           FluentIcon, InfoBar, InfoBarPosition,
                           ScrollArea, ImageLabel, TreeView,
                           BodyLabel, StrongBodyLabel, CaptionLabel)

# Define our own animation function since it's not in the current qfluentwidgets version
def expandWidgetAnimation(widget, duration=200):
    """Simple replacement for expandWidgetAnimation if not available"""
    widget.show()
    # In actual qfluentwidgets this would animate the expansion

class BasePage(QWidget):
    """Base class for all asset editor pages"""
    
    # Signal to notify when configuration changes
    assetConfigChanged = pyqtSignal(dict)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.parent = parent
        
    def createTabbedUI(self, tabs_dict):
        """Create a tabbed UI with the given tabs
        
        Args:
            tabs_dict: Dictionary mapping tab names to tab creation methods
        
        Returns:
            QTabWidget: The created tab widget
        """
        # Create main layout
        main_layout = QVBoxLayout(self)
        
        # Create tabs widget
        tabs = QTabWidget()
        
        # Add tabs
        for tab_name, create_method in tabs_dict.items():
            tab = create_method()
            tabs.addTab(tab, tab_name)
        
        # Add tabs to layout
        main_layout.addWidget(tabs)
        
        return tabs
    
    def createPropertyGroup(self, title, form_layout):
        """Create a property group with the given title and form layout
        
        Args:
            title: Title for the group box
            form_layout: QFormLayout to use for the group
            
        Returns:
            QGroupBox: The created group box
        """
        group_box = QGroupBox(title)
        group_box.setLayout(form_layout)
        return group_box
    
    def showMessage(self, title, content, is_error=False):
        """Show a message to the user
        
        Args:
            title: Message title
            content: Message content
            is_error: Whether this is an error message
        """
        try:
            if is_error:
                InfoBar.error(
                    title=title,
                    content=content,
                    orient=Qt.Horizontal,
                    isClosable=True,
                    position=InfoBarPosition.TOP,
                    duration=2000,
                    parent=self
                )
            else:
                InfoBar.success(
                    title=title,
                    content=content,
                    orient=Qt.Horizontal,
                    isClosable=True,
                    position=InfoBarPosition.TOP,
                    duration=2000,
                    parent=self
                )
        except ImportError:
            from PyQt5.QtWidgets import QMessageBox
            if is_error:
                QMessageBox.critical(self, title, content)
            else:
                QMessageBox.information(self, title, content)


class EntityBasePage(BasePage):
    """Base class for entity-based pages (vehicles, buildings, etc.)
    
    This adds common functionality for entity editing, including:
    - Basic properties tab
    - Advanced properties tab
    - Preview functionality
    - Entity list panel
    - Editing, saving, and copying functionality
    - FlatBuffer export
    """
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.is_new_entity = False
        self.is_editing = False
        self.current_entity = None
        self.entity_type = "Entity"  # Override in subclasses ("Building", "Vehicle", etc)
        self.preview_image = None    # For preview tab
        self.entity_list = None      # List of entities (for subclasses)
        
    def initEntityUI(self, tabs_dict, entity_list_panel=None, details_panel=None):
        """Initialize UI with tabs and action buttons
        
        Args:
            tabs_dict: Dictionary mapping tab names to create methods
            entity_list_panel: Optional panel for entity list
            details_panel: Optional panel for entity details
        """
        # Create main layout
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(2, 2, 2, 2)
        main_layout.setSpacing(2)
        
        # Create master vertical splitter to allow collapsing all sections
        master_splitter = QSplitter(Qt.Vertical)
        
        # Create content area (contains panels or tabs)
        content_widget = QWidget()
        content_layout = QVBoxLayout(content_widget)
        content_layout.setContentsMargins(0, 0, 0, 0)
        content_layout.setSpacing(0)
        
        # Create splitter if both panels are provided
        if entity_list_panel and details_panel:
            # Store entity list panel reference
            self.entity_list_panel = entity_list_panel
            
            # Create horizontal splitter for panels
            panel_splitter = QSplitter(Qt.Horizontal)
            
            # Add toggle button to entity_list_panel to make it collapsible
            if hasattr(entity_list_panel, 'layout'):
                collapse_button = PushButton("◀")
                collapse_button.setFixedSize(20, 20)
                collapse_button.setStyleSheet("""
                    QPushButton {
                        border: none;
                        background-color: transparent;
                        color: #aaaaaa;
                        font-weight: bold;
                    }
                    QPushButton:hover {
                        color: #ffffff;
                    }
                """)
                
                # Add toggle button to the top-right of the panel
                entity_list_panel.layout().insertWidget(0, collapse_button, 0, Qt.AlignRight)
                
                # Connect toggle button
                self.panel_collapsed = False
                collapse_button.clicked.connect(lambda: self.togglePanelCollapse(panel_splitter))
            
            # Add panels to splitter
            panel_splitter.addWidget(entity_list_panel)
            panel_splitter.addWidget(details_panel)
            panel_splitter.setSizes([200, 800])
            
            # Add splitter to content layout
            content_layout.addWidget(panel_splitter)
            
            # Store reference to panel splitter
            self.panel_splitter = panel_splitter
            
        elif details_panel:
            # Only details panel provided
            content_layout.addWidget(details_panel)
        
        # Create tabs widget if not using a details panel
        if not details_panel:
            # Add preview tab to tabs dictionary if not present
            if "Preview" not in tabs_dict:
                tabs_dict["Preview"] = self.createPreviewTab
                
            # Create tabbed UI
            self.tabs = self.createTabbedUI(tabs_dict)
            content_layout.addWidget(self.tabs)

        # Add content widget to master splitter
        master_splitter.addWidget(content_widget)

        # Add compact button bar
        button_bar = self.createCompactButtonBar()
        
        # Add status bar with version info
        status_bar = self.createStatusBar()
        
        # Create bottom container for buttons and status
        bottom_container = QWidget()
        bottom_layout = QVBoxLayout(bottom_container)
        bottom_layout.setContentsMargins(0, 0, 0, 0)
        bottom_layout.setSpacing(0)
        bottom_layout.addWidget(button_bar)
        bottom_layout.addWidget(status_bar)
        
        # Add bottom container to master splitter
        master_splitter.addWidget(bottom_container)
        
        # Set sizes to make content area dominant
        master_splitter.setSizes([900, 100])
        
        # Add master splitter to main layout
        main_layout.addWidget(master_splitter)
        
        # Store reference to master splitter
        self.master_splitter = master_splitter
    
    def createCompactButtonBar(self):
        """Create a compact button bar with combined icons and text"""
        button_container = QWidget()
        button_container.setFixedHeight(40)
        button_layout = QHBoxLayout(button_container)
        button_layout.setContentsMargins(10, 5, 10, 5)
        button_layout.setSpacing(20)
        
        # Define icon size
        icon_size = 24
        
        # Create compact button style
        button_style = """
            QPushButton {
                padding: 4px 12px;
                border-radius: 4px;
                font-weight: bold;
                color: #f0f0f0;
                min-width: 90px;
                font-size: 11px;
            }
            QPushButton:hover {
                background-color: rgba(255, 255, 255, 0.1);
            }
            QPushButton:disabled {
                color: #666666;
                background-color: #333333;
                border: 1px solid #444444;
            }
        """
        
        # Create action buttons with only icons
        self.new_button = PushButton(f"New")
        self.new_button.setIcon(FluentIcon.ADD)
        self.new_button.setToolTip(f"New {self.entity_type}")
        
        self.clone_button = PushButton("Clone")
        self.clone_button.setIcon(FluentIcon.COPY)
        self.clone_button.setToolTip(f"Clone {self.entity_type}")
        
        self.save_button = PushButton("Save")
        self.save_button.setIcon(FluentIcon.SAVE)
        self.save_button.setToolTip("Save Changes")
        
        self.export_button = PushButton("Export")
        self.export_button.setIcon(FluentIcon.SAVE)
        self.export_button.setToolTip("Export to FlatBuffer")
            
        # Apply button styles with colored backgrounds
        self.new_button.setStyleSheet(button_style + """
            background-color: #2a6e2a;
            border: 1px solid #5dca31;
        """)
        
        self.clone_button.setStyleSheet(button_style + """
            background-color: #1a3a5a;
            border: 1px solid #2a6ad2;
        """)
        
        self.save_button.setStyleSheet(button_style + """
            background-color: #3a3a1a;
            border: 1px solid #b2a832;
        """)
        
        self.export_button.setStyleSheet(button_style + """
            background-color: #3a1a3a;
            border: 1px solid #b232a8;
        """)
        
        # Add buttons to layout directly
        button_layout.addStretch(1)
        button_layout.addWidget(self.new_button)
        button_layout.addWidget(self.clone_button)
        button_layout.addWidget(self.save_button)
        button_layout.addWidget(self.export_button)
        button_layout.addStretch(1)
        
        # Add styled container
        button_container.setStyleSheet("""
            QWidget {
                background-color: rgba(40, 40, 40, 0.95);
                border-top: 1px solid #3a3a3a;
            }
        """)
        
        # Connect button signals to handlers
        self.new_button.clicked.connect(self.createNew)
        self.clone_button.clicked.connect(self.cloneEntity)
        self.save_button.clicked.connect(self.saveEntity)
        self.export_button.clicked.connect(self.exportFlatBuffer)
        
        # Set initial button states
        self.clone_button.setEnabled(False)
        self.save_button.setEnabled(False)
        self.export_button.setEnabled(False)
        
        return button_container
    
    def createStatusBar(self):
        """Create a status bar with version and export information"""
        status_bar = QWidget()
        status_bar.setFixedHeight(20)
        status_layout = QHBoxLayout(status_bar)
        status_layout.setContentsMargins(5, 1, 5, 1)
        status_layout.setSpacing(10)
        
        # Version info
        self.version_label = QLabel("Version: New")
        self.version_label.setStyleSheet("color: #8dc9c8; font-size: 10px;")
        
        # Export status
        self.export_status = QLabel("Not exported")
        self.export_status.setStyleSheet("color: #ff9966; font-size: 10px;")
        
        # Add spacers and labels
        status_layout.addWidget(self.version_label)
        status_layout.addStretch(1)
        status_layout.addWidget(self.export_status)
        
        # Style the status bar
        status_bar.setStyleSheet("""
            QWidget {
                background-color: #252525;
                border-top: 1px solid #2a2a2a;
            }
        """)
        
        return status_bar
    
    def togglePanelCollapse(self, splitter):
        """Toggle the collapse state of the panel"""
        if self.panel_collapsed:
            # Expand panel
            splitter.setSizes([200, 800])
            self.panel_collapsed = False
            # Change button text
            sender = self.sender()
            if sender:
                sender.setText("◀")
        else:
            # Collapse panel
            splitter.setSizes([0, 1000])
            self.panel_collapsed = True
            # Change button text
            sender = self.sender()
            if sender:
                sender.setText("▶")
        
        # Set initial state
        self.clone_button.setEnabled(False)
        self.save_button.setEnabled(False)
        self.export_button.setEnabled(False)
        
        # Connect button signals with clearer actions
        self.new_button.clicked.connect(self.createNew)
        self.clone_button.clicked.connect(self.cloneEntity)
        self.save_button.clicked.connect(self.saveEntity)
        self.export_button.clicked.connect(self.exportFlatBuffer)
        
    def createPreviewTab(self):
        """Create a default preview tab with image and information display"""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        
        # Create preview splitter (horizontal)
        preview_splitter = QSplitter(Qt.Horizontal)
        
        # Create left side: image preview
        image_widget = QWidget()
        image_layout = QVBoxLayout(image_widget)
        
        # Add title
        image_title = QLabel("Visual Preview")
        image_title.setStyleSheet("font-weight: bold; font-size: 14px;")
        image_layout.addWidget(image_title)
        
        # Add image label
        self.preview_image = ImageLabel("No preview available")
        self.preview_image.setFixedSize(400, 300)
            
        image_layout.addWidget(self.preview_image)
        image_layout.addStretch()
        
        # Create right side: data preview
        data_widget = QWidget()
        data_layout = QVBoxLayout(data_widget)
        
        # Add title
        data_title = QLabel("Data Preview")
        data_title.setStyleSheet("font-weight: bold; font-size: 14px;")
        data_layout.addWidget(data_title)
        
        # Create text area for data display
        self.preview_data = QTextEdit()
        self.preview_data.setReadOnly(True)
        self.preview_data.setPlaceholderText("Entity data will appear here")
        
        data_layout.addWidget(self.preview_data)
        
        # Add widgets to splitter
        preview_splitter.addWidget(image_widget)
        preview_splitter.addWidget(data_widget)
        preview_splitter.setSizes([400, 600])
        
        # Add splitter to tab layout
        layout.addWidget(preview_splitter)
        
        return tab
        
    # Methods to be implemented by subclasses
    def createNew(self):
        """Create a new entity"""
        # Set state for creating a new entity
        self.is_new_entity = True
        self.is_editing = True
        
        # Update button states
        self.updateButtonStates()
        
        # Switch to first tab (basic properties)
        if hasattr(self, 'tabs'):
            self.tabs.setCurrentIndex(0)
    
    def cloneEntity(self):
        """Clone the selected entity as a new one"""
        if not self.current_entity:
            self.showMessage("Clone Error", f"No {self.entity_type.lower()} selected to clone", is_error=True)
            return
        
        # Set state for cloning an entity
        self.is_new_entity = True
        self.is_editing = True
        
        # Show feedback to the user
        self.showMessage("Clone", f"Created clone of {self.entity_type}", is_error=False)
        
        # Update button states
        self.updateButtonStates()
    
    def saveEntity(self):
        """Save the entity (new or existing)"""
        # Validate entity has minimum required data
        if not self.validateEntity():
            return
            
        # Perform the save operation
        self.performSave()
        
        # Reset editing states
        self.is_editing = False
        self.is_new_entity = False
        
        # Update button states
        self.updateButtonStates()
        
        # Switch to preview tab
        if hasattr(self, 'tabs'):
            preview_index = self.tabs.count() - 1  # Usually the last tab
            self.tabs.setCurrentIndex(preview_index)
        
        # Show success message
        self.showMessage("Success", f"{self.entity_type} saved successfully", is_error=False)
    
    def validateEntity(self):
        """Validate that the entity has required fields before saving
        
        Returns:
            bool: True if valid, False otherwise
        """
        # Base implementation - should be overridden in subclasses
        return True
        
    def performSave(self):
        """Perform the actual save operation (to be implemented in subclasses)"""
        # Base implementation - should be overridden in subclasses
        pass
    
    def exportFlatBuffer(self):
        """Export entity configuration as a FlatBuffer file"""
        if not self.current_entity:
            self.showMessage("Export Error", f"No {self.entity_type.lower()} selected to export", is_error=True)
            return
        
        # Update the preview with current data
        self.updatePreview()
            
        # Open file dialog for export (implemented in subclasses)
        self.exportEntityData()
    
    def exportEntityData(self):
        """Export entity data to a file (to be implemented in subclasses)"""
        # Base implementation - should be overridden in subclasses
        self.showMessage("Export", "Export functionality must be implemented in subclasses", is_error=True)
    
    def updatePreview(self):
        """Update the preview with current entity data"""
        # Base implementation - should be overridden in subclasses
        if hasattr(self, 'preview_data'):
            if self.current_entity:
                # Format entity data as JSON for display
                try:
                    formatted_data = json.dumps(self.current_entity, indent=2)
                    self.preview_data.setText(formatted_data)
                except (TypeError, AttributeError):
                    self.preview_data.setText(str(self.current_entity))
            else:
                self.preview_data.clear()
    
    def updateButtonStates(self):
        """Update button states based on current state"""
        # Set button states based on entity selection and editing mode
        has_entity = self.current_entity is not None
        
        # New button is always enabled
        self.new_button.setEnabled(True)
        
        # Clone and export only enabled when an entity is selected and not editing
        self.clone_button.setEnabled(has_entity and not self.is_editing)
        self.export_button.setEnabled(has_entity and not self.is_editing)
        
        # Save only enabled when editing
        self.save_button.setEnabled(self.is_editing)
        
        # Update button titles based on entity type and state
        self.new_button.setText(f"New {self.entity_type}")
        self.clone_button.setText(f"Clone {self.entity_type}")
        
        if self.is_new_entity:
            self.save_button.setText(f"Create {self.entity_type}")
        else:
            self.save_button.setText(f"Save Changes")