# T005 Plan: Update test/benchmark config instantiations for secrecy

## Task Description
- Update `Config` struct instantiations in test and benchmark files to properly use the SecretString type for API keys
- Wrap dummy API key strings with SecretString::new() in these instantiations
- Ensure tests and benchmarks compile successfully

## Files to Modify
1. `tests/common/mod.rs` - likely contains shared test config setup 
2. Any direct `tests/config_test.rs` - any config tests
3. `benches/*.rs` - benchmark files

## Implementation Steps
1. Check `src/config.rs` to understand the current SecretString implementation
2. Find all Config instantiations in test and benchmark files
3. Update each instantiation to wrap API keys with SecretString::new()
4. Verify compilation with cargo test --no-run and cargo bench --no-run

## Implementation Notes
- Need to import SecretString from the secrecy crate in each file
- Pay attention to Option<SecretString> vs SecretString patterns
- Make sure to use the correct initialization pattern (SecretString::new("".to_string().into()))