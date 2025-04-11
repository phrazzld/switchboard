# Rust Testing Strategy

This document outlines our philosophy and strategy for automated testing in Rust projects. Effective testing is critical for ensuring software correctness, preventing regressions, enabling confident refactoring, providing living documentation, and ultimately driving better software design, leveraging Rust's strengths in safety and expressiveness. Our approach aligns directly with our Core Principles, especially *Design for Testability* and *Modularity*.

---

## 1. Guiding Principles: Why and How We Test

Our testing efforts are guided by the following fundamental principles:

*   **Purpose:** Tests exist primarily to verify that the software meets its functional requirements reliably, to prevent regressions as the codebase evolves, and to increase our confidence in deploying changes frequently and safely. `cargo test` should be a reliable gatekeeper.
*   **Simplicity & Clarity:** Test code *is* production code. It must be simple, clear, concise, and maintainable using idiomatic Rust. Overly complex test setups (`#[cfg(test)]` modules, helper functions) or convoluted assertions often indicate complexity or poor design in the code under test itself. Leverage Rust's type system and `Result` type for clear error handling tests.
*   **Behavior Over Implementation:** Tests should primarily validate the observable *behavior* of a component through its public interface (functions, methods, traits), not its internal implementation details. Testing behavior makes tests more resilient to refactoring and less brittle. Focus on the contracts defined by `pub` items.
*   **Testability is a Design Constraint:** We do not write code and *then* figure out how to test it. Testability is a primary consideration *during* design and implementation, often guided by defining clear `trait` boundaries. **Crucially: If code proves difficult to test without resorting to extensive mocking of internal collaborators (concrete types or private modules within the same crate), this is a strong signal that the *code under test* likely needs refactoring (e.g., to improve separation of concerns, clarify interfaces/traits, or reduce coupling) *before* attempting to write complex tests.**

---

## 2. Test Focus and Types: The Rust Testing Landscape

We employ different types of automated tests provided by Rust's ecosystem, strategically focusing our efforts where they provide the most value and confidence relative to their cost.

*   **Unit Tests:** Highly focused tests verifying small, isolated pieces of logic, typically placed in a `mod tests { ... }` submodule within the same file or module they are testing. (See Section 3).
*   **Integration Tests:** Verify the collaboration *between* different parts of a crate or between multiple crates, typically placed in the `tests/` directory at the crate root. **These often provide the highest return on investment for ensuring library APIs or application features work correctly.** (See Section 4).
*   **Doc Tests:** Examples embedded directly in Rust documentation (`///`) that are compiled and run by `cargo test`. Excellent for verifying API examples and usage patterns remain correct.
*   **Property-Based Tests:** Define properties or invariants that should hold true for a wide range of generated inputs, rather than testing specific examples. Useful for uncovering edge cases in algorithms or complex logic. (Tools: `proptest`, `quickcheck`).
*   **Snapshot Tests:** Compare the output of a function against a stored "snapshot" file. Useful for testing complex data structures, serialization formats (JSON, YAML), or UI component rendering. (Tools: `insta`).
*   **Benchmarking:** Measure the performance characteristics of code. While not strictly correctness tests, they are crucial for performance-sensitive code. Run via `cargo bench`. (Tools: `criterion`).
*   **End-to-End (E2E) Tests:** Validate complete user journeys or critical paths through the entire deployed system. Used sparingly due to cost and potential flakiness.

We prioritize *effectiveness*. We favor integration tests that exercise public APIs over heavily mocked unit tests when verifying component interactions. Doc tests are highly encouraged for public APIs.

---

## 3. Unit Testing Approach: Verifying Isolated Logic

Unit tests are most valuable for verifying the correctness of specific algorithms, complex calculations, data transformations, state transitions within a struct, or other logic units that can be tested in relative isolation.

