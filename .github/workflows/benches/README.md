# Benchmarks Directory

This directory contains files that should be excluded from the main CI test suite to prevent
issues with tracing subscribers.

The primary issue is that the benchmark code tries to set up global tracing subscribers,
but the test harness already has its own subscribers configured, causing conflicts.

## Benchmark Files

- `logging_benchmarks.rs` - Tests different logging configurations and performance
- `bench_utils.rs` - Utilities for setting up benchmark environments

## Running Benchmarks

These benchmarks should be run manually with:

```bash
cargo bench
```

Or as part of a separate workflow dedicated to benchmarking, not as part of the main test suite.