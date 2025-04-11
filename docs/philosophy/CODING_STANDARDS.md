# Rust Coding Standards

This document defines the concrete rules, conventions, and best practices for writing Rust code day-to-day in our projects. Adherence to these standards is essential for creating code that is readable, consistent, predictable, maintainable, safe, and performant. These standards directly support our Core Principles, particularly aiming for Simplicity, Maintainability, Explicit code, and leveraging Rust's safety guarantees.

---

## 1. Maximize Language Strictness: Catch Errors Early

**Standard:** Leverage Rust's compiler (`rustc`) and the `clippy` linter to their fullest potential. Configure `clippy` with strict settings to catch potential errors, inconsistencies, performance pitfalls, and non-idiomatic patterns at compile-time or lint-time.

**Rationale:** Rust's compiler is already strict, preventing many classes of bugs by default (e.g., data races, use-after-free). `clippy` provides hundreds of additional lints beyond the compiler's checks. Catching issues early drastically reduces debugging time and improves code quality. This supports the *Explicit is Better than Implicit* principle by making assumptions visible and verifiable by the compiler.

**Implementation:**
*   **Compiler:** Rely on `cargo check` for fast feedback during development and `cargo build` for full compilation. Address all compiler warnings; treat them as errors in CI environments (`-D warnings`).
*   **Clippy:** Integrate `clippy` into the development workflow and CI pipeline. Use a shared `clippy.toml` configuration (or configure via `Cargo.toml` or source attributes) checked into version control. Start with a strict default lint level (e.g., `#![deny(clippy::all)]` or `#![deny(warnings)]` at the crate root) and explicitly `#[allow(...)]` specific lints *with justification* only when necessary.
*   Run `cargo clippy -- -D warnings` in CI to enforce compliance.

---

## 2. Leverage Rust's Type System Diligently: Express Intent Clearly

**Standard:** Utilize Rust's strong static type system fully and precisely. Model data using `struct` and `enum`. Define clear function signatures using specific types, generics, and traits. Employ `Option<T>` for optional values and `Result<T, E>` for fallible operations.

**Rationale:** Types serve as invaluable, machine-checked documentation. They improve code clarity, eliminate entire classes of runtime errors (null pointer exceptions, type mismatches), enable powerful compiler optimizations, and facilitate safe refactoring via compiler guarantees. This directly supports *Explicit is Better than Implicit*.

**Implementation:**
*   Define custom types (`struct`, `enum`) for domain concepts. Use tuple structs or unit-like structs where appropriate. Leverage pattern matching on enums for exhaustive handling of variants.
*   Use generics (`<T>`) and traits to write reusable, abstract code while maintaining type safety. Prefer static dispatch (generics/`impl Trait`) over dynamic dispatch (`dyn Trait`) unless runtime polymorphism is explicitly required, as static dispatch is generally more performant.
*   **Embrace `Option<T>` and `Result<T, E>`:** Never simulate null values. Model absence with `Option`. Model recoverable errors with `Result`. Propagate errors concisely using the `?` operator. Define specific, meaningful error types (often enums implementing `std::error::Error`).

    ```rust
    // Good: Clear intent, handles absence and errors idiomatically
    #[derive(Debug)]
    struct User { id: u32, name: String }
    #[derive(Debug, thiserror::Error)] // Using thiserror for convenience
    enum ProcessingError {
        #[error("User not found: {0}")]
        UserNotFound(u32),
        #[error("Invalid data format")]
        InvalidFormat,
        #[error("Database error: {0}")]
        Database(#[from] sqlx::Error), // Example: Propagating underlying errors
    }

    fn find_user(id: u32) -> Option<User> {
        // ... database lookup logic ...
        if id == 1 { Some(User { id: 1, name: "Alice".to_string() }) } else { None }
    }

    fn process_user_data(user: &User) -> Result<(), ProcessingError> {
        if user.name.is_empty() {
            return Err(ProcessingError::InvalidFormat);
        }
        // ... processing logic that might fail ...
        Ok(())
    }

    fn run(user_id: u32) -> Result<(), ProcessingError> {
        let user = find_user(user_id)
            .ok_or(ProcessingError::UserNotFound(user_id))?; // Use Option::ok_or
        process_user_data(&user)?; // Propagate Result errors with ?
        println!("Successfully processed user: {:?}", user);
        Ok(())
    }
    ```
