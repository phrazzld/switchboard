mod bench_utils;

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkId, Criterion, Throughput,
};
use hyper::{HeaderMap, Method, Uri};
use reqwest::StatusCode;
use std::time::Duration;
use switchboard::proxy_handler::{log_request_details, log_response_details};

use bench_utils::{
    generate_test_data, setup_logging, simulate_processing_delay, teardown_logging, LoggingMode,
};

// Reduced benchmark configuration for faster execution
const SMALL_BODY_SIZE: usize = 1_000; // 1 KB
const MEDIUM_BODY_SIZE: usize = 10_000; // 10 KB
const SAMPLES: usize = 50; // Reduced number of samples

/// Benchmark for request processing with different logging modes
fn bench_request_processing(c: &mut Criterion<WallTime>) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_time()
        .build()
        .expect("Failed to build Tokio runtime");

    let mut group = c.benchmark_group("request_processing");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(SAMPLES);

    // Test with two body sizes
    for size in [SMALL_BODY_SIZE, MEDIUM_BODY_SIZE] {
        group.throughput(Throughput::Bytes(size as u64));

        // Generate test data once
        let test_body = generate_test_data(size);
        let method = Method::POST;
        let uri = Uri::from_static("https://example.com/v1/messages");
        let headers = HeaderMap::new();
        let status = StatusCode::OK;

        // Test with different logging modes
        for mode in [
            LoggingMode::Disabled,
            LoggingMode::StdoutOnly,
            LoggingMode::FileOnly,
            LoggingMode::DualOutput,
        ] {
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", mode), size),
                &(mode, test_body.clone()),
                |b, (mode, body)| {
                    // Setup logging for this benchmark
                    let (config, guard) = setup_logging(*mode);

                    b.to_async(&runtime).iter(|| async {
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
                            simulate_processing_delay().await;

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
                            // For disabled logging mode
                            log_request_details(&method, &uri, &headers, body, false, 0);
                            simulate_processing_delay().await;
                            log_response_details(&status, &headers, body, false, 0, None);
                        }
                    });

                    // Teardown logging
                    teardown_logging(*mode, config, guard);
                },
            );
        }
    }

    group.finish();
}

/// Tests high-volume logging to verify non-blocking behavior
fn bench_high_volume_logging(c: &mut Criterion<WallTime>) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_time()
        .build()
        .expect("Failed to build Tokio runtime");

    let mut group = c.benchmark_group("high_volume_logging");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(10); // Fewer samples due to longer runs

    // Parameters for high-volume testing
    let num_logs = 100; // Reduced number of logs for faster execution
    let body_size = SMALL_BODY_SIZE;

    group.throughput(Throughput::Elements(num_logs as u64));

    // Generate test data once
    let test_body = generate_test_data(body_size);
    let method = Method::POST;
    let uri = Uri::from_static("https://example.com/v1/messages");
    let headers = HeaderMap::new();

    // Only test file logging modes here
    for mode in [LoggingMode::FileOnly, LoggingMode::DualOutput] {
        group.bench_with_input(
            BenchmarkId::new(format!("{:?}", mode), num_logs),
            &(mode, test_body.clone()),
            |b, (mode, body)| {
                // Setup logging for this benchmark
                let (config, guard) = setup_logging(*mode);

                // Ensure we have a config
                let config = config.expect("Config should be available for this test");

                b.to_async(&runtime).iter(|| async {
                    for _ in 0..num_logs {
                        // Log a request (this will generate a lot of logging activity)
                        log_request_details(
                            &method,
                            &uri,
                            &headers,
                            body,
                            config.log_bodies,
                            config.log_max_body_size,
                        );
                    }

                    // Add a small delay to allow the non-blocking writer to process some logs
                    tokio::time::sleep(Duration::from_millis(5)).await;
                });

                // Teardown logging (this will flush the logs)
                teardown_logging(*mode, Some(config), guard);
            },
        );
    }

    group.finish();
}

// Configure the benchmark groups with reduced settings for faster execution
criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5))
        .sample_size(SAMPLES)
        .significance_level(0.05)
        .noise_threshold(0.05);
    targets = bench_request_processing, bench_high_volume_logging
}

criterion_main!(benches);
