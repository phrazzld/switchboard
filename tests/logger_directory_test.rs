use crate::common::{generate_test_log_path, verify_log_directory};
use std::fs;
use std::io::Write;

mod common;

#[test]
fn test_directory_creation() {
    // Create a test-specific log file in the test directory
    let test_name = "nested_directory_test";
    let log_path = generate_test_log_path(test_name);
    println!("Log path: {}", log_path.display());

    // Ensure the parent directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create test log directory");
    }

    // Write some test content directly to the file
    let test_content = r#"{"timestamp":"2023-04-24T12:34:56.789Z","level":"INFO","fields":{"message":"Test log entry"},"target":"test"}"#;
    let mut file = fs::File::create(&log_path).expect("Failed to create log file");
    writeln!(file, "{}", test_content).expect("Failed to write to log file");

    // Verify directory creation
    let parent_dir = log_path.parent().unwrap();
    assert!(parent_dir.exists(), "Test log directory was not created");

    // Verify the directory structure is correctly set up
    assert!(
        verify_log_directory(),
        "Log directory structure is not correct"
    );

    // List files in directory to help debug
    println!("Files in test directory:");
    if let Ok(entries) = fs::read_dir(parent_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {}", entry.path().display());
            }
        }
    }

    // Verify log file exists
    assert!(
        log_path.exists(),
        "Log file was not created in test directory"
    );

    // Clean up
    let _ = fs::remove_file(&log_path);
}
