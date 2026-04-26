import os
from PyQt5.QtCore import Qt, pyqtSignal
from PyQt5.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                            QListWidget, QFileDialog, QGroupBox,
                            QSplitter, QFileSystemModel, QTreeView)
from PyQt5.QtGui import QPixmap

# Import base page and widgets
from .base_page import BasePage
from .base_page import (PushButton, FluentIcon, TreeView, ImageLabel)
    
# Try to import from config directory
try:
    from ..config.asset_config import TEXTURE_MAP_STATES
except ImportError:
    # Fallback constants if not available
    TEXTURE_MAP_STATES = ["Default", "Dirty", "Damaged", "Destroyed"]

class TexturePage(BasePage):
    """Texture editor page"""
    
    # Signal to notify when texture is selected
    textureSelected = pyqtSignal(str)
    
    def __init__(self, parent=None):
        super().__init__(parent)
        self.current_path = None
        self.initUI()
        
    def createNew(self):
        """Import a new texture (For compatibility with other pages)"""
        self.importExternalTexture()
        
    def initUI(self):
        """Initialize the UI"""
        layout = QHBoxLayout(self)
        
        # Create a splitter for file browser and preview
        splitter = QSplitter(Qt.Horizontal)
        
        # Left panel - file browser
        left_panel = self.createFileBrowserPanel()
        
        # Right panel - preview
        right_panel = self.createPreviewPanel()
        
        # Add panels to splitter
        splitter.addWidget(left_panel)
        splitter.addWidget(right_panel)
        
        # Set initial sizes - give more space to preview
        splitter.setSizes([300, 700])
        
        layout.addWidget(splitter)
    
    def createFileBrowserPanel(self):
        """Create the file browser panel"""
        panel = QWidget()
        layout = QVBoxLayout(panel)
        
        # Create a file system model
        file_model = QFileSystemModel()
        file_model.setNameFilters(["*.png", "*.jpg", "*.jpeg", "*.bmp", "*.tif", "*.tiff"])
        file_model.setNameFilterDisables(False)
        file_model.setRootPath(os.path.expanduser("~"))
        
        # Create a tree view - note we use QTreeView instead of TreeView because
        # qfluentwidgets' TreeView doesn't have necessary file system methods
        self.file_tree = QTreeView()
        self.file_tree.setModel(file_model)
        # Set root index to the root path
        root_path = os.path.expanduser("~")
        self.file_tree.setRootIndex(file_model.index(root_path))
            
        # Add buttons
        browse_button = PushButton("Browse Game Assets")
        browse_button.setIcon(FluentIcon.FOLDER)
        import_button = PushButton("Import External Texture")
        import_button.setIcon(FluentIcon.DOWNLOAD)
        
        # Hide columns we don't need
        self.file_tree.setColumnHidden(1, True) # Size
        self.file_tree.setColumnHidden(2, True) # Type
        self.file_tree.setColumnHidden(3, True) # Date Modified
        
        # Add widgets to layout
        layout.addWidget(QLabel("Browse for textures:"))
        layout.addWidget(browse_button)
        layout.addWidget(import_button)
        layout.addWidget(self.file_tree)
        
        # Connect signals
        self.file_tree.clicked.connect(self.onFileSelected)
        browse_button.clicked.connect(self.browseGameAssets)
        import_button.clicked.connect(self.importExternalTexture)
        
        return panel
    
    def createPreviewPanel(self):
        """Create the preview panel"""
        panel = QWidget()
        layout = QVBoxLayout(panel)
        layout.setContentsMargins(10, 10, 10, 10)
        layout.setSpacing(15)
        
        # Add title
        title_label = QLabel("Texture Preview")
        title_label.setStyleSheet("font-weight: bold; font-size: 16px; color: #5dca31;")
        
        # Create the preview label
        self.preview_label = ImageLabel("Select a texture to preview")
        self.preview_label.setAlignment(Qt.AlignCenter)
        self.preview_label.setMinimumSize(400, 300)
        self.preview_label.setStyleSheet("border: 1px solid #444; background-color: #222; border-radius: 4px; padding: 5px;")
        
        # Create info section in a group box for better organization
        info_group = QGroupBox("Image Information")
        info_layout = QVBoxLayout()
        
        # Create info rows using simple layouts
        filename_layout = QHBoxLayout()
        filename_layout.addWidget(QLabel("Filename:"))
        self.filename_label = QLabel("None selected")
        filename_layout.addWidget(self.filename_label)
        filename_layout.addStretch()
        
        dimensions_layout = QHBoxLayout()
        dimensions_layout.addWidget(QLabel("Dimensions:"))
        self.dimensions_label = QLabel("N/A")
        dimensions_layout.addWidget(self.dimensions_label)
        dimensions_layout.addStretch()
        
        filesize_layout = QHBoxLayout()
        filesize_layout.addWidget(QLabel("File size:"))
        self.filesize_label = QLabel("N/A")
        filesize_layout.addWidget(self.filesize_label)
        filesize_layout.addStretch()
        
        # Add all layouts to the info layout
        info_layout.addLayout(filename_layout)
        info_layout.addLayout(dimensions_layout)
        info_layout.addLayout(filesize_layout)
        
        info_group.setLayout(info_layout)
        
        # Create map states group
        map_states_group = QGroupBox("Map States")
        map_states_layout = QVBoxLayout()
        map_states_layout.setSpacing(10)
        
        # Add map states explanation
        map_states_info = QLabel("Textures can have multiple states (e.g., clean, dirty, damaged)")
        map_states_info.setWordWrap(True)
        map_states_layout.addWidget(map_states_info)
        
        # Add map states buttons
        map_button = PushButton("Map Texture States")
        map_button.setIcon(FluentIcon.PHOTO)
        map_states_layout.addWidget(map_button)
        
        # Set group layout
        map_states_group.setLayout(map_states_layout)
        
        # Create actions section
        actions_group = QGroupBox("Actions")
        actions_layout = QHBoxLayout()
        actions_layout.setContentsMargins(10, 10, 10, 10)
        actions_layout.setSpacing(10)
        
        add_button = PushButton("Add to Library")
        add_button.setIcon(FluentIcon.ADD)
        
        export_button = PushButton("Export")
        export_button.setIcon(FluentIcon.SAVE)
        
        actions_layout.addWidget(add_button)
        actions_layout.addWidget(export_button)
        actions_layout.addStretch()
        actions_group.setLayout(actions_layout)
        
        # Add all widgets to main layout
        layout.addWidget(title_label)
        layout.addWidget(self.preview_label)
        layout.addWidget(info_group)
        layout.addWidget(map_states_group)
        layout.addWidget(actions_group)
        layout.addStretch()
        
        # Connect signals
        map_button.clicked.connect(self.mapTextureStates)
        add_button.clicked.connect(self.addToLibrary)
        export_button.clicked.connect(self.exportTexture)
        
        return panel
    
    def onFileSelected(self, index):
        """Handle file selection from tree view"""
        # Get the file path
        model = self.file_tree.model()
        file_path = model.filePath(index)
        
        # Check if it's a file
        if os.path.isfile(file_path):
            # Store the current path
            self.current_path = file_path
            
            # Update preview
            self.updatePreview(file_path)
            
            # Emit signal that texture was selected
            self.textureSelected.emit(file_path)
    
    def updatePreview(self, file_path):
        """Update the preview with the selected texture"""
        try:
            # Load the image
            pixmap = QPixmap(file_path)
            
            if pixmap.isNull():
                # Failed to load - show error
                self.showMessage("Error", f"Failed to load image: {file_path}", is_error=True)
                return
                
            # Set image to preview label
            self.preview_label.setPixmap(pixmap.scaled(
                400, 300, 
                Qt.KeepAspectRatio, 
                Qt.SmoothTransformation
            ))
            
            # Update info labels
            filename = os.path.basename(file_path)
            dimensions = f"{pixmap.width()} x {pixmap.height()}"
            filesize = os.path.getsize(file_path) / 1024  # in KB
            
            self.filename_label.setText(f"{filename}")
            self.dimensions_label.setText(f"{dimensions}")
            self.filesize_label.setText(f"{filesize:.1f} KB")
            
        except Exception as e:
            # Show error message
            self.showMessage("Error", f"Error loading image: {str(e)}", is_error=True)
    
    def browseGameAssets(self):
        """Browse game assets directory"""
        assets_dir = os.path.join(os.path.expanduser("~"), "game_assets", "textures")
        
        # Create directory if it doesn't exist
        os.makedirs(assets_dir, exist_ok=True)
        
        # Set the root path to assets directory
        self.file_tree.setRootIndex(
            self.file_tree.model().index(assets_dir)
        )
    
    def importExternalTexture(self):
        """Import texture from external source"""
        file_dialog = QFileDialog()
        file_path, _ = file_dialog.getOpenFileName(
            self,
            "Import Texture",
            os.path.expanduser("~"),
            "Image files (*.png *.jpg *.jpeg *.bmp *.tif *.tiff)"
        )
        
        if file_path:
            # Update preview
            self.updatePreview(file_path)
            self.current_path = file_path
    
    def mapTextureStates(self):
        """Map texture states dialog"""
        if not self.current_path:
            self.showMessage("Error", "Please select a texture first", is_error=True)
            return
        
        # For this example, just show a message
        states_str = ", ".join(TEXTURE_MAP_STATES)
        self.showMessage(
            "Map Texture States", 
            f"Would map the following states for {os.path.basename(self.current_path)}: {states_str}",
            is_error=False
        )
    
    def addToLibrary(self):
        """Add texture to game library"""
        if not self.current_path:
            self.showMessage("Error", "Please select a texture first", is_error=True)
            return
            
        # For this example, just show a message
        self.showMessage(
            "Add to Library", 
            f"Added {os.path.basename(self.current_path)} to game library",
            is_error=False
        )
    
    def exportTexture(self):
        """Export texture to another format"""
        if not self.current_path:
            self.showMessage("Error", "Please select a texture first", is_error=True)
            return
            
        # For this example, just show a message
        self.showMessage(
            "Export Texture", 
            f"Exported {os.path.basename(self.current_path)}",
            is_error=False
        )