*   **Scope:** Test a single function, method, or a small, cohesive set of functions/methods (e.g., methods on a `struct` or functions within a private module) in isolation from its *internal* collaborators within the same crate.
*   **Location:** Typically reside within a `#[cfg(test)] mod tests { ... }` block inside the `.rs` file containing the code under test.
*   **Dependencies:** **Crucially, do not mock internal collaborators** (other structs, functions, modules defined *within* the same crate). If a piece of code requires extensive internal mocking to be tested, it's a design smell – refactor the code (see Principle 1d and Section 5). Abstracted *external* dependencies (represented by traits) may be mocked according to the Mocking Policy (Section 5).
*   **Goal:** Verify the logical correctness of the unit under test, including its handling of various inputs, outputs, `Result` variants (`Ok`/`Err`), panics (using `#[should_panic]`), edge cases, and boundary conditions. Pure functions are ideal candidates.
*   **Rationale:** Unit tests provide fast, precise feedback. Keeping them truly "unit" (without internal mocks) ensures they are robust and don't hinder refactoring.
*   **Implementation:**
    *   Use the `#[test]` attribute.
    *   Use standard assertion macros: `assert!`, `assert_eq!`, `assert_ne!`.
    *   Use `#[should_panic(expected = "panic message")]` to test expected panics.
    *   Test `Result` return types explicitly for both `Ok` and `Err` variants.
    *   Leverage `#[cfg(test)]` helper functions or modules for test setup within the same file/module.

    ```rust
    // src/calculator.rs
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
        if b == 0 {
            Err("Cannot divide by zero")
        } else {
            Ok(a / b)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*; // Import items from outer module

        #[test]
        fn test_add_positive() {
            assert_eq!(add(2, 3), 5);
        }

        #[test]
        fn test_divide_ok() {
            assert_eq!(divide(10, 2), Ok(5));
        }

        #[test]
        fn test_divide_by_zero() {
            assert_eq!(divide(10, 0), Err("Cannot divide by zero"));
        }

        // Example using should_panic (less common than checking Result)
        // fn potentially_panicking_function(input: &str) {
        //     if input.is_empty() { panic!("Input cannot be empty"); }
        //     // ...
        // }
        //
        // #[test]
        // #[should_panic(expected = "Input cannot be empty")]
        // fn test_panic_on_empty_input() {
        //     potentially_panicking_function("");
        // }
    }
    ```

---

## 4. Integration Testing Approach (High Priority): Verifying Collaboration

These tests ensure that distinct parts of our crate (or multiple crates in a workspace) work together correctly through their public APIs. They operate at a higher level than unit tests.

