use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use switchboard::config::Config;
use switchboard::logger;
use switchboard::proxy_handler::{log_request_details, log_response_details};

use hyper::{HeaderMap, Method, Uri};
use reqwest::StatusCode;

fn main() {
    println!("Running manual benchmarks for logging performance...");
    println!("This will measure the performance impact of different logging configurations.");

    // Define test parameters
    let iterations = 1000;
    let body_sizes = [1000, 10000]; // 1KB and 10KB

    // Test different logging configurations
    run_benchmark("No Logging", None, iterations, &body_sizes);
    run_benchmark(
        "Stdout Only",
        Some(LogConfig::StdoutOnly),
        iterations,
        &body_sizes,
    );
    run_benchmark(
        "File Only",
        Some(LogConfig::FileOnly),
        iterations,
        &body_sizes,
    );
    run_benchmark(
        "Dual Output",
        Some(LogConfig::DualOutput),
        iterations,
        &body_sizes,
    );

    // Test high volume logging for non-blocking I/O verification
    test_high_volume_logging();

    println!("\nBenchmark completed!");
}

#[derive(Debug, Clone, Copy)]
enum LogConfig {
    StdoutOnly,
    FileOnly,
    DualOutput,
}

fn run_benchmark(name: &str, config: Option<LogConfig>, iterations: usize, body_sizes: &[usize]) {
    println!("\n--- {} ---", name);

    for &size in body_sizes {
        // Set up the test environment
        let (config_obj, guard) = setup_logging(config);

        // Prepare test data
        let body = generate_test_body(size);
        let method = Method::POST;
        let uri = Uri::from_static("https://example.com/v1/messages");
        let headers = HeaderMap::new();
        let status = StatusCode::OK;

        // Run the benchmark
        let start = Instant::now();

        for _ in 0..iterations {
            // Only pass log_bodies and log_max_body_size if we have a config
            if let Some(cfg) = &config_obj {
                // Log request
                log_request_details(
                    &method,
                    &uri,
                    &headers,
                    &body,
                    cfg.log_bodies,
                    cfg.log_max_body_size,
                );

                // Simulate processing
                std::thread::sleep(Duration::from_micros(100));

                // Log response
                log_response_details(
                    &status,
                    &headers,
                    &body,
                    cfg.log_bodies,
                    cfg.log_max_body_size,
                    None,
                );
            } else {
                // For disabled logging mode
                log_request_details(&method, &uri, &headers, &body, false, 0);
                std::thread::sleep(Duration::from_micros(100));
                log_response_details(&status, &headers, &body, false, 0, None);
            }
        }

        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

        println!("Body Size: {}KB", size / 1000);
        println!("  Time: {:.2?}", elapsed);
        println!("  Throughput: {:.2} ops/sec", ops_per_sec);
        println!(
            "  Average: {:.2} microseconds/op",
            elapsed.as_micros() as f64 / iterations as f64
        );

        // Clean up
        drop(guard);
    }
}

