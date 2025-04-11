# Rust Documentation Approach

This document outlines our philosophy and practices for documenting Rust projects. Our goal is effective communication and knowledge sharing—whether with team members, future contributors, or our future selves—achieved with minimal friction and maximum clarity. We prioritize documentation methods that are tightly integrated with the Rust ecosystem, leveraging `rustdoc` and standard conventions. This aligns with the Core Principles of *Explicit is Better than Implicit* and *Document Decisions, Not Mechanics*.

---

## 1. Prioritize Self-Documenting Code and Idiomatic Rust

**Approach:** The codebase itself is the primary and most accurate source of documentation regarding *how* the system works. We achieve this through idiomatic Rust practices:
*   **Clear Naming:** Adhering strictly to Rust naming conventions (e.g., `snake_case` for functions/variables, `PascalCase` for types/traits/enums). Use `clippy` to help enforce conventions.
*   **Strong Typing & Expressive Types:** Leveraging Rust's powerful type system, including `enum`s, `struct`s, generics, and especially `Option<T>` and `Result<T, E>` to make contracts and potential failure states explicit.
*   **Logical Structure:** Organizing code into small, focused modules (`mod`) within crates, using visibility (`pub`, `pub(crate)`, etc.) effectively to define clear public APIs (see `ARCHITECTURE_GUIDELINES.md`). Consider workspaces for larger projects.
*   **Readability:** Following consistent formatting using `rustfmt`, writing straightforward logic, and utilizing standard Rust idioms identified by `clippy`.
*   **Well-Written Tests:** Unit tests (`#[test]`), integration tests (in `/tests`), and documentation tests serve as executable examples of how components are intended to be used (see `TESTING_STRATEGY.md`).

**Rationale:** Code is the ultimate source of truth. Relying heavily on external documentation for low-level mechanics creates a significant synchronization burden. Well-factored, idiomatic Rust code minimizes the need for supplementary explanation of *what* it does. `Option`/`Result` explicitly document potential absence or failure.

**Implementation:** Before writing extensive comments or external documents explaining code mechanics, always ask: "Can I refactor the code itself—using clearer names, types, or structure—to make its purpose and operation clearer?"

---

## 2. README.md: The Essential Crate Entry Point

**Approach:** Every crate *must* have a `README.md` file at its root. This file is the front door, providing essential context and practical instructions. It must be kept concise and up-to-date.

**Standard Structure:**
*   **Crate Name & Title:** Clear and prominent.
*   **Brief Description:** Concisely explain the crate's purpose and what problem it solves (1-3 sentences). Often mirrors the `[package]` description in `Cargo.toml`.
*   **Status Badges:** (Recommended) crates.io version, docs.rs build status, CI build status, test coverage, license.
*   **Getting Started / Setup:**
    *   Minimum prerequisites (e.g., Rust toolchain version via `rustup`).
    *   Instructions for adding the crate as a dependency (`cargo add crate-name` or `Cargo.toml` entry).
    *   Instructions for building the project if it's a binary or workspace (`cargo build --release`).
    *   Any necessary environment setup (e.g., required environment variables, config files).
*   **Running Tests:** Clear command(s) to execute the full test suite (`cargo test`).
*   **Usage / Running the Application:**
    *   Basic usage examples (often linking to crate/module documentation or `/examples`).
    *   For binaries: How to run (`cargo run --bin <n> -- <args>`, `target/release/<n> <args>`).
    *   Reference the `/examples` directory if present.
*   **Key Cargo Commands:** Reference common commands like `cargo check`, `cargo fmt`, `cargo clippy`, `cargo test`, `cargo build`, `cargo run`, `cargo doc --open`.
*   **Architecture Overview (Optional):** A brief description or link to key architectural documents (like ADRs or module-level docs) if helpful for orientation.
*   **How to Contribute:** Link to contribution guidelines, code of conduct, and the expected Pull Request process.
*   **License:** Specify the project's software license (should match `Cargo.toml`).

**Rationale:** A well-structured README significantly lowers the barrier to entry, enabling others (and future you) to quickly understand, set up, use, test, and contribute to the crate.

---

## 3. Documentation Comments (`///`, `//!`): Explaining Intent and Public API

**Approach:** Use Rust's built-in documentation comments (`///` for outer documentation, `//!` for inner/module/crate documentation) to explain the *why* and the *purpose* of public API items. Adhere strictly to the commenting philosophy: comments explain *intent* and *context*, not trivial mechanics.

