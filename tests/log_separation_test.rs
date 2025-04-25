use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use switchboard::{
    config::Config,
    logger::{LogPathResolver, LogType, APP_LOG_SUBDIR, DEFAULT_LOG_DIR, TEST_LOG_SUBDIR},
};

/// Creates a test config with a specific log filename
fn create_test_config(log_filename: &str) -> Config {
    Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "debug".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: log_filename.to_string(),
        log_file_level: "debug".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
        log_max_age_days: None,
    }
}

/// Resolves a log path for a specific log type
fn resolve_log_path(log_filename: &str, log_type: LogType) -> PathBuf {
    let config = create_test_config(log_filename);
    let resolver = LogPathResolver::new(&config, log_type);
    resolver.resolve().expect("Failed to resolve log path")
}

/// Creates a test log file with specified content
fn create_log_file(path: &Path, content: &str) {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create log directory");
    }

    // Write the content to the file
    let mut file = fs::File::create(path).expect("Failed to create log file");
    writeln!(file, "{}", content).expect("Failed to write to log file");
    file.flush().expect("Failed to flush file");
}

/// Verifies that a log file exists and contains the expected content
fn verify_log_file(path: &Path, expected_content: &str) -> bool {
    if !path.exists() {
        println!("Log file does not exist: {}", path.display());
        return false;
    }

    // Read the file content
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            println!("Failed to read log file {}: {}", path.display(), e);
            return false;
        }
    };

    // Check if it contains the expected content
    if !content.contains(expected_content) {
        println!(
            "Log file does not contain expected content. Expected: '{}', File content: '{}'",
            expected_content, content
        );
        return false;
    }

    true
}

/// Ensures the directories have the correct structure
fn verify_directory_structure() -> bool {
    let base_dir = PathBuf::from(DEFAULT_LOG_DIR);
    let app_dir = base_dir.join(APP_LOG_SUBDIR);
    let test_dir = base_dir.join(TEST_LOG_SUBDIR);

    if !base_dir.exists() || !base_dir.is_dir() {
        println!("Base log directory does not exist: {}", base_dir.display());
        return false;
    }

    if !app_dir.exists() || !app_dir.is_dir() {
        println!("App log directory does not exist: {}", app_dir.display());
        return false;
    }

    if !test_dir.exists() || !test_dir.is_dir() {
        println!("Test log directory does not exist: {}", test_dir.display());
        return false;
    }

    true
}

#[test]
fn test_app_test_log_separation() {
    // Create unique test identifiers to avoid conflicts with other tests
    let test_id = format!("sep_test_{}", std::process::id());
    let app_log_name = format!("{}_app.log", test_id);
    let test_log_name = format!("{}_test.log", test_id);

    // Clean up any existing log files from previous test runs
    let test_base_dir = PathBuf::from(DEFAULT_LOG_DIR);
    let expected_app_dir = test_base_dir.join(APP_LOG_SUBDIR);
    let expected_test_dir = test_base_dir.join(TEST_LOG_SUBDIR);
    let _ = fs::create_dir_all(&expected_app_dir);
    let _ = fs::create_dir_all(&expected_test_dir);

    // Resolve log paths for both types
    let app_log_path = resolve_log_path(&app_log_name, LogType::Application);
    let test_log_path = resolve_log_path(&test_log_name, LogType::Test);

    // Create log files with distinctive content
    let app_log_content = format!("Application log content {}", test_id);
    let test_log_content = format!("Test log content {}", test_id);

    create_log_file(&app_log_path, &app_log_content);
    create_log_file(&test_log_path, &test_log_content);

    // Verify the paths are in the correct subdirectories
    assert!(
        app_log_path.to_string_lossy().contains(APP_LOG_SUBDIR),
        "App log path should be in app subdirectory: {}",
        app_log_path.display()
    );

    assert!(
        test_log_path.to_string_lossy().contains(TEST_LOG_SUBDIR),
        "Test log path should be in test subdirectory: {}",
        test_log_path.display()
    );

    // Verify base directory structure is correct
    assert!(
        verify_directory_structure(),
        "Directory structure is incorrect"
    );

    // Verify log files exist with the expected content
    assert!(
        verify_log_file(&app_log_path, &app_log_content),
        "App log verification failed"
    );

    assert!(
        verify_log_file(&test_log_path, &test_log_content),
        "Test log verification failed"
    );

    // Verify the files are in different directories
    let app_parent = app_log_path.parent().unwrap();
    let test_parent = test_log_path.parent().unwrap();
    assert_ne!(
        app_parent, test_parent,
        "App and test logs should be in different directories"
    );

    // Verify absolute paths
    let app_canonical = fs::canonicalize(&app_log_path).expect("Failed to canonicalize app path");
    let test_canonical =
        fs::canonicalize(&test_log_path).expect("Failed to canonicalize test path");

    println!("App log path: {}", app_canonical.display());
    println!("Test log path: {}", test_canonical.display());

    // Cleanup - remove the test log files
    let _ = fs::remove_file(app_log_path);
    let _ = fs::remove_file(test_log_path);
}