fn test_high_volume_logging() {
    println!("\n--- High Volume Logging Test (Non-blocking I/O) ---");

    // Set up file logging
    let (config, guard) = setup_logging(Some(LogConfig::DualOutput));
    let config = config.expect("Config should be available for this test");

    // Prepare test data
    let body = generate_test_body(1000);
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();

    // Parameters for high-volume testing
    let burst_size = 1000;

    println!("Generating burst of {} log messages...", burst_size);

    // Measure time to generate logs
    let start = Instant::now();

    for i in 0..burst_size {
        // Log a request (this will generate a lot of logging activity)
        log_request_details(
            &method,
            &uri,
            &headers,
            &body,
            config.log_bodies,
            config.log_max_body_size,
        );

        // Every 100 messages, print progress and check timing
        if (i + 1) % 100 == 0 {
            let elapsed = start.elapsed();
            println!(
                "  Generated {} messages in {:.2?} ({:.2} msgs/sec)",
                i + 1,
                elapsed,
                (i + 1) as f64 / elapsed.as_secs_f64()
            );
        }
    }

    let gen_time = start.elapsed();
    println!(
        "Total time to generate {} messages: {:.2?}",
        burst_size, gen_time
    );
    println!(
        "Message generation rate: {:.2} msgs/sec",
        burst_size as f64 / gen_time.as_secs_f64()
    );

    // Small pause to demonstrate non-blocking behavior
    println!("Pausing for 500ms to allow some non-blocking I/O to complete...");
    std::thread::sleep(Duration::from_millis(500));

    // Time to flush logs
    let flush_start = Instant::now();
    println!("Flushing logs...");
    drop(guard);
    let flush_time = flush_start.elapsed();

    println!("Time to flush all logs: {:.2?}", flush_time);
    println!(
        "Non-blocking I/O verification: {}",
        if flush_time > gen_time {
            "Non-blocking I/O confirmed - flush took longer than generation"
        } else {
            "Inconclusive - flush was faster than expected"
        }
    );
}

fn setup_logging(
    mode: Option<LogConfig>,
) -> (
    Option<Arc<Config>>,
    Option<tracing_appender::non_blocking::WorkerGuard>,
) {
    // Create a temporary directory for log files
    let log_file_path = Path::new("./benchmark.log").to_string_lossy().to_string();

    match mode {
        None => {
            // No logging
            (None, None)
        }
        Some(LogConfig::StdoutOnly) => {
            // Create config with stdout only (set file level to OFF)
            let config = Arc::new(Config {
                port: "0".to_string(),                     // Not used in benchmarks
                anthropic_api_key: "test-key".to_string(), // Not used in benchmarks
                anthropic_target_url: "https://example.com".to_string(), // Not used in benchmarks
                log_stdout_level: "debug".to_string(),
                log_format: "json".to_string(), // JSON format for consistency
                log_bodies: true,
                log_file_path,
                log_file_level: "off".to_string(), // Disable file logging
                log_max_body_size: 20480,
                log_directory_mode: switchboard::config::LogDirectoryMode::Default,
            });

            match logger::init_tracing(&config) {
                Ok(guard) => (Some(config), Some(guard)),
                Err(e) => {
                    panic!("Failed to initialize logging for benchmarks: {}", e);
                }
            }
        }
        Some(LogConfig::FileOnly) => {
            // Create config with file only (set stdout level to OFF)
            let config = Arc::new(Config {
                port: "0".to_string(),                     // Not used in benchmarks
                anthropic_api_key: "test-key".to_string(), // Not used in benchmarks
                anthropic_target_url: "https://example.com".to_string(), // Not used in benchmarks
                log_stdout_level: "off".to_string(),       // Disable stdout logging
                log_format: "json".to_string(),            // Not relevant when stdout disabled
                log_bodies: true,
                log_file_path,
                log_file_level: "debug".to_string(),
                log_max_body_size: 20480,
                log_directory_mode: switchboard::config::LogDirectoryMode::Default,
            });

            match logger::init_tracing(&config) {
                Ok(guard) => (Some(config), Some(guard)),
                Err(e) => {
                    panic!("Failed to initialize logging for benchmarks: {}", e);
                }
            }
        }
        Some(LogConfig::DualOutput) => {
            // Create config with both outputs enabled
            let config = Arc::new(Config {
                port: "0".to_string(),                     // Not used in benchmarks
                anthropic_api_key: "test-key".to_string(), // Not used in benchmarks
                anthropic_target_url: "https://example.com".to_string(), // Not used in benchmarks
                log_stdout_level: "debug".to_string(),
                log_format: "json".to_string(), // JSON format for consistency
                log_bodies: true,
                log_file_path,
                log_file_level: "debug".to_string(),
                log_max_body_size: 20480,
                log_directory_mode: switchboard::config::LogDirectoryMode::Default,
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

fn generate_test_body(size: usize) -> bytes::Bytes {
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
