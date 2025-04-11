# Implement Request Duration Logging

## Task Title
Implement Request Duration Logging: Calculate elapsed time and record `duration_ms` in the main handler span before returning `Ok(response)`.

## Implementation Approach
Add code to calculate the elapsed time using the `start` Instant already created at the beginning of the function. At the end of both the streaming and non-streaming response paths, calculate the duration using `start.elapsed()`, record it in the tracing span with `span.record("duration_ms", duration.as_millis())`, and add a log message with the duration before returning the response.