*   **Scope:** Test the public API of a crate (`lib.rs` or `main.rs`) or the interaction between multiple public modules/structs/functions. For applications, this often involves testing handlers, services, and repository layers together.
*   **Location:** Reside in the `tests/` directory at the crate root. Each file in `tests/` is compiled as a separate crate, linking the library/binary crate under test.
*   **Dependencies:** Use *real* implementations of internal collaborators (types defined within the crate being tested) whenever practical. Mocking should *only* occur at true external system boundaries, accessed via `trait`s defined within our crate (see Mocking Policy, Section 5).
*   **Goal:** Verify that components integrate correctly via their public interfaces, data flows accurately, contracts (especially trait bounds) are honored, side effects occur as expected, and key workflows produce the correct outcomes (including `Result::Err` states).
*   **Rationale:** Provides high confidence that the crate/application works cohesively as intended. Tests behavior closer to real-world usage. Less brittle to internal refactoring compared to heavily mocked unit tests. Directly validates the *Modularity* and *Interface* design.
*   **Implementation:**
    *   Create `.rs` files in the `tests/` directory.
    *   Use the crate under test by adding `use your_crate_name;`.
    *   For HTTP services (e.g., using Actix Web, Axum, Rocket): Utilize the framework's test utilities (e.g., `actix_web::test`, `axum::test_helpers`) or libraries like `reqwest` to make requests to a test server instance.
    *   For database interactions: Prefer using real database instances managed via Docker containers (e.g., using `testcontainers-rs`) with test-specific databases/schemas, or utilize test transaction features if provided by the database library (e.g., `sqlx::test`, Diesel's test transaction support).
    *   Test interactions between crates in a workspace by having integration tests in one crate depend on and call the public API of another.

    ```rust
    // tests/api_integration.rs
    // Assume `my_app` crate defines an Axum web server

    // #[cfg(test)] // Not needed in tests/ directory files
    // mod tests { // Not needed in tests/ directory files

    use my_app::{create_app, models::Item}; // Import app builder and types
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn test_create_and_get_item() {
        // Setup: Create app instance (potentially with test-specific dependencies)
        let app = create_app(/* pass test db connection pool or mocks here */);

        // 1. Create an item
        let create_response = app
            .clone() // Clone app for concurrent requests
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name": "Test Item", "value": 10}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::CREATED);
        let body = hyper::body::to_bytes(create_response.into_body()).await.unwrap();
        let created_item: Item = serde_json::from_slice(&body).unwrap();
        assert_eq!(created_item.name, "Test Item");
        let item_id = created_item.id;

        // 2. Get the created item
        let get_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/items/{}", item_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
        let fetched_item: Item = serde_json::from_slice(&body).unwrap();
        assert_eq!(fetched_item.id, item_id);
        assert_eq!(fetched_item.name, "Test Item");

        // Cleanup: (e.g., drop test database if using testcontainers)
    }
    // }
    ```

---

## 5. Mocking Policy: Use Sparingly, At Trait Boundaries Only (Crucial)

Our stance on mocking is conservative and critical for maintaining effective tests and good design in Rust.

*   **Minimize Mocking:** Actively strive to design code and tests that require minimal mocking. Test complexity often reflects code complexity or insufficient abstraction.
*   **Mock Only True External System Boundaries via Traits:** Mocking is permissible *only* for `trait`s defined *within our codebase* that represent systems or concerns genuinely *external* to the service/application being tested, and which are impractical or undesirable to use directly in automated tests. These typically include:
    *   Network I/O (Third-party APIs, other distinct microservices)
    *   Databases / External Data Stores (when not using test containers/in-memory fakes)
    *   Filesystem Operations
    *   System Clock (use injectable clock abstractions like `trait Clock { fn now() -> Instant; }`)
    *   External Message Brokers / Caches
*   **Define Traits for External Dependencies:** Ensure external dependencies are *always* accessed through `trait`s defined *within* your own crate (following the Dependency Inversion principle). Your core logic depends on the trait, not the concrete external implementation. Tests provide a *test implementation* (mock or fake) of that trait.
*   **NO Mocking Internal Collaborators:** **It is an anti-pattern to mock concrete structs, functions, or private modules that are defined and owned *within* the same application/crate solely for the purpose of isolating another internal component.** Feeling the need to do this indicates a design flaw (likely high coupling, poor separation of concerns, violation of dependency inversion, or unclear public API). **The correct action is to refactor the code under test to improve its design and testability, often by introducing clearer interfaces (potentially traits, but often just better function/struct boundaries).**

**Rationale:** This strict policy ensures tests verify realistic interactions through defined contracts (traits), remain robust against internal refactoring, provide genuine confidence, and act as accurate indicators of design health. Over-mocking hides design problems and leads to tests that verify the mocks, not the actual integrated behavior.

**Implementation:**
*   Define `trait`s in your core logic for external dependencies.
*   Use mocking libraries like `mockall` or `mockiato` to generate mock implementations of these traits for your tests.
*   Alternatively, create simple hand-rolled "fake" implementations of the trait using `struct`s, often defined within the test module (`#[cfg(test)]`) or integration test file.
*   Inject mock/fake implementations into the code under test during test setup (e.g., via constructor arguments, builder patterns, or function parameters).

```rust
// src/notifications.rs
use async_trait::async_trait; // If async methods are needed in the trait

#[async_trait]
pub trait Notifier {
    async fn send_notification(&self, user_id: &str, message: &str) -> Result<(), String>;
}

// Your service depends on the trait, not a concrete implementation
pub struct UserService<N: Notifier> {
    notifier: N,
    // ... other fields
}

impl<N: Notifier> UserService<N> {
    pub async fn notify_user(&self, user_id: &str, message: &str) {
        // ... logic ...
        if let Err(e) = self.notifier.send_notification(user_id, message).await {
            eprintln!("Failed to send notification: {}", e);
            // Handle error appropriately
        }
        // ... more logic ...
    }
}

// --- In your test file (e.g., tests/user_service_integration.rs) ---
#[cfg(test)] // Or directly in tests/ directory file
mod tests {
    use super::*; // Assuming UserService and Notifier are in scope
    use mockall::mock;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // Option 1: Using mockall
    mock! {
        pub Notifier {} // Name of the mock struct, can be different
        #[async_trait]
        impl Notifier for Notifier { // The trait being mocked
            async fn send_notification(&self, user_id: &str, message: &str) -> Result<(), String>;
        }
    }

    #[tokio::test]
    async fn test_user_service_sends_notification_mockall() {
        let mut mock_notifier = MockNotifier::new(); // Use mock struct name

        mock_notifier.expect_send_notification()
            .withf(|uid, msg| uid == "user123" && msg == "Hello!") // Match arguments
            .times(1) // Expect it to be called once
            .returning(|_, _| Ok(())); // Define return value

        let user_service = UserService { notifier: mock_notifier /* ... other fields */ };
        user_service.notify_user("user123", "Hello!").await;
        // Mockall verifies expectations automatically on drop
    }

    // Option 2: Hand-rolled Fake
    #[derive(Clone, Default)]
    struct FakeNotifier {
        calls: Arc<Mutex<Vec<(String, String)>>>,
    }

    #[async_trait]
    impl Notifier for FakeNotifier {
        async fn send_notification(&self, user_id: &str, message: &str) -> Result<(), String> {
            let mut calls = self.calls.lock().unwrap();
            calls.push((user_id.to_string(), message.to_string()));
            // Simulate success or failure based on test needs
            Ok(())
            // Err("Simulated network error".to_string())
        }
    }

    #[tokio::test]
    async fn test_user_service_sends_notification_fake() {
        let fake_notifier = FakeNotifier::default();
        let user_service = UserService { notifier: fake_notifier.clone() /* ... */ };

        user_service.notify_user("user456", "Welcome!").await;

        let calls = fake_notifier.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], ("user456".to_string(), "Welcome!".to_string()));
    }
}

```

---

## 6. Desired Test Characteristics: The FIRST Properties

We aim for tests that embody the FIRST principles, making our test suite (`cargo test`) trustworthy and effective:

*   **Fast:** Tests must run quickly via `cargo test` to provide rapid feedback. Slow tests hinder development flow. Use `cargo test -- --ignored` to run specific, potentially slower tests (like those needing external resources) separately.
*   **Independent / Isolated:** Each `#[test]` function must be runnable independently and in any order. Tests should manage their own state and not rely on shared mutable global state or the side effects of other tests. Leverage Rust's ownership and borrowing rules to help enforce isolation.
*   **Repeatable / Reliable:** Tests must consistently produce the same pass/fail result. Eliminate non-determinism (e.g., uncontrolled concurrency without proper synchronization, reliance on `SystemTime::now()` without abstraction, random data without seeding). Flaky tests (`cargo test` sometimes passes, sometimes fails) destroy confidence.
*   **Self-Validating:** Tests must use `assert!` macros or `#[should_panic]` to explicitly check outcomes and report pass/fail clearly. No manual inspection of `println!` output should be required (though `println!` is useful for debugging via `cargo test -- --nocapture`).
*   **Timely / Thorough:** Tests should ideally be written alongside or slightly before the production code (TDD/BDD). They need to provide *thorough* coverage of the intended functionality, including happy paths (`Ok` variants), error conditions (`Err` variants, panics), edge cases, and boundary values relevant to the component's responsibility. Consider using code coverage tools (`cargo-tarpaulin`, `grcov`) as a guide, but focus on testing behavior, not just hitting lines.

---

## 7. Test Data Management: Realistic and Maintainable

Effective management of test data is crucial for readable and robust Rust tests.

*   **Clarity & Intent:** Test data setup should clearly reveal the specific scenario being tested. Use descriptive variable names and well-structured data (`struct`s). Make the relationship between inputs and expected outputs obvious.
*   **Realism:** Use test data that reasonably approximates real-world data characteristics, especially for integration tests. Libraries like `fake-rs` can help generate realistic data. Consider property-based testing (`proptest`) for exploring a wider range of valid inputs automatically.
*   **Maintainability:** Avoid duplicating complex struct creation logic. Employ patterns like the Builder Pattern, factory functions (often within `#[cfg(test)]` modules), or dedicated test fixture modules to create test data consistently and reduce boilerplate.
*   **Isolation:** Ensure data used or created by one test does not interfere with subsequent tests. For database tests, use techniques like per-test transactions that roll back automatically (supported by `sqlx::test` and Diesel) or manage isolated database instances/schemas (e.g., via `testcontainers-rs`). Clean up any created files or other external state.

---

By adhering to this strategy, we aim to build robust, maintainable, and well-designed Rust applications and libraries, using testing as a fundamental tool for quality and confidence.