*   **Avoid panics in library code.** Use `Result` for recoverable errors. Panics should typically be reserved for unrecoverable states or programming errors detected at runtime (e.g., assertion failures in tests, index out of bounds if logic guarantees validity, critical initialization failures). Use methods like `expect()` judiciously, primarily when a condition *should* logically never fail based on program invariants.
*   Prefer borrowing (`&T`, `&mut T`) over taking ownership (`T`) in function arguments when full ownership is not required. This increases flexibility for the caller and often improves performance by avoiding unnecessary clones or moves.

---

## 3. Prefer Immutability: Simplify State Management

**Standard:** Leverage Rust's default immutability. Declare variables with `let` unless mutability is explicitly required, in which case use `let mut`. Prefer data structures that encourage immutable operations where practical.

**Rationale:** Immutability, enforced by Rust's ownership and borrowing rules, significantly simplifies reasoning about program state, as data doesn't change unexpectedly. It eliminates data races at compile time and makes code easier to understand, especially in concurrent scenarios. Supports *Simplicity*.

**Implementation:**
*   Default to `let` bindings. Use `let mut` only when mutation is necessary and within the smallest practical scope.
*   Understand the difference between `Copy` types (implicitly copied) and `Clone` types (explicitly cloned).
*   When updating data structures (like `Vec` or `HashMap`), be mindful of ownership. If immutable updates are desired, create new instances (e.g., using iterator methods like `map`, `filter`, which produce new collections) rather than mutating in place, though performance implications should be considered for large structures.
*   Use types from immutable collection libraries (like `im` or `rpds`) if complex persistent data structures are needed, though often Rust's ownership model provides sufficient guarantees without them.

---

## 4. Favor Pure Functions & Isolate Side Effects: Enhance Predictability

**Standard:** Strive to implement core logic, data transformations, and calculations as functions that primarily operate on their inputs without causing external side effects (I/O, mutating global state, etc.). Concentrate necessary side effects (interacting with databases, file systems, networks, FFI) at the boundaries of your application or library.

**Rationale:** Functions with limited or no side effects are easier to reason about, test in isolation (often without complex setup or mocking), reuse, and parallelize. Rust's type system (lifetimes, `Send`/`Sync` traits) helps manage state and concurrency, supporting this principle. Supports *Simplicity*, *Modularity*, and *Testability*.

**Implementation:**
*   Actively identify opportunities to extract pure computational logic from functions that mix calculations with I/O or state mutation.
*   Pass dependencies (like database connections, configuration) explicitly as arguments or struct fields rather than relying on global statics or implicit context.
*   Leverage traits to abstract away side effects for easier testing (e.g., define a `DatabaseReader` trait and provide both a real implementation and a mock/test implementation).

---

## 5. Meaningful Naming: Communicate Purpose

**Standard:** Choose clear, descriptive, and unambiguous names adhering strictly to Rust's established naming conventions. Names should effectively communicate the entity's purpose, scope, and usage.

**Rationale:** Code is read far more often than it's written. Meaningful names are crucial for *Maintainability* and readability, reducing the need for explanatory comments and lowering cognitive load. Consistent conventions improve predictability. Supports *Self-Documenting Code*.

**Implementation:**
*   **`snake_case`**: Variables, function names, method names, module names, crate names (generally).
*   **`PascalCase`**: Type names (`struct`, `enum`, `trait`), enum variants, type aliases.
*   **`SCREAMING_SNAKE_CASE`**: Constants (`const`) and statics (`static`).
*   Follow the Rust API Guidelines ([https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/)) for naming conventions and API design principles (e.g., methods for conversions like `as_`, `to_`, `into_`).
*   Avoid vague terms like `data`, `info`, `temp`, `handle`, `process` unless the scope is extremely limited and context makes the meaning obvious. Use domain-specific terminology where appropriate.
*   Module names should clearly indicate their contents.

