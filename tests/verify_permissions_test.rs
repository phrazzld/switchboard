// Tests for the verify_directory_permissions helper function in common/mod.rs

use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Import common module from tests
mod common;
use common::verify_directory_permissions;

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_verify_directory_permissions() {
    // Skip this test on non-Unix platforms
    #[cfg(not(target_family = "unix"))]
    {
        println!("Skipping test_verify_directory_permissions on non-Unix platform");
        return;
    }

    // Run the actual test on Unix platforms
    #[cfg(target_family = "unix")]
    {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();

        // Test case 1: Permissions match (0o750)
        let expected_mode = 0o750;
        set_dir_permissions(temp_path, expected_mode).expect("Failed to set directory permissions");

        let result = verify_directory_permissions(temp_path, expected_mode);
        assert!(
            result.is_ok(),
            "Permissions should match: {}",
            result.err().unwrap_or_default()
        );

        // Test case 2: Permissions don't match (0o755 vs 0o750)
        let wrong_mode = 0o755;
        set_dir_permissions(temp_path, wrong_mode).expect("Failed to set directory permissions");

        let result = verify_directory_permissions(temp_path, expected_mode);
        assert!(result.is_err(), "Should detect mismatched permissions");
        if let Err(err) = result {
            assert!(
                err.contains("incorrect permissions"),
                "Error message should mention incorrect permissions: {}",
                err
            );
        }
    }
}

#[cfg(target_family = "unix")]
fn set_dir_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(path, perms)
}
