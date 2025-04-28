# Todo

## Pre-commit Hooks Configuration
- [x] **T001 · Chore · P2: create `.pre-commit-config.yaml` file**
    - **Context:** PLAN.md > Detailed Build Steps > #2
    - **Action:**
        1. Create an empty file named `.pre-commit-config.yaml` at the repository root.
        2. Initialize it with a top-level `repos:` key as an empty list.
    - **Done‑when:**
        1. `.pre-commit-config.yaml` exists at the repository root with valid YAML syntax.
    - **Depends‑on:** none
- [x] **T002 · Feature · P2: add rustfmt hook to pre-commit config**
    - **Context:** PLAN.md > Detailed Build Steps > #3
    - **Action:**
        1. Add the `local` repo entry for `rustfmt` to `.pre-commit-config.yaml` as specified (id, name, entry, language, types, pass_filenames).
    - **Done‑when:**
        1. `pre-commit run rustfmt --all-files` executes `cargo fmt -- --check` successfully on compliant code.
        2. `pre-commit run rustfmt --all-files` fails on non-compliant code.
    - **Depends‑on:** [T001]
- [x] **T003 · Feature · P2: add clippy hook to pre-commit config**
    - **Context:** PLAN.md > Detailed Build Steps > #4
    - **Action:**
        1. Add the `local` repo entry for `clippy` to `.pre-commit-config.yaml` as specified (id, name, entry, language, types, pass_filenames).
    - **Done‑when:**
        1. `pre-commit run clippy --all-files` executes `cargo clippy --all-targets -- -D warnings` successfully on compliant code.
        2. `pre-commit run clippy --all-files` fails on code with clippy warnings.
    - **Depends‑on:** [T001]
- [x] **T004 · Feature · P2: add commitlint hook to pre-commit config**
    - **Context:** PLAN.md > Detailed Build Steps > #5
    - **Action:**
        1. Add the community repo entry for `commitlint` (`alessandrojcm/commitlint-pre-commit-hook`) to `.pre-commit-config.yaml` with the specified `rev`, `id`, `stages`, and `additional_dependencies`.
    - **Done‑when:**
        1. `pre-commit run commitlint --hook-stage commit-msg --commit-msg-filename <path_to_valid_message>` passes.
        2. `pre-commit run commitlint --hook-stage commit-msg --commit-msg-filename <path_to_invalid_message>` fails.
    - **Depends‑on:** [T001, T005]
- [x] **T005 · Chore · P2: create `commitlint.config.js` file**
    - **Context:** PLAN.md > Detailed Build Steps > #6
    - **Action:**
        1. Create `commitlint.config.js` at the repository root.
        2. Populate it with `module.exports = { extends: ['@commitlint/config-conventional'] };`.
    - **Done‑when:**
        1. `commitlint.config.js` exists with the specified content.
    - **Depends‑on:** none
- [x] **T006 · Feature · P2: create post-commit hook template**
    - **Context:** PLAN.md > Detailed Build Steps > #7
    - **Action:**
        1. Create `templates/post-commit.template` with the exact shell script content provided in the plan.
        2. Ensure the script checks for `glance` and runs it asynchronously, redirecting output.
    - **Done‑when:**
        1. `templates/post-commit.template` exists with the specified script content.
        2. The script is syntactically valid shell script.
    - **Depends‑on:** none
- [x] **T007 · Chore · P1: update developer documentation for hooks**
    - **Context:** PLAN.md > Detailed Build Steps > #8, Risk Matrix > Developer Setup Friction
    - **Action:**
        1. Update `README.md` or `CONTRIBUTING.md` with prerequisites (`python`, `pip`, `glance`).
        2. Add setup instructions: `pip install pre-commit`, `pre-commit install --hook-type pre-commit --hook-type commit-msg`, and the manual `cp`/`chmod` for the post-commit hook.
        3. Explain the purpose of each hook and document the `SKIP=` mechanism, discouraging `--no-verify`.
    - **Done‑when:**
        1. Documentation accurately reflects prerequisites, setup steps, hook purposes, and skip usage.
        2. Following the setup instructions results in functional local hooks.
    - **Depends‑on:** [T010]
- [x] **T008 · Chore · P2: remove legacy hook system artifacts**
    - **Context:** PLAN.md > Detailed Build Steps > #9
    - **Action:**
        1. Identify and delete any previously tracked Git hook scripts (e.g., `.git/hooks/pre-commit`, `hooks/pre-commit`).
        2. Remove any documentation referencing the old hook system.
    - **Done‑when:**
        1. Legacy hook files are removed from the repository history (if applicable).
        2. No documentation refers to the old system.
    - **Depends‑on:** [T007]
- [x] **T009 · Chore · P2: stage hook config files and update `.gitignore`**
    - **Context:** PLAN.md > Detailed Build Steps > #10
    - **Action:**
        1. Add `.pre-commit-config.yaml`, `commitlint.config.js`, and `templates/post-commit.template` to Git tracking.
        2. Add `.pre-commit-cache/` to the project's `.gitignore` file.
    - **Done‑when:**
        1. The three configuration files are tracked by Git.
        2. `.gitignore` includes the `.pre-commit-cache/` entry.
    - **Depends‑on:** [T001, T005, T006]
- [x] **T010 · Test · P1: add ci check for pre-commit hooks**
    - **Context:** PLAN.md > Detailed Build Steps > #11, Testing Strategy > CI Tests
    - **Action:**
        1. Add a step/job to the existing CI pipeline.
        2. This step must install `pre-commit` and execute `pre-commit run --all-files`.
        3. Ensure the CI pipeline fails if this step reports any hook failures.
    - **Done‑when:**
        1. CI pipeline executes `pre-commit run --all-files` on relevant code changes.
        2. CI fails when code violates hook rules (`rustfmt`, `clippy`, `commitlint`).
        3. CI passes when code adheres to hook rules.
    - **Depends‑on:** [T002, T003, T004, T009]

### Clarifications & Assumptions
- [ ] **Issue:** Confirm `glance` tool availability and expected location in standard developer and CI environments.
    - **Context:** PLAN.md > Open Questions > #1; Risk Matrix > `glance` Tool Unavailable
    - **Blocking?:** no
- [ ] **Issue:** Verify team acceptance of the chosen `alessandrojcm/commitlint-pre-commit-hook` repository.
    - **Context:** PLAN.md > Open Questions > #2; Risk Matrix > `commitlint` Node.js Dependency Management
    - **Blocking?:** no