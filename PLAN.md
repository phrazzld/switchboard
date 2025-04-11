```markdown
# PLAN.MD: GitHub Actions CI Setup for Switchboard Proxy

## Task Description

Set up GitHub Actions for the Switchboard proxy project to automate testing, linting, and build verification. This involves creating a CI workflow that runs on pushes and pull requests to the main branch, executing checks like `cargo fmt --check`, `cargo clippy`, `cargo test`, and `cargo build`.

## Recommended Plan: Single Workflow, Multiple Jobs

This plan utilizes a single GitHub Actions workflow file (`.github/workflows/ci.yml`) containing multiple distinct jobs that can run in parallel for faster feedback. This approach balances simplicity, clarity, speed, and resource utilization, aligning well with the project's core principles.

**Workflow Structure:**

1.  **Trigger:** The workflow will trigger on `push` events to the `main` branch and on `pull_request` events targeting the `main` branch.
2.  **Environment:** Define `CARGO_TERM_COLOR: always` for better log output.
3.  **Jobs:** Define separate jobs for each primary check:
    *   `format`: Checks code formatting using `cargo fmt --check`.
    *   `lint`: Lints code using `cargo clippy -- -D warnings` (treating warnings as errors).
    *   `test`: Runs the full test suite using `cargo test --all-features`.
    *   `build`: Verifies the release build using `cargo build --release`.
4.  **Job Steps:** Each job will perform the following steps:
    *   Check out the repository code (`actions/checkout@v4`).
    *   Set up the stable Rust toolchain (`dtolnay/rust-toolchain@stable`), specifying necessary components (`rustfmt`, `clippy`).
    *   Configure caching for `cargo` dependencies (`~/.cargo/registry`, `~/.cargo/git`) and the job-specific `target` directory (`actions/cache@v4`) based on the `Cargo.lock` hash to speed up subsequent runs.
    *   Execute the specific `cargo` command for the job's purpose.
5.  **Runner:** All jobs will run on `ubuntu-latest`.

## Implementation Steps

1.  Create the directory `.github/workflows/` in the project root if it doesn't exist.
2.  Create a file named `ci.yml` inside `.github/workflows/`.
3.  Add the following content to `ci.yml`:

    ```yaml
    # .github/workflows/ci.yml
    name: Rust CI

    # Controls when the workflow will run
    on:
      push:
        branches: [ "main" ]
      pull_request:
        branches: [ "main" ]

    # Environment variables available to all jobs and steps
    env:
      CARGO_TERM_COLOR: always

    jobs:
      format:
        name: Format Check
        runs-on: ubuntu-latest
        steps:
          - uses: actions/checkout@v4
          - uses: dtolnay/rust-toolchain@stable
            with:
              components: rustfmt # Ensure rustfmt component is installed
          - name: Cache cargo dependencies
            uses: actions/cache@v4
            with:
              # Cache paths for cargo registry, git dependencies, and target directory
              path: |
                ~/.cargo/bin/
                ~/.cargo/registry/index/
                ~/.cargo/registry/cache/
                ~/.cargo/git/db/
                target/
              # Create a unique key based on OS, job name, and Cargo.lock hash
              key: ${{ runner.os }}-cargo-fmt-${{ hashFiles('**/Cargo.lock') }}
              # Fallback keys if the exact key is not found
              restore-keys: |
                ${{ runner.os }}-cargo-fmt-
          - name: Run cargo fmt check
            run: cargo fmt --check # Check if code needs formatting

      lint:
        name: Lint Check
        runs-on: ubuntu-latest
        steps:
          - uses: actions/checkout@v4
          - uses: dtolnay/rust-toolchain@stable
            with:
              components: clippy # Ensure clippy component is installed
          - name: Cache cargo dependencies
            uses: actions/cache@v4
            with:
              path: |
                ~/.cargo/bin/
                ~/.cargo/registry/index/
                ~/.cargo/registry/cache/
                ~/.cargo/git/db/
                target/
              key: ${{ runner.os }}-cargo-lint-${{ hashFiles('**/Cargo.lock') }}
              restore-keys: |
                ${{ runner.os }}-cargo-lint-
          - name: Run cargo clippy
            # Run clippy with strict settings: treat warnings as errors (-D warnings)
            run: cargo clippy -- -D warnings

      test:
        name: Run Tests
        runs-on: ubuntu-latest
        steps:
          - uses: actions/checkout@v4
          - uses: dtolnay/rust-toolchain@stable
          - name: Cache cargo dependencies
            uses: actions/cache@v4
            with:
              path: |
                ~/.cargo/bin/
                ~/.cargo/registry/index/
                ~/.cargo/registry/cache/
                ~/.cargo/git/db/
                target/
              key: ${{ runner.os }}-cargo-test-${{ hashFiles('**/Cargo.lock') }}
              restore-keys: |
                ${{ runner.os }}-cargo-test-
          - name: Run cargo test
            # Run all tests, including unit, integration, and doc tests
            # --all-features ensures features are enabled if needed for tests
            run: cargo test --all-features

      build:
        name: Build Verification
        runs-on: ubuntu-latest
        steps:
          - uses: actions/checkout@v4
          - uses: dtolnay/rust-toolchain@stable
          - name: Cache cargo dependencies
            uses: actions/cache@v4
            with:
              path: |
                ~/.cargo/bin/
                ~/.cargo/registry/index/
                ~/.cargo/registry/cache/
                ~/.cargo/git/db/
                target/
              key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}
              restore-keys: |
                ${{ runner.os }}-cargo-build-
          - name: Run cargo build (release)
            # Build the release artifact to verify compilation in release mode
            # --verbose provides more output if the build fails
            run: cargo build --release --verbose
    ```

4.  Commit and push the `.github/workflows/ci.yml` file to the repository.
5.  Verify that the action runs correctly on the next push or pull request to the `main` branch by checking the "Actions" tab in the GitHub repository.

## Justification for Recommendation

This plan (Single Workflow, Multiple Jobs) is recommended because it provides the best balance according to the project's standards hierarchy:

1.  **Simplicity/Clarity (`CORE_PRINCIPLES.md`):** This is a standard, widely understood pattern for GitHub Actions CI. The single workflow file keeps all CI logic centralized, and the distinct jobs clearly map to specific checks (format, lint, test, build).
2.  **Separation of Concerns (`ARCHITECTURE_GUIDELINES.md`):** Each job handles a distinct aspect of CI verification, aligning with the principle of modularity. Parallel execution enhances this separation.
3.  **Testability (`TESTING_STRATEGY.md`):** The `test` job directly implements the requirement to automate test execution (`cargo test`). Parallelism allows tests to start running alongside other checks, potentially providing faster feedback. The strict mocking policy is unaffected by the CI setup itself but is supported by running integration tests effectively.
4.  **Coding Conventions (`CODING_STANDARDS.md`):** The `format` and `lint` jobs directly enforce the mandatory formatting (`rustfmt`) and linting (`clippy`) standards, using strict checks (`--check`, `-D warnings`).
5.  **Documentability (`DOCUMENTATION_APPROACH.md`):** The workflow file itself serves as clear documentation of the automated CI process.

**Benefits over Alternatives:**

*   **Faster Feedback:** Compared to a single-job sequential plan, parallel execution of independent checks (format, lint, test) provides quicker results.
*   **Clear Failure Isolation:** GitHub's UI clearly indicates which specific job failed, simplifying debugging.
*   **Maintainability:** Easier to manage than multiple workflow files, keeping the CI configuration cohesive.
*   **Automation:** Directly fulfills the "Automate Everything" core principle for essential development tasks.

**Trade-offs:**

*   **Resource Usage:** Uses slightly more runner resources than a single sequential job due to parallel execution. This is generally acceptable for the faster feedback provided.
*   **Caching:** Caching the `target` directory across *different* jobs can be complex. This plan simplifies by caching `target` *within* each job, accepting that some recompilation might occur between jobs but leveraging the more impactful source/registry cache. This is a reasonable trade-off for simplicity and robustness.
```