use std::fs;
use std::path::Path;
use tempfile::TempDir;

mod common;
use common::verify_directory_permissions;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_permissions_matching() {
        // Skip test on non-Unix platforms
        if cfg!(not(target_family = "unix")) {
            println!("Skipping test on non-Unix platform");
            return;
        }

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;

            // Create a temporary directory
            let temp_dir = TempDir::new().expect("Failed to create temporary directory");
            let temp_dir_path = temp_dir.path();

            // Set specific permissions on the directory (0o750)
            let expected_mode = 0o750;
            fs::set_permissions(temp_dir_path, fs::Permissions::from_mode(expected_mode))
                .expect("Failed to set directory permissions");

            // Verify the permissions match
            let result = verify_directory_permissions(temp_dir_path, expected_mode);
            assert!(
                result.is_ok(),
                "Directory permissions should match: {}",
                result.err().unwrap_or_default()
            );
        }
    }

    #[test]
    fn test_directory_permissions_not_matching() {
        // Skip test on non-Unix platforms
        if cfg!(not(target_family = "unix")) {
            println!("Skipping test on non-Unix platform");
            return;
        }

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;

            // Create a temporary directory
            let temp_dir = TempDir::new().expect("Failed to create temporary directory");
            let temp_dir_path = temp_dir.path();

            // Set specific permissions on the directory (0o750)
            fs::set_permissions(temp_dir_path, fs::Permissions::from_mode(0o750))
                .expect("Failed to set directory permissions");

            // Try to verify with different required permissions (0o755)
            let result = verify_directory_permissions(temp_dir_path, 0o755);
            assert!(
                result.is_err(),
                "Verification should fail when permissions don't match"
            );

            // Verify the error message contains useful information
            let error_message = result.err().unwrap_or_default();
            assert!(
                error_message.contains("incorrect permissions"),
                "Error message should indicate incorrect permissions: {}",
                error_message
            );
            assert!(
                error_message.contains("750"),
                "Error message should show actual permissions: {}",
                error_message
            );
            assert!(
                error_message.contains("755"),
                "Error message should show expected permissions: {}",
                error_message
            );
        }
    }

    #[test]
    fn test_directory_permissions_non_unix() {
        // This test should run on all platforms
        // but will have different behavior based on the platform
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_dir_path = temp_dir.path();

        // On non-Unix systems, it should always succeed
        // On Unix systems, we test the actual behavior
        #[cfg(not(target_family = "unix"))]
        {
            // The function should return Ok regardless of the permission bits on non-Unix
            let result = verify_directory_permissions(temp_dir_path, 0o755);
            assert!(
                result.is_ok(),
                "On non-Unix systems, verify_directory_permissions should always succeed"
            );
        }

        #[cfg(target_family = "unix")]
        {
            // On Unix, we should get accurate verification
            // This is covered by the other tests, so no need to duplicate here
        }
    }

    #[test]
    fn test_directory_permissions_nonexistent_path() {
        // Create a path that doesn't exist
        let nonexistent_path = Path::new("/tmp/nonexistent_directory_for_testing");

        // Ensure the path doesn't exist
        if nonexistent_path.exists() {
            fs::remove_dir_all(nonexistent_path).expect("Failed to remove test directory");
        }

        // Try to verify permissions on a non-existent path
        let result = verify_directory_permissions(nonexistent_path, 0o755);

        // Should fail with an appropriate error message
        assert!(
            result.is_err(),
            "Should fail when checking permissions on non-existent path"
        );

        let error_message = result.err().unwrap_or_default();
        assert!(
            error_message.contains("Failed to get metadata"),
            "Error message should indicate metadata issue: {}",
            error_message
        );
    }

    #[test]
    fn test_directory_permissions_on_file() {
        // Skip test on non-Unix platforms
        if cfg!(not(target_family = "unix")) {
            println!("Skipping test on non-Unix platform");
            return;
        }

        #[cfg(target_family = "unix")]
        {
            // Create a temporary directory
            let temp_dir = TempDir::new().expect("Failed to create temporary directory");
            let temp_dir_path = temp_dir.path();

            // Create a file inside the temporary directory
            let file_path = temp_dir_path.join("test_file.txt");
            fs::write(&file_path, "test content").expect("Failed to create test file");

            // Try to verify permissions on a file (not a directory)
            let result = verify_directory_permissions(&file_path, 0o644);

            // Should fail with an appropriate error message
            assert!(
                result.is_err(),
                "Should fail when checking permissions on a file"
            );

            let error_message = result.err().unwrap_or_default();
            assert!(
                error_message.contains("is not a directory"),
                "Error message should indicate it's not a directory: {}",
                error_message
            );
        }
    }
}
