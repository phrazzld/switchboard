use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::Duration;
use switchboard::config::Config;
use switchboard::logger;
use tracing::{debug, error, info, trace, warn};

// Helper function to check if a file exists and is readable
fn file_exists_and_readable(path: &Path) -> bool {
    match File::open(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

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

// Check if a specific level is in a log file
fn check_log_level_present(path: &Path, level: &str) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        content.contains(&format!("\"level\":\"{}\"", level))
    } else {
        false
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
fn test_log_level_filtering() {
    // Test with debug level
    let test_dir = setup_test_dir();
    let debug_base_path = test_dir.join("debug_level_test.log");
    println!("Debug log base path: {}", debug_base_path.display());

    // Clean up any previous test artifacts
    if let Some(old_log) = find_log_file(&debug_base_path) {
        let _ = fs::remove_file(old_log);
    }

    // Set up debug level logging
    let config = create_test_config(&debug_base_path, "debug");
    let _guard = logger::init_tracing(&config);

    // Write logs at all levels
    trace!("TRACE message");
    debug!("DEBUG message");
    info!("INFO message");
    warn!("WARN message");
    error!("ERROR message");

    // Allow time for logs to be written
    std::thread::sleep(Duration::from_millis(1000));

    // Find the actual log file
    let actual_debug_path = find_log_file(&debug_base_path);

    // Verify debug level filtering (debug and above should appear)
    assert!(
        actual_debug_path.is_some(),
        "Debug log file was not created"
    );
    let debug_path = actual_debug_path.unwrap();

    assert!(
        check_log_level_present(&debug_path, "DEBUG"),
        "DEBUG level missing in debug log"
    );
    assert!(
        check_log_level_present(&debug_path, "INFO"),
        "INFO level missing in debug log"
    );
    assert!(
        check_log_level_present(&debug_path, "WARN"),
        "WARN level missing in debug log"
    );
    assert!(
        check_log_level_present(&debug_path, "ERROR"),
        "ERROR level missing in debug log"
    );

    // Clean up
    let _ = fs::remove_file(debug_path);
}