*   **`///` (Outer Doc Comments):** Use for documenting items like functions, structs, enums, traits, modules, etc., *from the outside*. Placed *before* the item.
*   **`//!` (Inner Doc Comments):** Use for documenting the enclosing item, typically the *crate* (in `lib.rs` or `main.rs`) or a *module* (in `mod.rs` or `module_name.rs`). Placed *inside* the item, usually at the top of the file.
*   **Regular Comments (`//`):** Use sparingly *inside* function bodies or private code sections to clarify non-obvious logic, workarounds, or the rationale for a specific implementation choice (`// WHY: ...`). Avoid comments that merely paraphrase the code.
*   **`// SAFETY:` Comments:** Mandatory for documenting the invariants and requirements callers must uphold when calling `unsafe fn` or interacting with `unsafe` blocks. Explain *why* the code is safe under those conditions.
*   **`// TODO:` / `// FIXME:`:** Use standard markers for tracking future work or known issues.

**Rationale:** `rustdoc` processes `///` and `//!` comments (which support Markdown) to generate high-quality, navigable HTML documentation. This keeps API documentation tightly coupled with the code. Good comments provide insight; redundant comments create noise. `// SAFETY:` comments are crucial for maintaining soundness in `unsafe` contexts.

**Implementation:**
*   **Document all `pub` items.** This is the contract your crate provides.
*   Explain the item's purpose, its parameters (if any), return values (especially `Result`/`Option` variants), potential panics (`# Panics` section), error conditions (`# Errors` section), and safety considerations (`# Safety` section for `unsafe fn`).
*   Use Markdown for formatting, including code blocks (see Section 4).
*   If code requires extensive `//` comments to explain its mechanics, refactor it for clarity first.
*   **Delete commented-out code.** Use version control history.

**Example (`///`):**

```rust
/// Calculates the square root of a number.
///
/// Returns `None` if `n` is negative, as the square root of a negative
/// number is not a real number.
///
/// # Examples
///
/// ```
/// let root = my_crate::sqrt(9.0);
/// assert_eq!(root, Some(3.0));
///
/// let imaginary = my_crate::sqrt(-4.0);
/// assert!(imaginary.is_none());
/// ```
///
/// # Panics
///
/// This function does not panic. // Explicitly state if it doesn't panic.
pub fn sqrt(n: f64) -> Option<f64> {
    if n < 0.0 {
        None
    } else {
        Some(n.sqrt())
    }
}
```

**Example (`//!` in `lib.rs`):**

```rust
//! # My Awesome Crate
//!
//! `my_awesome_crate` provides utility functions for doing
//! amazing things. This crate aims to be simple, fast, and reliable.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! my_awesome_crate = "0.1.0"
//! ```
//!
//! And then use it in your code:
//! ```rust
//! use my_awesome_crate::do_stuff;
//!
//! fn main() {
//!     do_stuff();
//! }
//! ```

// ... rest of lib.rs content
pub mod utils;
pub use utils::do_stuff;
```

---

## 4. `rustdoc`: Generating Comprehensive API Documentation

**Approach:** Leverage `rustdoc` as the primary tool for generating and consuming API documentation. Write documentation comments with `rustdoc` features in mind.

*   **Crate and Module Documentation (`//!`):** Provide high-level overviews, usage examples, and explanations of the crate's or module's purpose and structure using inner doc comments (`//!`) at the top of `lib.rs`/`main.rs` or module files.
*   **Item Documentation (`///`):** Document every public (`pub`) function, struct, enum, trait, type alias, and constant using outer doc comments (`///`). Explain purpose, parameters, returns, errors (`# Errors`), panics (`# Panics`), safety requirements (`# Safety`), and provide examples (`# Examples`).
*   **Examples (`# Examples` and `/examples`):**
    *   Include concise, runnable examples directly within `///` documentation blocks using Markdown code fences (```rust ... ```). These serve as documentation tests (`doctests`).
    *   For more complex scenarios, create standalone example programs in the `/examples` directory at the crate root. `cargo run --example <n>` executes them. Reference these from doc comments or the README.
