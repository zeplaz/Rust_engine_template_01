"""
Simple placeholder terrain generator for testing
"""
import numpy as np

class TerrainGenerator:
    """Simple terrain generator for testing"""
    
    def __init__(self, settings):
        self.settings = settings
        self.reset()
    
    def reset(self):
        """Reset the generator"""
        self.seed = np.random.randint(0, 1000000)
    
    def generate_heightmap(self):
        """Generate a simple heightmap using sine waves"""
        width = getattr(self.settings, 'world_width', 256)
        height = getattr(self.settings, 'world_height', 256)
        scale = getattr(self.settings, 'noise_scale', 0.1)
        
        # Create heightmap
        heightmap = np.zeros((height, width), dtype=np.float32)
        
        for y in range(height):
            for x in range(width):
                nx = x / width * 10.0 * scale
                ny = y / height * 10.0 * scale
                
                # Simple noise using sine waves
                value = (np.sin(nx) * np.cos(ny) * 0.5 + 0.5 +
                         np.sin(nx * 2) * np.cos(ny * 2) * 0.25)
                
                heightmap[y, x] = value
        
        # Apply simple island falloff if desired
        if getattr(self.settings, 'island_mode', True):
            center_x = width / 2
            center_y = height / 2
            
            for y in range(height):
                for x in range(width):
                    dx = (x - center_x) / (width / 2)
                    dy = (y - center_y) / (height / 2)
                    dist = np.sqrt(dx * dx + dy * dy)
                    
                    # Circular falloff
                    if dist < 1.0:
                        falloff = 1.0 - dist
                    else:
                        falloff = 0.0
                    
                    heightmap[y, x] *= falloff
        
        return heightmap
    
    def to_dict(self):
        """Convert to dictionary for serialization"""
        return {
            'seed': self.seed,
            'settings': {
                'world_width': getattr(self.settings, 'world_width', 256),
                'world_height': getattr(self.settings, 'world_height', 256),
                'noise_scale': getattr(self.settings, 'noise_scale', 0.1),
                'island_mode': getattr(self.settings, 'island_mode', True),
            }
        }
    
    def from_dict(self, data):
        """Load from dictionary"""
        self.seed = data.get('seed', np.random.randint(0, 1000000))
        
        # Update settings if needed
        settings_data = data.get('settings', {})
        if hasattr(self.settings, 'world_width'):
            self.settings.world_width = settings_data.get('world_width', self.settings.world_width)
        
        if hasattr(self.settings, 'world_height'):
            self.settings.world_height = settings_data.get('world_height', self.settings.world_height)
        
        if hasattr(self.settings, 'noise_scale'):
            self.settings.noise_scale = settings_data.get('noise_scale', self.settings.noise_scale)
        
        if hasattr(self.settings, 'island_mode'):
            self.settings.island_mode = settings_data.get('island_mode', self.settings.island_mode)