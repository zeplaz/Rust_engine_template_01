#!/usr/bin/env python3
"""World Builder for Processor Alpha Dine"""

# Fix import path issue
import os
import sys

# Add the current directory to the path so we can import the src package
current_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, current_dir)

from src.main import main

if __name__ == "__main__":
    main()