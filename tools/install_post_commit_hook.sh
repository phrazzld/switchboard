#!/bin/sh
# Script to install the post-commit hook for glance

set -e  # Exit immediately if any command fails

# Determine the project root directory
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null)
if [ -z "$PROJECT_ROOT" ]; then
    echo "Error: Not in a git repository"
    exit 1
fi

# Paths
TEMPLATE_PATH="$PROJECT_ROOT/templates/post-commit.template"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"
POST_COMMIT_PATH="$GIT_HOOKS_DIR/post-commit"

# Make sure the template file exists
if [ ! -f "$TEMPLATE_PATH" ]; then
    echo "Error: Template file not found: $TEMPLATE_PATH"
    exit 1
fi

# Check if glance is installed
if ! command -v glance > /dev/null 2>&1; then
    echo "Warning: 'glance' command not found."
    echo "The hook will be installed but won't execute until glance is installed."
    echo "You can install glance following the instructions at: https://github.com/vforgione/glance.md"
    read -p "Continue with installation? [Y/n] " response
    response=${response:-Y}  # Default to Y if empty
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Installation aborted"
        exit 0
    fi
fi

# Check if hook already exists and is different
if [ -f "$POST_COMMIT_PATH" ]; then
    if ! cmp -s "$TEMPLATE_PATH" "$POST_COMMIT_PATH"; then
        echo "A different post-commit hook already exists at: $POST_COMMIT_PATH"
        read -p "Do you want to overwrite it? [y/N] " response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            echo "Installation aborted"
            exit 0
        fi
    else
        echo "Post-commit hook is already installed and up to date."
    fi
fi

# Copy the template to the hooks directory
cp "$TEMPLATE_PATH" "$POST_COMMIT_PATH"
chmod +x "$POST_COMMIT_PATH"

echo "Post-commit hook successfully installed at: $POST_COMMIT_PATH"
echo "The hook will run 'glance ./' asynchronously after each commit."
echo "Logs will be written to: $GIT_HOOKS_DIR/logs/glance-post-commit.log"

exit 0