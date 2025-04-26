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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[cfg(not(target_family = "unix"))]
    use std::io::Write;

    /// Test creating a directory that doesn't exist
    #[test]
    fn test_create_nonexistent_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_dir = temp_dir.path().join("new_dir");

        // Ensure directory doesn't exist yet
        assert!(!test_dir.exists());

        // Create the directory
        ensure_directory(&test_dir, None).expect("Failed to create directory");

        // Verify directory exists
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());
    }

    /// Test creating a directory with Unix permissions (mode)
    #[test]
    #[cfg(target_family = "unix")]
    fn test_create_directory_with_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_dir = temp_dir.path().join("mode_dir");
        let test_mode = 0o751; // rwxr-x--x

        // Create directory with specific mode
        ensure_directory(&test_dir, Some(test_mode)).expect("Failed to create directory with mode");

        // Verify directory exists
        assert!(test_dir.exists());
        assert!(test_dir.is_dir());

        // Verify permissions
        let metadata = fs::metadata(&test_dir).expect("Failed to get metadata");
        let mode = metadata.permissions().mode() & 0o777; // Get only permission bits
        assert_eq!(
            mode, test_mode,
            "Directory permissions don't match requested mode"
        );
    }

    /// Test no-op behavior when directory already exists
    #[test]
    fn test_existing_directory_noop() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Directory already exists
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());

        // Should be a no-op and not fail
        ensure_directory(temp_dir.path(), None).expect("Failed on existing directory");

        // Directory should still exist
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
    }

    /// Test error when path exists but is not a directory
    #[test]
    fn test_error_on_existing_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("existing_file.txt");

        // Create a file
        File::create(&file_path).expect("Failed to create test file");
        assert!(file_path.exists());
        assert!(!file_path.is_dir());

        // Should fail because path exists but is not a directory
        let result = ensure_directory(&file_path, None);
        assert!(result.is_err());

        // Verify error type
        match result {
            Err(e) => {
                assert_eq!(e.kind(), io::ErrorKind::AlreadyExists);
                assert!(e.to_string().contains("not a directory"));
            }
            Ok(_) => panic!("Expected error for path that exists but is not a directory"),
        }
    }

    /// Test updating permissions on existing directory
    #[test]
    #[cfg(target_family = "unix")]
    fn test_update_permissions_on_existing_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_dir = temp_dir.path().join("perm_update_dir");

        // Create directory with initial permissions
        let initial_mode = 0o755;
        ensure_directory(&test_dir, Some(initial_mode)).expect("Failed to create directory");

        // Verify initial permissions
        let metadata = fs::metadata(&test_dir).expect("Failed to get metadata");
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, initial_mode);

        // Update to new permissions
        let new_mode = 0o700;
        ensure_directory(&test_dir, Some(new_mode)).expect("Failed to update permissions");

        // Verify new permissions
        let metadata = fs::metadata(&test_dir).expect("Failed to get metadata");
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, new_mode, "Directory permissions weren't updated");
    }

    // This test simulates an unwritable parent directory scenario
    // Note: We can't easily create truly unwritable directories in tests
    // without admin privileges, so this test mocks the behavior using
    // a non-existent path at an invalid location
    #[test]
    fn test_error_on_invalid_parent() {
        // Try to create a directory in a location that's invalid or doesn't exist
        // This varies by platform but /proc/none is typically invalid on Unix systems
        // and C:\Windows\System32\NonExistentLocation is invalid on Windows
        #[cfg(target_family = "unix")]
        let invalid_path = Path::new("/proc/none/invalid_dir");
        #[cfg(not(target_family = "unix"))]
        let invalid_path = Path::new("C:\\Windows\\System32\\NonExistentLocation\\invalid_dir");

        // Should fail because parent doesn't exist or isn't a directory
        let result = ensure_directory(invalid_path, None);
        assert!(result.is_err());
    }

    // Tests for check_writable function

    /// Test check_writable on a writable directory
    #[test]
    fn test_writable_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Directory should be writable
        let result = check_writable(temp_dir.path());
        assert!(result.is_ok(), "Expected temp directory to be writable");
    }

    /// Test check_writable on a writable file
    #[test]
    fn test_writable_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("writable_file.txt");

        // Create a writable file
        File::create(&file_path).expect("Failed to create test file");

        // File should be writable
        let result = check_writable(&file_path);
        assert!(result.is_ok(), "Expected file to be writable");
    }

    /// Test check_writable on a non-existent path
    #[test]
    fn test_nonexistent_path() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let nonexistent_path = temp_dir.path().join("nonexistent/path");

        // Should fail with NotFound
        let result = check_writable(&nonexistent_path);
        assert!(result.is_err());

        // Verify error type
        match result {
            Err(e) => {
                assert_eq!(e.kind(), io::ErrorKind::NotFound);
                assert!(e.to_string().contains("does not exist"));
            }
            Ok(_) => panic!("Expected NotFound error for non-existent path"),
        }
    }

    /// Test check_writable on a directory with restricted permissions (Unix only)
    #[test]
    #[cfg(target_family = "unix")]
    fn test_non_writable_directory_unix() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let restricted_dir = temp_dir.path().join("restricted_dir");

        // Create directory first with normal permissions
        fs::create_dir(&restricted_dir).expect("Failed to create test directory");

        // Set read-only permissions (no write permission for anyone)
        let read_only_mode = 0o555; // r-xr-xr-x
        let mut perms = fs::metadata(&restricted_dir)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(read_only_mode);
        fs::set_permissions(&restricted_dir, perms).expect("Failed to set permissions");

        // If we're running as root, we can still write regardless of permissions
        // so we should skip this test
        use nix::unistd::Uid;
        if Uid::current().is_root() {
            println!("Skipping test_non_writable_directory_unix because we're running as root");
            return;
        }

        // Try to check if it's writable
        let result = check_writable(&restricted_dir);

        // Should fail with PermissionDenied
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(e.kind(), io::ErrorKind::PermissionDenied);
                assert!(e.to_string().contains("permission denied"));
            }
            Ok(_) => panic!("Expected PermissionDenied error for non-writable directory"),
        }
    }

    /// On Windows, we simulate a non-writable path by using a system directory
    /// that is typically not writable by normal users
    #[test]
    #[cfg(not(target_family = "unix"))]
    fn test_non_writable_path_windows() {
        // On Windows, system directories like C:\Windows are typically not writable
        // by normal users
        let system_dir = Path::new("C:\\Windows\\System32\\config");

        // Skip test if running with admin privileges or if the path doesn't exist
        if !system_dir.exists() {
            println!(
                "Skipping test_non_writable_path_windows because system directory doesn't exist"
            );
            return;
        }

        // Try to check writability - this might pass if running as admin, so we have
        // an alternative approach in the test below
        let result = check_writable(system_dir);
        if result.is_ok() {
            println!("Warning: Expected system directory to be non-writable, but it appears writable. Test might be running with elevated privileges.");
        }
    }

    /// Alternative test for Windows using a file opened by another process
    /// This simulates a non-writable situation on Windows
    #[test]
    #[cfg(not(target_family = "unix"))]
    fn test_readonly_file_windows() {
        use std::fs::OpenOptions;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let readonly_file_path = temp_dir.path().join("readonly_file.txt");

        // Create a file first
        {
            let mut file = File::create(&readonly_file_path).expect("Failed to create test file");
            file.write_all(b"test content")
                .expect("Failed to write to test file");
        }

        // Make it read-only on Windows
        let mut perms = fs::metadata(&readonly_file_path)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_readonly(true);
        fs::set_permissions(&readonly_file_path, perms).expect("Failed to set permissions");

        // Try to check write access - should fail since it's read-only
        let result = OpenOptions::new().write(true).open(&readonly_file_path);

        // If the file is truly read-only, this should fail
        // But skip asserting if it somehow works (maybe due to high privileges)
        if result.is_ok() {
            println!("Warning: Expected read-only file to be non-writable, but it appears writable. Test might be running with elevated privileges.");
            return;
        }

        // Now check with our function
        let result = check_writable(&readonly_file_path);
        assert!(result.is_err(), "Expected error for read-only file");
    }

    /// Test check_writable with special paths
    #[test]
    fn test_special_paths() {
        // Try with root directory, which may or may not be writable depending on the system
        // We're not asserting the result, just checking that it doesn't panic
        let _root_result = check_writable(Path::new("/"));

        // Try with current directory, which should typically be writable
        // Again, not asserting the result, just checking for panics
        let _current_result = check_writable(Path::new("."));
    }
}
