# Remove `dead_code` suppression from CI lint job

## Implementation Approach
Modify the lint job in `.github/workflows/ci.yml` to remove the `-A dead_code` flag from the `cargo clippy` command, forcing clippy to report dead code warnings. This will help enforce the code standard that violations should be addressed rather than suppressed, aligning with `CODING_STANDARDS.md` Standard 8.