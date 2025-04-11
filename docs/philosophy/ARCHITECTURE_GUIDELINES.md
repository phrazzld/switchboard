# Rust Architecture Guidelines

This document translates our Core Principles—particularly Simplicity, Modularity, and Testability—into actionable guidelines for structuring our software applications in Rust. Architecture is not about rigid dogma, but about making conscious design choices that lead to systems that are maintainable, adaptable, testable, easy to reason about, and leverage Rust's strengths in safety, performance, and concurrency. These guidelines provide a default approach optimized for building focused, composable, and robust applications in Rust.

---

## 1. Embrace the Unix Philosophy: Build Focused, Composable Crates and Modules

**Guideline:** Design components (crates, modules, functions, structs, CLIs) to "do one thing and do it well." Aim for minimal, well-defined responsibilities, clear inputs (leveraging Rust's type system), and predictable outputs for each unit. Prefer building larger systems by composing these smaller, specialized units rather than creating expansive, monolithic entities. Utilize Rust's crate and module system effectively to enforce boundaries.

**Rationale:** Directly embodies *Simplicity* and *Modularity*. Focused components are significantly easier to understand, develop, test independently (often as unit tests within the module or crate), deploy, and potentially replace. Composition provides flexibility and encourages reuse, leading to more resilient and adaptable systems. Rust's strong type system helps enforce these contracts at compile time.

**Implementation:** Think in terms of clear data flows and ownership transfer: inputs -> transformation -> outputs. Resist adding unrelated responsibilities to an existing module or struct; create a new, focused one instead. Define clear public APIs (`pub`) for crates and modules, keeping implementation details private. Use Cargo workspaces to manage multiple related crates.

---

## 2. Strict Separation of Concerns: Isolate the Core Logic

**Guideline:** Ruthlessly separate the application's core business logic and domain knowledge from its infrastructure concerns. Infrastructure includes anything related to external interactions: UI, database access (e.g., using `sqlx`, `diesel`), network calls (e.g., `reqwest`, `hyper`, `tonic`), file system operations (`std::fs`), FFI calls, third-party API integrations, etc. The core logic should remain "pure," unaware of the specific mechanisms used for I/O or external communication, and ideally contain no `unsafe` code.

**Rationale:** Paramount for *Modularity* and *Design for Testability*. The core business rules—the most valuable part of the application—can be developed and tested (using `#[cfg(test)]`) in complete isolation, without needing slow or complex setups. It allows infrastructure details to be changed (e.g., swapping a REST API for gRPC, changing database drivers) with minimal impact on the stable core logic. Isolating `unsafe` code to specific infrastructure modules improves overall system safety.

**Implementation:** Employ architectural patterns that enforce this separation, such as Hexagonal Architecture (Ports & Adapters) or Clean Architecture. The key is defining clear boundaries using Rust's `trait` system:
*   **Core Crate/Module(s):** Define `trait`s representing necessary collaborations (e.g., `UserRepository`, `OrderNotifier`). These traits specify the *what*, not the *how*. The core logic uses these traits generically or via trait objects.
    ```rust
    // In core crate (e.g., src/domain/orders.rs or its own crate `my_app_core`)
    pub trait OrderRepository {
        fn save(&self, order: &Order) -> Result<(), RepositoryError>;
        fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, RepositoryError>;
    }

    pub struct OrderService<R: OrderRepository> {
        repo: R,
    }

    impl<R: OrderRepository> OrderService<R> {
        pub fn new(repo: R) -> Self {
            Self { repo }
        }
        // ... methods using self.repo ...
    }
    ```
*   **Infrastructure Crate/Module(s):** Implement the core-defined `trait`s using specific technologies.
    ```rust
    // In infrastructure crate (e.g., `my_app_db_postgres`)
    use my_app_core::domain::orders::{Order, OrderId, OrderRepository, RepositoryError};
    use sqlx::PgPool;

    pub struct PostgresOrderRepository {
        pool: PgPool,
    }

    impl OrderRepository for PostgresOrderRepository {
        fn save(&self, order: &Order) -> Result<(), RepositoryError> {
            // ... implementation using self.pool ...
            todo!()
        }
        // ... other trait methods ...
    }
    ```

---

## 3. Dependency Inversion Principle: Point Dependencies Inward via Traits

**Guideline:** Ensure that high-level policy (core application/business logic) does not depend directly on low-level details (infrastructure). Instead, both should depend on abstractions (`trait`s defined by the core). Consequently, source code dependencies (in `Cargo.toml` and `use` statements) must always point *inwards*, from outer crates/modules (infrastructure) towards the central core logic. Infrastructure code implements traits defined *in* the core.

**Rationale:** This principle is the mechanism that enables effective *Separation of Concerns* and *Testability*. It decouples the stable, high-value core logic from volatile, low-level implementation details. This makes the system more flexible, maintainable, and robust against changes in external technologies or libraries. Rust's compile-time checks help enforce this.

**Implementation:**
*   **Core Crate(s):** Define `pub trait`s and core data structures (`struct`, `enum`). Core crates **never** depend on infrastructure crates in their `[dependencies]` section of `Cargo.toml`.
*   **Infrastructure Crate(s):** Depend on the core crate(s) to access and implement the defined `trait`s and use core types.
*   **Application/Binary Crate (`main.rs`):** Acts as the composition root. It depends on both core and infrastructure crates. It instantiates concrete infrastructure types (like `PostgresOrderRepository`) and injects them into core logic components (like `OrderService`) where traits are expected. This injection typically happens via:
    *   **Generic Parameters:** `OrderService::new(postgres_repo)` where `OrderService` is generic over `T: OrderRepository`. Offers static dispatch (performance).
    *   **Trait Objects:** `OrderService::new(Box::new(postgres_repo))` where `OrderService` holds a `Box<dyn OrderRepository>`. Offers dynamic dispatch (flexibility).

---

## 4. Crate and Module Organization: Structure by Feature/Domain

**Guideline:** Structure your codebase primarily around business features, domains, or capabilities, rather than grouping files solely by their technical type (e.g., `models`, `repositories`, `services`). Use Cargo workspaces and crates for coarse-grained separation (e.g., `core`, `web_api`, `database_impl`) and modules (`mod`) for finer-grained organization within crates.

**Rationale:** Enhances *Modularity* and *Maintainability*. Co-locating all code related to a specific feature (its domain logic, traits, potentially even specific infra implementations if simple) makes it easier to find, understand, and modify. It improves cohesion within feature modules/crates and reduces coupling between unrelated features. This structure simplifies refactoring or removing features.

**Implementation:**
*   **Cargo Workspaces:** For larger projects, use a workspace defined in the root `Cargo.toml` to manage multiple interdependent crates.
    ```toml
    # Root Cargo.toml
    [workspace]
    members = [
        "crates/my_app_core",
        "crates/my_app_infra_db",
        "crates/my_app_infra_web",
        "crates/my_app_cli",
        # ... other crates ...
    ]
    ```
*   **Crate Structure:**
    *   `crates/my_app_core`: Contains domain logic, core data types, and infrastructure traits. No dependencies on other workspace crates (except potentially shared utility crates).
    *   `crates/my_app_infra_*`: Implement traits from `core` using specific technologies (e.g., `my_app_infra_db` depends on `my_app_core` and `sqlx`).
    *   `crates/my_app_web` / `crates/my_app_cli`: Binary crates that act as composition roots, depend on `core` and necessary `infra` crates. Contain `main.rs`.
*   **Module Structure (within a crate):** Organize modules by feature.
    ```rust
    // Inside crates/my_app_core/src/lib.rs
    pub mod domain; // Contains submodules like orders, products, users
    pub mod services; // Contains application services orchestrating domain logic

    // Inside crates/my_app_core/src/domain/mod.rs
    pub mod orders;
    pub mod products;
    // ...

    // Inside crates/my_app_core/src/domain/orders/mod.rs
    pub mod model; // struct Order, OrderId, etc.
    pub mod repository; // trait OrderRepository, RepositoryError enum
    // Potentially: pub mod rules; // Specific business rules for orders
    ```

---

## 5. API Design: Leverage Types, Traits, and Define Clear Boundaries

**Guideline:** Whether designing internal APIs between Rust modules/crates or external APIs (REST, gRPC, FFI, CLIs), define clear, explicit, and robust contracts using Rust's type system. Precisely define expected inputs, outputs (especially `Result<T, E>`), behavior, invariants, and potential error conditions (`enum` error types). For external APIs, prioritize stability (`#[non_exhaustive]`), adopt versioning, and consider operational aspects. Pay special attention to FFI boundaries.

**Rationale:** Supports *Modularity* and *Explicit is Better than Implicit*. Rust's strong type system is a powerful tool for defining contracts. Well-defined APIs are essential for independent development, testing, and evolution. They reduce integration friction and serve as compile-time checked documentation.

**Implementation:**
*   **Internal APIs:** Use `pub trait`s, `pub struct`, `pub enum`, and clear function signatures. Leverage generics for flexibility and performance. Use the `Result<T, E>` type extensively for fallible operations, with specific error enums (`thiserror` helps). Keep interfaces minimal. Consider ownership patterns carefully (borrowing `&T`, mutable borrowing `&mut T`, moving `T`).
*   **Serialization Boundaries (e.g., REST/JSON):** Use `serde` with strongly-typed request/response structs. Define these DTO (Data Transfer Object) structs separately from core domain models if they diverge significantly.
*   **External gRPC APIs:** Define services rigorously using `.proto` files and use crates like `tonic-build` to generate Rust code.
*   **FFI Boundaries:** This requires extreme care.
    *   Define a dedicated module (e.g., `ffi`) for C-compatible interfaces.
    *   Use `extern "C"` function signatures.
    *   Use `#[repr(C)]` on structs/enums passed across the boundary.
    *   Carefully manage memory (e.g., functions to allocate/free Rust data from the C side, using `Box::into_raw` and `Box::from_raw`).
    *   Convert Rust `Result`s into C-style error codes or status indicators.
    *   Thoroughly document safety requirements and invariants. Mark FFI functions clearly as `unsafe` if they rely on the caller upholding contracts.
*   **CLIs:** Use crates like `clap` to define arguments, flags, and subcommands clearly. Provide help messages. Use specific exit codes (`std::process::exit`).

---

## 6. Configuration Management: Externalize and Type-Check

**Guideline:** Never hardcode configuration values—especially those varying between environments or containing secrets—directly in the source code. Externalize database connection strings, API endpoints, keys, ports, feature flags, etc. Load configuration at application startup.

**Rationale:** Crucial for *Maintainability*, deployment flexibility, and security. Externalized configuration allows the same compiled binary to run correctly in different environments. It keeps secrets out of version control. Rust's type system can ensure configuration is parsed correctly at startup.

**Implementation:** Prefer environment variables for deployment flexibility (containers). Use configuration files (e.g., TOML, YAML) for local development or complex setups. Employ crates like `config`, `figment`, or `serde` combined with `dotenvy` to load, parse, and validate configuration from multiple sources into strongly-typed Rust structs.

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub listen_address: String,
    pub database: DatabaseConfig,
    // ... other config sections
}