---

## 6. Mandatory Code Formatting: Ensure Consistency

**Standard:** All Rust code committed to the repository *must* be automatically formatted using `rustfmt`. Formatting style is not subject to individual preference.

**Rationale:** Enforces a consistent visual style across the entire codebase, drastically improving readability and reducing cognitive friction. Eliminates time wasted on style debates in code reviews. Ensures code diffs only show substantive logical changes. Supports *Maintainability* and *Automation*.

**Implementation:**
*   **`rustfmt`** is mandatory. Install via `rustup component add rustfmt`.
*   Configure `rustfmt` via a `rustfmt.toml` file (or `.rustfmt.toml`) checked into the repository root if non-default settings are required (though sticking to defaults is often preferred).
*   Run `cargo fmt` to format the entire project.
*   Integrate `cargo fmt --check` into pre-commit hooks and the CI pipeline to enforce compliance automatically.

---

## 7. Mandatory Linting: Catch Problems Proactively

**Standard:** All Rust code committed to the repository *must* pass analysis by `clippy` using the strict, shared configuration defined for the project.

**Rationale:** `clippy` acts as an automated code reviewer, identifying potential bugs, anti-patterns, stylistic inconsistencies, performance issues, and deviations from idiomatic Rust *before* they are manually reviewed or merged. This improves overall code quality, consistency, safety, and *Maintainability*. Supports *Automation*.

**Implementation:**
*   **`clippy`** is mandatory. Install via `rustup component add clippy`.
*   Configure `clippy` via `clippy.toml`, `Cargo.toml`, or source attributes (`#![deny(...)]`, `#![warn(...)]`). Ensure this configuration is shared and version-controlled. Aim for a strict default (e.g., denying most warnings).
*   Run `cargo clippy` regularly during development.
*   Integrate `cargo clippy -- -D warnings` (or specific denied lints) into pre-commit hooks and the CI pipeline, treating violations as build failures.

---

## 8. Address Violations, Don't Suppress: Fix the Root Cause

**Standard:** Avoid using `#[allow(...)]` attributes to suppress compiler or `clippy` warnings/lints. Investigate the underlying issue and refactor the code to satisfy the check correctly and idiomatically. Use of `unsafe` blocks requires extreme caution and justification.

**Rationale:** Suppression mechanisms often hide genuine bugs, technical debt, non-idiomatic code, or safety issues. Bypassing these checks defeats their purpose and degrades code quality and safety over time. Fixing the root cause leads to more robust, maintainable, and understandable code. `unsafe` bypasses Rust's core safety guarantees and must be treated with exceptional care. Supports *Maintainability* and *Explicit is Better than Implicit*.

**Implementation:**
*   Legitimate exceptions for `#[allow(...)]` are rare and require an explicit code comment justifying *why* the suppression is necessary and safe in that specific context.
*   **`unsafe` Code:**
    *   Minimize the use of `unsafe` to the absolute smallest scope necessary.
    *   Every `unsafe` block *must* be accompanied by a comment explaining *why* it is necessary and *what invariants* must be upheld by the surrounding safe code to ensure the `unsafe` operations are actually sound.
    *   Prefer creating safe abstractions around `unsafe` blocks rather than exposing `unsafe` in public APIs.

---

## 9. Purposeful Comments: Explain the *Why* and the *Unsafe*

**Standard:** Write comments primarily to explain the *intent*, *rationale*, or *context* behind non-obvious code. Focus on the "why," not the "what" or "how"—idiomatic Rust should clearly express the mechanics. Use Rustdoc comments (`///`, `//!`) for documenting public APIs (crates, modules, functions, types, traits, methods). Document the soundness invariants for all `unsafe` blocks.

**Rationale:** Well-written, idiomatic Rust often minimizes the need for implementation comments. Valuable comments provide context the code cannot, such as design trade-offs or links to requirements. Rustdoc comments are essential for usability and discoverability. Documenting `unsafe` is critical for safety audits and maintenance. Supports *Self-Documenting Code* and *Maintainability*.

