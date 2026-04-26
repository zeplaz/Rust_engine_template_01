#!/usr/bin/env python3
import sys
import os
import json
import subprocess
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QLabel, QSlider, QCheckBox, QComboBox, QFileDialog,
    QTabWidget, QSpinBox, QDoubleSpinBox, QGroupBox, QFormLayout,
    QRadioButton, QButtonGroup, QSplitter, QScrollArea
)
from PySide6.QtCore import Qt, QProcess, Signal, Slot
from PySide6.QtGui import QPixmap, QImage

class WorldGeneratorTool(QMainWindow):
    def __init__(self):
        super().__init__()
        
        self.setWindowTitle("World Generator Tool")
        self.setMinimumSize(1024, 768)
        
        # Main widget and layout
        self.central_widget = QWidget()
        self.setCentralWidget(self.central_widget)
        self.main_layout = QVBoxLayout(self.central_widget)
        
        # Top toolbar with actions
        self.create_toolbar()
        
        # Main content splitter
        self.splitter = QSplitter(Qt.Horizontal)
        self.main_layout.addWidget(self.splitter)
        
        # Left panel: Settings
        self.settings_panel = QWidget()
        self.settings_layout = QVBoxLayout(self.settings_panel)
        self.create_settings_panel()
        self.splitter.addWidget(self.settings_panel)
        
        # Right panel: Preview
        self.preview_panel = QWidget()
        self.preview_layout = QVBoxLayout(self.preview_panel)
        self.create_preview_panel()
        self.splitter.addWidget(self.preview_panel)
        
        # Status bar
        self.statusBar().showMessage("Ready")
        
        # Set splitter sizes
        self.splitter.setSizes([400, 600])
        
        # Initialize the world generator process
        self.world_gen_process = None
        
    def create_toolbar(self):
        toolbar_layout = QHBoxLayout()
        toolbar_layout.setContentsMargins(0, 0, 0, 10)
        
        self.new_button = QPushButton("New World")
        self.new_button.clicked.connect(self.new_world)
        toolbar_layout.addWidget(self.new_button)
        
        self.load_button = QPushButton("Load")
        self.load_button.clicked.connect(self.load_world)
        toolbar_layout.addWidget(self.load_button)
        
        self.save_button = QPushButton("Save")
        self.save_button.clicked.connect(self.save_world)
        toolbar_layout.addWidget(self.save_button)
        
        self.export_button = QPushButton("Export")
        self.export_button.clicked.connect(self.export_world)
        toolbar_layout.addWidget(self.export_button)
        
        toolbar_layout.addStretch()
        
        self.launch_button = QPushButton("Launch Generator")
        self.launch_button.clicked.connect(self.launch_world_generator)
        toolbar_layout.addWidget(self.launch_button)
        
        self.main_layout.addLayout(toolbar_layout)
    
    def create_settings_panel(self):
        # Create tabs for different settings categories
        self.settings_tabs = QTabWidget()
        self.settings_layout.addWidget(self.settings_tabs)
        
        # General settings tab
        self.general_tab = QWidget()
        general_layout = QVBoxLayout(self.general_tab)
        self.settings_tabs.addTab(self.general_tab, "General")
        
        # World size
        size_group = QGroupBox("World Size")
        size_layout = QFormLayout()
        self.width_spin = QSpinBox()
        self.width_spin.setRange(128, 2048)
        self.width_spin.setValue(512)
        self.width_spin.setSingleStep(128)
        size_layout.addRow("Width:", self.width_spin)
        
        self.height_spin = QSpinBox()
        self.height_spin.setRange(128, 2048)
        self.height_spin.setValue(512)
        self.height_spin.setSingleStep(128)
        size_layout.addRow("Height:", self.height_spin)
        size_group.setLayout(size_layout)
        general_layout.addWidget(size_group)
        
        # Seed
        seed_group = QGroupBox("Seed")
        seed_layout = QHBoxLayout()
        self.seed_spin = QSpinBox()
        self.seed_spin.setRange(0, 9999999)
        self.seed_spin.setValue(12345)
        seed_layout.addWidget(self.seed_spin)
        self.random_seed_button = QPushButton("Random")
        seed_layout.addWidget(self.random_seed_button)
        seed_group.setLayout(seed_layout)
        general_layout.addWidget(seed_group)
        
        # Region settings tab
        self.regions_tab = QWidget()
        regions_layout = QVBoxLayout(self.regions_tab)
        self.settings_tabs.addTab(self.regions_tab, "Regions")
        
        # Region count
        region_count_group = QGroupBox("Region Count")
        region_count_layout = QVBoxLayout()
        self.region_count_slider = QSlider(Qt.Horizontal)
        self.region_count_slider.setRange(4, 64)
        self.region_count_slider.setValue(24)
        self.region_count_label = QLabel("24 regions")
        self.region_count_slider.valueChanged.connect(
            lambda v: self.region_count_label.setText(f"{v} regions")
        )
        region_count_layout.addWidget(self.region_count_slider)
        region_count_layout.addWidget(self.region_count_label)
        region_count_group.setLayout(region_count_layout)
        regions_layout.addWidget(region_count_group)
        
        # Region method
        region_method_group = QGroupBox("Region Generation Method")
        region_method_layout = QVBoxLayout()
        self.region_method_buttons = QButtonGroup()
        
        methods = [
            ("Regular Voronoi", "Regular"),
            ("Manhattan", "Manhattan"),
            ("Weighted", "Weighted"),
            ("Centroidal", "Centroidal"),
            ("Circular", "Circular"),
            ("Power", "Power")
        ]
        
        for i, (text, value) in enumerate(methods):
            radio = QRadioButton(text)
            radio.setChecked(i == 3)  # Default to Centroidal
            self.region_method_buttons.addButton(radio, i)
            region_method_layout.addWidget(radio)
        
        region_method_group.setLayout(region_method_layout)
        regions_layout.addWidget(region_method_group)
        
        # Terrain settings tab
        self.terrain_tab = QWidget()
        terrain_layout = QVBoxLayout(self.terrain_tab)
        self.settings_tabs.addTab(self.terrain_tab, "Terrain")
        
        # Noise settings
        noise_group = QGroupBox("Noise Settings")
        noise_layout = QFormLayout()
        
        self.noise_scale = QDoubleSpinBox()
        self.noise_scale.setRange(0.01, 0.1)
        self.noise_scale.setValue(0.03)
        self.noise_scale.setSingleStep(0.01)
        self.noise_scale.setDecimals(3)
        noise_layout.addRow("Noise Scale:", self.noise_scale)
        
        self.noise_octaves = QSpinBox()
        self.noise_octaves.setRange(1, 8)
        self.noise_octaves.setValue(6)
        noise_layout.addRow("Noise Octaves:", self.noise_octaves)
        
        self.moisture_bias = QDoubleSpinBox()
        self.moisture_bias.setRange(-0.5, 0.5)
        self.moisture_bias.setValue(0.0)
        self.moisture_bias.setSingleStep(0.1)
        self.moisture_bias.setDecimals(2)
        noise_layout.addRow("Moisture Bias:", self.moisture_bias)
        
        self.temperature_bias = QDoubleSpinBox()
        self.temperature_bias.setRange(-0.5, 0.5)
        self.temperature_bias.setValue(0.0)
        self.temperature_bias.setSingleStep(0.1)
        self.temperature_bias.setDecimals(2)
        noise_layout.addRow("Temperature Bias:", self.temperature_bias)
        
        noise_group.setLayout(noise_layout)
        terrain_layout.addWidget(noise_group)
        
        # Features settings tab
        self.features_tab = QWidget()
        features_layout = QVBoxLayout(self.features_tab)
        self.settings_tabs.addTab(self.features_tab, "Features")
        
        # Water features
        water_group = QGroupBox("Water Features")
        water_layout = QFormLayout()
        
        self.river_count = QSpinBox()
        self.river_count.setRange(0, 10)
        self.river_count.setValue(3)
        water_layout.addRow("Rivers:", self.river_count)
        
        self.lake_count = QSpinBox()
        self.lake_count.setRange(0, 5)
        self.lake_count.setValue(2)
        water_layout.addRow("Lakes:", self.lake_count)
        
        water_group.setLayout(water_layout)
        features_layout.addWidget(water_group)
        
        # Terrain features
        terrain_features_group = QGroupBox("Terrain Features")
        terrain_features_layout = QFormLayout()
        
        self.mountain_threshold = QDoubleSpinBox()
        self.mountain_threshold.setRange(0.5, 0.9)
        self.mountain_threshold.setValue(0.7)
        self.mountain_threshold.setSingleStep(0.05)
        self.mountain_threshold.setDecimals(2)
        terrain_features_layout.addRow("Mountain Threshold:", self.mountain_threshold)
        
        self.island_mode = QCheckBox("Island Mode")
        self.island_mode.setChecked(True)
        terrain_features_layout.addRow("", self.island_mode)
        
        self.island_falloff = QDoubleSpinBox()
        self.island_falloff.setRange(1.0, 5.0)
        self.island_falloff.setValue(3.0)
        self.island_falloff.setSingleStep(0.5)
        self.island_falloff.setDecimals(1)
        terrain_features_layout.addRow("Island Falloff:", self.island_falloff)
        
        terrain_features_group.setLayout(terrain_features_layout)
        features_layout.addWidget(terrain_features_group)
        
        # Add generate button at the bottom
        self.generate_button = QPushButton("Generate World")
        self.generate_button.clicked.connect(self.generate_world)
        self.settings_layout.addWidget(self.generate_button)
    
    def create_preview_panel(self):
        # Preview controls
        preview_controls = QWidget()
        preview_controls_layout = QHBoxLayout(preview_controls)
        preview_controls_layout.setContentsMargins(0, 0, 0, 0)
        
        preview_label = QLabel("Preview Mode:")
        preview_controls_layout.addWidget(preview_label)
        
        self.preview_mode = QComboBox()
        self.preview_mode.addItems(["Height Map", "Moisture Map", "Temperature Map", "Biome Map", "Region Map"])
        self.preview_mode.currentIndexChanged.connect(self.update_preview)
        preview_controls_layout.addWidget(self.preview_mode)
        
        preview_controls_layout.addStretch()
        
        self.preview_layout.addWidget(preview_controls)
        
        # Preview image
        scroll_area = QScrollArea()
        scroll_area.setWidgetResizable(True)
        
        self.preview_image_label = QLabel("No preview available")
        self.preview_image_label.setAlignment(Qt.AlignCenter)
        self.preview_image_label.setMinimumSize(400, 400)
        
        scroll_area.setWidget(self.preview_image_label)
        self.preview_layout.addWidget(scroll_area)
    
    def new_world(self):
        # Reset all settings to defaults
        self.width_spin.setValue(512)
        self.height_spin.setValue(512)
        self.seed_spin.setValue(12345)
        self.region_count_slider.setValue(24)
        self.region_method_buttons.button(3).setChecked(True)  # Centroidal
        self.noise_scale.setValue(0.03)
        self.noise_octaves.setValue(6)
        self.moisture_bias.setValue(0.0)
        self.temperature_bias.setValue(0.0)
        self.river_count.setValue(3)
        self.lake_count.setValue(2)
        self.mountain_threshold.setValue(0.7)
        self.island_mode.setChecked(True)
        self.island_falloff.setValue(3.0)
        
        # Clear preview
        self.preview_image_label.setText("No preview available")
        self.preview_image_label.setPixmap(QPixmap())
        
        self.statusBar().showMessage("New world initialized")
    
    def load_world(self):
        filename, _ = QFileDialog.getOpenFileName(
            self, "Load World", "", "World Files (*.json)"
        )
        if filename:
            try:
                with open(filename, 'r') as f:
                    data = json.load(f)
                
                # Load settings from file
                params = data.get('params', {})
                
                self.width_spin.setValue(params.get('width', 512))
                self.height_spin.setValue(params.get('height', 512))
                self.seed_spin.setValue(params.get('seed', 12345))
                self.region_count_slider.setValue(params.get('num_regions', 24))
                
                region_method = params.get('region_method', 'Centroidal')
                method_mapping = {
                    'Regular': 0, 'Manhattan': 1, 'Weighted': 2, 
                    'Centroidal': 3, 'Circular': 4, 'Power': 5
                }
                method_idx = method_mapping.get(region_method, 3)
                self.region_method_buttons.button(method_idx).setChecked(True)
                
                self.noise_scale.setValue(params.get('noise_scale', 0.03))
                self.noise_octaves.setValue(params.get('noise_octaves', 6))
                self.moisture_bias.setValue(params.get('moisture_bias', 0.0))
                self.temperature_bias.setValue(params.get('temperature_bias', 0.0))
                self.river_count.setValue(params.get('river_count', 3))
                self.lake_count.setValue(params.get('lake_count', 2))
                self.mountain_threshold.setValue(params.get('mountain_threshold', 0.7))
                self.island_mode.setChecked(params.get('island_mode', True))
                self.island_falloff.setValue(params.get('island_falloff', 3.0))
                
                # Update preview based on loaded data
                # This is a placeholder - in a real implementation you would
                # visualize the actual loaded world data
                self.generate_placeholder_preview()
                
                self.statusBar().showMessage(f"Loaded world from {filename}")
            except Exception as e:
                self.statusBar().showMessage(f"Error loading world: {e}")
    
    def save_world(self):
        filename, _ = QFileDialog.getSaveFileName(
            self, "Save World", "", "World Files (*.json)"
        )
        if filename:
            if not filename.endswith('.json'):
                filename += '.json'
            
            # Gather settings to save
            world_data = {
                'params': {
                    'width': self.width_spin.value(),
                    'height': self.height_spin.value(),
                    'seed': self.seed_spin.value(),
                    'num_regions': self.region_count_slider.value(),
                    'region_method': self.get_region_method(),
                    'noise_scale': self.noise_scale.value(),
                    'noise_octaves': self.noise_octaves.value(),
                    'moisture_bias': self.moisture_bias.value(),
                    'temperature_bias': self.temperature_bias.value(),
                    'river_count': self.river_count.value(),
                    'lake_count': self.lake_count.value(),
                    'mountain_threshold': self.mountain_threshold.value(),
                    'island_mode': self.island_mode.isChecked(),
                    'island_falloff': self.island_falloff.value(),
                },
                # In a real implementation, you would add the actual world data here
                'height_map': [],
                'moisture_map': [],
                'temperature_map': [],
                'region_map': [],
                'biome_map': []
            }
            
            try:
                with open(filename, 'w') as f:
                    json.dump(world_data, f, indent=2)
                self.statusBar().showMessage(f"Saved world to {filename}")
            except Exception as e:
                self.statusBar().showMessage(f"Error saving world: {e}")
    
    def export_world(self):
        filename, _ = QFileDialog.getSaveFileName(
            self, "Export World", "", "World Data Files (*.dat)"
        )
        if filename:
            if not filename.endswith('.dat'):
                filename += '.dat'
            
            # In a real implementation, you would export the world data
            # to a binary format for the game engine to consume
            
            # For now, we'll just create a placeholder file
            try:
                with open(filename, 'wb') as f:
                    # Write a simple header
                    header = f"WORLD_DATA_V1\n{self.width_spin.value()}\n{self.height_spin.value()}\n{self.seed_spin.value()}\n"
                    f.write(header.encode('utf-8'))
                    
                    # Write placeholder data
                    for _ in range(self.width_spin.value() * self.height_spin.value()):
                        f.write(b'\x00\x00\x00\x00')
                
                self.statusBar().showMessage(f"Exported world to {filename}")
            except Exception as e:
                self.statusBar().showMessage(f"Error exporting world: {e}")
    
    def launch_world_generator(self):
        if self.world_gen_process and self.world_gen_process.state() == QProcess.Running:
            self.statusBar().showMessage("World generator is already running")
            return
        
        # Find the path to the world generator executable
        # This path needs to be updated based on your project structure
        base_path = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        world_gen_path = os.path.join(base_path, "target", "debug", "world_generator")
        
        if not os.path.exists(world_gen_path):
            # Try to build it
            self.statusBar().showMessage("Building world generator...")
            try:
                result = subprocess.run(
                    ["cargo", "build", "--bin", "world_generator"], 
                    cwd=base_path,
                    check=True
                )
                if result.returncode != 0:
                    self.statusBar().showMessage("Failed to build world generator")
                    return
            except Exception as e:
                self.statusBar().showMessage(f"Error building world generator: {e}")
                return
        
        # Launch the process
        self.world_gen_process = QProcess()
        self.world_gen_process.finished.connect(self.on_world_gen_finished)
        self.world_gen_process.start(world_gen_path)
        
        if self.world_gen_process.state() == QProcess.Running:
            self.statusBar().showMessage("World generator launched")
        else:
            self.statusBar().showMessage("Failed to launch world generator")
    
    def on_world_gen_finished(self, exit_code, exit_status):
        if exit_code == 0:
            self.statusBar().showMessage("World generator exited successfully")
        else:
            self.statusBar().showMessage(f"World generator exited with code {exit_code}")
    
    def generate_world(self):
        # This would trigger the actual world generation process
        # For now, we'll just update the preview with a placeholder
        self.statusBar().showMessage("Generating world...")
        self.generate_placeholder_preview()
        self.statusBar().showMessage("World generated")
    
    def generate_placeholder_preview(self):
        # Generate a placeholder preview image
        width = self.width_spin.value()
        height = self.height_spin.value()
        
        # Create a simple gradient image
        image = QImage(width, height, QImage.Format_RGB32)
        
        preview_type = self.preview_mode.currentText()
        
        for y in range(height):
            for x in range(width):
                if preview_type == "Height Map":
                    # Gray gradient
                    val = int(255 * (x / width * 0.5 + y / height * 0.5))
                    image.setPixel(x, y, qRgb(val, val, val))
                elif preview_type == "Moisture Map":
                    # Blue gradient
                    val = int(255 * (x / width))
                    image.setPixel(x, y, qRgb(0, 0, val))
                elif preview_type == "Temperature Map":
                    # Red gradient
                    val = int(255 * (y / height))
                    image.setPixel(x, y, qRgb(val, 0, 0))
                elif preview_type == "Biome Map":
                    # Colorful grid
                    cx = x // 32
                    cy = y // 32
                    biome = (cx + cy) % 8
                    colors = [
                        qRgb(0, 0, 128),     # Deep water
                        qRgb(0, 0, 255),     # Shallow water
                        qRgb(240, 240, 64),  # Beach
                        qRgb(0, 255, 0),     # Grassland
                        qRgb(0, 128, 0),     # Forest
                        qRgb(128, 128, 128), # Mountain
                        qRgb(255, 255, 255), # Snow
                        qRgb(255, 255, 128)  # Desert
                    ]
                    image.setPixel(x, y, colors[biome])
                elif preview_type == "Region Map":
                    # Random colored regions
                    cx = x // 64
                    cy = y // 64
                    region = (cx * 4 + cy) % 6
                    colors = [
                        qRgb(255, 0, 0),     # Red
                        qRgb(0, 255, 0),     # Green
                        qRgb(0, 0, 255),     # Blue
                        qRgb(255, 255, 0),   # Yellow
                        qRgb(255, 0, 255),   # Magenta
                        qRgb(0, 255, 255),   # Cyan
                    ]
                    image.setPixel(x, y, colors[region])
        
        # Scale the image to fit the view if necessary
        preview_pixmap = QPixmap.fromImage(image).scaled(
            800, 600, Qt.KeepAspectRatio, Qt.SmoothTransformation
        )
        
        self.preview_image_label.setPixmap(preview_pixmap)
    
    def update_preview(self):
        # Update the preview based on the selected mode
        if self.preview_image_label.pixmap():
            self.generate_placeholder_preview()
    
    def get_region_method(self):
        method_mapping = {
            0: "Regular",
            1: "Manhattan",
            2: "Weighted",
            3: "Centroidal",
            4: "Circular",
            5: "Power"
        }
        return method_mapping.get(self.region_method_buttons.checkedId(), "Centroidal")

if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = WorldGeneratorTool()
    window.show()
    sys.exit(app.exec())