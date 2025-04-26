#!/bin/bash
# Script to clean up log files after tests
# This script is added to ensure that log files don't remain in the repository
# even if the tests don't clean up after themselves properly

echo "🧹 Cleaning up log files after tests..."

# Remove all log files in the logs directory while preserving .gitkeep files
find ./logs -type f \( -name "*.log" -o -name "*.log.*" \) \
  -not -name ".gitkeep" -delete || true

# Ensure directories exist but are empty except for .gitkeep
mkdir -p ./logs/app
mkdir -p ./logs/test

# Make sure .gitkeep files exist
touch ./logs/app/.gitkeep ./logs/test/.gitkeep

# Verify all logs have been removed
leftover_logs=$(find ./logs -type f -not -name ".gitkeep" 2>/dev/null || true)
if [ -n "$leftover_logs" ]; then
  echo "⚠️ Warning: Some log files could not be removed:"
  echo "$leftover_logs"
else
  echo "✅ Log cleanup successful - no log files remaining."
fi