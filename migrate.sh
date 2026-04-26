#!/bin/bash

# Migration script for Processor Alpha Dine codebase reorganization

echo "Starting migration process..."

# Create backup of original src directory
echo "Creating backup of original src directory..."
cp -r src src_backup_$(date +%Y%m%d)

# Ensure the src_reorganized directory exists
if [ ! -d "src_reorganized" ]; then
  echo "src_reorganized directory not found! Please run the reorganization script first."
  exit 1
fi

# Move the reorganized files to replace the original src directory
echo "Moving reorganized files to src directory..."
mv src src_old
mv src_reorganized src

# Update Cargo.toml to use the new structure
echo "Updating Cargo.toml..."
sed -i 's/path = "src\/lib.rs"/path = "src\/lib.rs"/' Cargo.toml

echo "Migration complete!"
echo "The original source code is preserved in src_old"
echo "A backup of the original source is also available in src_backup_*"
echo
echo "Next steps:"
echo "1. Build the project to verify everything works"
echo "2. Fix any import issues"
echo "3. Remove the backup directories once everything is confirmed working"

# Recommendations for manual tasks
echo
echo "Manual tasks required:"
echo "- Update imports in all files"
echo "- Check for any path references that need updating"
echo "- Review the README.md in the src directory for more information"