#!/bin/bash

# Asset Tool Installation Script

echo "Installing Asset Tool dependencies..."

# Determine Python command
if command -v python3 &>/dev/null; then
    PYTHON_CMD=python3
elif command -v python &>/dev/null; then
    PYTHON_CMD=python
else
    echo "ERROR: Python not found. Please install Python 3.6+ to continue."
    exit 1
fi

# Check Python version
PYTHON_VERSION=$($PYTHON_CMD -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')
PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d. -f1)
PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d. -f2)

if [ "$PYTHON_MAJOR" -lt 3 ] || ([ "$PYTHON_MAJOR" -eq 3 ] && [ "$PYTHON_MINOR" -lt 6 ]); then
    echo "ERROR: Python 3.6+ required, found $PYTHON_VERSION. Please update Python to continue."
    exit 1
fi

echo "Using Python $PYTHON_VERSION"

# Install pip if not available
if ! $PYTHON_CMD -m pip --version &>/dev/null; then
    echo "Installing pip..."
    $PYTHON_CMD -m ensurepip --upgrade
fi

# Install required packages
echo "Installing required packages..."
$PYTHON_CMD -m pip install -r requirements.txt

# Check if PyQt-Fluent-Widgets was installed successfully
if $PYTHON_CMD -c "import qfluentwidgets" &>/dev/null; then
    echo "PyQt-Fluent-Widgets installed successfully!"
else
    echo "Warning: PyQt-Fluent-Widgets not installed. The tool will use standard PyQt widgets instead."
    echo "For a better experience, try manually installing PyQt-Fluent-Widgets:"
    echo "  $PYTHON_CMD -m pip install PyQt-Fluent-Widgets"
fi

echo "Installation complete! Run the tool with: $PYTHON_CMD run.py"