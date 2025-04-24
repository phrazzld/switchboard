use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;
use switchboard::config::Config;
use switchboard::logger;
use tracing::{debug, error, info, warn};

// Helper function to count lines in a file
fn count_lines(path: &Path) -> io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let count = reader.lines().count();
    println!("Line count: {} -> {}", path.display(), count);
    Ok(count)
}

// Helper function to check if file content is valid JSON
fn is_valid_json(path: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(_) => {}
                Err(_) => return false,
            }
        }
        true
    } else {
        false
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
fn test_file_creation_and_json_format() {
    // Create a test-specific log path
    let test_dir = setup_test_dir();
    let log_base_path = test_dir.join("file_creation_test.log");
    println!("Log base path: {}", log_base_path.display());

    // Create config and initialize logging
    let config = create_test_config(&log_base_path, "debug");
    let _guard = logger::init_tracing(&config).expect("Failed to initialize logging for file test");

    // Write test logs with structured fields
    debug!(value = 42, "Debug message with structured data");
    info!(user_id = 1001, "Info message with structured data");
    warn!(latency_ms = 150, "Warning message with structured data");
    error!(error_code = 500, "Error message with structured data");

    // Allow time for logs to be written (non-blocking I/O)
    std::thread::sleep(Duration::from_millis(1000));

    // Find the actual log file
    let actual_log_path = find_log_file(&log_base_path);

    // List files in directory to help debug
    if let Ok(entries) = fs::read_dir(&test_dir) {
        println!("Files in test directory:");
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {}", entry.path().display());
            }
        }
    }

    // Verify file creation
    assert!(actual_log_path.is_some(), "Log file was not created");
    let log_path = actual_log_path.unwrap();

    // Verify non-empty content
    let line_count = count_lines(&log_path).unwrap_or(0);
    assert!(line_count > 0, "Log file is empty");

    // Verify JSON format
    assert!(
        is_valid_json(&log_path),
        "Log file doesn't contain valid JSON"
    );

    // Clean up
    let _ = fs::remove_file(&log_path);
}
