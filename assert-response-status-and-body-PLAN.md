# Assert Response Status and Body in Basic Test

## Implementation Approach
Add assertions to the test function that check the HTTP response status code is 200 OK, then extract and parse the response body bytes to verify it matches the expected JSON structure {"status": "ok"}. Use http_body_util::BodyExt to aggregate the body stream and assert_eq! to verify equality.