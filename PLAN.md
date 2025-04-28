```markdown
# Plan: Refactor Pre-commit Hooks using pre-commit Framework

## Chosen Approach (One-liner)
Adopt the standard `pre-commit` framework to manage formatting, linting, and commit message validation hooks, while using a separate standard Git post-commit hook for the asynchronous `glance` execution.

## Architecture Blueprint

- **Modules / Packages**
  - `.pre-commit-config.yaml` → Declarative configuration for `pre-commit` managed hooks (formatting, linting, commit message validation). Single source-of-truth for these checks.
  - `commitlint.config.js` → Configuration file defining the Conventional Commits rules enforced by `commitlint`.
  - `templates/post-commit.template` → A template shell script for the standard Git `post-commit` hook, responsible for running `glance ./` asynchronously. Developers copy this to `.git/hooks/post-commit`.
  - `README.md` / `CONTRIBUTING.md` → Essential documentation covering setup, usage, rationale, and troubleshooting for the new hook system.

- **Public Interfaces / Contracts**
  - **`pre-commit` CLI:**
    - `pre-commit install --hook-type pre-commit --hook-type commit-msg`: Installs the managed hooks.
    - `pre-commit run [--all-files]`: Runs hooks manually.
    - `SKIP=<hook_id>[,<hook_id>...] git commit ...`: Environment variable mechanism provided by `pre-commit` to bypass specific hooks.
  - **Conventional Commits Standard:** Enforced by the `commitlint` hook during the `commit-msg` stage.
  - **`post-commit` Hook:** Standard Git hook mechanism (`.git/hooks/post-commit`), not managed by the `pre-commit` framework installation command.

- **Data Flow Diagram** (mermaid)
  ```mermaid
  graph TD
      A[Developer: git commit -m "feat: Implement X"] --> B{Git Pre-Commit Trigger};
      B --> C[pre-commit framework];
      C -- Runs hooks defined in .pre-commit-config.yaml --> D{Run 'rustfmt' hook};
      D -- Success --> E{Run 'clippy' hook};
      E -- Success --> F[Commit process continues];
      F --> G{Git Commit-Msg Trigger};
      G --> C;
      C -- Runs hooks defined in .pre-commit-config.yaml --> H{Run 'commitlint' hook};
      H -- Success --> I[Commit is created];
      I --> J{Git Post-Commit Trigger};
      J --> K[Execute .git/hooks/post-commit];
      K --> L[Run 'glance ./' async];
      L --> M[Commit process complete];

      D -- Failure --> Z{Abort Commit & Display Error};
      E -- Failure --> Z;
      H -- Failure --> Z;
  ```

- **Error & Edge-Case Strategy**
  - **Hook Failures (`pre-commit`, `commit-msg`):** Any failure (non-zero exit code) from a hook managed by `pre-commit` will block the commit process immediately. The framework outputs the failing hook's ID and its stdout/stderr, providing clear feedback.
  - **Skipping Hooks:** Use the documented `SKIP` environment variable for exceptional cases. Discourage the use of `git commit --no-verify`.
  - **`post-commit` Failures:** The template script checks for `glance` existence and warns if not found. The `glance` command itself is run asynchronously (`&`) and its output redirected (`>/dev/null 2>&1`) to prevent blocking the terminal or failing the overall commit flow if `glance` itself errors. Failures here are non-blocking by design.
  - **Setup Issues:** Clear documentation is the primary mitigation. CI checks (`pre-commit run --all-files`) ensure the configuration is valid and tools run correctly in a clean environment. Failure to install `pre-commit` or copy the `post-commit` hook will mean hooks don't run locally, but CI provides a safety net.

## Detailed Build Steps

1.  **Add `pre-commit` Dependency:** Document the requirement for developers to install `pre-commit` (e.g., `pip install pre-commit` or via package manager).
2.  **Create `.pre-commit-config.yaml`:** Initialize the configuration file at the repository root.
3.  **Configure `rustfmt` Hook:** Add an entry for `cargo fmt --check`. Use a `language: system` hook for simplicity if Rust toolchain is assumed present.
    ```yaml
    repos:
    - repo: local
      hooks:
        - id: rustfmt
          name: Rust Formatter Check
          entry: cargo fmt -- --check
          language: system
          types: [rust]
          pass_filenames: false # Run against the whole project state if any .rs file changed
    ```
4.  **Configure `clippy` Hook:** Add an entry for `cargo clippy -- -D warnings`.
    ```yaml
    # In .pre-commit-config.yaml, within the same repo: local or a new one
        - id: clippy
          name: Rust Linter Check
          entry: cargo clippy --all-targets -- -D warnings # Fail on any warnings
          language: system
          types: [rust]
          pass_filenames: false
    ```
5.  **Configure `commitlint` Hook:** Add a hook using a community repository to manage Node.js execution and dependencies for commit message validation.
    ```yaml
    # In .pre-commit-config.yaml
    - repo: https://github.com/alessandrojcm/commitlint-pre-commit-hook
      rev: v9.13.0 # Specify a pinned, stable version
      hooks:
        - id: commitlint
          stages: [commit-msg]
          additional_dependencies: ['@commitlint/config-conventional'] # Use conventional commit rules
    ```
6.  **Create `commitlint.config.js`:** Add the configuration file at the repository root to specify the rule set.
    ```javascript
    // commitlint.config.js
    module.exports = {
      extends: ['@commitlint/config-conventional']
    };
    ```
7.  **Create `post-commit` Hook Template:** Create `templates/post-commit.template` with robust async execution and checks.
    ```sh
    #!/bin/sh
    # Template for .git/hooks/post-commit
    # Runs 'glance ./' asynchronously after a successful commit.

    # Check if glance command exists
    if ! command -v glance > /dev/null 2>&1; then
        echo "post-commit hook: Warning: 'glance' command not found. Skipping execution." >&2
        exit 0
    fi

    echo "post-commit hook: Running 'glance ./' in background..."

    # Execute glance asynchronously, detaching completely
    ( glance ./ & ) > /dev/null 2>&1 &

    exit 0
    ```
8.  **Update Documentation (`README.md` / `CONTRIBUTING.md`):**
    *   Clearly state prerequisites: `python`, `pip` (for `pre-commit`), `glance`.
    *   Provide setup instructions:
        *   `pip install pre-commit`
        *   `pre-commit install --hook-type pre-commit --hook-type commit-msg` (Installs hooks defined in `.pre-commit-config.yaml`)
        *   `cp templates/post-commit.template .git/hooks/post-commit && chmod +x .git/hooks/post-commit` (Manual step for post-commit)
    *   Explain the purpose of each hook (Format, Lint, Commit Message Syntax, Post-Commit Glance).
    *   Document how to skip hooks using `SKIP=hook_id,... git commit ...`. Strongly discourage `--no-verify`.
9.  **Remove Old Hook System:** Delete any legacy hook scripts (e.g., `.git/hooks/pre-commit`, `hooks/pre-commit` if tracked). Ensure documentation reflects the removal.
10. **Add Configuration Files to Git:** Ensure `.pre-commit-config.yaml`, `commitlint.config.js`, and `templates/post-commit.template` are tracked by Git. Add `.pre-commit-cache/` to `.gitignore`.
11. **Integrate with CI:** Add a CI step that installs `pre-commit` and runs `pre-commit run --all-files` to validate all files against all hooks.

## Testing Strategy

- **Unit Tests:** N/A for the configuration files or the simple post-commit script. `commitlint` itself has unit tests.
- **Integration Tests (Manual):** Developers perform these implicitly during setup and use. Explicit tests:
    - Run setup instructions verbatim.
    - Stage code requiring formatting -> `git commit` -> Verify `rustfmt` fails and blocks.
    - Fix formatting, stage code with clippy warnings -> `git commit` -> Verify `clippy` fails and blocks.
    - Fix clippy warnings, attempt commit with invalid message -> `git commit -m "bad message"` -> Verify `commitlint` fails and blocks.
    - Attempt commit with valid code and message -> `git commit -m "feat: Add valid feature"` -> Verify commit succeeds.
    - Immediately after success, check running processes for `glance ./` or evidence of its execution (if it creates files/logs). Verify no errors were printed to the terminal from the post-commit hook.
    - Test skipping: `SKIP=clippy git commit ...` with clippy warnings present -> Verify commit succeeds (if fmt/commitlint pass).
- **CI Tests:** The `pre-commit run --all-files` step in CI acts as the automated integration test for the `pre-commit` managed hooks against the entire codebase.

## Logging & Observability

- **`pre-commit` Framework:** Logs hook execution status (Passed/Failed), timing, and any stdout/stderr from the hooks directly to the console during the commit attempt. Verbosity can be increased if needed (`pre-commit run -v`).
- **`commitlint`:** Outputs specific rule violations for commit messages.
- **`post-commit` Script:** Logs a warning to stderr if `glance` is not found. Logs an informational message indicating background execution start. `glance`'s own output is intentionally discarded (`>/dev/null 2>&1`) to avoid cluttering the user's terminal after a successful commit.
- **Correlation IDs:** Not applicable for local developer tooling.

## Security & Config

- **Input Validation:** Handled by the underlying tools (`cargo fmt`, `cargo clippy`, `commitlint`). The `post-commit` script validates the existence of `glance` before attempting execution.
- **Secrets Handling:** No secrets are used, processed, or stored by this tooling.
- **Least Privilege:** Hooks run with the privileges of the developer invoking `git commit`. The `post-commit` script does not require elevated privileges. Ensure `glance` itself follows least privilege principles.
- **Dependencies:** Using pinned versions (`rev`) in `.pre-commit-config.yaml` for third-party hook repositories mitigates risks from upstream changes.

## Documentation

- **Code Self-doc:**
    - `.pre-commit-config.yaml`: YAML structure is declarative. Use `name:` fields for clarity. Add comments (`#`) explaining non-obvious choices.
    - `commitlint.config.js`: Minimal, relies on standard conventional commit rules.
    - `templates/post-commit.template`: Includes comments explaining its purpose, checks, and asynchronous execution logic.
