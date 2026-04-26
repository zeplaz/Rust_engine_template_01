import os
import sys

# Add the current directory to the path so we can import the src package
current_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, current_dir)

# Import the main window module
try:
    from src.main_window import main
except ImportError:
    # Fall back to main_entity_editor if main_window doesn't exist
    from src import main_entity_editor
    main = main_entity_editor.main

if __name__ == "__main__":
    main()