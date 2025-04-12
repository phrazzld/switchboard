# Resolve `unused_imports` suppression in integration tests

## Implementation Approach
Investigate the `futures_util::StreamExt` import in `tests/proxy_integration_tests.rs` to determine if it's needed for current or future test functionality. If it's truly unused, remove both the import and the `#[allow(unused_imports)]` attribute. If it's intended for future use, keep the import but replace the suppress attribute with a descriptive `// TODO:` comment explaining its intended purpose.