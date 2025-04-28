```markdown
# Todo

## Pre-commit & Local Hooks
- [x] **T001 · Refactor · P1: replace local rustfmt hook with remote**
    - **Context:** PLAN.md § cr-03 Fix Fragile Local Pre-commit Hooks Definition
    - **Action:**
        1. Edit `.pre-commit-config.yaml`.
        2. Replace the `repo: local` entry for `rustfmt` with a standard remote repository (e.g., `pre-commit/mirrors-rustfmt`).
    - **Done‑when:**
        1. The `rustfmt` hook in `.pre-commit-config.yaml` uses a remote `repo:` URL.
    - **Verification:**
        1. Run `pre-commit install --install-hooks`.
        2. Run `pre-commit run rustfmt` and confirm it downloads and runs the remote hook.
    - **Depends‑on:** none

- [x] **T002 · Refactor · P1: replace local clippy hook with remote**
    - **Context:** PLAN.md § cr-03 Fix Fragile Local Pre-commit Hooks Definition
    - **Action:**
        1. Edit `.pre-commit-config.yaml`.
        2. Replace the `repo: local` entry for `clippy` with a standard remote repository (e.g., `doublify/pre-commit-clippy`).
    - **Done‑when:**
        1. The `clippy` hook in `.pre-commit-config.yaml` uses a remote `repo:` URL.
    - **Verification:**
        1. Run `pre-commit install --install-hooks`.
        2. Run `pre-commit run clippy` and confirm it downloads and runs the remote hook.
    - **Depends‑on:** none

- [x] **T003 · Chore · P1: pin remote rustfmt hook version**
    - **Context:** PLAN.md § cr-03 Fix Fragile Local Pre-commit Hooks Definition
    - **Action:**
        1. Edit `.pre-commit-config.yaml`.
        2. Set the `rev:` field for the remote `rustfmt` hook to a specific, audited tag or commit hash.
    - **Done‑when:**
        1. The `rev:` for the remote `rustfmt` hook is pinned to a specific version.
    - **Depends‑on:** [T001]

- [x] **T004 · Chore · P1: pin remote clippy hook version**
    - **Context:** PLAN.md § cr-03 Fix Fragile Local Pre-commit Hooks Definition
    - **Action:**
        1. Edit `.pre-commit-config.yaml`.
        2. Set the `rev:` field for the remote `clippy` hook to a specific, audited tag or commit hash.
    - **Done‑when:**
        1. The `rev:` for the remote `clippy` hook is pinned to a specific version.
    - **Depends‑on:** [T002]

- [x] **T005 · Chore · P1: pin @commitlint/config-conventional dependency version**
    - **Context:** PLAN.md § cr-09 Pin Unpinned Commitlint Dependency
    - **Action:**
        1. Edit `.pre-commit-config.yaml`.
        2. Modify the `additional_dependencies` for the `commitlint` hook to pin `@commitlint/config-conventional` to a specific version (e.g., `'@commitlint/config-conventional@<specific_version>'`).
    - **Done‑when:**
        1. `@commitlint/config-conventional` dependency is pinned to a specific version string.
    - **Verification:**
        1. Run `pre-commit run commitlint --hook-stage commit-msg -m "feat: test pinning"` locally; verify it passes.
    - **Depends‑on:** none

- [x] **T006 · Feature · P1: add pre-commit hook for file size check**
    - **Context:** PLAN.md § cr-01 Reinstate Lost Pre-commit File Length & Test Checks (Step 2)
    - **Action:**
        1. Edit `.pre-commit-config.yaml`.
        2. Add a hook using `pre-commit-hooks` `check-added-large-files` or a custom script to check line counts (e.g., warn 500, error 1000).
    - **Done‑when:**
        1. `pre-commit run -a` fails locally if a file exceeds the defined ERROR threshold.
    - **Verification:**
        1. Add a file exceeding the threshold and attempt `git commit`. Verify the hook fails.
    - **Depends‑on:** none

- [x] **T007 · Feature · P1: add pre-commit hook for local cargo test execution**
    - **Context:** PLAN.md § cr-01 Reinstate Lost Pre-commit File Length & Test Checks (Step 3 & 4)
    - **Action:**
        1. Edit `.pre-commit-config.yaml`.
        2. Add a `repo: local` hook to execute `cargo test --no-fail-fast` (or a documented faster subset like `cargo test --lib`). Set `stages: [commit]`, `language: system`, `pass_filenames: false`.
    - **Done‑when:**
        1. `.pre-commit-config.yaml` includes a hook that runs `cargo test` (or subset) on commit.
    - **Verification:**
        1. Introduce a failing test. Attempt `git commit`. Verify the hook fails. Fix the test. Attempt `git commit`. Verify the hook passes.
    - **Depends‑on:** none

- [ ] **T008 · Documentation · P2: document file size and cargo test pre-commit hooks**
    - **Context:** PLAN.md § cr-01 Reinstate Lost Pre-commit File Length & Test Checks (Step 5)
    - **Action:**
        1. Edit `CONTRIBUTING.md`.
        2. Add/update a section describing the file size check (thresholds) and the `cargo test` hook (command run, scope/subset if applicable).
    - **Done‑when:**
        1. `CONTRIBUTING.md` accurately documents the behavior of the file size and `cargo test` hooks.
    - **Depends‑on:** [T006, T007]

## CI Automation
- [ ] **T009 · Feature · P1: add commit message validation step to ci**
    - **Context:** PLAN.md § cr-04 Fix Missing Commit Message Enforcement in CI
    - **Action:**
        1. Edit `.github/workflows/ci.yml`.
        2. Add a step after checkout in a primary job to run `pre-commit run --hook-stage commit-msg --from <base> --to <head>` (using appropriate event context variables for base/head).
    - **Done‑when:**
        1. CI pipeline fails if any commit message in the PR/push range does not conform to Conventional Commits.
    - **Verification:**
        1. Create a test PR with an invalid commit message. Verify the CI step fails.
        2. Create a test PR with only valid commit messages. Verify the CI step passes.
    - **Depends‑on:** [T005]

- [ ] **T010 · Chore · P2: remove redundant pre-commit ci jobs**
    - **Context:** PLAN.md § cr-06 Remove Redundant and Misleading Pre-commit CI Jobs (Step 2)
    - **Action:**
        1. Edit `.github/workflows/ci.yml`.
        2. Delete the `pre-commit-linux` and `pre-commit-macos` job definitions entirely.
    - **Done‑when:**
        1. The `pre-commit-linux` and `pre-commit-macos` jobs are removed from `ci.yml`.
    - **Depends‑on:** none

- [ ] **T011 · Chore · P2: update needs section of ci summary job**
    - **Context:** PLAN.md § cr-06 Remove Redundant and Misleading Pre-commit CI Jobs (Step 3)
    - **Action:**
        1. Edit `.github/workflows/ci.yml`.
        2. Locate the job previously named `cross-platform-verification`.
        3. Update its `needs:` section to remove dependencies on the deleted jobs (`pre-commit-linux`, `pre-commit-macos`).
    - **Done‑when:**
        1. The `needs:` section of the summary job no longer lists the removed jobs.
    - **Depends‑on:** [T010]

- [ ] **T012 · Chore · P2: rename ci summary job**
    - **Context:** PLAN.md § cr-06 Remove Redundant and Misleading Pre-commit CI Jobs (Step 4)
    - **Action:**
        1. Edit `.github/workflows/ci.yml`.
        2. Rename the job previously named `cross-platform-verification` (both ID and `name:`) to `Linux_macOS_Verification_Summary` or similar.
    - **Done‑when:**
        1. The summary CI job has been renamed.
    - **Depends‑on:** [T011]

## Repository & Code Cleanup
- [ ] **T013 · Chore · P3: remove temporary files from repository**
    - **Context:** PLAN.md § cr-07 Remove Committed Temporary/Example Files (Steps 1, 2, 4)
    - **Action:**
        1. Delete `test-file.txt` from the repository.
        2. Delete `valid-commit-msg.txt` from the repository.
        3. Commit the deletions.
    - **Done‑when:**
        1. `test-file.txt` and `valid-commit-msg.txt` are no longer present in the repository HEAD.
    - **Depends‑on:** none

- [ ] **T014 · Documentation · P3: embed commit message example in documentation if needed**
    - **Context:** PLAN.md § cr-07 Remove Committed Temporary/Example Files (Step 3)
    - **Action:**
        1. Determine if the content of the deleted `valid-commit-msg.txt` is a useful example.
        2. If yes, copy the example content into a relevant section (e.g., Conventional Commits) in `README.md` or `CONTRIBUTING.md`.
    - **Done‑when:**
        1. A useful commit message example is embedded in documentation, or a decision was made not to include it.
    - **Depends‑on:** [T013]

- [ ] **T015 · Refactor · P1: remove dead_code allowances in tests/common/mod.rs**
    - **Context:** PLAN.md § cr-02 Remove Undocumented Suppression of Dead Code Warnings (Steps 1, 2)
    - **Action:**
        1. Open `tests/common/mod.rs`.
        2. Delete every instance of the `#[allow(dead_code)]` attribute.
    - **Done‑when:**
        1. No `#[allow(dead_code)]` attributes remain in `tests/common/mod.rs`.
    - **Depends‑on:** none

