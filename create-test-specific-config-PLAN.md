# Create Test-Specific Config - Plan

## Task Title
Create Test-Specific `Config` in `setup_test_environment`

## Implementation Approach
Modify the `setup_test_environment` function in `tests/common/mod.rs` to create an instance of `switchboard::config::Config` using the mock server URI as the `anthropic_target_url` value. Set appropriate dummy values for other configuration fields like `port`, `anthropic_api_key`, `log_level`, and `log_format` that are suitable for test environments.