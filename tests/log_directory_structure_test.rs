use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use switchboard::config::Config;
use switchboard::logger::{
    LogPathResolver, LogType, APP_LOG_SUBDIR, DEFAULT_LOG_DIR, TEST_LOG_SUBDIR,
};

mod common;
use common::{cleanup_test_log_file, verify_log_directory};

// Structure to hold paths for different log types
struct LogPaths {
    app_log_path: PathBuf,
    test_log_path: PathBuf,
}

/// Helper function to generate log paths for different log types
fn generate_log_paths(base_name: &str) -> LogPaths {
    // Create config with the provided base name
    let config = Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "debug".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: format!("{}.log", base_name),
        log_file_level: "trace".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
        log_max_age_days: None,
    };

    // Create resolvers for both app and test logs
    let app_resolver = LogPathResolver::new(&config, LogType::Application);
    let test_resolver = LogPathResolver::new(&config, LogType::Test);

    // Resolve paths
    let app_log_path = app_resolver
        .resolve()
        .expect("Failed to resolve app log path");
    let test_log_path = test_resolver
        .resolve()
        .expect("Failed to resolve test log path");

    LogPaths {
        app_log_path,
        test_log_path,
    }
}

/// Helper function to create a log file and write a test entry
fn create_test_log_file(path: &Path) {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create log directory");
    }

    // Write test content to the file
    let test_content = r#"{"timestamp":"2023-04-24T12:34:56.789Z","level":"INFO","fields":{"message":"Test log entry"},"target":"test"}"#;
    let mut file = fs::File::create(path).expect("Failed to create log file");
    writeln!(file, "{}", test_content).expect("Failed to write to log file");
}

#[test]
fn test_log_directory_structure() {
    // Verify the basic directory structure is correct
    assert!(
        verify_log_directory(),
        "Basic log directory structure is not correct"
    );

    // Generate paths for different log types
    let paths = generate_log_paths("directory_structure_test");

    // Create test log files
    create_test_log_file(&paths.app_log_path);
    create_test_log_file(&paths.test_log_path);

    // Verify log files were created in the correct locations
    assert!(
        paths.app_log_path.exists(),
        "App log file was not created: {}",
        paths.app_log_path.display()
    );
    assert!(
        paths.test_log_path.exists(),
        "Test log file was not created: {}",
        paths.test_log_path.display()
    );

    // Verify the paths have the correct structure - app log should be in the app subdirectory
    let app_dir = paths.app_log_path.parent().unwrap();
    let app_dir_name = app_dir.file_name().unwrap().to_string_lossy();
    assert_eq!(
        app_dir_name,
        APP_LOG_SUBDIR,
        "App log is not in the correct directory: {}",
        app_dir.display()
    );

    // Test log should be in the test subdirectory
    let test_dir = paths.test_log_path.parent().unwrap();
    let test_dir_name = test_dir.file_name().unwrap().to_string_lossy();
    assert_eq!(
        test_dir_name,
        TEST_LOG_SUBDIR,
        "Test log is not in the correct directory: {}",
        test_dir.display()
    );

    // Verify directory permissions (Unix-specific)
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;

        // Check app directory permissions
        let app_dir_perms = fs::metadata(app_dir)
            .expect("Failed to get app directory metadata")
            .permissions()
            .mode();

        // Check test directory permissions
        let test_dir_perms = fs::metadata(test_dir)
            .expect("Failed to get test directory metadata")
            .permissions()
            .mode();

        // Check that the mode matches 0o750 (rwxr-x---)
        // We use bitwise AND with 0o777 to mask out non-permission bits
        assert_eq!(
            app_dir_perms & 0o777,
            0o750,
            "App directory permissions are incorrect: {:o}",
            app_dir_perms & 0o777
        );
        assert_eq!(
            test_dir_perms & 0o777,
            0o750,
            "Test directory permissions are incorrect: {:o}",
            test_dir_perms & 0o777
        );
    }

    // Clean up created files
    cleanup_test_log_file(&paths.app_log_path);
    cleanup_test_log_file(&paths.test_log_path);
}

#[test]
fn test_log_path_resolution() {
    // Generate paths for different log types
    let paths = generate_log_paths("path_resolution_test");

    // Verify that the paths have the expected structure
    let app_path_str = paths.app_log_path.to_string_lossy();
    let test_path_str = paths.test_log_path.to_string_lossy();

    // Both paths should contain the default log directory
    assert!(
        app_path_str.contains(DEFAULT_LOG_DIR),
        "App log path does not contain default log directory: {}",
        app_path_str
    );
    assert!(
        test_path_str.contains(DEFAULT_LOG_DIR),
        "Test log path does not contain default log directory: {}",
        test_path_str
    );

    // App path should contain the app subdirectory
    assert!(
        app_path_str.contains(APP_LOG_SUBDIR),
        "App log path does not contain app subdirectory: {}",
        app_path_str
    );

    // Test path should contain the test subdirectory
    assert!(
        test_path_str.contains(TEST_LOG_SUBDIR),
        "Test log path does not contain test subdirectory: {}",
        test_path_str
    );

    // Check that the base filename is preserved
    assert!(
        app_path_str.ends_with("path_resolution_test.log"),
        "App log path does not end with correct filename: {}",
        app_path_str
    );
    assert!(
        test_path_str.ends_with("path_resolution_test.log"),
        "Test log path does not end with correct filename: {}",
        test_path_str
    );
}

#[test]
fn test_all_log_types() {
    // Instead of using the global logger, we'll test the path resolution directly
    // Test application logs path
    let app_config = Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "debug".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: "app_type_test.log".to_string(),
        log_file_level: "trace".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
        log_max_age_days: None,
    };

    // Get app log path
    let app_resolver = LogPathResolver::new(&app_config, LogType::Application);
    let app_log_path = app_resolver
        .resolve()
        .expect("Failed to resolve app log path");
    create_test_log_file(&app_log_path);

    // Test log path
    let test_config = Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "debug".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: "test_type_test.log".to_string(),
        log_file_level: "trace".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
        log_max_age_days: None,
    };

    // Get test log path
    let test_resolver = LogPathResolver::new(&test_config, LogType::Test);
    let test_log_path = test_resolver
        .resolve()
        .expect("Failed to resolve test log path");
    create_test_log_file(&test_log_path);

    // Verify log files were created in the correct locations
    assert!(
        app_log_path.exists(),
        "App log file was not created: {}",
        app_log_path.display()
    );
    assert!(
        test_log_path.exists(),
        "Test log file was not created: {}",
        test_log_path.display()
    );

    // Verify the log files are in the correct directories
    let app_dir = app_log_path.parent().unwrap();
    let test_dir = test_log_path.parent().unwrap();

    assert!(
        app_dir.to_string_lossy().contains(APP_LOG_SUBDIR),
        "App log is not in the app subdirectory: {}",
        app_dir.display()
    );
    assert!(
        test_dir.to_string_lossy().contains(TEST_LOG_SUBDIR),
        "Test log is not in the test subdirectory: {}",
        test_dir.display()
    );

    // Clean up created files
    cleanup_test_log_file(&app_log_path);
    cleanup_test_log_file(&test_log_path);
}