- [ ] **T016 · Refactor · P1: resolve dead_code warnings in tests/common/mod.rs**
    - **Context:** PLAN.md § cr-02 Remove Undocumented Suppression of Dead Code Warnings (Steps 3, 4, 5)
    - **Action:**
        1. Run `cargo clippy --tests -- -D warnings` focusing on `tests/common/mod.rs`.
        2. For each reported `dead_code` warning: investigate usage (find references). Delete if truly unused.
        3. If usage confirmed (warning disappears) or compiler error requires justified allowance, add specific `// ALLOWANCE: Used by X, compiler fails...` comment.
    - **Done‑when:**
        1. `cargo clippy --tests -- -D warnings` passes cleanly regarding dead code in `tests/common/mod.rs`.
        2. Any remaining allowances have explicit `// ALLOWANCE:` comments.
    - **Depends‑on:** [T015]

- [ ] **T017 · Refactor · P2: resolve unused variable in test_directory_permissions_non_unix**
    - **Context:** PLAN.md § cr-08 Clarify Unclear Test Variable Usage (Step 1)
    - **Action:**
        1. Edit `tests/common_utils_test.rs:96`.
        2. Remove `#[allow(unused_variables)]` from `temp_dir_path`.
        3. Determine if `temp_dir_path` is needed; remove the binding if not, or update the test logic to use it.
    - **Done‑when:**
        1. The `unused_variables` allowance is removed and `cargo clippy --tests -- -D warnings` passes for this variable.
    - **Depends‑on:** none

