"""
Settings for the World Builder
"""

import json
import os
from typing import Dict, Any, List, Optional

class Settings:
    """Settings class for the World Builder"""
    
    def __init__(self):
        """Initialize with default settings"""
        # Game asset paths
        self.game_dir = self._find_game_dir()
        self.data_dir = os.path.join(self.game_dir, "data")
        self.texture_dir = os.path.join(self.data_dir, "textures")
        
        # Default world generation settings
        self.world_width = 1024
        self.world_height = 1024
        self.tile_size = 64
        self.region_count = 16
        self.region_generation_method = "voronoi"
        
        # Terrain generation settings
        self.use_perlin_noise = True
        self.noise_scale = 0.1
        self.noise_octaves = 4
        self.noise_persistence = 0.5
        self.noise_lacunarity = 2.0
        self.water_level = 0.3
        self.mountain_level = 0.7
        
        # Resource generation settings
        self.resource_density = 0.1
        self.resource_types = ["Metal", "Oil", "Water", "Food", "Coal"]
        self.resource_distribution = {
            "Metal": 0.2,
            "Oil": 0.15,
            "Water": 0.3,
            "Food": 0.25,
            "Coal": 0.1
        }
        
        # Visual settings
        self.terrain_color_map = {
            "water": (0, 0, 200),
            "sand": (240, 240, 120),
            "grass": (30, 180, 30),
            "forest": (0, 100, 0),
            "mountain": (120, 120, 120),
            "snow": (255, 255, 255)
        }
        
        # UI settings
        self.dark_mode = True
        self.autosave_enabled = True
        self.autosave_interval = 5  # minutes
        
    def to_dict(self) -> Dict[str, Any]:
        """Convert settings to dictionary for serialization"""
        return {
            "game_dir": self.game_dir,
            "data_dir": self.data_dir,
            "texture_dir": self.texture_dir,
            "world_width": self.world_width,
            "world_height": self.world_height,
            "tile_size": self.tile_size,
            "region_count": self.region_count,
            "region_generation_method": self.region_generation_method,
            "use_perlin_noise": self.use_perlin_noise,
            "noise_scale": self.noise_scale,
            "noise_octaves": self.noise_octaves,
            "noise_persistence": self.noise_persistence,
            "noise_lacunarity": self.noise_lacunarity,
            "water_level": self.water_level,
            "mountain_level": self.mountain_level,
            "resource_density": self.resource_density,
            "resource_types": self.resource_types,
            "resource_distribution": self.resource_distribution,
            "terrain_color_map": self.terrain_color_map,
            "dark_mode": self.dark_mode,
            "autosave_enabled": self.autosave_enabled,
            "autosave_interval": self.autosave_interval
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Settings':
        """Create settings from dictionary"""
        settings = cls()
        for key, value in data.items():
            if hasattr(settings, key):
                setattr(settings, key, value)
        return settings
    
    def _find_game_dir(self) -> str:
        """Find the game directory"""
        # Start with current directory
        current_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        
        # Try to navigate up to find the game directory
        if os.path.basename(current_dir) == "utils":
            game_dir = os.path.dirname(current_dir)
        else:
            game_dir = current_dir
            
        # Validate that it's the correct directory
        if not os.path.exists(os.path.join(game_dir, "data")):
            # If data directory doesn't exist, create it
            os.makedirs(os.path.join(game_dir, "data"), exist_ok=True)
            
        return game_dir

def load_settings(file_path: str) -> Settings:
    """Load settings from a JSON file"""
    settings = Settings()
    
    if os.path.exists(file_path):
        try:
            with open(file_path, 'r') as f:
                data = json.load(f)
                settings = Settings.from_dict(data)
        except (json.JSONDecodeError, IOError):
            print(f"Error loading settings from {file_path}, using defaults")
    else:
        # Save default settings
        save_settings(file_path, settings)
    
    return settings

def save_settings(file_path: str, settings: Settings) -> bool:
    """Save settings to a JSON file"""
    try:
        # Ensure directory exists
        os.makedirs(os.path.dirname(file_path), exist_ok=True)
        
        # Save settings
        with open(file_path, 'w') as f:
            json.dump(settings.to_dict(), f, indent=4)
        return True
    except IOError:
        print(f"Error saving settings to {file_path}")
        return False