# Todo

## Pre-Commit Hook Enhancement
- [x] **T001 · Feature · P2: define line count threshold variables**
    - **Context:** PLAN.md Detailed Build Steps #2
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Add shell variables `WARN_LINES=500` and `ERROR_LINES=1000` near the top.
    - **Done‑when:**
        1. Variables are defined in the script.
    - **Depends‑on:** none

- [x] **T002 · Feature · P2: get list of staged rust files**
    - **Context:** PLAN.md Detailed Build Steps #3.a
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Add command to get staged `.rs` files (e.g., using `git diff --cached --name-only --diff-filter=ACM | grep '\.rs$'`).
        3. Store the list in a variable or use directly in a loop.
    - **Done‑when:**
        1. Script correctly identifies staged `.rs` files.
        2. Script handles the case where no `.rs` files are staged.
    - **Depends‑on:** none

- [ ] **T003 · Feature · P2: implement line count check logic**
    - **Context:** PLAN.md Detailed Build Steps #3.b, #3.c
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Loop through the list of staged `.rs` files identified in T002.
        3. For each file, use `wc -l` to get the line count and compare against `WARN_LINES` and `ERROR_LINES`.
    - **Done‑when:**
        1. Script correctly calculates line counts for staged `.rs` files.
        2. Comparisons against thresholds are performed.
    - **Depends‑on:** [T001, T002]

- [ ] **T004 · Feature · P2: implement line count warning**
    - **Context:** PLAN.md Detailed Build Steps #3.c, Error & Edge-Case Strategy
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. If a file's line count is > `WARN_LINES` and <= `ERROR_LINES`, print a warning message (including filename and line count).
        3. Ensure the script continues execution after a warning.
    - **Done‑when:**
        1. Warning message is printed to stderr for files between 501-1000 lines.
        2. Commit is not blocked by a warning.
    - **Depends‑on:** [T003]

- [ ] **T005 · Feature · P1: implement line count error**
    - **Context:** PLAN.md Detailed Build Steps #3.c, Error & Edge-Case Strategy
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. If a file's line count is > `ERROR_LINES`, print an error message (including filename and line count) and exit with status 1.
    - **Done‑when:**
        1. Error message is printed to stderr for files > 1000 lines.
        2. Script exits with non-zero status, blocking the commit.
    - **Depends‑on:** [T003]

- [ ] **T006 · Bugfix · P2: handle no staged rust files gracefully**
    - **Context:** PLAN.md Error & Edge-Case Strategy
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Ensure the line count check logic (T003, T004, T005) is skipped or handles empty input correctly if no `.rs` files are staged.
    - **Done‑when:**
        1. Committing non-Rust files does not trigger line count checks or errors.
        2. Script exits successfully (status 0) if only non-Rust files are staged (assuming other checks pass).
    - **Depends‑on:** [T002]

- [x] **T007 · Feature · P1: add cargo test execution**
    - **Context:** PLAN.md Detailed Build Steps #4
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Add `cargo test` command after clippy check.
        3. Check the exit code of `cargo test` and exit with status 1 if it fails.
    - **Done‑when:**
        1. `cargo test` runs during pre-commit hook execution.
        2. Hook fails (exits non-zero) if any tests fail.
        3. Hook passes if tests pass.
    - **Depends‑on:** none

- [ ] **T008 · Refactor · P3: add check summary messages**
    - **Context:** PLAN.md Detailed Build Steps #5, Logging & Observability
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Add echo statements before/after `cargo fmt`, `cargo clippy`, line count checks (if any RS files), and `cargo test` indicating start/pass/fail.
    - **Done‑when:**
        1. User sees clear start/end messages for each check run.
    - **Depends‑on:** [T003, T007]

- [ ] **T009 · Refactor · P3: implement colored output**
    - **Context:** PLAN.md Logging & Observability
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Define standard ANSI color code variables (RED, GREEN, YELLOW, NC).
        3. Apply colors to summary messages (T008), warnings (T004), and errors (T005, existing fmt/clippy/test failures).
    - **Done‑when:**
        1. Hook output uses colors for enhanced readability of status/warnings/errors.
    - **Depends‑on:** [T004, T005, T008]

- [ ] **T010 · Chore · P2: update README with new checks**
    - **Context:** PLAN.md Documentation
    - **Action:**
        1. Edit `README.md`.
        2. Add descriptions of the new line count warning (>500 lines) and error (>1000 lines) checks.
        3. Add description of the `cargo test` execution check.
    - **Done‑when:**
        1. README accurately reflects the behavior of the updated pre-commit hook.
    - **Depends‑on:** [T005, T007]

- [ ] **T011 · Chore · P3: update README installation instructions**
    - **Context:** PLAN.md Documentation
    - **Action:**
        1. Edit `README.md`.
        2. Ensure the pre-commit hook installation instructions clearly mention copying or symlinking `hooks/pre-commit` to `.git/hooks/pre-commit` and making it executable.
    - **Done‑when:**
        1. Installation instructions in README are clear and complete.
    - **Depends‑on:** none

- [ ] **T012 · Test · P2: manually test hook failure scenarios**
    - **Context:** PLAN.md Detailed Build Steps #7, Testing Strategy
    - **Action:**
        1. Introduce a formatting error and attempt commit.
        2. Introduce a clippy error and attempt commit.
        3. Introduce a failing test and attempt commit.
    - **Done‑when:**
        1. Hook correctly blocks commit for formatting errors.
        2. Hook correctly blocks commit for clippy errors.
        3. Hook correctly blocks commit for test failures.
    - **Depends‑on:** [T007]

- [ ] **T013 · Test · P2: manually test line count scenarios**
    - **Context:** PLAN.md Detailed Build Steps #7, Testing Strategy
    - **Action:**
        1. Create/stage files with ~501, ~999, ~1000, ~1001 lines and attempt commit.
        2. Verify warning message for 501-1000 lines (commit allowed).
        3. Verify error message and blocked commit for >1000 lines.
    - **Done‑when:**
        1. Hook correctly warns for files between 501-1000 lines.
        2. Hook correctly errors and blocks commit for files > 1000 lines.
        3. Boundary conditions (499, 500, 1000, 1001) behave as expected.
    - **Depends‑on:** [T004, T005]

- [ ] **T014 · Test · P2: manually test no rust files scenario**
    - **Context:** PLAN.md Detailed Build Steps #7, Testing Strategy, Error & Edge-Case Strategy
    - **Action:**
        1. Stage only non-`.rs` files and attempt commit.
        2. Stage no files and attempt commit (empty commit).
    - **Done‑when:**
        1. Hook does not run line count checks or `cargo test` if no `.rs` files are staged.
        2. Hook allows commit (assuming no other errors) when only non-`.rs` files are staged.
        3. Hook handles empty commits gracefully.
    - **Depends‑on:** [T006]

- [ ] **T015 · Refactor · P3: add messaging for slow tests**
    - **Context:** PLAN.md Risk Matrix (Hook execution time)
    - **Action:**
        1. Edit `hooks/pre-commit`.
        2. Add an informational message before running `cargo test` indicating that tests might take some time.
    - **Done‑when:**
        1. User is informed before potentially slow test execution begins.
    - **Depends‑on:** [T007]