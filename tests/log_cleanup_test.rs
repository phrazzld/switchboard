//! Tests for the log cleanup functionality

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};
use switchboard::config::{Config, LogDirectoryMode};
use switchboard::log_cleanup::cleanup_logs;
use switchboard::logger::{APP_LOG_SUBDIR, DEFAULT_LOG_DIR, TEST_LOG_SUBDIR};

// Utility function to create a log file with a specific age
fn create_test_log_file(path: &Path, content: &str, age_days: u64) -> std::io::Result<()> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create and write to the file
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    // Set the modification time
    if age_days > 0 {
        let now = SystemTime::now();
        let mtime = now - Duration::from_secs(age_days * 24 * 60 * 60);
        filetime::set_file_mtime(path, filetime::FileTime::from_system_time(mtime))?;
    }

    Ok(())
}

#[test]
fn test_log_cleanup_with_max_age() {
    // Create test directory structure
    let app_dir = Path::new(DEFAULT_LOG_DIR).join(APP_LOG_SUBDIR);
    let test_dir = Path::new(DEFAULT_LOG_DIR).join(TEST_LOG_SUBDIR);

    // Create directories if they don't exist
    fs::create_dir_all(&app_dir).expect("Failed to create app log directory");
    fs::create_dir_all(&test_dir).expect("Failed to create test log directory");

    // Create test log files with different ages
    let files = [
        // App logs
        (app_dir.join("recent.log"), "recent app log", 1), // Should not be deleted
        (app_dir.join("old.log"), "old app log", 10),      // Should be deleted with max_age=7
        (
            app_dir.join("rotated.log.2023-01-01"),
            "rotated app log",
            20,
        ), // Should be deleted
        // Test logs
        (test_dir.join("recent_test.log"), "recent test log", 2), // Should not be deleted
        (test_dir.join("old_test.log"), "old test log", 15),      // Should be deleted
    ];

    // Create all test files
    for (path, content, age) in &files {
        create_test_log_file(path, content, *age).expect("Failed to create test file");
    }

    // Create a non-log file that should not be deleted
    let non_log_file = app_dir.join("data.txt");
    create_test_log_file(&non_log_file, "not a log file", 30)
        .expect("Failed to create non-log file");

    // Create a test config with max_age of 7 days
    let config = Config {
        openai_api_key: Some("test-openai-api-key".to_string()),
        openai_api_base_url: "https://api.openai.com".to_string(),
        openai_enabled: false,
        port: "0".to_string(),
        anthropic_api_key: "test-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "info".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: "./switchboard.log".to_string(),
        log_file_level: "debug".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: LogDirectoryMode::Default,
        log_max_age_days: Some(7),
    };

    // Run the cleanup
    let result = cleanup_logs(&config);

    // Check the results
    assert_eq!(result.files_removed, 3); // Should remove 3 old files
    assert!(result.bytes_removed > 0);
    assert!(result.failed_files.is_empty());

    // Verify the expected files still exist
    assert!(files[0].0.exists()); // recent.log still exists
    assert!(!files[1].0.exists()); // old.log deleted
    assert!(!files[2].0.exists()); // rotated.log.2023-01-01 deleted
    assert!(files[3].0.exists()); // recent_test.log still exists
    assert!(!files[4].0.exists()); // old_test.log deleted
    assert!(non_log_file.exists()); // data.txt still exists

    // Clean up test directories
    for (path, _, _) in &files {
        if path.exists() {
            fs::remove_file(path).ok();
        }
    }
    if non_log_file.exists() {
        fs::remove_file(non_log_file).ok();
    }
}

#[test]
fn test_log_cleanup_with_disabled_max_age() {
    // Create test directory structure
    let app_dir = Path::new(DEFAULT_LOG_DIR).join(APP_LOG_SUBDIR);
    let test_dir = Path::new(DEFAULT_LOG_DIR).join(TEST_LOG_SUBDIR);

    // Create directories if they don't exist
    fs::create_dir_all(&app_dir).expect("Failed to create app log directory");
    fs::create_dir_all(&test_dir).expect("Failed to create test log directory");

    // Create a test log file that would be deleted if cleanup was enabled
    let old_log = app_dir.join("should_not_delete.log");
    create_test_log_file(&old_log, "old log", 100).expect("Failed to create test file");

    // Create a test config with max_age of None (disabled)
    let config = Config {
        openai_api_key: Some("test-openai-api-key".to_string()),
        openai_api_base_url: "https://api.openai.com".to_string(),
        openai_enabled: false,
        port: "0".to_string(),
        anthropic_api_key: "test-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "info".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: "./switchboard.log".to_string(),
        log_file_level: "debug".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: LogDirectoryMode::Default,
        log_max_age_days: None,
    };

    // Run the cleanup
    let result = cleanup_logs(&config);

    // Check the results
    assert_eq!(result.files_removed, 0); // No files should be removed
    assert_eq!(result.bytes_removed, 0);
    assert!(result.failed_files.is_empty());

    // Verify the file still exists
    assert!(old_log.exists());

    // Clean up test file
    if old_log.exists() {
        fs::remove_file(old_log).ok();
    }
}

#[test]
fn test_log_cleanup_with_zero_max_age() {
    // Create test directory structure
    let app_dir = Path::new(DEFAULT_LOG_DIR).join(APP_LOG_SUBDIR);
    let test_dir = Path::new(DEFAULT_LOG_DIR).join(TEST_LOG_SUBDIR);

    // Create directories if they don't exist
    fs::create_dir_all(&app_dir).expect("Failed to create app log directory");
    fs::create_dir_all(&test_dir).expect("Failed to create test log directory");

    // Create a test log file that would be deleted if cleanup was enabled
    let old_log = app_dir.join("should_not_delete_zero.log");
    create_test_log_file(&old_log, "old log", 100).expect("Failed to create test file");

    // Create a test config with max_age of 0 (disabled)
    let config = Config {
        openai_api_key: Some("test-openai-api-key".to_string()),
        openai_api_base_url: "https://api.openai.com".to_string(),
        openai_enabled: false,
        port: "0".to_string(),
        anthropic_api_key: "test-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "info".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: "./switchboard.log".to_string(),
        log_file_level: "debug".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: LogDirectoryMode::Default,
        log_max_age_days: Some(0),
    };

    // Run the cleanup
    let result = cleanup_logs(&config);

    // Check the results
    assert_eq!(result.files_removed, 0); // No files should be removed
    assert_eq!(result.bytes_removed, 0);
    assert!(result.failed_files.is_empty());

    // Verify the file still exists
    assert!(old_log.exists());

    // Clean up test file
    if old_log.exists() {
        fs::remove_file(old_log).ok();
    }
}
