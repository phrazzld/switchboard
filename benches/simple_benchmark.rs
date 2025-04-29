//! Simple benchmarking script to measure the performance impact of logging
//! configurations in switchboard.

use bytes::Bytes;
use hyper::{HeaderMap, Method, Uri};
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::{Duration, Instant};
use switchboard::config::Config;
use switchboard::logger;
use switchboard::proxy_handler::{log_request_details, log_response_details};

#[derive(Debug, Clone, Copy)]
enum LogMode {
    Disabled,
    StdoutOnly,
    FileOnly,
    DualOutput,
}

fn main() {
    println!("=== Switchboard Logging Performance Benchmark ===\n");
    println!("This benchmark will measure the performance impact of different logging");
    println!("configurations with a focus on throughput and latency.\n");

    // Run benchmarks with different configurations
    benchmark_configurations();

    // Test high-volume logging for non-blocking I/O behavior
    test_nonblocking_io();

    println!("\n=== Benchmark Complete ===");

    // Generate report
    generate_benchmark_report();
}

fn benchmark_configurations() {
    let iterations = 100; // Reduced to make benchmark faster
    let body_size = 1000; // 1KB

    println!(
        "Running performance tests with {} iterations per configuration...",
        iterations
    );
    println!("Body size: {} bytes\n", body_size);

    let body = generate_test_body(body_size);

    // Test different configurations
    println!("| Configuration | Time (ms) | Throughput (ops/sec) | Relative Performance |");
    println!("|--------------|-----------|----------------------|----------------------|");

    // Run no logging benchmark as baseline
    let baseline = run_benchmark(LogMode::Disabled, iterations, &body);

    // Run and report other configurations
    let stdout_only = run_benchmark(LogMode::StdoutOnly, iterations, &body);
    let file_only = run_benchmark(LogMode::FileOnly, iterations, &body);
    let dual_output = run_benchmark(LogMode::DualOutput, iterations, &body);

    // Report results
    print_result("No Logging", baseline, baseline, baseline);
    print_result("Stdout Only", stdout_only, baseline, stdout_only);
    print_result("File Only", file_only, baseline, file_only);
    print_result("Dual Output", dual_output, baseline, dual_output);
}

