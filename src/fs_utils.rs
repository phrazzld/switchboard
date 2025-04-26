//! Filesystem utilities for directory management and permissions
//!
//! This module provides cross-platform abstractions for common filesystem
//! operations needed by the application, including:
//!
//! - Creating directories with proper permissions
//! - Checking if directories are writable
//!
//! Functions handle platform-specific behavior (Unix vs Windows) where relevant,
//! particularly for permission management.

use std::fs;
use std::io;
use std::path::Path;
use tracing::info;

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

/// Ensures a directory exists with the specified permissions
///
/// This function creates a directory and all of its parent directories
/// if they don't exist already. On Unix systems, it also sets the specified
/// permissions (mode) if provided.
///
/// # Arguments
///
/// * `path` - The path to the directory to create
/// * `mode` - Optional Unix permission mode (e.g., 0o750)
///
/// # Returns
///
/// * `io::Result<()>` - Ok if the directory exists or was created successfully, Err otherwise
///
/// # Platform-specific behavior
///
/// On Unix systems, the permissions are set to the provided mode if specified.
/// On Windows, the mode parameter is ignored.
///
/// # Logging
///
/// Emits an INFO log when a directory is created, including the path.
pub fn ensure_directory(path: &Path, mode: Option<u32>) -> io::Result<()> {
    // Check if the directory already exists
    if path.exists() {
        if path.is_dir() {
            // Directory already exists, check if we need to set permissions on Unix
            #[cfg(target_family = "unix")]
            if let Some(mode_value) = mode {
                // Only change permissions if they're different to avoid unnecessary operations
                let metadata = fs::metadata(path)?;
                let current_mode = metadata.permissions().mode() & 0o777; // Get only permission bits

                if current_mode != mode_value {
                    let mut perms = metadata.permissions();
                    perms.set_mode(mode_value);
                    fs::set_permissions(path, perms)?;

                    info!(
                        event = "update_permissions",
                        path = ?path,
                        old_mode = format!("0o{:o}", current_mode),
                        new_mode = format!("0o{:o}", mode_value),
                        "Updated directory permissions"
                    );
                }
            }

            // Directory exists with correct permissions
            return Ok(());
        } else {
            // Path exists but is not a directory
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Path exists but is not a directory: {:?}", path),
            ));
        }
    }

    // Create the directory and its parents
    fs::create_dir_all(path)?;

    // Log the creation
    info!(
        event = "create_dir",
        path = ?path,
        "Created directory"
    );

    // Set permissions on Unix systems if requested
    #[cfg(target_family = "unix")]
    if let Some(mode_value) = mode {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(mode_value);
        fs::set_permissions(path, perms)?;

        info!(
            event = "set_permissions",
            path = ?path,
            mode = format!("0o{:o}", mode_value),
            "Set directory permissions"
        );
    }

    Ok(())
}

// Function to be implemented in a later task:
// - check_writable
