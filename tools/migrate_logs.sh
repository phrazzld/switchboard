#!/bin/bash
# Log migration utility for Switchboard project
# Moves existing log files to the appropriate subdirectories based on naming patterns

set -e

# Ensure we're in the project root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}/.."

# Define color codes for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Create directories if they don't exist
mkdir -p ./logs/app
mkdir -p ./logs/test

echo -e "${GREEN}Starting log file migration...${NC}"

# Count files for summary
total_files=0
moved_app_files=0
moved_test_files=0
skipped_files=0

# Function to move a file to a target directory
move_file() {
    local file=$1
    local target_dir=$2
    
    # Create target directory if it doesn't exist
    mkdir -p "$target_dir"
    
    # Check if file exists in target directory already
    if [ -f "${target_dir}/$(basename "$file")" ]; then
        echo -e "${YELLOW}Skipping $file - already exists in target directory${NC}"
        ((skipped_files++))
        return
    fi
    
    # Move the file
    mv "$file" "$target_dir"
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Moved $file to $target_dir${NC}"
        return 0
    else
        echo -e "${RED}Failed to move $file to $target_dir${NC}"
        return 1
    fi
}

# Find log files in the root directory
for file in *.log*; do
    # Skip if no files match pattern
    [ -e "$file" ] || continue
    
    ((total_files++))
    
    # Determine target directory based on naming pattern
    if [[ "$file" == *"_test.log"* || "$file" == "test_"* || "$file" == "benchmark.log"* ]]; then
        # Test-related log files
        move_file "$file" "./logs/test" && ((moved_test_files++))
    else
        # Application log files
        move_file "$file" "./logs/app" && ((moved_app_files++))
    fi
done

# Look for legacy logs in logs/ that should be in subdirectories
for file in logs/*.log*; do
    # Skip if no files match pattern
    [ -e "$file" ] || continue
    
    ((total_files++))
    
    # Determine target directory based on naming pattern
    if [[ "$file" == *"_test.log"* || "$file" == "*/test_"* || "$file" == "*/benchmark.log"* ]]; then
        # Test-related log files
        move_file "$file" "./logs/test" && ((moved_test_files++))
    else
        # Application log files
        move_file "$file" "./logs/app" && ((moved_app_files++))
    fi
done

echo -e "${GREEN}Log migration complete!${NC}"
echo -e "Total files processed: $total_files"
echo -e "Moved to app directory: $moved_app_files"
echo -e "Moved to test directory: $moved_test_files"
echo -e "Skipped files: $skipped_files"

# Check if there are still any log files in the root directory
remaining=$(find . -maxdepth 1 -name "*.log*" | wc -l)
if [ "$remaining" -gt 0 ]; then
    echo -e "${YELLOW}Warning: $remaining log files still remain in the root directory${NC}"
    echo -e "These may require manual attention or might be actively in use."
fi

# Make the tool directory if it doesn't exist already
mkdir -p ./tools