// In main.rs or setup code:
// Use the `config` crate or similar to load from files/env vars
// let config: AppConfig = load_config_somehow()?;
```

---

## 7. Leverage Rust's Error Handling: `Result` and Custom Errors

**Guideline:** Embrace Rust's standard error handling mechanisms: `Result<T, E>` for recoverable errors and `panic!` for unrecoverable programming errors/bugs. Propagate recoverable errors using `Result` and the `?` operator. Define custom, specific error types (typically enums using crates like `thiserror`) to provide context and allow programmatic handling. Avoid `unwrap()` or `expect()` in library or core logic code; handle potential errors explicitly. Establish clear error handling boundaries (e.g., in request handlers, `main` function) to log errors and convert them into appropriate responses (e.g., HTTP status codes, exit codes).

**Rationale:** Supports *Maintainability*, robustness, and operational clarity. Rust's `Result` makes error paths explicit and checked by the compiler. Consistent use of `Result` and `?` simplifies error propagation. Custom error types make debugging easier and enable callers to react differently to specific failures. Distinguishing `Result` from `panic!` clarifies intent.

**Implementation:**
*   **Return `Result<T, E>`:** Functions that can fail should return a `Result`.
*   **Use `?`:** Propagate errors concisely up the call stack.
*   **Define Custom Error Types:** Use enums, often deriving `thiserror::Error` for boilerplate implementation (`Display`, `Error` trait, `source()`).
    ```rust
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum OrderProcessingError {
        #[error("Order not found: {0}")]
        NotFound(String), // Include relevant context

        #[error("Invalid order state: {0}")]
        InvalidState(String),

        #[error("Repository error: {source}")] // Propagate underlying errors
        Repository { #[from] source: RepositoryError },

        #[error("Payment failed: {0}")]
        PaymentFailed(String),

        #[error("An unexpected error occurred")]
        Unexpected, // For truly unknown issues
    }

    // Function signature
    fn process_order(order_id: &str) -> Result<(), OrderProcessingError> {
        // ... logic using `?` ...
        Ok(())
    }
    ```
*   **Error Boundaries:** In `main.rs` or top-level handlers (e.g., web framework request handlers), match on the `Result` and handle errors appropriately (log, return HTTP error, exit).
*   **`panic!`:** Reserve for unrecoverable states indicating a bug (e.g., broken invariants, impossible conditions).
*   **`anyhow`:** Consider using the `anyhow` crate for error handling in application-level code (like `main.rs` or CLIs) where you primarily need to report errors rather than inspect specific types, simplifying error type conversions. Use specific error types (`thiserror`) in library/core logic.

---

## 8. Concurrency and Asynchronicity: Design for Safety and Performance

**Guideline:** Leverage Rust's strong concurrency safety guarantees. Choose appropriate concurrency models: `async/await` (with runtimes like Tokio or async-std) for I/O-bound tasks, standard threads (`std::thread`) for CPU-bound tasks, or channel-based communication (`std::sync::mpsc`, `crossbeam-channel`, `tokio::sync::mpsc`) for message passing. Use synchronization primitives like `Arc<Mutex<T>>` or `Arc<RwLock<T>>` judiciously for shared mutable state, being mindful of potential deadlocks and performance implications. Structure code to minimize contention and maximize parallelism where beneficial.

**Rationale:** Rust's ownership and borrowing system prevents data races at compile time, a major source of concurrency bugs. Choosing the right model improves performance and responsiveness. Explicit architectural separation (e.g., isolating I/O) often simplifies concurrent design.

**Implementation:**
*   **`async/await`:** Preferred for network services, database interactions, and other I/O-heavy workloads. Ensure functions interacting with async resources are `async fn`. Be mindful of `.await` points and potential blocking within async tasks.
*   **Threading:** Use for computationally intensive tasks that can be parallelized. Consider thread pools (e.g., `rayon`) for managing CPU-bound work efficiently.
*   **Channels:** Excellent for decoupling components and communicating between threads or async tasks.
*   **Shared State:** Minimize shared mutable state. When necessary, use `Arc` for shared ownership and `Mutex` or `RwLock` for interior mutability, ensuring locks are held for the shortest duration possible. Consider lock-free data structures for high-contention scenarios if performance profiling justifies the complexity.
*   **Architecture:** Design components with concurrency in mind. For example, ensure repository traits can be implemented efficiently in an async context (`async_trait` crate might be needed for traits with async methods until async fn in traits is stabilized).

---

By adhering to these Rust-specific guidelines, we aim to build systems that are not only robust, maintainable, and testable but also fully leverage the unique safety, performance, and concurrency features offered by the Rust language and ecosystem.