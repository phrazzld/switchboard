# Core Principles (Rust Edition)

This document outlines the fundamental beliefs and guiding principles that shape our approach to Rust software design, development, and maintenance. These principles are the bedrock upon which our more specific guidelines for architecture, coding patterns, testing, and documentation are built. Adhering to these principles helps us create Rust software that is simple, robust, maintainable, safe, and performant.

---

## 1. Simplicity First: Complexity is the Enemy

**Principle:** Always seek the simplest possible solution in Rust that correctly meets the requirements. Actively resist complexity – unnecessary features, premature or overly-abstract generics/traits, complex lifetime annotations where simpler ownership suffices, overly clever macros, or convoluted layers of indirection. Leverage Rust's type system for clarity, not obfuscation.

**Rationale:** Simplicity is the ultimate sophistication. Simple Rust code is drastically easier to understand, reason about (especially concerning ownership and borrowing), debug, test, modify, and maintain. Complexity, even enabled by Rust's powerful features, remains the primary source of bugs, development friction, and long-term maintenance costs.

**Implications:** We rigorously apply principles like YAGNI (You Ain't Gonna Need It). We question the value proposition of every added crate, trait bound, or generic parameter. We favor straightforward, readable code using idiomatic Rust patterns over compact but obscure implementations. Simplicity is a constant goal, achieved by carefully considering Rust's features, not just avoiding them. Avoid `unsafe` unless absolutely necessary and justified.

---

## 2. Modularity is Mandatory: Crates, Modules, and Traits

**Principle:** Construct software from small, well-defined, independent components – primarily crates and modules – with clear responsibilities and explicit public APIs (`pub` items). Define interfaces using traits where polymorphism or abstraction is needed. Strive for high internal cohesion (logic within a module/crate is strongly related) and low external coupling (components depend only on the public APIs of others, not their internal details). Embrace the Unix philosophy: build focused crates/modules that perform a single task well, and compose them.

**Rationale:** Modularity tames complexity in Rust projects. It enables parallel development (e.g., using Cargo workspaces), allows for independent testing (`cargo test`) and compilation, facilitates code reuse (publishing crates), improves fault isolation, and makes the overall system easier to understand and evolve. It is essential for building scalable and maintainable Rust applications.

**Implications:** This demands careful attention to API design (`pub` visibility) and boundary definition between modules and crates. Traits are the primary mechanism for defining contracts and enabling dependency inversion, promoting loose coupling. We enforce a strong separation of concerns. Our architectural guidelines leverage Rust's module system and crate structure.

---

## 3. Design for Testability: Confidence Through Verification

**Principle:** Testability is a fundamental, non-negotiable design constraint considered from the beginning. Structure Rust code—using techniques like clear public APIs (`pub`), trait-based dependency injection where appropriate, and separation of concerns—so that its behavior can be easily and reliably verified through automated tests (`cargo test`). Focus tests on *what* the code achieves (its public API, its behavior, its contracts via types and traits), not *how* it achieves it (internal implementation details).

**Rationale:** Automated tests (unit, integration, documentation tests) are crucial for building confidence, preventing regressions (especially memory safety regressions caught by the compiler, but also logic errors), enabling safe refactoring, and acting as precise, executable documentation (`cargo doc --open`). Code that is inherently difficult to test without complex setup or mocking internal details often indicates poor design (e.g., high coupling, mixed concerns, overly concrete dependencies).

**Implications:** Testability requirements directly influence architectural choices. We leverage Rust's built-in testing framework. We often favor integration tests (in the `tests/` directory) verifying component interactions through public APIs. Difficulty in testing is a strong signal to refactor the *code under test* first. `unsafe` code requires particularly rigorous testing to validate its claimed invariants.

---

## 4. Maintainability Over Premature Optimization: Idiomatic Rust First

**Principle:** Write code primarily for human understanding and ease of future modification. Clarity, readability, and consistency using idiomatic Rust patterns are paramount. Aggressively resist the urge to optimize code for performance (e.g., using `unsafe`, complex low-level tricks, or non-standard data structures) before identifying *actual*, *measured* performance bottlenecks using profiling tools in realistic scenarios. Trust the Rust compiler's optimizations first.

**Rationale:** The vast majority of software development time is spent reading and maintaining existing Rust code. While Rust offers high performance potential, premature optimization often introduces significant complexity (especially with `unsafe` or intricate lifetimes), obscures the original intent, makes the code harder to debug and modify, violates safety guarantees, and frequently targets non-critical performance paths, yielding negligible real-world benefit while incurring high maintenance and safety risks.

**Implications:** We prioritize clear, descriptive naming and leverage Rust's strong type system (`enum`, `struct`, `Result`, `Option`). We enforce consistent code style using `rustfmt`. We favor straightforward algorithms and standard library data structures unless profiling proves a more complex or `unsafe` approach is necessary *and* justified. Optimization is a targeted activity driven by data, not speculation, and the safety implications of using `unsafe` must be carefully considered and documented.

---

## 5. Explicit is Better than Implicit: Leverage Rust's Strengths

**Principle:** Make dependencies, data flow (ownership/borrowing), control flow (e.g., `match`, `if let`), contracts (traits, types), error handling (`Result`, `panic!`), and side effects as clear and obvious as possible within the code. Avoid "magic" behavior. Leverage Rust's strong static typing, explicit ownership model, and comprehensive standard library types (`Option`, `Result`) to enforce clarity at compile time.

**Rationale:** Explicit code is easier to understand, reason about (especially regarding memory safety and concurrency), debug, and refactor safely. Rust's design philosophy heavily favors explicitness (e.g., `mut`, explicit lifetimes where needed, `Result` for error handling). Implicit behavior obscures dependencies and control flow, making code harder to follow and potentially hiding bugs or unexpected side effects.

**Implications:** We rely heavily on Rust's type system, including `Result` for recoverable errors and `panic!` only for unrecoverable programmer errors. We prefer passing explicit dependencies (e.g., function arguments, struct fields) over implicit context or global state. Function signatures should accurately reflect dependencies, borrowing requirements, and potential failure modes (`-> Result<T, E>`). Avoid overly complex macros that hide control flow or dependencies.

---

## 6. Automate Everything: Harness the Cargo Ecosystem

**Principle:** Automate every feasible repetitive task in the Rust development lifecycle using `cargo` and related tooling. This includes, but is not limited to: running tests (`cargo test`), linting for style and potential errors (`cargo clippy`), formatting code (`cargo fmt`), building artifacts (`cargo build`), generating documentation (`cargo doc`), managing dependencies (`Cargo.toml`, `cargo update`), and deploying applications (via CI/CD scripts invoking `cargo`).

**Rationale:** Automation via `cargo` and CI/CD drastically reduces the potential for manual error, ensures consistency across developers and environments, frees up developer time, provides faster feedback loops (compile-time checks, test results), and makes processes repeatable and reliable. The Rust ecosystem is built around this tooling.

**Implications:** This requires an upfront and ongoing investment in robust CI/CD pipelines (e.g., GitHub Actions using `cargo test`, `cargo fmt --check`, `cargo clippy -- -D warnings`). We standardize on using `rustfmt` and `clippy` configurations. The goal is to make the correct, idiomatic, and safe way the easy way through tooling.

---

## 7. Document Decisions and Safety, Not Just Mechanics: Explain the *Why* and the *Invariant*

**Principle:** Strive for code that is largely self-documenting through clear naming, logical structure, idiomatic patterns, and effective use of Rust's type system – this covers the *how*. Use Rustdoc (`///`, `//!`) primarily for documenting the public API – the *what* and usage examples. Reserve comments (`//`) and external documentation (like ADRs) primarily for explaining the *why*: the rationale behind a non-obvious design choice, the context surrounding complex logic, critical constraints, or trade-offs considered. Crucially, **any use of `unsafe` MUST be accompanied by comments explaining the invariants the programmer is upholding that the compiler cannot verify.**

**Rationale:** The mechanics of code change frequently. Rustdoc ensures API documentation stays synchronized via `cargo doc`. The *reasoning* behind a design or the *safety invariants* of an `unsafe` block provide enduring value for maintainers. Self-documenting code reduces the burden of keeping separate documentation synchronized. Missing `unsafe` justifications is a critical documentation failure.

**Implications:** We prioritize writing clean, expressive, idiomatic Rust. Rustdoc comments are mandatory for public APIs (`pub` items). Inline comments explain intent or complex local logic. `unsafe` blocks require detailed `// SAFETY:` comments explaining why the code is safe. Architecture Decision Records (ADRs) track significant design choices and their justifications.