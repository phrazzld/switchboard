use secrecy::SecretString;
use std::sync::Arc;
use std::time::Duration;
use switchboard::config::Config;
use switchboard::logger;
use tempfile::tempdir;

/// Benchmark logging modes
#[derive(Debug, Clone, Copy)]
pub enum LoggingMode {
    /// No logging (completely disabled)
    Disabled,
    /// Stdout logging only
    StdoutOnly,
    /// File logging only
    FileOnly,
    /// Both stdout and file logging (dual-output)
    DualOutput,
}

/// Initialize a specific logging configuration for benchmarking
pub fn setup_logging(
    mode: LoggingMode,
) -> (
    Option<Arc<Config>>,
    Option<tracing_appender::non_blocking::WorkerGuard>,
) {
    // Create a temp directory for log files
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let log_file_path = temp_dir
        .path()
        .join("benchmark.log")
        .to_string_lossy()
        .to_string();

    match mode {
        LoggingMode::Disabled => {
            // For disabled mode, just return None values without setting up a subscriber
            // This avoids the SetGlobalDefaultError when running in CI
            (None, None)
        }
        LoggingMode::StdoutOnly => {
            // Create config with stdout only (set file level to OFF)
            let config = Arc::new(Config {
                port: "0".to_string(), // Not used in benchmarks
                anthropic_api_key: SecretString::new("test-key".to_string().into()), // Not used in benchmarks
                anthropic_target_url: "https://example.com".to_string(), // Not used in benchmarks
                openai_api_key: None,                                    // Not used in benchmarks
                openai_api_base_url: "https://api.openai.com".to_string(), // Not used in benchmarks
                openai_enabled: false,                                   // Not used in benchmarks
                log_stdout_level: "debug".to_string(),
                log_format: "json".to_string(), // JSON format for consistency
                log_bodies: true,
                log_file_path,
                log_file_level: "off".to_string(), // Disable file logging
                log_max_body_size: 20480,
                log_directory_mode: switchboard::config::LogDirectoryMode::Default,
                log_max_age_days: None,
            });

            match logger::init_tracing(&config) {
                Ok(guard) => (Some(config), Some(guard)),
                Err(e) => {
                    panic!("Failed to initialize logging for benchmarks: {}", e);
                }
            }
        }
        LoggingMode::FileOnly => {
            // Create config with file only (set stdout level to OFF)
            let config = Arc::new(Config {
                port: "0".to_string(), // Not used in benchmarks
                anthropic_api_key: SecretString::new("test-key".to_string().into()), // Not used in benchmarks
                anthropic_target_url: "https://example.com".to_string(), // Not used in benchmarks
                openai_api_key: None,                                    // Not used in benchmarks
                openai_api_base_url: "https://api.openai.com".to_string(), // Not used in benchmarks
                openai_enabled: false,                                   // Not used in benchmarks
                log_stdout_level: "off".to_string(),                     // Disable stdout logging
                log_format: "json".to_string(), // Not relevant when stdout disabled
                log_bodies: true,
                log_file_path,
                log_file_level: "debug".to_string(),
                log_max_body_size: 20480,
                log_directory_mode: switchboard::config::LogDirectoryMode::Default,
                log_max_age_days: None,
            });

            match logger::init_tracing(&config) {
                Ok(guard) => (Some(config), Some(guard)),
                Err(e) => {
                    panic!("Failed to initialize logging for benchmarks: {}", e);
                }
            }
        }
        LoggingMode::DualOutput => {
            // Create config with both outputs enabled
            let config = Arc::new(Config {
                port: "0".to_string(), // Not used in benchmarks
                anthropic_api_key: SecretString::new("test-key".to_string().into()), // Not used in benchmarks
                anthropic_target_url: "https://example.com".to_string(), // Not used in benchmarks
                openai_api_key: None,                                    // Not used in benchmarks
                openai_api_base_url: "https://api.openai.com".to_string(), // Not used in benchmarks
                openai_enabled: false,                                   // Not used in benchmarks
                log_stdout_level: "debug".to_string(),
                log_format: "json".to_string(), // JSON format for consistency
                log_bodies: true,
                log_file_path,
                log_file_level: "debug".to_string(),
                log_max_body_size: 20480,
                log_directory_mode: switchboard::config::LogDirectoryMode::Default,
                log_max_age_days: None,
            });

            match logger::init_tracing(&config) {
                Ok(guard) => (Some(config), Some(guard)),
                Err(e) => {
                    panic!("Failed to initialize logging for benchmarks: {}", e);
                }
            }
        }
    }
}

/// Generate sample test data for benchmarks
pub fn generate_test_data(size: usize) -> bytes::Bytes {
    let mut data = String::with_capacity(size);

    // Create a sample JSON string of approximately the specified size
    data.push_str(r#"{"message":"#);

    // Fill with placeholder data to reach the target size
    while data.len() < size - 2 {
        data.push('a');
    }

    data.push('}');
    bytes::Bytes::from(data)
}

/// Helper function to simulate a typical processing delay
/// This helps benchmark how logging affects the rest of the application
pub async fn simulate_processing_delay() {
    tokio::time::sleep(Duration::from_millis(5)).await;
}

/// Clean up after running a benchmark with the specified logging mode
pub fn teardown_logging(
    _mode: LoggingMode,
    _config: Option<Arc<Config>>,
    guard: Option<tracing_appender::non_blocking::WorkerGuard>,
) {
    // Explicitly drop the worker guard to ensure logs are flushed
    if let Some(g) = guard {
        drop(g);
    }

    // Note: We don't try to reset the global default subscriber here anymore
    // Setting it multiple times causes a panic with SetGlobalDefaultError
    // Each benchmark should ideally run in its own process to avoid this issue
}
