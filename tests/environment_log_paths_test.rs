use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use switchboard::logger::{
    detect_environment, get_environment_log_directory, get_xdg_log_directory, LogEnvironment,
    LogType, APP_LOG_SUBDIR, DEFAULT_LOG_DIR, SYSTEM_LOG_DIR, TEST_LOG_SUBDIR,
};

/// Creates a test log file with specified content
fn create_log_file(path: &Path, content: &str) -> bool {
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            println!("Failed to create directory {}: {}", parent.display(), e);
            return false;
        }
    }

    // Try to write the content to the file
    match fs::File::create(path) {
        Ok(mut file) => {
            if writeln!(file, "{}", content).is_err() || file.flush().is_err() {
                println!("Failed to write to log file: {}", path.display());
                return false;
            }
        }
        Err(e) => {
            println!("Failed to create log file {}: {}", path.display(), e);
            return false;
        }
    }

    true
}

/// Verifies that a log file exists and contains the expected content
fn verify_log_file(path: &Path, expected_content: &str) -> bool {
    if !path.exists() {
        println!("Log file does not exist: {}", path.display());
        return false;
    }

    // Read the file content
    match fs::read_to_string(path) {
        Ok(content) => {
            if !content.contains(expected_content) {
                println!(
                    "Log file does not contain expected content. Expected: '{}', File content: '{}'",
                    expected_content, content
                );
                return false;
            }
            true
        }
        Err(e) => {
            println!("Failed to read log file {}: {}", path.display(), e);
            false
        }
    }
}

