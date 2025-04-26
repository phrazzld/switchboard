//! Filesystem utilities for directory management and permissions
//!
//! This module provides cross-platform abstractions for common filesystem
//! operations needed by the application's logging and configuration systems.
//! These utilities safely handle platform-specific behaviors while providing
//! a consistent API.
//!
//! ## Functions
//!
//! - [`ensure_directory`] - Creates a directory (and parent directories) with specified permissions
//! - [`check_writable`] - Verifies that a path exists and is writable by the current process
//!
//! ## Platform-specific behavior
//!
//! This module handles platform differences transparently for callers:
//!
//! **Unix systems**:
//! - Permission management using Unix permission bits (0o777, etc.)
//! - UID/GID-based access control
//! - Special handling for root permissions
//!
//! **Windows systems**:
//! - Graceful fallback for permission operations
//! - Runtime writability tests instead of permission bits
//!
//! ## Integration
//!
//! These utilities are primarily used by:
//! - The logging system (`logger.rs`) for log directory management
//! - Configuration validation to ensure specified paths are valid
//!
//! ## Example usage
//!
//! ```no_run
//! use std::path::Path;
//! use switchboard::fs_utils;
//!
//! // Create a directory with specific permissions on Unix (no-op on Windows)
//! let log_dir = Path::new("/var/log/switchboard");
//! fs_utils::ensure_directory(log_dir, Some(0o750))?;
//!
//! // Check if a directory is writable
//! fs_utils::check_writable(log_dir)?;
//! ```
//!
//! ## Error handling
//!
//! All functions return `io::Result<()>` with appropriate error kinds:
//! - `NotFound` for missing paths
//! - `PermissionDenied` for permission issues
//! - `AlreadyExists` when a non-directory exists at the target path
//!
//! ## Logging
//!
//! Functions emit structured logs using the `tracing` crate for observability:
//! - Directory creation events
//! - Permission changes
//!

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

/// Checks if a path exists and is writable by the current process
///
/// This function verifies if the given path exists and the current process
/// has write permissions to it. The implementation is platform-specific:
///
/// # Arguments
///
/// * `path` - The path to check for writeability
///
/// # Returns
///
/// * `io::Result<()>` - Ok if the path is writable, Err otherwise
///
/// # Platform-specific behavior
///
/// On Unix systems, this checks the permission bits of the file/directory.
/// On Windows, this attempts to create a temporary file in the directory.
///
/// # Error cases
///
/// * Returns `io::ErrorKind::NotFound` if the path doesn't exist
/// * Returns `io::ErrorKind::PermissionDenied` if write permission is denied
/// * Returns other IO errors as appropriate for other failure cases
pub fn check_writable(path: &Path) -> io::Result<()> {
    // First check if the path exists
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Path does not exist: {:?}", path),
        ));
    }

    // Platform-specific writability check
    #[cfg(target_family = "unix")]
    {
        // On Unix, check the file permissions
        let metadata = fs::metadata(path)?;
        let mode = metadata.permissions().mode();

        // Get current user and group ids
        use std::os::unix::fs::MetadataExt;
        let file_uid = metadata.uid();
        let file_gid = metadata.gid();

        // Get process user and group ids
        use nix::unistd::{Gid, Uid};
        let process_uid = Uid::current().as_raw();
        let process_gid = Gid::current().as_raw();

        // Check if we're the owner and owner has write permission
        let owner_writable = file_uid == process_uid && (mode & 0o200) != 0;
        // Check if we're in the group and group has write permission
        let group_writable = file_gid == process_gid && (mode & 0o020) != 0;
        // Check if others have write permission
        let other_writable = (mode & 0o002) != 0;

        // Special case for root user (can write regardless of permissions)
        let is_root = process_uid == 0;

        if is_root || owner_writable || group_writable || other_writable {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Write permission denied on path: {:?}", path),
            ))
        }
    }

    #[cfg(not(target_family = "unix"))]
    {
        // On non-Unix platforms (e.g., Windows), try to create a temp file
        if path.is_dir() {
            // For directories, try to create a temporary file inside
            use rand::Rng;
            use std::fs::File;
            use std::io::Write;

            let mut rng = rand::thread_rng();
            let random_suffix: u32 = rng.gen();
            let temp_filename = format!(".tmp_write_test_{}.tmp", random_suffix);
            let test_path = path.join(temp_filename);

            let result = (|| {
                let mut file = File::create(&test_path)?;
                file.write_all(b"test")?;
                file.sync_all()?;
                Ok(())
            })();

            // Clean up regardless of success
            let _ = fs::remove_file(&test_path);

            return result;
        } else {
            // For files, try to open in write mode
            use std::fs::OpenOptions;

            match OpenOptions::new().write(true).open(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }
    }
}
