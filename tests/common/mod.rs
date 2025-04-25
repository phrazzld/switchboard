// Common test utilities for integration tests

use axum::Router;
use reqwest::Client;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use switchboard::config::Config;
use switchboard::logger::{self, LogPathResolver, LogType};
use switchboard::proxy_handler::create_router;
use tracing_appender::non_blocking::WorkerGuard;
use wiremock::MockServer;

/// Represents the setup needed for integration tests.
pub struct TestSetup {
    /// HTTP client for the application to use
    /// Note: This field is required during setup even though tests may not access it directly,
    /// as it's captured by the router and used internally during request processing.
    #[allow(dead_code)]
    pub client: Client,

    /// Application configuration pointing to the mock server
    /// Note: This field is retained for potential future tests that might need to verify config values,
    /// though it's not directly accessed in current tests.
    #[allow(dead_code)]
    pub config: Config,

    /// The WireMock server instance
    pub mock_server: MockServer,

    /// The Axum router configured for testing
    pub app: Router,
}

/// Represents the setup needed for logging in tests.
pub struct TestLoggingSetup {
    /// Worker guard that must be kept alive for the duration of the test
    pub _guard: WorkerGuard,

    /// Path to the log file that was created
    pub log_path: PathBuf,

    /// Configuration used for logging
    pub config: Config,
}

/// Sets up the test environment with all necessary components for integration testing.
///
/// This function:
/// 1. Starts a WireMock server to mock the Anthropic API
/// 2. Creates a test-specific configuration pointing to the mock server
/// 3. Creates a reqwest Client with appropriate timeouts
/// 4. Instantiates the application router using the test client and config
///
/// Returns a TestSetup instance containing all components needed for testing.
pub async fn setup_test_environment() -> TestSetup {
    // Start a WireMock server on a random available port
    // This will be used to mock the Anthropic API during tests
    let mock_server = MockServer::start().await;

    // Create a test-specific configuration pointing to the mock server
    // Use dummy values for fields that are suitable for testing
    let config = Config {
        port: "0".to_string(), // Use 0 to let OS assign a random port if needed
        anthropic_api_key: "test-api-key".to_string(), // Dummy API key for testing
        anthropic_target_url: mock_server.uri(), // Point to the mock server
        log_stdout_level: "debug".to_string(), // Use debug level for more verbose test logs
        log_format: "pretty".to_string(), // Use pretty format for readability in tests
        log_bodies: true,      // Enable body logging for verbose testing
        log_file_path: "./test-switchboard.log".to_string(), // Test-specific log file
        log_file_level: "trace".to_string(), // Most verbose for file logs in tests
        log_max_body_size: 20480, // Default size for tests
        log_directory_mode: switchboard::config::LogDirectoryMode::Default, // Use automatic detection for tests
    };

    // Create a reqwest client with appropriate timeouts for testing
    // Using shorter timeouts than production to avoid long-running tests
    let client = Client::builder()
        .timeout(Duration::from_secs(5)) // Overall request timeout
        .connect_timeout(Duration::from_secs(2)) // Connection establishment timeout
        .pool_idle_timeout(Duration::from_secs(10)) // Keep idle connections for reuse
        .build()
        .expect("Failed to build test reqwest client");

    // Create a thread-safe reference-counted config
    // This avoids the memory leak from Box::leak while still allowing sharing
    let config_arc = Arc::new(config.clone());

    // Create the application router with our test client and config
    let app = create_router(client.clone(), config_arc);

    // Return the complete TestSetup with all components
    TestSetup {
        client,
        config,
        mock_server,
        app,
    }
}

/// Sets up logging for tests using the Test log type.
///
/// This function:
/// 1. Creates a unique log file path for the test to avoid conflicts
/// 2. Creates a test configuration with appropriate settings
/// 3. Initializes the logger with the TestLog type to ensure logs go to the test subdirectory
/// 4. Returns a TestLoggingSetup struct with the guard, path, and config
///
/// # Arguments
/// * `test_name` - Name of the test, used to create a unique log file name
///
/// # Returns
/// A TestLoggingSetup instance containing the worker guard, log path, and config
pub fn setup_test_logging(test_name: &str) -> TestLoggingSetup {
    // Create a unique log file name based on the test name
    let log_file_name = format!("{}_test.log", test_name);

    // Create a test-specific configuration
    let config = Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "debug".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: log_file_name.clone(),
        log_file_level: "trace".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
    };

    // Use LogPathResolver to get the correct path for test logs
    let resolver = LogPathResolver::new(&config, LogType::Test);
    let log_path = resolver.resolve().expect("Failed to resolve test log path");

    // Ensure the parent directory exists
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create test log directory");
    }

    // Initialize logging with our test configuration
    let guard = logger::init_tracing(&config).expect("Failed to initialize test logging");

    TestLoggingSetup {
        _guard: guard,
        log_path,
        config,
    }
}

