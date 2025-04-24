use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use switchboard::config::Config;
use switchboard::logger;
use tracing::info;

// Create a test-specific temp directory that won't be auto-cleaned
fn setup_test_dir() -> PathBuf {
    let temp_dir = env::temp_dir().join("switchboard_tests");
    fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
    temp_dir
}

// Create a test config with specific settings
fn create_test_config(log_path: &Path, log_level: &str) -> Config {
    Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "info".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: log_path.to_string_lossy().to_string(),
        log_file_level: log_level.to_string(),
        log_max_body_size: 1024,
    }
}

// Find the actual log file, accounting for the date suffix
fn find_log_file(base_path: &Path) -> Option<PathBuf> {
    // Check for the base path first (unlikely)
    if base_path.exists() {
        return Some(base_path.to_path_buf());
    }

    // Check for the base path with today's date suffix
    let date_suffix = chrono::Local::now().format(".%Y-%m-%d").to_string();
    let dated_path = PathBuf::from(format!("{}{}", base_path.display(), date_suffix));

    if dated_path.exists() {
        return Some(dated_path);
    }

    // If not found, check the directory for files with similar names
    if let Some(parent) = base_path.parent() {
        if let Ok(entries) = fs::read_dir(parent) {
            let base_name = base_path.file_name().unwrap().to_string_lossy();
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with(base_name.as_ref()) {
                        return Some(entry.path());
                    }
                }
            }
        }
    }

    None
}

#[test]
fn test_directory_creation() {
    // Create a nested path that doesn't exist yet
    let test_dir = setup_test_dir();
    let nested_dir = test_dir.join("nested/directory/for/logs");
    let log_base_path = nested_dir.join("nested_test.log");
    println!("Log base path: {}", log_base_path.display());

    // Remove if it exists from a previous test
    if nested_dir.exists() {
        let _ = fs::remove_dir_all(&nested_dir);
    }

    // Create config and initialize logging
    let config = create_test_config(&log_base_path, "debug");
    let _guard =
        logger::init_tracing(&config).expect("Failed to initialize logging for directory test");

    // Write a test log
    info!("Test message for directory creation test");

    // Allow time for directories and file to be created
    std::thread::sleep(Duration::from_millis(1000));

    // Verify directory creation
    assert!(nested_dir.exists(), "Nested directory was not created");

    // Find the actual log file (with date suffix)
    let actual_log_path = find_log_file(&log_base_path);

    // List files in directory to help debug
    println!("Files in nested directory:");
    if let Ok(entries) = fs::read_dir(&nested_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {}", entry.path().display());
            }
        }
    }

    // Verify log file exists
    assert!(
        actual_log_path.is_some(),
        "Log file was not created in nested directory"
    );

    // Clean up
    if let Some(log_path) = actual_log_path {
        let _ = fs::remove_file(log_path);
    }
}
