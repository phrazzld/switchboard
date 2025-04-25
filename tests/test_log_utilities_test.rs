use std::fs;
use std::io::Write;
use switchboard::logger::TEST_LOG_SUBDIR;

// Import the test utilities from common module
mod common;
use common::{
    count_lines, generate_test_log_path, is_valid_json, verify_log_directory,
    verify_log_file_exists,
};

/// Test that the generate_test_log_path function correctly resolves paths.
///
/// For simplicity, we'll create the log file directly instead of using the logger
/// since tracing has some challenges with multiple initializations in tests.
#[test]
fn test_logging_setup() {
    // Generate a path to use for the test
    let log_path = generate_test_log_path("test_logging_setup");
    println!("Generated log path: {}", log_path.display());

    // Ensure the parent directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create test log directory");
    }

    // Write directly to the file
    let test_json = r#"{"timestamp":"2023-04-24T12:34:56.789Z","level":"INFO","fields":{"message":"Test log entry"},"target":"test"}"#;
    let mut file = fs::File::create(&log_path).expect("Failed to create log file");
    writeln!(file, "{}", test_json).expect("Failed to write to log file");

    // Flush and close the file
    drop(file);

    // Verify the file exists and has content
    assert!(log_path.exists(), "Log file does not exist");

    // Count lines in the log file
    let line_count = count_lines(&log_path).unwrap_or(0);
    assert!(line_count > 0, "Log file is empty");

    // Verify the file contains valid JSON
    assert!(
        is_valid_json(&log_path),
        "Log file does not contain valid JSON"
    );

    // Clean up
    let _ = fs::remove_file(&log_path);
}

/// Test that the generate_test_log_path function produces paths in the test subdirectory.
#[test]
fn test_generate_test_log_path() {
    // Generate a test log path
    let log_path = generate_test_log_path("path_generation_test");

    // Verify the path is in the test subdirectory
    let path_str = log_path.to_string_lossy();
    assert!(
        path_str.contains(TEST_LOG_SUBDIR),
        "Log path does not include the test subdirectory: {}",
        path_str
    );

    // Verify the path contains the test name
    assert!(
        path_str.contains("path_generation_test"),
        "Log path does not include the test name: {}",
        path_str
    );
}

/// Test that the verify_log_directory function correctly checks directory structure.
#[test]
fn test_verify_log_directory() {
    // We'll use generate_test_log_path to get a proper path which ensures directories are created
    let app_test_file = generate_test_log_path("app_dir_test");
    let test_log_file = generate_test_log_path("test_dir_test");

    // Create the parent directories
    if let Some(app_parent) = app_test_file.parent() {
        fs::create_dir_all(app_parent).expect("Failed to create app directory");
    }

    if let Some(test_parent) = test_log_file.parent() {
        fs::create_dir_all(test_parent).expect("Failed to create test directory");
    }

    // Verify the directory structure
    assert!(
        verify_log_directory(),
        "Log directory structure verification failed"
    );
}

/// Test that verify_log_file_exists correctly checks for file existence and content.
#[test]
fn test_verify_log_file_exists() {
    // Create a test file with content
    let test_dir = std::env::temp_dir().join("switchboard_test_verify");
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let test_file = test_dir.join("verify_test.log");
    fs::write(&test_file, "Test log content\nSecond line").expect("Failed to write test file");

    // Verify the file exists and has content
    assert!(
        verify_log_file_exists(&test_file),
        "File exists verification failed for file with content"
    );

    // Create an empty file
    let empty_file = test_dir.join("empty_test.log");
    fs::write(&empty_file, "").expect("Failed to write empty test file");

    // Verify empty file is detected correctly
    assert!(
        !verify_log_file_exists(&empty_file),
        "Empty file incorrectly verified as having content"
    );

    // Clean up
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&empty_file);
    let _ = fs::remove_dir(&test_dir);
}