*   **Documentation Tests (`doctests`):** Ensure examples in doc comments are correct and stay up-to-date by running `cargo test`, which automatically executes them. Hide setup code with `#` prefix if needed (e.g., `# let my_var = setup();`).
*   **Documenting Traits and Implementations:** Document traits thoroughly. For `impl Trait for Type`, document specific implementation details or behaviors if they differ significantly from the trait's general contract or have notable performance characteristics. `rustdoc` automatically links implementations back to the trait and type.
*   **Documenting `unsafe` Code:** Use the `# Safety` section in `rustdoc` comments for `unsafe fn` to explain the conditions under which the function is safe to call. Use `// SAFETY:` comments within `unsafe` blocks to justify why the operation is sound.
*   **Error Handling (`Result`/`Option`):** Clearly document the meaning of `Ok` and `Err` variants in the `# Errors` section for functions returning `Result`. Document the meaning of `Some` and `None` for functions returning `Option`.
*   **Linking:** Use `rustdoc`'s automatic linking or explicit intra-doc links (`[path::to::Item]`) to connect related parts of the API documentation.
*   **API Guidelines:** Refer to the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) for best practices on API design and documentation conventions.

**Rationale:** `rustdoc` provides a standardized, high-quality, navigable, and testable way to document Rust APIs. Keeping documentation close to the code (`///`, `//!`) and making it executable (`doctests`) significantly reduces drift and improves reliability.

**Implementation:**
*   Run `cargo doc --open` frequently during development to preview documentation.
*   Ensure `cargo test` passes (including doctests).
*   Publish documentation to `docs.rs` for library crates by default upon publishing to `crates.io`.
*   For external APIs (REST, gRPC):
    *   **REST APIs:** Maintain an OpenAPI spec. Consider Rust crates like `utoipa` or `aide` to generate specs from code annotations/types.
    *   **gRPC APIs:** The `.proto` files are the contract. Use comments within `.proto`. Leverage Rust crates like `tonic`.

---

## 5. Architecture Decision Records (ADRs): Capturing Design Rationale

**Approach:** **ADRs are mandatory for recording significant architectural or design decisions.** This includes choices about major dependencies (e.g., async runtime, web framework, database client), structural patterns, cross-cutting concerns, public API design philosophy, or non-trivial trade-offs. **This practice is vital even for solo developers.**

**Format & Storage:**
*   Use simple, numbered Markdown files (e.g., `001-use-tokio-as-async-runtime.md`) stored in a dedicated `/docs/adrs/` directory within the repository.
*   Use a consistent template (see `ADR_TEMPLATE.md`).

**Key Template Sections:**
*   **Title:** Short, descriptive summary (e.g., "Use Tokio as the Async Runtime").
*   **Status:** (e.g., Proposed, Accepted, Rejected, Deprecated, Superseded by ADR-XXX).
*   **Context:** Problem statement, background, relevant constraints (e.g., performance needs, ecosystem compatibility).
*   **Decision:** Clearly state the chosen approach (e.g., "We will use the Tokio async runtime for all asynchronous operations").
*   **Consequences:** Positive and negative outcomes, trade-offs accepted, impact on the codebase, potential future issues, required follow-up actions.
*   *(Optional) Alternatives Considered:* Briefly describe other options (e.g., `async-std`, `smol`) and why they were not chosen.

**Rationale:** ADRs provide an immutable log of *why* the system evolved the way it did, preserving crucial context for future development, onboarding, and refactoring. Directly implements *Document Decisions, Not Mechanics*.

---

## 6. Diagrams: Visualizing Structure and Complex Flows

**Approach:** Use diagrams judiciously when a visual representation significantly clarifies high-level crate architecture, module interactions, complex data flows (especially with async/concurrency), state machines, or type relationships. Prioritize maintainable formats.

**Tools & Storage:**
*   **Preferred:** Text-based diagramming tools embeddable in Markdown (e.g., **MermaidJS**, PlantUML) stored directly in relevant documentation files (READMEs, module docs, ADRs) or in `/docs/diagrams/`. They are version-controlled and diff-friendly.
*   **Acceptable:** Simple vector drawing tools (e.g., **Excalidraw**, diagrams.net/draw.io) where the source file (e.g., `.excalidraw`, `.drawio.svg`) is committed to `/docs/diagrams/`. Avoid opaque binary formats.
*   Store diagram source files and/or exported images (SVG preferred over PNG) in `/docs/diagrams/`.

**Rationale:** Diagrams offer quick visual comprehension. Using text-based or versionable source formats keeps them maintainable alongside the code.

**Implementation:**
*   Keep diagrams focused and avoid excessive detail.
*   Ensure clear titles, legends, and context.
*   Reference diagrams from `rustdoc` comments, READMEs, or ADRs.
*   Review and update diagrams when relevant code structures change significantly.

---

By consistently applying these Rust-specific practices, we aim to create documentation that is accurate, useful, easy to maintain, and deeply integrated with the code itself.