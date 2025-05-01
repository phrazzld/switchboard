# T005 Plan: Strengthen `test_config_debug_redaction` unit test

## Context
Ticket T005 is part of cr-02 Bulletproof API key redaction, Step 2. The goal is to improve and enhance the test that verifies API key redaction works properly to provide more thorough and robust testing.

## Approach
1. Locate the existing test and analyze its strengths and weaknesses
2. Enhance the test with more realistic API key values and more thorough assertions
3. Add testing for potential edge cases in the redaction process

## Implementation Steps
1. Update the test to use more complex and realistic API key patterns
2. Add additional assertions that check:
   - The output contains appropriate non-secret fields
   - No parts or substrings of the API keys appear in the debug output
   - Various segments and identifiable parts of the keys are properly redacted
3. Verify that the SecretString::expose_secret method still works correctly despite redaction

## Changes Made
1. Used more realistic API key patterns similar to actual Anthropic and OpenAI key formats
2. Added pattern markers like "DEADBEEF", "CAFEBABE", "TESTKEY" to make substring testing more effective
3. Added assertions to verify the output:
   - Contains the struct name and non-secret fields
   - Contains the [REDACTED] placeholder
   - Does not contain any specific key prefixes (sk-ant, sk-oai)
   - Does not contain any of the test markers in the keys
4. Added sliding window substring tests that check for any 10-character subsequence of the keys
5. Added verification that expose_secret() still provides access to the unredacted values when needed

## Verification
The enhanced test provides more thorough verification of the redaction behavior and will catch any inadvertent changes to the Debug formatting that might expose secrets.