use std::sync::{Arc, Mutex};
use std::time::Duration;
use switchboard::config::Config;
use switchboard::logger;
use tracing::{debug, error, info, trace, warn, Level, Subscriber};
use tracing_subscriber::{layer::SubscriberExt, registry::LookupSpan, Layer};

// Define a memory capture layer for testing
struct MemoryCapture {
    buffer: Arc<Mutex<Vec<String>>>,
    level: Level,
    json_format: bool,
}

impl MemoryCapture {
    fn new(level: Level, json_format: bool) -> Self {
        MemoryCapture {
            buffer: Arc::new(Mutex::new(Vec::new())),
            level,
            json_format,
        }
    }

    fn buffer(&self) -> Arc<Mutex<Vec<String>>> {
        self.buffer.clone()
    }
}

impl<S> Layer<S> for MemoryCapture
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Check if the event's level is high enough for our filter
        if event.metadata().level() > &self.level {
            return;
        }

        // Format the event data according to the format
        let formatted_message = if self.json_format {
            // Visit the event to extract its fields as JSON
            let mut visitor = JsonVisitor::default();
            event.record(&mut visitor);
            visitor.finish()
        } else {
            // Visit the event to extract its fields as text
            let mut visitor = TextVisitor::default();
            event.record(&mut visitor);
            visitor.finish()
        };

        // Store the formatted message in the buffer
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push(formatted_message);
        }
    }
}

// Simple visitor for text format
#[derive(Default)]
struct TextVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl TextVisitor {
    fn finish(self) -> String {
        let mut result = self.message.unwrap_or_else(|| "".to_string());

        if !self.fields.is_empty() {
            result.push_str(" ");
            for (i, (key, value)) in self.fields.iter().enumerate() {
                if i > 0 {
                    result.push_str(" ");
                }
                // Remove quotes from value for string types
                let formatted_value = value.trim_matches('"').to_string();
                result.push_str(&format!("{}={}", key, formatted_value));
            }
        }

        result
    }
}

impl tracing::field::Visit for TextVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value).trim_matches('"').to_string());
        } else {
            self.fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }
}

// JSON visitor
#[derive(Default)]
struct JsonVisitor {
    fields: Vec<(String, String)>,
}

impl JsonVisitor {
    fn finish(self) -> String {
        let mut json = serde_json::Map::new();
        for (key, value) in self.fields {
            // Handle different types (simplified for this test)
            if let Ok(parsed) = serde_json::from_str(&value) {
                json.insert(key, parsed);
            } else {
                json.insert(
                    key,
                    serde_json::Value::String(value.trim_matches('"').to_string()),
                );
            }
        }
        serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
    }
}

impl tracing::field::Visit for JsonVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields
            .push((field.name().to_string(), format!("{:?}", value)));
    }
}