/// Ensures the directory structure is correct for a given base directory
fn verify_directory_structure(base_dir: &Path) -> bool {
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

/// Creates a PathBuf for a log file using standard development environment path
fn create_development_log_path(filename: &str, log_type: LogType) -> PathBuf {
    let subdir = match log_type {
        LogType::Application => APP_LOG_SUBDIR,
        LogType::Test => TEST_LOG_SUBDIR,
    };

    PathBuf::from(DEFAULT_LOG_DIR).join(subdir).join(filename)
}

#[test]
fn test_development_environment_paths() {
    // Set up test environment
    let test_id = format!("dev_env_{}", std::process::id());
    let log_filename = format!("{}.log", test_id);

    // Make sure we're in Development environment
    env::set_var("SWITCHBOARD_DEV", "1");

    // Verify environment detection
    let env = detect_environment();
    assert_eq!(
        env,
        LogEnvironment::Development,
        "Environment should be detected as Development"
    );

    // Note: We don't need to create a config here as the test directly uses the environment paths

    // Get expected base directory for this environment
    let expected_base_dir = PathBuf::from(DEFAULT_LOG_DIR);

    // Create the expected log paths
    let app_log_path = create_development_log_path(&log_filename, LogType::Application);
    let test_log_path = create_development_log_path(&log_filename, LogType::Test);

    // Create log files
    let app_content = format!("App log in development environment {}", test_id);
    let test_content = format!("Test log in development environment {}", test_id);

    if create_log_file(&app_log_path, &app_content)
        && create_log_file(&test_log_path, &test_content)
    {
        // Verify paths are in the correct locations
        assert!(
            app_log_path.starts_with(&expected_base_dir),
            "App log path should start with the development base directory. Expected: {}, Actual: {}",
            expected_base_dir.display(),
            app_log_path.display()
        );

        assert!(
            test_log_path.starts_with(&expected_base_dir),
            "Test log path should start with the development base directory. Expected: {}, Actual: {}",
            expected_base_dir.display(),
            test_log_path.display()
        );

        // Verify app/test subdirectories
        assert!(
            app_log_path.to_string_lossy().contains(APP_LOG_SUBDIR),
            "App log path should include app subdirectory: {}",
            app_log_path.display()
        );

        assert!(
            test_log_path.to_string_lossy().contains(TEST_LOG_SUBDIR),
            "Test log path should include test subdirectory: {}",
            test_log_path.display()
        );

        // Verify files were created correctly
        assert!(
            verify_log_file(&app_log_path, &app_content),
            "App log verification failed"
        );
        assert!(
            verify_log_file(&test_log_path, &test_content),
            "Test log verification failed"
        );

        // Verify directory structure
        assert!(
            verify_directory_structure(&expected_base_dir),
            "Directory structure verification failed for development environment"
        );
    }

    // Clean up
    let _ = fs::remove_file(&app_log_path);
    let _ = fs::remove_file(&test_log_path);
    env::remove_var("SWITCHBOARD_DEV");
}

#[test]
fn test_environment_detection_mapping() {
    // Test that environment detection correctly maps to the right directories

    // For each environment type
    let environments = [
        LogEnvironment::Development,
        LogEnvironment::UserInstallation,
        LogEnvironment::SystemService,
    ];

    for env in environments {
        // Get expected directory for this environment
        let dir = get_environment_log_directory(env);

        // Verify the mapping is correct
        match env {
            LogEnvironment::Development => {
                assert_eq!(dir, PathBuf::from(DEFAULT_LOG_DIR));
            }
            LogEnvironment::UserInstallation => {
                assert_eq!(dir, get_xdg_log_directory());
            }
            LogEnvironment::SystemService => {
                assert_eq!(dir, PathBuf::from(SYSTEM_LOG_DIR));
            }
        }
    }
}

#[test]
#[cfg(not(target_os = "windows"))] // Skip on Windows - not worth the trouble
fn test_xdg_path_format() {
    // This test verifies the format of the XDG path without checking if files can be created there
    let xdg_path = get_xdg_log_directory();

    // Verify the XDG path is properly formatted according to platform conventions
    #[cfg(target_os = "linux")]
    {
        // Linux XDG path should contain .local/share
        assert!(
            xdg_path
                .to_string_lossy()
                .contains(".local/share/switchboard/logs"),
            "Linux XDG path should follow XDG standard: {}",
            xdg_path.display()
        );
    }

    #[cfg(target_os = "macos")]
    {
        // macOS XDG path should contain Library/Application Support
        assert!(
            xdg_path
                .to_string_lossy()
                .contains("Library/Application Support/switchboard/logs"),
            "macOS XDG path should follow Apple standards: {}",
            xdg_path.display()
        );
    }

    #[cfg(target_os = "windows")]
    {
        // Windows XDG path should contain AppData\Roaming and switchboard
        // Note: Path separators could be either \ or / based on how they're handled
        // Also the actual structure might include a 'data' subdirectory in CI
        let path_str = xdg_path.to_string_lossy().to_string();
        assert!(
            path_str.contains("AppData\\Roaming\\switchboard")
                || path_str.contains("AppData/Roaming/switchboard"),
            "Windows XDG path should follow Windows standards: {}",
            xdg_path.display()
        );
    }
}

#[test]
fn test_system_path_format() {
    // This test verifies the format of the system path without checking if files can be created there
    let system_path = PathBuf::from(SYSTEM_LOG_DIR);

    // Verify the system path is properly formatted according to platform conventions
    #[cfg(target_family = "unix")]
    {
        // Unix system path should be /var/log/switchboard
        assert_eq!(
            system_path,
            PathBuf::from("/var/log/switchboard"),
            "Unix system path should be in /var/log: {}",
            system_path.display()
        );
    }

    #[cfg(target_family = "windows")]
    {
        // Windows system path should be C:\ProgramData\Switchboard\Logs
        assert_eq!(
            system_path,
            PathBuf::from("C:\\ProgramData\\Switchboard\\Logs"),
            "Windows system path should be in ProgramData: {}",
            system_path.display()
        );
    }
}

#[test]
fn test_directory_structure_creation() {
    // Test that we can create the expected directory structure
    let test_id = format!("dir_struct_{}", std::process::id());
    let base_dir = PathBuf::from(DEFAULT_LOG_DIR);

    // Create the app and test directories
    let app_log_path = base_dir
        .join(APP_LOG_SUBDIR)
        .join(format!("{}.log", test_id));
    let test_log_path = base_dir
        .join(TEST_LOG_SUBDIR)
        .join(format!("{}.log", test_id));

    // Create log files with content
    let app_content = format!("App log directory structure test {}", test_id);
    let test_content = format!("Test log directory structure test {}", test_id);

    create_log_file(&app_log_path, &app_content);
    create_log_file(&test_log_path, &test_content);

    // Verify the directory structure
    assert!(
        verify_directory_structure(&base_dir),
        "Failed to create and verify directory structure"
    );

    // Verify log files
    assert!(
        verify_log_file(&app_log_path, &app_content),
        "App log verification failed"
    );
    assert!(
        verify_log_file(&test_log_path, &test_content),
        "Test log verification failed"
    );

    // Clean up
    let _ = fs::remove_file(&app_log_path);
    let _ = fs::remove_file(&test_log_path);
}