**Implementation:**
*   Before writing a comment to explain *how* code works, try refactoring for clarity.
*   Use line comments (`//`) for implementation details within function bodies where necessary.
*   Use doc comments (`///` for items, `//!` for modules/crates) for anything exposed publicly or intended for library use. Write clear, concise summaries and include examples where helpful (````rust ... ```` blocks in doc comments are tested by `cargo test`).
*   **Crucially, document the justification and required invariants for every `unsafe` block.**
*   Delete commented-out code; rely on version control history.

---

## 10. Disciplined Dependency Management: Keep It Lean and Secure

**Standard:** Minimize the number of third-party dependencies (`crates`) declared in `Cargo.toml`. Each external dependency adds compile time, potential security risks, maintenance overhead, and licensing considerations. Keep essential dependencies reasonably up-to-date and audit them for security vulnerabilities.

**Rationale:** Fewer dependencies result in a smaller attack surface, faster build times (especially clean builds), reduced binary size, less potential for version conflicts, and easier long-term maintenance. Supports *Simplicity* and *Maintainability*.

**Implementation:**
*   Thoroughly evaluate the necessity and value before adding a new dependency. Can the functionality be achieved reasonably with the standard library, existing dependencies, or a small amount of custom code?
*   Prefer dependencies that are well-maintained, widely used, have clear documentation, compatible licenses (check `cargo-deny` or `cargo-license`), and minimal transitive dependencies. Check crates.io stats, repository activity, and community trust signals.
*   Use `cargo audit` (install via `cargo install cargo-audit`) regularly and in CI to check for known security vulnerabilities in the dependency tree.
*   Keep `Cargo.toml` tidy. Use features to enable optional dependencies where appropriate (`[features]`, `[dependencies.some_crate.optional]`). Run `cargo update` periodically to update dependencies according to semantic versioning rules specified in `Cargo.toml`. Consider tools like Dependabot or Renovate Bot for automated update PRs.
*   Review `Cargo.lock` changes carefully during code reviews.

---

## 11. Use Macros Judiciously: Prioritize Clarity

**Standard:** Employ macros (declarative and procedural) when they provide significant improvements in code conciseness or capability (e.g., DSLs, boilerplate reduction via `derive`). However, prioritize code clarity and debuggability. Avoid overly complex or "magical" macros that obscure control flow or data transformation.

**Rationale:** Macros are a powerful feature of Rust but can decrease readability and make debugging harder if overused or poorly implemented. Simple, well-documented macros (like common derives) are beneficial; overly complex ones can hinder maintainability. Supports *Simplicity* and *Maintainability*.

**Implementation:**
*   Prefer functions and traits over macros when they can achieve the same result with comparable ergonomics and sufficient clarity.
*   When writing macros (especially procedural macros), document their usage and behavior thoroughly.
*   Be mindful of the impact of complex macros on compile times.

---

## 12. Leverage Built-in Testing: Ensure Correctness

**Standard:** Write unit and integration tests using Rust's built-in testing framework (`#[test]`, `cargo test`). Aim for good test coverage of public APIs and critical logic paths. Use documentation tests (`/// ```rust`) to ensure examples are correct and APIs are usable.

**Rationale:** Automated tests are crucial for verifying correctness, preventing regressions, and enabling confident refactoring. Rust's integrated testing support makes writing and running tests straightforward. Supports *Maintainability* and *Reliability*.

**Implementation:**
*   Place unit tests in `#[cfg(test)]` modules within the same file or in a `tests` subdirectory alongside the code they test (`src/my_module/tests.rs`).
*   Place integration tests in the `tests/` directory at the crate root. These test the crate's public API as an external user would.
*   Write documentation tests within `///` comments to provide usage examples that are automatically verified.
*   Use `assert!`, `assert_eq!`, `assert_ne!`, and `#[should_panic]` appropriately.
*   Consider property-based testing (e.g., with the `proptest` crate) for more robust testing of complex logic.
*   Ensure tests are run as part of the CI pipeline using `cargo test --all-features`.

---

Adherence to these standards fosters a codebase that is not only functional but also safe, maintainable, efficient, and a pleasure to work with, fully leveraging the strengths of the Rust language and ecosystem.