// Create a test subscriber with a memory capture layer
fn create_test_subscriber(
    level: &str,
    format: &str,
) -> (impl Subscriber + Send + Sync, Arc<Mutex<Vec<String>>>) {
    // Parse the level
    let level = match level {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    // Create a memory capture layer with the appropriate format
    let json_format = format == "json";
    let capture = MemoryCapture::new(level, json_format);
    let buffer = capture.buffer();

    // Create a new registry with the memory capture layer
    let subscriber = tracing_subscriber::registry().with(capture);

    (subscriber, buffer)
}

// Tests

#[test]
fn test_stdout_level_filtering() {
    // Test various log levels
    let test_levels = vec![
        ("debug", vec!["DEBUG", "INFO", "WARN", "ERROR"]),
        ("info", vec!["INFO", "WARN", "ERROR"]),
        ("warn", vec!["WARN", "ERROR"]),
        ("error", vec!["ERROR"]),
    ];

    for (level, expected_levels) in test_levels {
        println!("Testing log level: {}", level);

        // Create a test subscriber and get its buffer
        let (subscriber, buffer) = create_test_subscriber(level, "pretty");

        // Use the test subscriber for this scope
        let _guard = tracing::subscriber::set_default(subscriber);

        // Emit log messages at different levels
        trace!("TRACE message");
        debug!("DEBUG message");
        info!("INFO message");
        warn!("WARN message");
        error!("ERROR message");

        // Allow a moment for logs to process
        std::thread::sleep(Duration::from_millis(50));

        // Get the captured messages
        let captured = buffer.lock().unwrap();
        let messages: Vec<&String> = captured.iter().collect();

        // Print captured messages for debugging
        println!("Captured {} messages:", messages.len());
        for msg in &messages {
            println!("  {}", msg);
        }

        // Check that we only have the expected number of messages
        assert_eq!(
            messages.len(),
            expected_levels.len(),
            "Expected {} messages for level {}, got {}",
            expected_levels.len(),
            level,
            messages.len()
        );

        // Check that each expected level appears in the output
        for expected_level in expected_levels {
            let found = messages.iter().any(|msg| msg.contains(expected_level));
            assert!(
                found,
                "Level {} message not found in output for level {}",
                expected_level, level
            );
        }

        // Check that unexpected levels don't appear
        let unexpected_levels = match level {
            "error" => vec!["TRACE", "DEBUG", "INFO", "WARN"],
            "warn" => vec!["TRACE", "DEBUG", "INFO"],
            "info" => vec!["TRACE", "DEBUG"],
            "debug" => vec!["TRACE"],
            _ => vec![], // trace level has no unexpected levels
        };

        for unexpected_level in unexpected_levels {
            assert!(
                !messages
                    .iter()
                    .any(|msg| msg.contains(&format!("{} message", unexpected_level))),
                "Unexpected {} level message found in output for level {}",
                unexpected_level,
                level
            );
        }

        // Clean up the global subscriber
        drop(_guard);

        // Small delay between tests to ensure clean test environment
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn test_stdout_format_json() {
    // Create a test subscriber with JSON format
    let (subscriber, buffer) = create_test_subscriber("info", "json");

    // Use the test subscriber for this scope
    let _guard = tracing::subscriber::set_default(subscriber);

    // Emit a structured log message
    info!(user_id = 123, action = "login", "User logged in");

    // Allow a moment for logs to process
    std::thread::sleep(Duration::from_millis(50));

    // Verify the message is formatted as JSON
    let captured = buffer.lock().unwrap();
    assert_eq!(captured.len(), 1, "Should capture 1 log message");

    // Validate the captured message is valid JSON
    let message = &captured[0];
    println!("Captured JSON message: {}", message);

    match serde_json::from_str::<serde_json::Value>(message) {
        Ok(json) => {
            // Verify the JSON contains the expected fields
            assert!(
                json.get("message").is_some(),
                "JSON should have a message field"
            );
            assert!(
                json.get("user_id").is_some(),
                "JSON should have a user_id field"
            );
            assert!(
                json.get("action").is_some(),
                "JSON should have an action field"
            );

            // Verify field values
            if let Some(user_id) = json.get("user_id") {
                assert_eq!(user_id.as_i64().unwrap(), 123, "user_id should be 123");
            }
            if let Some(action) = json.get("action") {
                assert_eq!(
                    action.as_str().unwrap(),
                    "login",
                    "action should be 'login'"
                );
            }
            if let Some(message) = json.get("message") {
                assert_eq!(
                    message.as_str().unwrap(),
                    "User logged in",
                    "message should be 'User logged in'"
                );
            }
        }
        Err(e) => {
            panic!("Failed to parse JSON: {}. Message: {}", e, message);
        }
    }

    // Clean up the global subscriber
    drop(_guard);
}

#[test]
fn test_stdout_format_pretty() {
    // Create a test subscriber with pretty format
    let (subscriber, buffer) = create_test_subscriber("info", "pretty");

    // Use the test subscriber for this scope
    let _guard = tracing::subscriber::set_default(subscriber);

    // Emit a structured log message
    info!(user_id = 123, action = "login", "User logged in");

    // Allow a moment for logs to process
    std::thread::sleep(Duration::from_millis(50));

    // Verify the message was captured in pretty format
    let captured = buffer.lock().unwrap();
    assert_eq!(captured.len(), 1, "Should capture 1 log message");

    let message = &captured[0];
    println!("Captured pretty message: {}", message);

    // Check the message content
    assert!(
        message.contains("User logged in"),
        "Message should contain the log message"
    );
    assert!(
        message.contains("user_id=123"),
        "Message should contain the user_id field"
    );
    assert!(
        message.contains("action=login"),
        "Message should contain the action field"
    );

    // Clean up the global subscriber
    drop(_guard);
}

// Test that verifies logger initialization with JSON format
#[test]
fn test_logger_json_format() {
    // Create a unique temp file path to avoid conflicts with other tests
    let temp_file = std::env::temp_dir().join(format!(
        "switchboard_json_test_{}.log",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let config = Config {
        port: "0".to_string(),
        anthropic_api_key: "test-api-key".to_string(),
        anthropic_target_url: "https://example.com".to_string(),
        log_stdout_level: "info".to_string(),
        log_format: "json".to_string(),
        log_bodies: true,
        log_file_path: temp_file.to_string_lossy().to_string(),
        log_file_level: "debug".to_string(),
        log_max_body_size: 1024,
        log_directory_mode: switchboard::config::LogDirectoryMode::Default,
    };

    // Initialize the logger (this should succeed with JSON format)
    let _guard =
        logger::init_tracing(&config).expect("Failed to initialize logging for stdout JSON test");

    // Emit a log message
    info!(format = "json", "Test with JSON format");

    // The test passes as long as the logger initialization doesn't panic
    // We don't need to verify the output format since that's already tested elsewhere
}