- [ ] **T018 · Refactor · P2: resolve unused _config in test_development_environment_paths**
    - **Context:** PLAN.md § cr-08 Clarify Unclear Test Variable Usage (Step 2)
    - **Action:**
        1. Edit `tests/environment_log_paths_test.rs:135`.
        2. Investigate `_config`: remove the binding if unused, add comment if needed for side-effects, or rename to `config` and use it.
    - **Done‑when:**
        1. The `_config` variable is appropriately used, removed, or justified with a comment, and `cargo clippy --tests -- -D warnings` passes.
    - **Depends‑on:** none

## Documentation Restoration
- [ ] **T019 · Chore · P1: retrieve deleted historical documentation content**
    - **Context:** PLAN.md § cr-05 Restore Essential Planning/Rationale Documentation (Steps 1, 2)
    - **Action:**
        1. Use `git log --diff-filter=D --summary` or history browsing to find commits deleting `PLAN.md`, `TODO.md`, `CI_FAILURES.md`, `CI_STATUS.md`.
        2. Use `git show <commit_hash>:<path>` to extract the content of these files before deletion.
    - **Done‑when:**
        1. Content from the specified deleted files has been retrieved locally.
    - **Depends‑on:** none

- [ ] **T020 · Documentation · P1: create decisions.md consolidating historical rationale**
    - **Context:** PLAN.md § cr-05 Restore Essential Planning/Rationale Documentation (Steps 3, 4)
    - **Action:**
        1. Create `DECISIONS.md` in the repository root.
        2. Review retrieved content (from T019), extracting key decisions, rationale, trade-offs, and learnings.
        3. Organize and write this extracted information into `DECISIONS.md`.
    - **Done‑when:**
        1. `DECISIONS.md` exists and contains key historical decisions and rationale.
    - **Depends‑on:** [T019]

- [ ] **T021 · Documentation · P1: update readme.md to reference decisions.md**
    - **Context:** PLAN.md § cr-05 Restore Essential Planning/Rationale Documentation (Step 5)
    - **Action:**
        1. Edit `README.md`.
        2. Add a reference and link to the new `DECISIONS.md` file, explaining its purpose.
    - **Done‑when:**
        1. `README.md` contains a link to `DECISIONS.md`.
    - **Depends‑on:** [T020]

### Clarifications & Assumptions
- [ ] **Issue:** Determine specific version for `@commitlint/config-conventional` pin.
    - **Context:** PLAN.md § cr-09 Pin Unpinned Commitlint Dependency (Step 3 mentions example but not required version).
    - **Blocking?:** no (T005 can proceed with latest stable, but clarification preferred).
- [ ] **Issue:** Decide scope of `cargo test` pre-commit hook (full suite vs. faster subset).
    - **Context:** PLAN.md § cr-01 Reinstate Lost Pre-commit File Length & Test Checks (Step 4 mentions options).
    - **Blocking?:** no (T007 implements default, T008 documents it; can be adjusted later if needed).
```