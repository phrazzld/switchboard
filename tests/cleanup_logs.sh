#!/bin/bash
# Script to clean up log files after tests
# This script is added to ensure that log files don't remain in the repository
# even if the tests don't clean up after themselves properly

echo "Cleaning up log files after tests..."

# Remove all log files in the logs directory
find ./logs -type f -name "*.log" -o -name "*.log.*" -delete

# Ensure directories exist but are empty
mkdir -p ./logs/app
mkdir -p ./logs/test

echo "Log cleanup complete."