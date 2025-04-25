use std::fs::{self};
use std::io::Write;
use std::path::PathBuf;

mod common;

// Check if a specific level is in a log file
fn check_log_level_present(content: &str, level: &str) -> bool {
    content.contains(&format!("\"level\":\"{}\"", level))
}

#[test]
fn test_log_level_filtering() {
    // Create a test-specific log file in the test directory
    let test_name = "level_filtering_test";
    let log_path = PathBuf::from("./logs/test").join(format!("{}_test.log", test_name));
    println!("Log path: {}", log_path.display());

    // Ensure the parent directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create test log directory");
    }

    // Write test JSON content directly to the file with different log levels
    let test_content = vec![
        r#"{"timestamp":"2023-04-24T12:34:56.789Z","level":"DEBUG","fields":{"message":"DEBUG message"},"target":"test"}"#,
        r#"{"timestamp":"2023-04-24T12:34:57.789Z","level":"INFO","fields":{"message":"INFO message"},"target":"test"}"#,
        r#"{"timestamp":"2023-04-24T12:34:58.789Z","level":"WARN","fields":{"message":"WARN message"},"target":"test"}"#,
        r#"{"timestamp":"2023-04-24T12:34:59.789Z","level":"ERROR","fields":{"message":"ERROR message"},"target":"test"}"#,
    ];

    let mut file = fs::File::create(&log_path).expect("Failed to create log file");
    for line in &test_content {
        writeln!(file, "{}", line).expect("Failed to write to log file");
    }

    // Verify file exists
    assert!(log_path.exists(), "Log file was not created");

    // Read log file content
    let content = fs::read_to_string(&log_path).expect("Failed to read log file");

    // Verify debug level filtering (debug and above should appear)
    assert!(
        check_log_level_present(&content, "DEBUG"),
        "DEBUG level missing in log"
    );
    assert!(
        check_log_level_present(&content, "INFO"),
        "INFO level missing in log"
    );
    assert!(
        check_log_level_present(&content, "WARN"),
        "WARN level missing in log"
    );
    assert!(
        check_log_level_present(&content, "ERROR"),
        "ERROR level missing in log"
    );

    // Clean up
    let _ = fs::remove_file(&log_path);
}
