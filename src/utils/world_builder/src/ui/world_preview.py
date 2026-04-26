import numpy as np
from PyQt5.QtWidgets import QWidget, QVBoxLayout, QLabel, QComboBox, QHBoxLayout
from PyQt5.QtGui import QPixmap, QImage
from PyQt5.QtCore import Qt

try:
    from qfluentwidgets import ComboBox, ImageLabel
    HAS_FLUENT = True
except ImportError:
    HAS_FLUENT = False

class WorldPreviewWidget(QWidget):
    """Widget for previewing the generated world"""
    
    def __init__(self, settings, parent=None):
        super().__init__(parent)
        self.settings = settings
        self.terrain_map = None
        self.region_map = None
        self.resource_map = None
        self.preview_mode = "Terrain"
        self.initUI()
    
    def initUI(self):
        """Initialize the UI"""
        layout = QVBoxLayout(self)
        
        # Preview mode selection
        preview_layout = QHBoxLayout()
        preview_layout.addWidget(QLabel("Preview Mode:"))
        
        if HAS_FLUENT:
            self.preview_mode_combo = ComboBox()
        else:
            self.preview_mode_combo = QComboBox()
        
        self.preview_mode_combo.addItems(["Terrain", "Regions", "Resources", "Height", "Biomes"])
        self.preview_mode_combo.currentTextChanged.connect(self.onPreviewModeChanged)
        preview_layout.addWidget(self.preview_mode_combo)
        layout.addLayout(preview_layout)
        
        # Preview image
        if HAS_FLUENT:
            self.preview_label = ImageLabel("No preview available")
            self.preview_label.setMinimumSize(600, 400)
        else:
            self.preview_label = QLabel("No preview available")
            self.preview_label.setAlignment(Qt.AlignCenter)
            self.preview_label.setMinimumSize(600, 400)
        
        layout.addWidget(self.preview_label)
        
        # Info label
        self.info_label = QLabel()
        layout.addWidget(self.info_label)
    
    def onPreviewModeChanged(self, mode):
        """Handle preview mode change"""
        self.preview_mode = mode
        self.updatePreview()
    
    def updatePreview(self):
        """Update the preview based on current maps and mode"""
        if self.preview_mode == "Terrain" and self.terrain_map is not None:
            self.showTerrainPreview()
        elif self.preview_mode == "Regions" and self.region_map is not None:
            self.showRegionPreview()
        elif self.preview_mode == "Resources" and self.resource_map is not None:
            self.showResourcePreview()
        elif self.preview_mode == "Height" and self.terrain_map is not None:
            self.showHeightPreview()
        elif self.preview_mode == "Biomes" and self.terrain_map is not None:
            self.showBiomePreview()
        else:
            # Show placeholder
            self.preview_label.setText("No data available for this preview mode")
            self.info_label.setText("")
    
    def updateTerrainLayer(self, terrain_data=None):
        """Update terrain layer data"""
        if terrain_data is not None:
            self.terrain_map = terrain_data
        
        if self.preview_mode in ["Terrain", "Height", "Biomes"]:
            self.updatePreview()
    
    def updateRegionLayer(self, region_data=None):
        """Update region layer data"""
        if region_data is not None:
            self.region_map = region_data
        
        if self.preview_mode == "Regions":
            self.updatePreview()
    
    def updateResourceLayer(self, resource_data=None):
        """Update resource layer data"""
        if resource_data is not None:
            self.resource_map = resource_data
        
        if self.preview_mode == "Resources":
            self.updatePreview()
    
    def showTerrainPreview(self):
        """Show terrain preview"""
        # Create a color map based on terrain types
        colormap = np.zeros((self.terrain_map.shape[0], self.terrain_map.shape[1], 3), dtype=np.uint8)
        
        for y in range(self.terrain_map.shape[0]):
            for x in range(self.terrain_map.shape[1]):
                terrain_type = self.terrain_map[y, x]
                colormap[y, x] = self._get_terrain_color(terrain_type)
        
        self._show_image(colormap)
        self.info_label.setText(f"Terrain map ({self.terrain_map.shape[1]}x{self.terrain_map.shape[0]})")
    
    def showRegionPreview(self):
        """Show region preview"""
        # Create a color map based on regions
        colormap = np.zeros((self.region_map.shape[0], self.region_map.shape[1], 3), dtype=np.uint8)
        
        # Generate random colors for regions if not already done
        unique_regions = np.unique(self.region_map)
        region_colors = {}
        
        for region in unique_regions:
            if region not in region_colors:
                # Generate random color (avoiding dark colors)
                r = np.random.randint(100, 256)
                g = np.random.randint(100, 256)
                b = np.random.randint(100, 256)
                region_colors[region] = (r, g, b)
        
        for y in range(self.region_map.shape[0]):
            for x in range(self.region_map.shape[1]):
                region = self.region_map[y, x]
                colormap[y, x] = region_colors[region]
        
        self._show_image(colormap)
        self.info_label.setText(f"Region map ({self.region_map.shape[1]}x{self.region_map.shape[0]}) - {len(unique_regions)} regions")
    
    def showResourcePreview(self):
        """Show resource preview"""
        # Create a color map based on resources
        colormap = np.zeros((self.resource_map.shape[0], self.resource_map.shape[1], 3), dtype=np.uint8)
        
        # Define resource colors
        resource_colors = {
            0: (200, 200, 200),  # No resource
            1: (255, 215, 0),    # Gold/metal
            2: (0, 0, 0),        # Coal
            3: (139, 69, 19),    # Wood
            4: (0, 128, 0),      # Food
            5: (0, 0, 255),      # Water
            6: (0, 0, 139),      # Oil
            7: (255, 0, 0)       # Iron
        }
        
        for y in range(self.resource_map.shape[0]):
            for x in range(self.resource_map.shape[1]):
                resource = self.resource_map[y, x]
                colormap[y, x] = resource_colors.get(resource, (128, 128, 128))
        
        self._show_image(colormap)
        self.info_label.setText(f"Resource map ({self.resource_map.shape[1]}x{self.resource_map.shape[0]})")
    
    def showHeightPreview(self):
        """Show height preview as grayscale"""
        # Assuming self.terrain_map contains height information in the first channel
        if len(self.terrain_map.shape) == 3:
            heightmap = self.terrain_map[:, :, 0]
        else:
            heightmap = self.terrain_map
        
        # Create a grayscale image
        colormap = np.zeros((heightmap.shape[0], heightmap.shape[1], 3), dtype=np.uint8)
        
        # Normalize height to 0-255
        min_height = np.min(heightmap)
        max_height = np.max(heightmap)
        normalized = (heightmap - min_height) / (max_height - min_height) * 255
        
        for y in range(heightmap.shape[0]):
            for x in range(heightmap.shape[1]):
                value = int(normalized[y, x])
                colormap[y, x] = (value, value, value)
        
        self._show_image(colormap)
        self.info_label.setText(f"Height map ({heightmap.shape[1]}x{heightmap.shape[0]}) - Min: {min_height:.2f}, Max: {max_height:.2f}")
    
    def showBiomePreview(self):
        """Show biome preview based on terrain data"""
        # This is a placeholder implementation that would need to be customized
        # based on how biomes are determined in your world generation
        
        # Create a color map based on a combination of factors
        colormap = np.zeros((self.terrain_map.shape[0], self.terrain_map.shape[1], 3), dtype=np.uint8)
        
        # Define biome colors
        biome_colors = {
            0: (0, 0, 128),       # Deep ocean
            1: (0, 0, 255),       # Shallow water
            2: (240, 240, 64),    # Beach
            3: (32, 160, 0),      # Grassland
            4: (0, 100, 0),       # Forest
            5: (128, 128, 128),   # Mountain
            6: (255, 255, 255),   # Snow
            7: (224, 224, 0),     # Desert
            8: (160, 82, 45),     # Savanna
            9: (64, 64, 0)        # Swamp
        }
        
        # Map terrain types to biomes (simplified)
        terrain_to_biome = {
            0: 0,  # Water -> Deep ocean
            1: 1,  # Shallow water -> Shallow water
            2: 2,  # Sand -> Beach
            3: 3,  # Grass -> Grassland
            4: 4,  # Forest -> Forest
            5: 5,  # Mountain -> Mountain
            6: 6   # Snow -> Snow
        }
        
        for y in range(self.terrain_map.shape[0]):
            for x in range(self.terrain_map.shape[1]):
                terrain_type = self.terrain_map[y, x]
                biome = terrain_to_biome.get(terrain_type, 0)
                colormap[y, x] = biome_colors.get(biome, (128, 128, 128))
        
        self._show_image(colormap)
        self.info_label.setText(f"Biome map ({self.terrain_map.shape[1]}x{self.terrain_map.shape[0]})")
    
    def _get_terrain_color(self, terrain_type):
        """Get color for terrain type"""
        terrain_colors = {
            0: (0, 0, 128),     # Deep water
            1: (0, 0, 255),     # Shallow water
            2: (240, 240, 64),  # Sand
            3: (32, 160, 0),    # Grass
            4: (0, 100, 0),     # Forest
            5: (128, 128, 128), # Mountain
            6: (255, 255, 255)  # Snow
        }
        
        return terrain_colors.get(terrain_type, (128, 128, 128))
    
    def _show_image(self, colormap):
        """Display the image in the preview label"""
        height, width, _ = colormap.shape
        
        # Create QImage from numpy array
        qimg = QImage(colormap.data, width, height, width * 3, QImage.Format_RGB888)
        
        # Create pixmap and scale to fit
        pixmap = QPixmap.fromImage(qimg)
        
        # Get label size
        label_width = self.preview_label.width()
        label_height = self.preview_label.height()
        
        # Scale pixmap to fit label, preserving aspect ratio
        pixmap = pixmap.scaled(label_width, label_height, Qt.KeepAspectRatio, Qt.SmoothTransformation)
        
        # Display pixmap
        if HAS_FLUENT:
            self.preview_label.setPixmap(pixmap)
        else:
            self.preview_label.setPixmap(pixmap)
    
    def updateSettings(self, settings):
        """Update settings"""
        self.settings = settings
    
    def resetPreview(self):
        """Reset preview data"""
        self.terrain_map = None
        self.region_map = None
        self.resource_map = None
        
        # Clear preview
        if HAS_FLUENT:
            self.preview_label.setText("No preview available")
        else:
            self.preview_label.setText("No preview available")
            self.preview_label.setPixmap(QPixmap())
        
        self.info_label.setText("")