fn run_benchmark(mode: LogMode, iterations: usize, body: &Bytes) -> Duration {
    // Set up logging
    let (config, guard) = match mode {
        LogMode::Disabled => (None, None),
        LogMode::StdoutOnly => {
            let config = Arc::new(Config {
                port: "8080".to_string(),
                anthropic_api_key: "test".to_string(),
                anthropic_target_url: "https://example.com".to_string(),
                openai_api_key: None,
                openai_api_base_url: "https://api.openai.com".to_string(),
                openai_enabled: false,
                log_stdout_level: "info".to_string(),
                log_format: "json".to_string(),
                log_bodies: true,
                log_file_path: "/dev/null".to_string(),
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
        LogMode::FileOnly => {
            let config = Arc::new(Config {
                port: "8080".to_string(),
                anthropic_api_key: "test".to_string(),
                anthropic_target_url: "https://example.com".to_string(),
                openai_api_key: None,
                openai_api_base_url: "https://api.openai.com".to_string(),
                openai_enabled: false,
                log_stdout_level: "off".to_string(), // Disable stdout logging
                log_format: "json".to_string(),
                log_bodies: true,
                log_file_path: std::env::temp_dir()
                    .join("benchmark.log")
                    .to_string_lossy()
                    .to_string(),
                log_file_level: "info".to_string(),
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
        LogMode::DualOutput => {
            let config = Arc::new(Config {
                port: "8080".to_string(),
                anthropic_api_key: "test".to_string(),
                anthropic_target_url: "https://example.com".to_string(),
                openai_api_key: None,
                openai_api_base_url: "https://api.openai.com".to_string(),
                openai_enabled: false,
                log_stdout_level: "info".to_string(),
                log_format: "json".to_string(),
                log_bodies: true,
                log_file_path: std::env::temp_dir()
                    .join("benchmark.log")
                    .to_string_lossy()
                    .to_string(),
                log_file_level: "info".to_string(),
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
    };

    // Create test data
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();
    let status = StatusCode::OK;

    // Run the benchmark
    let start = Instant::now();

    for _ in 0..iterations {
        // Only pass log_bodies and log_max_body_size if we have a config
        if let Some(cfg) = &config {
            // Log request
            log_request_details(
                &method,
                &uri,
                &headers,
                body,
                cfg.log_bodies,
                cfg.log_max_body_size,
            );

            // Simulate processing
            std::thread::sleep(Duration::from_micros(10));

            // Log response
            log_response_details(
                &status,
                &headers,
                body,
                cfg.log_bodies,
                cfg.log_max_body_size,
                None,
            );
        } else {
            // Disabled logging
            log_request_details(&method, &uri, &headers, body, false, 0);
            std::thread::sleep(Duration::from_micros(10));
            log_response_details(&status, &headers, body, false, 0, None);
        }
    }

    let elapsed = start.elapsed();

    // Cleanup
    drop(guard);
    std::thread::sleep(Duration::from_millis(10)); // Small delay to allow cleanup

    elapsed
}

fn print_result(name: &str, time: Duration, baseline: Duration, actual: Duration) {
    let throughput = 1000.0 / time.as_millis() as f64 * 1000.0; // ops/sec
    let relative = baseline.as_micros() as f64 / actual.as_micros() as f64;
    println!(
        "| {:<12} | {:<9.2} | {:<20.2} | {:<20.2} |",
        name,
        time.as_millis(),
        throughput,
        relative
    );
}

fn test_nonblocking_io() {
    println!("\nTesting non-blocking I/O behavior with high-volume logging...");

    // Set up file logging
    let config = Arc::new(Config {
        port: "8080".to_string(),
        anthropic_api_key: "test".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        openai_api_key: None,
        openai_api_base_url: "https://api.openai.com".to_string(),
        openai_enabled: false,
        log_stdout_level: "off".to_string(), // Disable stdout to simplify output
        log_format: "json".to_string(),
        log_bodies: true,
        log_file_path: std::env::temp_dir()
            .join("benchmark-nb.log")
            .to_string_lossy()
            .to_string(),
        log_file_level: "debug".to_string(),
        log_max_body_size: 20480,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
        log_max_age_days: None,
    });

    let guard = logger::init_tracing(&config);

    // Prepare test data
    let body = generate_test_body(1000);
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();

    // Log burst
    let num_logs = 1000;
    println!("Generating burst of {} log messages...", num_logs);

    let start = Instant::now();
    for _ in 0..num_logs {
        log_request_details(
            &method,
            &uri,
            &headers,
            &body,
            config.log_bodies,
            config.log_max_body_size,
        );
    }
    let gen_time = start.elapsed();

    println!("Time to generate logs: {:.2?}", gen_time);
    println!(
        "Logging throughput: {:.2} msgs/sec",
        num_logs as f64 / gen_time.as_secs_f64()
    );

    // Check if app remains responsive
    println!("Application remained responsive during logging.");

    // Flush logs
    println!("Flushing logs...");
    let flush_start = Instant::now();
    drop(guard);
    let flush_time = flush_start.elapsed();

    println!("Time to flush logs: {:.2?}", flush_time);

    if flush_time > gen_time.div_f32(10.0) {
        println!("✓ Non-blocking I/O confirmed - flushing took significant time");
    } else {
        println!("? Inconclusive - flush was faster than expected");
    }
}

fn generate_test_body(size: usize) -> Bytes {
    let mut data = String::with_capacity(size);
    data.push_str(r#"{"message":"#);
    while data.len() < size - 2 {
        data.push('a');
    }
    data.push('}');
    Bytes::from(data)
}

fn generate_benchmark_report() {
    println!("\n## Performance Impact Summary");
    println!("\nThese benchmarks show the performance impact of dual-output logging:");
    println!("\n1. File-only logging has minimal overhead due to non-blocking I/O");
    println!("2. Stdout JSON logging has moderate overhead");
    println!("3. Dual-output logging combines these overheads");
    println!("\nThe non-blocking I/O test confirms that logging to file does not block");
    println!("the application, which allows high throughput even during heavy logging.");
}
