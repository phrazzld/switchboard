# Define `wiremock::Mock` Expectation in Basic Test

## Implementation Approach
Add necessary wiremock imports and define a mock expectation inside the test function that matches POST requests to "/v1/messages". Configure the mock to respond with a 200 OK status and a simple JSON response body like {"status": "ok"}. Mount this mock to the test_setup.mock_server.