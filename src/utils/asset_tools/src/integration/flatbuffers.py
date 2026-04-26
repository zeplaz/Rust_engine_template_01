"""FlatBuffers integration for asset tool"""

import os
import json
import subprocess
import tempfile
from typing import Dict, Any, List, Optional, Union

# Check if flatc compiler is available
try:
    subprocess.run(["flatc", "--version"], capture_output=True, check=True)
    FLATC_AVAILABLE = True
except (subprocess.SubprocessError, FileNotFoundError):
    FLATC_AVAILABLE = False
    print("Warning: FlatBuffers compiler not found. FlatBuffers integration will be disabled.")

class FlatBuffersIntegration:
    """Integration with FlatBuffers for Rust"""
    
    def __init__(self, schema_dir: Optional[str] = None):
        """Initialize FlatBuffers integration
        
        Args:
            schema_dir: Directory containing FlatBuffer schema files (.fbs)
                       If None, will try to detect automatically
        """
        if schema_dir is None:
            # Try to find schema directory
            base_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
            game_dir = os.path.dirname(os.path.dirname(base_dir))
            schema_dir = os.path.join(game_dir, "data", "flatbuffers")
        
        self.schema_dir = schema_dir
        self.schemas = {}
        
        # Load schemas if flatc is available
        if FLATC_AVAILABLE and os.path.isdir(schema_dir):
            self._load_schemas()
    
    def _load_schemas(self) -> None:
        """Load FlatBuffer schema files"""
        for filename in os.listdir(self.schema_dir):
            if filename.endswith('.fbs'):
                schema_path = os.path.join(self.schema_dir, filename)
                schema_name = os.path.splitext(filename)[0]
                
                # Load schema content
                with open(schema_path, 'r') as f:
                    schema_content = f.read()
                
                self.schemas[schema_name] = {
                    'path': schema_path,
                    'content': schema_content
                }
                
                # Parse schema to extract information
                self._parse_schema(schema_name, schema_content)
    
    def _parse_schema(self, schema_name: str, schema_content: str) -> None:
        """Parse schema content to extract relevant information"""
        # Extract namespace
        namespace = None
        for line in schema_content.split('\n'):
            if line.strip().startswith('namespace '):
                namespace = line.strip().split(' ')[1].rstrip(';')
                break
        
        # Extract root type
        root_type = None
        for line in schema_content.split('\n'):
            if line.strip().startswith('root_type '):
                root_type = line.strip().split(' ')[1].rstrip(';')
                break
        
        # Extract enums
        enums = {}
        current_enum = None
        for line in schema_content.split('\n'):
            line = line.strip()
            if line.startswith('enum '):
                # Start of enum definition
                enum_def = line.split(':')[0].strip()
                enum_name = enum_def.split(' ')[1]
                current_enum = enum_name
                enums[current_enum] = []
            elif current_enum and line and line[0].isalpha() and '=' not in line and '//' not in line:
                # Enum value
                enum_value = line.split(',')[0].strip()
                if enum_value:
                    enums[current_enum].append(enum_value)
            elif current_enum and line == '}':
                # End of enum definition
                current_enum = None
        
        # Store parsed information
        self.schemas[schema_name].update({
            'namespace': namespace,
            'root_type': root_type,
            'enums': enums
        })
    
    def export_to_flatbuffers(self, data: Dict[str, Any], schema_name: str, output_path: str) -> bool:
        """Export data to FlatBuffers binary file
        
        Args:
            data: Data to export
            schema_name: Name of schema to use (without .fbs extension)
            output_path: Path to save FlatBuffers binary
            
        Returns:
            True if export successful, False otherwise
        """
        if not FLATC_AVAILABLE:
            print("Error: FlatBuffers compiler not available")
            return False
        
        if schema_name not in self.schemas:
            print(f"Error: Schema '{schema_name}' not found")
            return False
        
        # Create temporary JSON file
        with tempfile.NamedTemporaryFile(suffix='.json', mode='w', delete=False) as temp_file:
            json.dump(data, temp_file)
            temp_path = temp_file.name
        
        try:
            # Run flatc to compile JSON to binary
            schema_path = self.schemas[schema_name]['path']
            subprocess.run([
                "flatc",
                "-o", os.path.dirname(output_path),
                "--binary",
                "--strict-json",
                "--defaults-json",
                schema_path,
                temp_path
            ], check=True)
            
            # Rename output file
            temp_output = os.path.join(
                os.path.dirname(output_path),
                os.path.basename(temp_path).replace('.json', '.bin')
            )
            os.rename(temp_output, output_path)
            
            return True
        except subprocess.SubprocessError as e:
            print(f"Error running flatc: {e}")
            return False
        finally:
            # Clean up temporary file
            if os.path.exists(temp_path):
                os.unlink(temp_path)
    
    def import_from_flatbuffers(self, binary_path: str, schema_name: str) -> Optional[Dict[str, Any]]:
        """Import data from FlatBuffers binary file
        
        Args:
            binary_path: Path to FlatBuffers binary file
            schema_name: Name of schema to use (without .fbs extension)
            
        Returns:
            Dictionary of imported data, or None if import failed
        """
        if not FLATC_AVAILABLE:
            print("Error: FlatBuffers compiler not available")
            return None
        
        if schema_name not in self.schemas:
            print(f"Error: Schema '{schema_name}' not found")
            return None
        
        try:
            # Run flatc to convert binary to JSON
            schema_path = self.schemas[schema_name]['path']
            result = subprocess.run([
                "flatc",
                "--json",
                "--strict-json",
                "--defaults-json",
                schema_path,
                "--",
                binary_path
            ], capture_output=True, check=True, text=True)
            
            # Parse JSON output
            return json.loads(result.stdout)
        except (subprocess.SubprocessError, json.JSONDecodeError) as e:
            print(f"Error importing FlatBuffers data: {e}")
            return None
    
    def get_enum_values(self, schema_name: str, enum_name: str) -> List[str]:
        """Get values for enum from schema
        
        Args:
            schema_name: Name of schema containing enum
            enum_name: Name of enum
            
        Returns:
            List of enum values
        """
        if schema_name not in self.schemas:
            return []
        
        schema = self.schemas[schema_name]
        if 'enums' not in schema or enum_name not in schema['enums']:
            return []
        
        return schema['enums'][enum_name]
    
    def get_available_schemas(self) -> List[str]:
        """Get list of available schema names"""
        return list(self.schemas.keys())
    
    def get_schema_info(self, schema_name: str) -> Optional[Dict[str, Any]]:
        """Get information about a schema
        
        Args:
            schema_name: Name of schema
            
        Returns:
            Dictionary with schema information, or None if schema not found
        """
        return self.schemas.get(schema_name)
    
    def create_building_data(self, building_data: Dict[str, Any]) -> Dict[str, Any]:
        """Convert building data to FlatBuffers format
        
        Args:
            building_data: Building configuration data
            
        Returns:
            Data formatted for FlatBuffers serialization
        """
        # Example conversion for building data
        # This would need to be customized based on your specific FlatBuffers schema
        fb_data = {
            "name": building_data.get("asset_name", ""),
            "description": building_data.get("description", ""),
            "building_type": building_data.get("type_id", "generic"),
            "size": {
                "x": building_data.get("building_size_x", 1),
                "y": building_data.get("building_size_y", 1)
            },
            "production": {
                "produces": [
                    {"resource": res} for res in building_data.get("produces_resources", [])
                ],
                "consumes": [
                    {"resource": res} for res in building_data.get("consumes_resources", [])
                ],
                "production_rates": [
                    {"resource": res, "rate": rate} 
                    for res, rate in building_data.get("production_rate", {}).items()
                ],
                "consumption_rates": [
                    {"resource": res, "rate": rate} 
                    for res, rate in building_data.get("consumption_rate", {}).items()
                ]
            },
            "power": {
                "consumption": building_data.get("power_consumption", 0),
                "generation": -building_data.get("power_consumption", 0) if building_data.get("power_consumption", 0) < 0 else 0
            },
            "construction": {
                "cost": building_data.get("construction_cost", 100),
                "time": building_data.get("construction_time", 10)
            },
            "maintenance": {
                "cost": building_data.get("maintenance_cost", 1.0)
            }
        }
        
        return fb_data
    
    def create_vehicle_data(self, vehicle_data: Dict[str, Any]) -> Dict[str, Any]:
        """Convert vehicle data to FlatBuffers format
        
        Args:
            vehicle_data: Vehicle configuration data
            
        Returns:
            Data formatted for FlatBuffers serialization
        """
        # Example conversion for vehicle data
        # This would need to be customized based on your specific FlatBuffers schema
        fb_data = {
            "name": vehicle_data.get("asset_name", ""),
            "description": vehicle_data.get("description", ""),
            "vehicle_type": vehicle_data.get("vehicle_type", "Road"),
            "subtype": vehicle_data.get("road_vehicle_type", ""),
            "physics": {
                "mass": vehicle_data.get("mass", 1.0),
                "max_speed": vehicle_data.get("max_speed", 60)
            },
            "capacity": vehicle_data.get("capacity", 0),
            "segment": vehicle_data.get("segment", "Civilian")
        }
        
        return fb_data