/// Generates a test log file path for a specific test.
///
/// This function creates a unique log file path in the test logs directory,
/// ensuring that it won't conflict with other test logs.
///
/// # Arguments
/// * `test_name` - Name of the test, used to create a unique file name
///
/// # Returns
/// A PathBuf containing the resolved log file path
pub fn generate_test_log_path(test_name: &str) -> PathBuf {
    // Create a unique log file name based on the test name
    let log_file_name = format!("{}_test.log", test_name);

    // Create a dummy config for the resolver
    let config = Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "debug".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: log_file_name,
        log_file_level: "trace".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
    };

    // Use LogPathResolver to get the correct path for test logs
    let resolver = LogPathResolver::new(&config, LogType::Test);
    resolver.resolve().expect("Failed to resolve test log path")
}

/// Verifies that a log file exists and has content.
///
/// This function checks if the provided log file path exists and contains
/// at least one line, indicating that logging worked correctly.
///
/// # Arguments
/// * `log_path` - Path to the log file to verify
///
/// # Returns
/// true if the log file exists and has content, false otherwise
pub fn verify_log_file_exists(log_path: &Path) -> bool {
    if !log_path.exists() {
        return false;
    }

    match count_lines(log_path) {
        Ok(count) => count > 0,
        Err(_) => false,
    }
}

/// Verifies that the log directory structure is correctly set up.
///
/// This function checks if the log directory structure matches expectations using LogPathResolver:
/// - Base log directory exists
/// - Test subdirectory exists
/// - App subdirectory exists
///
/// # Returns
/// true if the directory structure is correct, false otherwise
pub fn verify_log_directory() -> bool {
    // Create a dummy config with LogDirectoryMode::Default
    let config = Config {
        port: "0".to_string(),
        anthropic_api_key: "test-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "info".to_string(),
        log_format: "pretty".to_string(),
        log_bodies: true,
        log_file_path: "test.log".to_string(),
        log_file_level: "debug".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
    };

    // Create resolvers for both app and test logs
    let app_resolver = LogPathResolver::new(&config, LogType::Application);
    let test_resolver = LogPathResolver::new(&config, LogType::Test);

    // Resolve paths to get the directories
    let app_path = match app_resolver.resolve() {
        Ok(path) => path,
        Err(_) => return false,
    };

    let test_path = match test_resolver.resolve() {
        Ok(path) => path,
        Err(_) => return false,
    };

    // Check if the parent directories exist
    let app_dir = match app_path.parent() {
        Some(dir) => dir,
        None => return false,
    };

    let test_dir = match test_path.parent() {
        Some(dir) => dir,
        None => return false,
    };

    // Verify both directories exist
    app_dir.exists() && test_dir.exists()
}

/// Helper function to find a log file, accounting for date suffixes.
///
/// Log files are often created with date suffixes for rotation purposes.
/// This function helps find the actual log file when given a base path.
///
/// # Arguments
/// * `base_path` - The base path of the log file (without date suffix)
///
/// # Returns
/// An Option containing the PathBuf of the found log file, or None if not found
pub fn find_log_file(base_path: &Path) -> Option<PathBuf> {
    // Check for the base path first
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

/// Helper function to count lines in a file.
///
/// # Arguments
/// * `path` - Path to the file to count lines in
///
/// # Returns
/// An io::Result containing the number of lines in the file
pub fn count_lines(path: &Path) -> io::Result<usize> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

/// Helper function to check if file content is valid JSON.
///
/// # Arguments
/// * `path` - Path to the file to check
///
/// # Returns
/// true if the file contains valid JSON (one object per line), false otherwise
pub fn is_valid_json(path: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if serde_json::from_str::<serde_json::Value>(line).is_err() {
                return false;
            }
        }
        true
    } else {
        false
    }
}
