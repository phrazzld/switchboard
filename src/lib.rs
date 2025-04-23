// Switchboard library entry point

// Re-export modules for use in integration tests and the main binary
pub mod config;
pub mod logger;
pub mod proxy_handler;

// Temporary test module for testing pre-commit hook failure detection
#[cfg(test)]
mod temp_test {
    #[test]
    fn intentionally_failing_test() {
        assert_eq!(1, 2, "This test is designed to fail");
    }
}