- **README/CONTRIBUTING Updates:** Critical section detailing prerequisites, installation steps (including the manual post-commit setup), purpose of hooks, and how to skip them. (See Detailed Build Steps #8).

## Risk Matrix

| Risk                                                     | Severity | Mitigation                                                                                                                               |
|----------------------------------------------------------|----------|------------------------------------------------------------------------------------------------------------------------------------------|
| Developer Setup Friction (Python/pre-commit/post-commit) | medium   | Clear, tested documentation in README/CONTRIBUTING.md. CI validation (`pre-commit run --all-files`) provides a safety net.                 |
| `commitlint` Node.js Dependency Management               | low      | Using a community `pre-commit` hook (`alessandrojcm/commitlint-pre-commit-hook`) abstracts away direct Node/npm management from the developer. |
| `post-commit` Hook Not Installed/Executable by Developer | medium   | Explicit manual setup steps in documentation. Consider adding a simple verification script (`./scripts/verify-hooks.sh`) if this becomes common. |
| `glance` Tool Unavailable or Failing                     | low      | Post-commit script includes existence check and warning. Async execution prevents blocking. Failure is non-critical for commit success.    |
| Hook Performance Impacting Commit Time                   | low      | `fmt` and `clippy` are generally fast. Avoiding `cargo test` in pre-commit mitigates major slowdowns. Monitor if needed.                     |
| Over-reliance on `SKIP` or `--no-verify`                 | low      | Document as escape hatches for exceptional cases only. CI enforces checks regardless of local skips.                                      |
| Platform Compatibility (Shell script, paths)             | low      | Post-commit script uses basic `sh` features. `pre-commit` handles cross-platform execution for managed hooks well. Test on target OSes.     |

## Open Questions

- Confirm `glance` is expected to be present in typical developer environments and CI. If not, the post-commit warning is sufficient, but documentation might need adjustment.
- Is the chosen `commitlint` hook repository (`alessandrojcm/commitlint-pre-commit-hook`) the best fit, or are there alternatives preferred by the team? (Chosen one seems robust and handles node environment).
- Should `cargo test` be added as a *manual* pre-commit stage (`stages: [manual]`) for optional execution via `pre-commit run --hook-stage manual test`, or solely rely on CI? (Decision: Rely solely on CI for tests to keep pre-commit fast).
```