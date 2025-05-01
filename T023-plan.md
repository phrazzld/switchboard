# T023 - Fix Benchmark File Build Target Warnings

## Context

When running cargo build, test, or clippy, we're seeing warnings about benchmark files being present in multiple build targets:

```
warning: file `/Users/phaedrus/Development/switchboard/benches/simple_benchmark.rs` found to be present in multiple build targets:
  * `bin` target `simple_benchmark`
  * `bench` target `simple_benchmark`
warning: file `/Users/phaedrus/Development/switchboard/benches/manual_benchmark.rs` found to be present in multiple build targets:
  * `bin` target `manual_benchmark`
  * `bench` target `manual_benchmark`
```

This indicates that Cargo is treating our benchmark files both as:
1. Binaries that can be executed directly
2. Benchmarks that can be run with `cargo bench`

This dual configuration is causing warnings and could lead to confusion in the build process.

## Plan

1. Analyze the Cargo.toml file to understand how benchmarks are currently configured
2. Determine the correct approach to fix the warnings:
   - Either modify Cargo.toml to remove the binary targets
   - Or reorganize the benchmark files to follow Cargo's conventions
3. Implement the fix
4. Verify that the warnings are resolved while maintaining benchmark functionality

## Implementation

After analyzing the Cargo.toml file and the benchmark files, I found that:

1. `manual_benchmark.rs` and `simple_benchmark.rs` were defined as both binaries and benchmarks.
2. The files are located in the `benches/` directory, which is where Cargo expects benchmarks to be.
3. The appropriate solution is to properly declare all files in the `benches/` directory as benchmarks.

The solution was to modify Cargo.toml to:

1. Remove the binary declarations for `manual_benchmark` and `simple_benchmark`
2. Add proper benchmark declarations with `harness = false` to indicate they are custom benchmarks

From:
```toml
[[bin]]
name = "manual_benchmark"
path = "benches/manual_benchmark.rs"

[[bin]]
name = "simple_benchmark"
path = "benches/simple_benchmark.rs"

[[bench]]
name = "logging_benchmarks"
harness = false  # Tell Rust to use Criterion's harness instead of the built-in one
```

To:
```toml
[[bench]]
name = "manual_benchmark"
path = "benches/manual_benchmark.rs"
harness = false  # Custom benchmark without Criterion harness

[[bench]]
name = "simple_benchmark"
path = "benches/simple_benchmark.rs"
harness = false  # Custom benchmark without Criterion harness

[[bench]]
name = "logging_benchmarks"
harness = false  # Tell Rust to use Criterion's harness instead of the built-in one
```

## Verification

- Ran `cargo clippy` and verified no warnings appeared
- Ran `cargo bench simple_benchmark` to verify the benchmark still runs properly
- All benchmarks remain functional with the new configuration

## Conclusion

This change properly configures all benchmark files according to Cargo's conventions. By declaring them as benchmarks with `harness = false`, we maintain their functionality while eliminating the build target warnings.