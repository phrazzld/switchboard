# T004 Plan: Audit and enforce safe `Config` logging

## Context
Ticket T004 is part of cr-02 Bulletproof API key redaction. The goal is to audit all code locations where `Config` struct or its fields might be formatted to string and ensure that only `Debug` formatting is used where secrets might be present.

## Approach
1. First, understand the `Config` struct and how it handles secret redaction.
2. Search the codebase for all instances where `Config` is logged or formatted.
3. Check each instance to ensure only `Debug` formatting is used for secrets.
4. Refactor any non-Debug logging/formatting to explicitly omit secret fields.

## Implementation Steps
1. Examine `Config` struct implementation to understand its fields, particularly those containing secrets.
2. Identify how `Debug` implementation handles secret redaction.
3. Search the codebase for uses of `Config` in:
   - log macros (`log::*`, `info!`, `debug!`, `error!`, etc.)
   - println!/format! macros
   - to_string() or similar methods
4. For each instance, verify if it's using `Debug` formatting (`{:?}`) or something else.
5. Fix any instances that don't use `Debug` formatting by either:
   - Converting to use `Debug` formatting
   - Creating a safe version that explicitly omits secret fields

## Verification Steps
1. Ensure all identified instances properly use `Debug` formatting.
2. Document findings in this plan file.
3. Make necessary code changes.
4. Run tests to ensure functionality is preserved.

## Audit Findings

### Secret Fields in Config
The `Config` struct contains the following secret fields:
- `anthropic_api_key: SecretString` 
- `openai_api_key: Option<SecretString>`

These use the `SecretString` type from the `secrecy` crate, which provides automatic redaction in Debug formatting.

### Current Redaction System
1. The struct derives `Debug` (line 250), which works with the `SecretString` type
2. `SecretString` automatically redacts secrets when using the Debug formatter `{:?}`
3. There is a test `test_config_debug_redaction` that verifies this behavior

### Usage Patterns Analysis
I audited all instances in the codebase where `Config` is used:

#### Safe Usages
- `config.rs` - Main module properly logs only non-secret fields in `load_config` using structured logging (lines 516-529)
- `logger/init.rs` - Uses `info!` with fields, accessing specific non-secret fields of Config
- `main.rs` - Uses `eprintln!` to log error messages but doesn't include Config values directly
- `proxy_handler.rs` - Uses `expose_secret()` to access the API key value for headers but doesn't log it

#### Test Usage
Test utilities create Config instances with dummy secrets, but they are not directly logged and use SecretString correctly.

### Improvements Made
1. Added a detailed doc comment warning to the `Config` struct emphasizing:
   - The presence of sensitive data in the struct
   - That Debug formatting (`{:?}`) must be used for safe formatting
   - Examples of correct and incorrect usage
   - Warning about `expose_secret()` usage

### Clippy Lint Investigation
I investigated the feasibility of creating a custom clippy lint to detect unsafe formatting of `Config`:

1. **Technical Approach**: A clippy lint would need to:
   - Detect instances where `Config` is used with non-Debug formatting
   - Identify format strings/macros that use `{}` instead of `{:?}` with `Config`
   - Associate the formatting with specific types (Config in this case)

2. **Implementation Feasibility**:
   - Custom clippy lints require creating a separate crate that depends on the clippy_utils crate
   - It would need to analyze the AST (Abstract Syntax Tree) to detect formatting patterns
   - Detecting format string contents is complex and potentially error-prone

3. **Decision**:
   - The doc comment warning is a more practical solution for this codebase size
   - The existing test `test_config_debug_redaction` already verifies the redaction behavior
   - A clippy lint would be a significant development effort compared to the value provided

4. **Alternative Approach**:
   - Consider adding a private constructor and making fields private in a future refactoring
   - This would enforce access only through safe accessors (rather than relying on documentation)
   - But this is a larger change that would affect many call sites

### Conclusion
The codebase is correctly handling Config values with appropriate redaction. The added doc comment warning will help future developers maintain this security practice. I found no instances where Config values were incorrectly logged without proper redaction of secrets.