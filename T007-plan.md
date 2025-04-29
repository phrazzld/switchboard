# T007: Document OpenAI env vars in README

## Task Description
Add documentation for the OpenAI environment variables to the README.md file.

## Changes Required
1. Add OpenAI environment variables to the "Environment Variables" table in README.md.
   - Add `OPENAI_API_KEY`, `OPENAI_API_BASE_URL`, and `OPENAI_ENABLED` with descriptions and default values.
   - Ensure the descriptions are concise and align with the documentation in config.rs.

2. Update the .env file example in the "Getting Started > Setup" section to include OpenAI configuration.
   - Add commented examples of the OpenAI environment variables.
   - Make it clear that OpenAI integration is optional and disabled by default.

3. Consider if any other sections need updates to mention OpenAI support, such as:
   - Features section
   - Prerequisites 
   - Usage section

## Implementation Approach
1. Add the OpenAI variables to the "Environment Variables" table, maintaining consistent formatting with existing entries.
2. Add the OpenAI variables to the .env file example in the Setup section.
3. Update the Features section to mention OpenAI API support as an opt-in feature.
4. Update the Usage section to mention both Anthropic and OpenAI API endpoints.

Will maintain the existing style and level of detail while ensuring the new documentation clearly explains that OpenAI support is opt-in and requires explicit configuration.