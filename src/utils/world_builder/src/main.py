"""
World Builder Tool for Processor Alpha Dine
Main application entry point
"""

import os
import sys
import json
import numpy as np
from PyQt5.QtWidgets import QApplication, QMainWindow
from PyQt5.QtCore import Qt

# Try to import PyQt-Fluent-Widgets
try:
    from qfluentwidgets import (NavigationInterface, NavigationItemPosition,
                                FluentIcon, SplitFluentWindow, Theme, setTheme, 
                                FluentTranslator)
    HAS_FLUENT = True
except ImportError:
    HAS_FLUENT = False
    print("PyQt-Fluent-Widgets not found. Falling back to standard PyQt widgets.")

# Import our modules - fix import paths
from src.ui.main_window import WorldBuilderWindow
from src.config.settings import Settings, load_settings, save_settings

def main():
    """Main application entry point"""
    # Create QApplication
    app = QApplication(sys.argv)
    
    # Apply Fluent style if available
    if HAS_FLUENT:
        # Set theme
        setTheme(Theme.DARK)
        
        # Set translator
        translator = FluentTranslator()
        app.installTranslator(translator)
    
    # Load settings
    settings_file = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'config', 'settings.json')
    settings = load_settings(settings_file)
    
    # Create and show main window
    window = WorldBuilderWindow(settings)
    window.show()
    
    # Run application
    sys.exit(app.exec_())

if __name__ == "__main__":
    main()