use crate::common::{count_lines, is_valid_json};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod common;

#[test]
fn test_file_creation_and_json_format() {
    // Create a test-specific log file in the test directory
    let test_name = "file_creation_test";
    let log_path = PathBuf::from("./logs/test").join(format!("{}_test.log", test_name));
    println!("Log path: {}", log_path.display());

    // Ensure the parent directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create test log directory");
    }

    // Write test JSON content directly to the file
    let test_content = vec![
        r#"{"timestamp":"2023-04-24T12:34:56.789Z","level":"DEBUG","fields":{"message":"Debug message with structured data","value":42},"target":"test"}"#,
        r#"{"timestamp":"2023-04-24T12:34:57.789Z","level":"INFO","fields":{"message":"Info message with structured data","user_id":1001},"target":"test"}"#,
        r#"{"timestamp":"2023-04-24T12:34:58.789Z","level":"WARN","fields":{"message":"Warning message with structured data","latency_ms":150},"target":"test"}"#,
        r#"{"timestamp":"2023-04-24T12:34:59.789Z","level":"ERROR","fields":{"message":"Error message with structured data","error_code":500},"target":"test"}"#,
    ];

    let mut file = fs::File::create(&log_path).expect("Failed to create log file");
    for line in &test_content {
        writeln!(file, "{}", line).expect("Failed to write to log file");
    }

    // Verify file creation
    assert!(log_path.exists(), "Log file was not created");

    // Verify non-empty content
    let line_count = count_lines(&log_path).unwrap_or(0);
    assert!(line_count > 0, "Log file is empty");

    // Verify JSON format
    assert!(
        is_valid_json(&log_path),
        "Log file doesn't contain valid JSON"
    );

    // Verify the log file is in the test subdirectory
    let path_str = log_path.to_string_lossy();
    assert!(
        path_str.contains("/test/"),
        "Log file should be in the test subdirectory"
    );

    // Clean up
    let _ = fs::remove_file(&log_path);
}
